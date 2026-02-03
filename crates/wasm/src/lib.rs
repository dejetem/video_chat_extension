mod chrome_api;
mod js_interop;
pub mod media;
mod schema;
mod utils;

#[cfg(test)]
mod tests;

use wasm_bindgen::prelude::*;

pub mod sqlite;

use crate::sqlite::Database;
use std::cell::RefCell;
use video_chat_sfu_client::SfuClient;

// Thread-local storage is safe for WASM (single-threaded) and allows !Send types like
// RtcPeerConnection
thread_local! {
    static SFU_CLIENT: RefCell<Option<SfuClient>> = const { RefCell::new(None) };
    static DB: RefCell<Option<Database>> = const { RefCell::new(None) };
    static REMOTE_TRACKS: RefCell<std::collections::HashMap<String, web_sys::MediaStreamTrack>> = RefCell::new(std::collections::HashMap::new());
}

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
    utils::init_logging();
    log::info!("Video Chat Extension WASM initialized");

    // Use a path that is likely to be mapped to OPFS by convention if VFS is set up
    // The prefix 'mvfs:/' or similar might be needed depending on the VFS implementation in
    // sqlite-wasm-rs For now, we stick to a simple filename which defaults to the mounted root.
    match Database::open_persistent("video_chat_logs.db") {
        Ok(db) => {
            DB.with(|d| {
                *d.borrow_mut() = Some(db);
            });
            log::info!("SQLite database initialized successfully");
        }
        Err(e) => {
            log::error!("Failed to initialize SQLite database: {}", e);
        }
    }
}

#[wasm_bindgen]
pub fn create_room(room_id: String, signaling_url: String) {
    log::info!(
        "Creating room: {} with signaling: {}",
        room_id,
        signaling_url
    );
    initialize_client(room_id.clone(), signaling_url);
    js_interop::dispatch_event("roomCreated", &room_id);
}

#[wasm_bindgen]
pub fn join_room(room_id: String, signaling_url: String) {
    log::info!(
        "Joining room: {} with signaling: {}",
        room_id,
        signaling_url
    );
    initialize_client(room_id.clone(), signaling_url);
    js_interop::dispatch_event("roomJoined", &room_id);
}

#[wasm_bindgen]
pub fn leave_room() {
    SFU_CLIENT.with(|c| {
        if let Some(client) = c.borrow_mut().take() {
            client.leave();
        }
    });
    log::info!("Left room");
    js_interop::dispatch_event("roomLeft", &"");
}

