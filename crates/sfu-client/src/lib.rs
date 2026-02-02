use std::sync::Arc;
use video_chat_core::error::Result;
use video_chat_media::PeerConnectionManager;
use video_chat_signaling::stun_config::StunConfig;
use video_chat_signaling::{Message, MessageType};

pub mod reconnection;
pub use reconnection::ReconnectionManager;

#[cfg(test)]
mod tests;

use std::sync::Mutex;
use tokio::sync::Mutex as AsyncMutex;
use video_chat_media::DataChannelManager;

use video_chat_signaling::websocket::WebSocketSignaling;

pub struct SfuClient {
    pub participant_id: String,
    #[allow(clippy::arc_with_non_send_sync)]
    pub peer_connection: Arc<PeerConnectionManager>,
    pub room_id: String,
    subscribed_streams: Arc<tokio::sync::RwLock<std::collections::HashSet<String>>>,
    _data_channel: Arc<Mutex<DataChannelManager>>,
    pub signaling: Arc<AsyncMutex<WebSocketSignaling>>,
}

impl SfuClient {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(
        room_id: String,
        signaling_url: &str,
        stun_config: &StunConfig,
        on_message_cb: Option<Box<dyn Fn(String) + Send + 'static>>,
    ) -> Result<Self> {
        // Generate unique participant ID
        #[cfg(target_arch = "wasm32")]
        let participant_id = format!("user-{}", js_sys::Date::now() as u64);
        #[cfg(not(target_arch = "wasm32"))]
        let participant_id = format!(
            "user-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        log::info!("Creating SfuClient with participant_id: {}", participant_id);

        let pc = Arc::new(PeerConnectionManager::new(stun_config)?);
        let dc_manager = DataChannelManager::new();

        // Signaling server with message callback
        let signaling = WebSocketSignaling::new(signaling_url);
        let signaling = Arc::new(AsyncMutex::new(signaling));

        // Set up ICE candidate handler
        {
            let signaling = signaling.clone();
            let _room_id = room_id.clone();
            let participant_id_for_ice = participant_id.clone();
            pc.set_onicecandidate(move |event: web_sys::RtcPeerConnectionIceEvent| {
                if let Some(candidate) = event.candidate() {
                    let candidate_str = candidate.candidate();
                    let sdp_mid = candidate.sdp_mid();
                    let sdp_m_line_index = candidate.sdp_m_line_index();

                    // Create IceCandidate message
                    use video_chat_signaling::messages::IceCandidate;
                    let ice_candidate = IceCandidate {
                        candidate: candidate_str,
                        sdp_mid,
                        sdp_m_line_index,
                    };

                    let participant_id = participant_id_for_ice.clone();
                    // Helper to send message
                    let signaling = signaling.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let msg = Message::new(
                            format!("ice-{}", js_sys::Date::now()),
                            MessageType::IceCandidate {
                                candidate: ice_candidate,
                                participant_id,
                            },
                        );
                        let sig = signaling.lock().await;
                        if let Err(e) = sig.send(msg).await {
                            log::error!("Failed to send ICE candidate: {}", e);
                        }
                    });
                }
            });
        }

        // Set up WebSocket message handler
        let pc_clone = pc.clone();
        let signaling_clone = signaling.clone();
        let participant_id_clone = participant_id.clone();

