use anyhow::Result;
use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::room_manager::RoomManager;
use crate::router::MediaRouter;
use video_chat_signaling::{messages::IceCandidate, Message, MessageType};

pub struct NegotiationManager;

impl NegotiationManager {
    pub async fn handle_offer(
        room_manager: Arc<RoomManager>,
        media_router: Arc<MediaRouter>,
        room_id: String,
        participant_id: String,
        sdp: String,
    ) -> Result<String> {
        let pc = match room_manager
            .get_peer_connection(&room_id, &participant_id)
            .await
        {
            Some(pc) => pc,
            None => {
                // Create MediaEngine and API
                let mut m = MediaEngine::default();
                m.register_default_codecs()?;

                let mut registry = Registry::new();
                registry = register_default_interceptors(registry, &mut m)?;

                let api = APIBuilder::new()
                    .with_media_engine(m)
                    .with_interceptor_registry(registry)
                    .build();

                // Load ICE server configuration from environment variables
                let stun_url = std::env::var("STUN_URL")
                    .unwrap_or_else(|_| "stun:stun.l.google.com:19302".to_string());
                let turn_username =
                    std::env::var("TURN_USERNAME").unwrap_or_else(|_| "".to_string());
                let turn_credential =
                    std::env::var("TURN_CREDENTIAL").unwrap_or_else(|_| "".to_string());

                // Build ICE servers list
                let mut ice_servers = vec![RTCIceServer {
                    urls: vec![stun_url],
                    ..Default::default()
                }];

                // Add TURN servers if credentials are provided
                if !turn_username.is_empty() && !turn_credential.is_empty() {
                    // Add all TURN server URLs from environment
                    for i in 1..=4 {
                        if let Ok(turn_url) = std::env::var(format!("TURN_URL_{}", i)) {
                            ice_servers.push(RTCIceServer {
                                urls: vec![turn_url.clone()],
                                username: turn_username.clone(),
                                credential: turn_credential.clone(),
                                ..Default::default()
                            });
                            tracing::info!("Added TURN server: {}", turn_url);
                        }
                    }
                }

                // Create PeerConnection
                let config = RTCConfiguration {
                    ice_servers,
                    ..Default::default()
                };
                let pc = api.new_peer_connection(config).await?;

                // Setup OnTrack handler - Forward tracks to all other participants
                let mr = media_router.clone();
                let rm = room_manager.clone();
                let rid = room_id.clone();
                let pid = participant_id.clone();

                pc.on_track(Box::new(move |track, receiver, _transceiver| {
                    let mr = mr.clone();
                    let rm = rm.clone();
                    let rid = rid.clone();
                    let pid = pid.clone();

                    Box::pin(async move {
                        let track_id = track.id();
                        let codec = track.codec();

                        tracing::info!(
                            "Track received from participant {}: id={}, kind={}",
                            pid,
                            track_id,
                            track.kind()
                        );

                        // Get all other participants' peer connections
                        let other_connections = rm.get_all_peer_connections_except(&rid, &pid).await;

                        for (other_pid, other_pc) in other_connections {
                            // Create output track for this participant
                            let output_track = Arc::new(
                                webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP::new(
                                    codec.capability.clone(),
                                    track_id.clone(),
                                    format!("stream-{}", pid),
                                )
                            );

                            // Add track to the other participant's peer connection
                            match other_pc.add_track(output_track.clone()).await {
                                Ok(_) => {
                                    tracing::info!(
                                        "Added output track {} to participant {}",
                                        track_id,
                                        other_pid
                                    );

                                    // Subscribe this output track to receive packets
                                    mr.add_subscriber(track_id.clone(), output_track).await;

                                    // Trigger renegotiation for the other participant
                                    let rm_clone = rm.clone();
                                    let rid_clone = rid.clone();
                                    let other_pid_clone = other_pid.clone();
                                    let other_pc_clone = other_pc.clone();

                                    tokio::spawn(async move {
                                        if let Err(e) = create_and_send_offer(
                                            &other_pc_clone,
                                            &rm_clone,
                                            &rid_clone,
                                            &other_pid_clone,
                                        ).await {
                                            tracing::error!(
                                                "Failed to renegotiate with {}: {}",
                                                other_pid_clone,
                                                e
                                            );
                                        }
                                    });
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to add track to participant {}: {}",
                                        other_pid,
                                        e
                                    );
                                }
                            }
                        }

                        // Start routing packets from this track to all subscribers
                        tokio::spawn(async move {
                            mr.route_track(receiver, None).await;
                        });
                    })
                }));

                // Logging ICE state changes
                pc.on_ice_connection_state_change(Box::new(move |state| {
                    tracing::info!("ICE Connection State has changed: {}", state);
                    Box::pin(async {})
                }));

                // Handle ICE Candidates -> Send to Client
                let rm = room_manager.clone();
                let rid = room_id.clone();
                let pid = participant_id.clone();
                pc.on_ice_candidate(Box::new(move |c| {
                    let rm = rm.clone();
                    let rid = rid.clone();
                    let pid = pid.clone();
                    Box::pin(async move {
                        if let Some(c) = c {
                            if let Ok(json_candidate) = c.to_json() {
                                let ice_msg = Message::new(
                                    format!("ice-{}", chrono::Utc::now().timestamp_millis()),
                                    MessageType::IceCandidate {
                                        candidate: IceCandidate {
                                            candidate: json_candidate.candidate,
                                            sdp_mid: json_candidate.sdp_mid,
                                            sdp_m_line_index: json_candidate.sdp_mline_index,
                                        },
                                        participant_id: "sfu".to_string(), // From SFU
                                    },
                                );
                                rm.send_message_to_participant(&rid, &pid, ice_msg).await;
                            }
                        }
                    })
                }));

                let pc = Arc::new(pc);
                room_manager
                    .set_peer_connection(&room_id, &participant_id, pc.clone())
                    .await;
                pc
            }
        };