fn initialize_client(room_id: String, signaling_url: String) {
    let stun_config = video_chat_signaling::stun_config::default_stun_config();

    let room_id_clone = room_id.clone();
    // Create a callback for incoming messages
    let on_msg = Some(Box::new(move |message_data: String| {
        log::info!("Dispatching received message: {}", message_data);

        // Check for special system messages from sfu-client (Participant Left)
        if let Some(participant_id) = message_data.strip_prefix("participant_left:") {
            js_interop::dispatch_event("participantLeft", &participant_id.to_string());
            return;
        }

        // Parse message format: "sender_id:text"
        let (sender_id, text) = if let Some(colon_pos) = message_data.find(':') {
            let (id, msg) = message_data.split_at(colon_pos);
            (id.to_string(), msg[1..].to_string()) // Skip the colon
        } else {
            ("Participant".to_string(), message_data)
        };

        // Check if this is a system command
        if text == "SYSTEM_COMMAND:CLEAR_LOGS" {
            log::info!("Received global clear logs command");
            let r_id = room_id_clone.clone();
            DB.with(|d| {
                if let Some(db) = d.borrow().as_ref() {
                    let _ = db.delete_messages(&r_id);
                }
            });
            js_interop::dispatch_event("logsCleared", &r_id);
            return;
        }

        // Persist incoming message with sender info
        DB.with(|d| {
            if let Some(db) = d.borrow().as_ref() {
                let id = format!("msg-{}", js_sys::Date::now());
                if let Err(e) = db.insert_message(
                    &id,
                    &room_id_clone,
                    &sender_id,
                    &text,
                    js_sys::Date::now() as i64,
                ) {
                    log::error!("Failed to persist incoming message: {}", e);
                }
            }
        });

        // Determine sender label: "You" if it's our message, "Participant" otherwise
        // Get our participant ID from the SFU_CLIENT
        let sender_label = SFU_CLIENT.with(|c| {
            if let Some(client) = c.borrow().as_ref() {
                if sender_id == client.participant_id {
                    "You"
                } else {
                    "Participant"
                }
            } else {
                "Participant"
            }
        });

        // Create message data with sender label and text
        let message_data = format!("{}:{}", sender_label, text);
        js_interop::dispatch_event("messageReceived", &message_data);
    }) as Box<dyn Fn(String) + Send + 'static>);

    match SfuClient::new(room_id, &signaling_url, &stun_config, on_msg) {
        Ok(client) => {
            // Set ontrack handler for remote tracks
            let pc = client.peer_connection.get_connection().clone();
            let on_track = Closure::wrap(Box::new(move |event: web_sys::RtcTrackEvent| {
                let track = event.track();
                let streams = event.streams();
                let _stream = streams.get(0).dyn_into::<web_sys::MediaStream>().ok();

                log::info!(
                    "Remote track received in WASM: kind={}, id={}",
                    track.kind(),
                    track.id()
                );

                let track_id = track.id();

                // Extract Participant ID from Stream ID (format: "stream-{pid}")
                let mut pid = "participant".to_string(); // Default fallback
                if let Ok(stream_obj) = streams.get(0).dyn_into::<web_sys::MediaStream>() {
                    let stream_id = stream_obj.id();
                    if let Some(stripped) = stream_id.strip_prefix("stream-") {
                        pid = stripped.to_string();
                    }
                }

                // Format payload as "trackId|participantId|kind"
                // This allows frontend to distinguish Audio (hidden) vs Video (visible)
                let payload = format!("{}|{}|{}", track_id, pid, track.kind());

                REMOTE_TRACKS.with(|t| {
                    t.borrow_mut().insert(track_id.clone(), track);
                });

                js_interop::dispatch_event("trackReceived", &payload);
            }) as Box<dyn FnMut(_)>);
            pc.set_ontrack(Some(on_track.as_ref().unchecked_ref()));
            on_track.forget();

            // Send Join message to server
            if let Err(e) = client.join() {
                log::error!("Failed to join room: {}", e);
            } else {
                log::info!(
                    "Successfully sent join message for room: {}",
                    client.room_id
                );
            }

            SFU_CLIENT.with(|c| {
                *c.borrow_mut() = Some(client);
            });
            log::info!("SFU Client initialized connected with ontrack handler");
        }
        Err(e) => {
            log::error!("Failed to initialize SFU Client: {}", e);
        }
    }
}

#[wasm_bindgen]
pub fn attach_remote_track(video_id: String, track_id: String) {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let video = document
        .get_element_by_id(&video_id)
        .expect("should have a video element")
        .dyn_into::<web_sys::HtmlMediaElement>()
        .expect("element is not a media element (video or audio)");

    REMOTE_TRACKS.with(|t| {
        if let Some(track) = t.borrow().get(&track_id) {
            let stream = web_sys::MediaStream::new().expect("failed to create media stream");
            stream.add_track(track);
            video.set_src_object(Some(&stream));
            let _ = video.play();
            log::info!("Attached track {} to video {}", track_id, video_id);
        } else {
            log::warn!("Track {} not found for attachment", track_id);
        }
    });
}
#[wasm_bindgen]
pub fn add_track(track: web_sys::MediaStreamTrack, stream: web_sys::MediaStream) {
    SFU_CLIENT.with(|c| {
        if let Some(client) = c.borrow().as_ref() {
            if let Err(e) = client.add_track(&track, &stream) {
                log::error!("Failed to add track: {}", e);
            } else {
                log::info!("Successfully added track to SFU connection");
            }
        }
    });
}

#[wasm_bindgen]
pub fn add_stream(stream: web_sys::MediaStream) {
    SFU_CLIENT.with(|c| {
        if let Some(client) = c.borrow().as_ref() {
            let tracks = stream.get_tracks();
            for i in 0..tracks.length() {
                let track = tracks
                    .get(i)
                    .dyn_into::<web_sys::MediaStreamTrack>()
                    .unwrap();
                if let Err(e) = client.add_track(&track, &stream) {
                    log::error!("Failed to add track from stream: {}", e);
                }
            }
            if let Err(e) = client.start_negotiation() {
                log::error!("Failed to start negotiation after adding stream: {}", e);
            } else {
                log::info!("Successfully added stream and started negotiation");
            }
        }
    });
}

#[wasm_bindgen]
pub fn start_negotiation() {
    SFU_CLIENT.with(|c| {
        if let Some(client) = c.borrow().as_ref() {
            if let Err(e) = client.start_negotiation() {
                log::error!("Failed to start negotiation: {}", e);
            }
        }
    });
}

#[wasm_bindgen]
pub fn toggle_microphone(enabled: bool) {
    log::info!("Microphone enabled: {}", enabled);
    SFU_CLIENT.with(|c| {
        if let Some(client) = c.borrow().as_ref() {
            let _ = client.set_audio_enabled(enabled);
        }
    });
    js_interop::dispatch_event("mediaStatus", &format!("mic:{}", enabled));
}

#[wasm_bindgen]
pub fn toggle_camera(enabled: bool) {
    log::info!("Camera enabled: {}", enabled);
    SFU_CLIENT.with(|c| {
        if let Some(client) = c.borrow().as_ref() {
            let _ = client.set_video_enabled(enabled);
        }
    });
    js_interop::dispatch_event("mediaStatus", &format!("cam:{}", enabled));
}

#[wasm_bindgen]
pub fn send_message(text: String) {
    log::info!("Sending message: {}", text);
    SFU_CLIENT.with(|c| {
        if let Some(client) = c.borrow().as_ref() {
            if let Err(e) = client.send_message(&text) {
                log::error!("Failed to send message: {}", e);
            } else {
                // Persist outgoing message
                let room_id = client.room_id.clone();
                DB.with(|d| {
                    if let Some(db) = d.borrow().as_ref() {
                        let id = format!("msg-{}", js_sys::Date::now());
                        if let Err(e) = db.insert_message(
                            &id,
                            &room_id,
                            "You",
                            &text,
                            js_sys::Date::now() as i64,
                        ) {
                            log::error!("Failed to persist outgoing message: {}", e);
                        }
                    }
                });
            }
        } else {
            log::warn!("SFU Client not initialized, cannot send message");
        }
    });
    js_interop::dispatch_event("messageSent", &text);
}
