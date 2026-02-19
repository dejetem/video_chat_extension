#![allow(dead_code)]
use anyhow::Result;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod bandwidth;
mod negotiation;
mod recorder;
mod room_manager;
mod router;
mod simulcast;
mod utils;

use crate::negotiation::NegotiationManager; // Import NegotiationManager
use crate::recorder::Recorder;
use crate::room_manager::RoomManager;
use crate::router::MediaRouter;
use axum::extract::State;
use std::sync::Arc;
use video_chat_signaling::{Message as SignalingMessage, MessageType};
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit; // Import RTCIceCandidateInit
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

// ... (AppState struct remains same)

// ... (main function remains same)

// ... (handlers remain same)

// Inside handle_socket, match signaling_msg.payload
// I'll replace the loop body or specifically the match block

#[derive(Clone)]
struct AppState {
    room_manager: Arc<RoomManager>,
    media_router: Arc<MediaRouter>,
    recorder: Arc<Recorder>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Rust Video Chat SFU Server...");

    let state = AppState {
        room_manager: Arc::new(RoomManager::new()),
        media_router: Arc::new(MediaRouter::new()),
        recorder: Arc::new(Recorder::new()),
    };

    // Build our application routes
    let static_dir = if std::path::Path::new("web").exists() {
        "web"
    } else {
        "../../web"
    };
    info!("Serving static files from: {}", static_dir);

    use axum::routing::post;
    let app = Router::new()
        .route("/ws", get(signaling_handler))
        .route("/api/config", get(config_handler))
        .route("/api/rooms", get(list_rooms_handler))
        .route("/api/close-room/:room_id", post(close_room_handler))
        .fallback_service(tower_http::services::ServeDir::new(static_dir))
        .with_state(state);

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn config_handler() -> axum::Json<serde_json::Value> {
    // Read tunnel URLs from environment variables
    let tunnel_url =
        std::env::var("TUNNEL_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let tunnel_ws_url = std::env::var("TUNNEL_WEB_S_SOCKET_URL")
        .unwrap_or_else(|_| "ws://localhost:8080/ws".to_string());

    axum::Json(serde_json::json!({
        "tunnelUrl": tunnel_url,
        "tunnelWsUrl": tunnel_ws_url
    }))
}

async fn list_rooms_handler(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let rooms = state.room_manager.rooms.read().await;
    let rooms_json: Vec<serde_json::Value> = rooms
        .values()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "participants": r.participants.len(),
                "created": "2026-01-25 16:00", // Placeholder
                "status": "Active"
            })
        })
        .collect();
    axum::Json(serde_json::json!(rooms_json))
}

async fn close_room_handler(
    State(state): State<AppState>,
    axum::extract::Path(room_id): axum::extract::Path<String>,
) -> String {
    info!("Closing room: {}", room_id);
    // Broadcast CLEAR_LOGS command
    let clear_msg = SignalingMessage::new(
        "system-clear",
        MessageType::Error {
            code: 200, // Using Error type as a hack for simplicity or we should add a Command type
            message: "SYSTEM_COMMAND:CLEAR_LOGS".to_string(),
        },
    );
    // Actually our WASM expects a raw string "SYSTEM_COMMAND:CLEAR_LOGS" in the data channel
    // message? No, it expects it in the signaling callback.

    // Let's just send a simple message that WASM can parse
    state
        .room_manager
        .broadcast_to_room(&room_id, clear_msg)
        .await;

    // In a real app, we'd wait a bit then close the room session
    "Room close signal sent".to_string()
}

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};