        let ws_callback = on_message_cb.map(|cb| {
            Box::new(move |msg: video_chat_signaling::Message| {
                let pc = pc_clone.clone();
                let signaling = signaling_clone.clone();
                let _participant_id = participant_id_clone.clone();

                match msg.payload {
                     video_chat_signaling::MessageType::Chat { text, participant_id: sender_id, .. } => {
                         // Pass both sender ID and text to callback
                         // Format: "sender_id:text"
                         cb(format!("{}:{}", sender_id, text));
                     }
                     video_chat_signaling::MessageType::Error { message, .. } => {
                         cb(message);
                     }
                     video_chat_signaling::MessageType::Offer { sdp, participant_id } => {
                         // Handle Offer -> Send Answer
                         wasm_bindgen_futures::spawn_local(async move {
                             log::info!("Received Offer from {}", participant_id);
                             if let Err(e) = pc.set_remote_description(&sdp.sdp, web_sys::RtcSdpType::Offer).await {
                                 log::error!("Failed to set remote description (Offer): {}", e);
                                 return;
                             }

                             match pc.create_answer().await {
                                 Ok(answer_sdp) => {
                                      // Send Answer
                                      let answer_msg = Message::new(
                                          format!("answer-{}", js_sys::Date::now()),
                                          MessageType::Answer {
                                              sdp: video_chat_signaling::messages::SessionDescription {
                                                  sdp_type: video_chat_signaling::messages::SdpType::Answer,
                                                  sdp: answer_sdp,
                                              },
                                              participant_id: participant_id.clone(),
                                          },
                                      );
                                      let sig = signaling.lock().await;
                                      if let Err(e) = sig.send(answer_msg).await {
                                            log::error!("Failed to send Answer: {}", e);
                                      }
                                 }
                                 Err(e) => log::error!("Failed to create answer: {}", e),
                             }
                         });
                     }
                     video_chat_signaling::MessageType::Answer { sdp, .. } => {
                         // Handle Answer
                         wasm_bindgen_futures::spawn_local(async move {
                             log::info!("Received Answer");
                             if let Err(e) = pc.set_remote_description(&sdp.sdp, web_sys::RtcSdpType::Answer).await {
                                 log::error!("Failed to set remote description (Answer): {}", e);
                             }
                         });
                     }
                     video_chat_signaling::MessageType::IceCandidate { candidate, .. } => {
                         // Handle ICE Candidate
                         wasm_bindgen_futures::spawn_local(async move {
                             // Create RtcIceCandidateInit
                             let init = web_sys::RtcIceCandidateInit::new(&candidate.candidate);
                             init.set_sdp_mid(candidate.sdp_mid.as_deref());
                             init.set_sdp_m_line_index(candidate.sdp_m_line_index);

                             if let Err(e) = pc.add_ice_candidate(&init).await {
                                 log::error!("Failed to add ICE candidate: {}", e);
                             }
                         });
                     }
                     _ => {}
                }
            }) as Box<dyn Fn(video_chat_signaling::Message) + 'static>
        });

        // Connect to signaling with callback
        // Signaling is now Arc<AsyncMutex<..>>, need inner mutex to connect?
        // Wait, WebSocketSignaling::connect_with_callback returns Result.
        // I need to unlock it.
        {
            let mut sig = signaling.try_lock().map_err(|_| {
                video_chat_core::Error::Signaling(
                    "Failed to lock signaling for connection".to_string(),
                )
            })?;
            sig.connect_with_callback(ws_callback)
                .map_err(|e| video_chat_core::Error::Signaling(e.to_string()))?;
        }