        // Set Remote Description (Offer)
        // Set Remote Description (Offer)
        let mut desc = RTCSessionDescription::default();
        desc.sdp = sdp;
        desc.sdp_type = RTCSdpType::Offer;

        pc.set_remote_description(desc).await?;

        // Create Answer
        let answer = pc.create_answer(None).await?;

        // Set Local Description
        let local_desc = answer.clone();
        pc.set_local_description(answer).await?;

        // Sanitize the SDP to remove simulcast-related attributes
        // This is a workaround for the webrtc-rs library automatically adding simulcast support
        let sanitized_sdp = Self::sanitize_sdp(&local_desc.sdp);

        tracing::debug!("Original SDP:\n{}", local_desc.sdp);
        tracing::debug!("Sanitized SDP:\n{}", sanitized_sdp);

        Ok(sanitized_sdp)
    }

    /// Remove simulcast-related attributes from SDP to avoid negotiation errors
    /// This is a temporary workaround until we can properly configure MediaEngine
    fn sanitize_sdp(sdp: &str) -> String {
        // SDP uses \r\n line endings, so we need to preserve them
        let lines: Vec<&str> = sdp.split("\r\n").collect();
        let filtered: Vec<&str> = lines
            .into_iter()
            .filter(|line| {
                // Remove simulcast and RID-related attributes
                !line.starts_with("a=simulcast:") && !line.starts_with("a=rid:")
            })
            .collect();

        // Rejoin with \r\n and ensure we end with \r\n if original did
        let result = filtered.join("\r\n");
        if sdp.ends_with("\r\n") && !result.ends_with("\r\n") {
            format!("{}\r\n", result)
        } else {
            result
        }
    }

    pub fn parse_sdp(sdp_str: &str, sdp_type: RTCSdpType) -> RTCSessionDescription {
        let mut desc = RTCSessionDescription::default();
        desc.sdp = sdp_str.to_string();
        desc.sdp_type = sdp_type;
        desc
    }
}

/// Helper function to create and send an offer to trigger renegotiation
async fn create_and_send_offer(
    pc: &Arc<RTCPeerConnection>,
    room_manager: &Arc<RoomManager>,
    room_id: &str,
    participant_id: &str,
) -> anyhow::Result<()> {
    use video_chat_signaling::{Message, MessageType};

    // Create offer
    let offer: RTCSessionDescription = pc.create_offer(None).await?;
    pc.set_local_description(offer.clone()).await?;

    // Sanitize SDP
    let sanitized_sdp = NegotiationManager::sanitize_sdp(&offer.sdp);

    // Send offer to participant
    let offer_msg = Message::new(
        format!(
            "offer-renegotiate-{}",
            chrono::Utc::now().timestamp_millis()
        ),
        MessageType::Offer {
            sdp: video_chat_signaling::SessionDescription {
                sdp_type: video_chat_signaling::SdpType::Offer,
                sdp: sanitized_sdp,
            },
            participant_id: "sfu".to_string(),
        },
    );

    room_manager
        .send_message_to_participant(room_id, participant_id, offer_msg)
        .await;

    Ok(())
}