async fn signaling_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SignalingMessage>();

    // Task to forward messages from mpsc to websocket
    let sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    info!("New signaling connection established");
    let mut current_room: Option<String> = None;
    let mut current_participant: Option<String> = None;

    while let Some(msg) = receiver.next().await {
        if let Ok(Message::Text(text)) = msg {
            // Check if this is a ping/pong message
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                if value.get("type").and_then(|t| t.as_str()) == Some("ping") {
                    // Respond with pong using the mpsc channel to avoid checking 'sender' which is
                    // moved
                    let pong_msg = SignalingMessage::new("pong", MessageType::Pong);
                    // We ignore send errors here, as the loop will handle disconnection eventually
                    let _ = tx.send(pong_msg);
                    continue;
                }
            }

            match serde_json::from_str::<SignalingMessage>(&text) {
                Ok(signaling_msg) => {
                    match signaling_msg.payload {
                        MessageType::Join {
                            room_id,
                            participant_id,
                        } => {
                            info!("Participant {} joining room {}", participant_id, room_id);
                            current_room = Some(room_id.clone());
                            current_participant = Some(participant_id.clone());
                            state
                                .room_manager
                                .join_room(
                                    room_id.clone(),
                                    participant_id.clone(),
                                    Some(tx.clone()),
                                )
                                .await;

                            // Send RoomJoined with ICE configuration
                            let ice_servers = crate::utils::load_ice_servers();
                            let joined_msg = SignalingMessage::new(
                                format!("joined-{}", signaling_msg.id),
                                MessageType::RoomJoined {
                                    room_id,
                                    participant_id,
                                    ice_servers,
                                },
                            );
                            let _ = tx.send(joined_msg);
                        }
                        MessageType::Subscribe {
                            target_id,
                            participant_id,
                        } => {
                            info!(
                                "Participant {} subscribing to {}",
                                participant_id, target_id
                            );
                            // Future: Trigger MediaRouter subscription
                        }
                        MessageType::Offer {
                            room_id,
                            sdp,
                            participant_id,
                        } => {
                            info!(
                                "Received Offer from {} for room {}",
                                participant_id, room_id
                            );

                            // Use room_id from message instead of current_room to avoid race
                            // condition
                            match NegotiationManager::handle_offer(
                                state.room_manager.clone(),
                                state.media_router.clone(),
                                room_id.clone(),
                                participant_id.clone(),
                                sdp.sdp,
                            )
                            .await
                            {
                                Ok(answer_sdp) => {
                                    info!("Generated Answer for {}", participant_id);
                                    let answer_msg = SignalingMessage::new(
                                        format!("answer-{}", signaling_msg.id),
                                        MessageType::Answer {
                                            sdp: video_chat_signaling::messages::SessionDescription {
                                                sdp_type: video_chat_signaling::messages::SdpType::Answer,
                                                sdp: answer_sdp,
                                            },
                                            participant_id: "sfu".to_string(),
                                        },
                                    );
                                    let _ = tx.send(answer_msg);
                                }
                                Err(e) => {
                                    warn!("Failed to handle offer from {}: {}", participant_id, e);
                                }
                            }
                        }
                        MessageType::Answer {
                            sdp,
                            participant_id: _, /* Ignore the participant_id from message (it's
                                                * "sfu") */
                        } => {
                            // Handle Answer from client (renegotiation response)
                            // Use current_participant from WebSocket context, not from message
                            if let (Some(room_id), Some(participant_id)) =
                                (&current_room, &current_participant)
                            {
                                info!("Received Answer from {} (renegotiation)", participant_id);
                                if let Some(pc) = state
                                    .room_manager
                                    .get_peer_connection(room_id, participant_id)
                                    .await
                                {
                                    // Set remote description to complete renegotiation
                                    let mut desc = RTCSessionDescription::default();
                                    desc.sdp = sdp.sdp;
                                    desc.sdp_type = RTCSdpType::Answer;

                                    if let Err(e) = pc.set_remote_description(desc).await {
                                        warn!(
                                            "Failed to set remote description (Answer) for {}: {}",
                                            participant_id, e
                                        );
                                    } else {
                                        info!(
                                            "Successfully processed renegotiation Answer from {}",
                                            participant_id
                                        );
                                    }
                                }
                            }
                        }

                        MessageType::IceCandidate {
                            candidate,
                            participant_id,
                        } => {
                            if let Some(room_id) = &current_room {
                                if let Some(pc) = state
                                    .room_manager
                                    .get_peer_connection(room_id, &participant_id)
                                    .await
                                {
                                    let init = RTCIceCandidateInit {
                                        candidate: candidate.candidate,
                                        sdp_mid: candidate.sdp_mid,
                                        sdp_mline_index: candidate.sdp_m_line_index,
                                        username_fragment: None,
                                    };
                                    if let Err(e) = pc.add_ice_candidate(init).await {
                                        warn!("Failed to add ICE candidate: {}", e);
                                    }
                                }
                            }
                        }
                        MessageType::Chat {
                            room_id,
                            participant_id,
                            text,
                        } => {
                            info!(
                                "Chat message from {} in room {}: {}",
                                participant_id, room_id, text
                            );
                            // Broadcast the chat message to all participants in the room
                            let chat_msg = SignalingMessage::new(
                                format!("chat-{}", signaling_msg.id),
                                MessageType::Chat {
                                    room_id: room_id.clone(),
                                    participant_id: participant_id.clone(),
                                    text: text.clone(),
                                },
                            );
                            state
                                .room_manager
                                .broadcast_to_room(&room_id, chat_msg)
                                .await;
                        }
                        _ => {
                            info!("Received signaling message: {:?}", signaling_msg);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to parse signaling message: {}", e);
                }
            }
        } else if msg.is_err() {
            break;
        }
    }

    if let (Some(r), Some(p)) = (current_room, current_participant) {
        state.room_manager.leave_room(&r, &p).await;
    }
    sender_task.abort();
    info!("Client disconnected");
}