        Ok(Self {
            participant_id,
            peer_connection: pc,
            room_id,
            subscribed_streams: Arc::new(
                tokio::sync::RwLock::new(std::collections::HashSet::new()),
            ),
            _data_channel: Arc::new(Mutex::new(dc_manager)),
            signaling,
        })
    }

    /// Create and send specific Offer (renegotiation)
    pub fn create_offer(&self) -> Result<()> {
        let pc = self.peer_connection.clone();
        let signaling = self.signaling.clone();
        let participant_id = self.participant_id.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match pc.create_offer().await {
                Ok(offer_sdp) => {
                    // Send Offer
                    let offer_msg = Message::new(
                        format!("offer-{}", js_sys::Date::now()),
                        MessageType::Offer {
                            sdp: video_chat_signaling::messages::SessionDescription {
                                sdp_type: video_chat_signaling::messages::SdpType::Offer,
                                sdp: offer_sdp,
                            },
                            participant_id,
                        },
                    );
                    let sig = signaling.lock().await;
                    if let Err(e) = sig.send(offer_msg).await {
                        log::error!("Failed to send Offer: {}", e);
                    }
                }
                Err(e) => log::error!("Failed to create offer: {}", e),
            }
        });
        Ok(())
    }

    /// Send a chat message via WebSocket signaling (relayed through SFU)
    pub fn send_message(&self, text: &str) -> Result<()> {
        // Create a Chat message with a simple timestamp-based ID
        let timestamp = js_sys::Date::now() as u64;

        let message = Message::new(
            format!("chat-{}", timestamp),
            MessageType::Chat {
                room_id: self.room_id.clone(),
                participant_id: self.participant_id.clone(),
                text: text.to_string(),
            },
        );

        // Send via signaling (async, but we'll use a blocking approach for WASM)
        wasm_bindgen_futures::spawn_local({
            let signaling = self.signaling.clone();
            async move {
                let sig = signaling.lock().await;
                if let Err(e) = sig.send(message).await {
                    log::error!("Failed to send chat message: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Send Join message to the SFU server to join a room
    pub fn join(&self) -> Result<()> {
        let timestamp = js_sys::Date::now() as u64;
        let room_id = self.room_id.clone();
        let signaling = self.signaling.clone();
        let participant_id = self.participant_id.clone();

        // Send join message with retry if socket not open yet
        wasm_bindgen_futures::spawn_local(async move {
            let message = Message::new(
                format!("join-{}", timestamp),
                MessageType::Join {
                    room_id,
                    participant_id,
                },
            );

            for i in 0..20 {
                {
                    let sig = signaling.lock().await;
                    match sig.send(message.clone()).await {
                        Ok(_) => {
                            log::info!("Join message sent successfully");
                            return;
                        }
                        Err(e) => {
                            if e.to_string().contains("not open") {
                                log::debug!(
                                    "WebSocket not open yet, retrying join... ({}/20)",
                                    i + 1
                                );
                            } else {
                                log::error!("Failed to send join message: {}", e);
                                return;
                            }
                        }
                    }
                }
                // Sleep for 200ms
                let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                    &mut |resolve, _| {
                        if let Some(window) = web_sys::window() {
                            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                                &resolve, 200,
                            );
                        }
                    },
                ))
                .await;
            }
            log::error!("Gave up sending Join message after 20 retries");
        });

        Ok(())
    }

    /// Subscribe to a participant's media stream
    /// This sends a signaling message to the SFU to start forwarding the stream
    pub async fn subscribe_to_stream(&self, participant_id: &str) -> Result<()> {
        log::info!("Subscribing to stream from participant: {}", participant_id);

        // Add to subscribed streams
        let mut streams = self.subscribed_streams.write().await;
        if streams.contains(participant_id) {
            log::warn!("Already subscribed to participant: {}", participant_id);
            return Ok(());
        }
        streams.insert(participant_id.to_string());
        drop(streams);

        // Create subscribe message for SFU
        let message = Message::new(
            format!("sub-{}", participant_id),
            MessageType::Subscribe {
                target_id: participant_id.to_string(),
                participant_id: self.participant_id.clone(),
            },
        );

        let sig = self.signaling.lock().await;
        sig.send(message)
            .await
            .map_err(|e| video_chat_core::Error::Signaling(e.to_string()))?;

        Ok(())
    }

    /// Unsubscribe from a participant's media stream
    /// This signals the SFU to stop forwarding the stream
    pub async fn unsubscribe_from_stream(&self, participant_id: &str) -> Result<()> {
        log::info!(
            "Unsubscribing from stream from participant: {}",
            participant_id
        );

        // Remove from subscribed streams
        let mut streams = self.subscribed_streams.write().await;
        if !streams.remove(participant_id) {
            log::warn!("Not subscribed to participant: {}", participant_id);
            return Ok(());
        }
        drop(streams);

        // Create unsubscribe message for SFU
        let message = Message::new(
            format!("unsub-{}", participant_id),
            MessageType::Unsubscribe {
                target_id: participant_id.to_string(),
                participant_id: "me".to_string(),
            },
        );

        let sig = self.signaling.lock().await;
        sig.send(message)
            .await
            .map_err(|e| video_chat_core::Error::Signaling(e.to_string()))?;

        Ok(())
    }

    /// Get list of currently subscribed participant IDs
    pub async fn get_subscribed_streams(&self) -> Vec<String> {
        self.subscribed_streams
            .read()
            .await
            .iter()
            .cloned()
            .collect()
    }

    /// Check if subscribed to a specific participant
    pub async fn is_subscribed(&self, participant_id: &str) -> bool {
        self.subscribed_streams
            .read()
            .await
            .contains(participant_id)
    }

    pub fn set_audio_enabled(&self, enabled: bool) -> Result<()> {
        self.peer_connection
            .set_audio_enabled(enabled)
            .map_err(|e| video_chat_core::Error::Media(e.to_string()))
    }

    pub fn set_video_enabled(&self, enabled: bool) -> Result<()> {
        self.peer_connection
            .set_video_enabled(enabled)
            .map_err(|e| video_chat_core::Error::Media(e.to_string()))
    }

    pub fn add_track(
        &self,
        track: &web_sys::MediaStreamTrack,
        stream: &web_sys::MediaStream,
    ) -> Result<()> {
        // Use standard track addition without simulcast for now
        self.peer_connection
            .add_track(track, stream)
            .map_err(|e| video_chat_core::Error::Media(e.to_string()))
    }

    /// Explicitly trigger WebRTC negotiation (create offer and send to server)
    pub fn start_negotiation(&self) -> Result<()> {
        self.create_offer()
    }

    /// Leave the room and close connections
    pub fn leave(&self) {
        log::info!("Leaving room: {}", self.room_id);
        self.peer_connection.close();
    }
}
