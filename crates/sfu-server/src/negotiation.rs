use anyhow::Result;
use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::sdp_type::RTCSdpType;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

use crate::room_manager::RoomManager;
use crate::router::MediaRouter;
use video_chat_signaling::{messages::IceCandidate, Message, MessageType};
use webrtc::rtcp::payload_feedbacks::full_intra_request::FullIntraRequest;
use webrtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;

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

                // Load ICE server configuration from environment variables via shared utility
                let ice_config = crate::utils::load_ice_servers();
                let ice_servers = crate::utils::stun_to_webrtc(&ice_config);

                let rtc_config = RTCConfiguration {
                    ice_servers,
                    ..Default::default()
                };
                let pc = Arc::new(api.new_peer_connection(rtc_config).await?);

                // Setup OnTrack handler - Forward tracks to all other participants
                let mr = media_router.clone();
                let rm = room_manager.clone();
                let rid = room_id.clone();
                let pid = participant_id.clone();

                let pc_clone = pc.clone();

                let pid_for_ontrack = pid.clone();
                pc.on_track(Box::new(move |track, receiver, _transceiver| {
                    let mr = mr.clone();
                    let pc = pc_clone.clone();
                    let rm = rm.clone();
                    let rid = rid.clone();
                    let pid = pid_for_ontrack.clone();

                    Box::pin(async move {
                        let track_id = track.id();
                        let codec = track.codec();

                        // Add track to room manager so late joiners can find it
                        rm.add_published_track(&rid, &pid, track_id.clone(), codec.clone()).await;

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
                                Ok(sender) => {
                                    tracing::info!(
                                        "Added output track {} (kind: {}, codec: {}) to participant {}",
                                        track_id,
                                        track.kind(),
                                        codec.capability.mime_type,
                                        other_pid
                                    );

                                    // Wait for SSRC and PT to be available (needed for rewriting)
                                    let mut target_ssrc = 0;
                                    let mut target_pt = 0;
                                    // Poll for parameters (simple retry)
                                    for _ in 0..10 {
                                        let params = sender.get_parameters().await;
                                        if let Some(encoding) = params.encodings.first() {
                                            target_ssrc = encoding.ssrc;
                                        }
                                        // Find PT that matches the Publisher's codec MimeType AND (if possible) FMTP parameters
                                        for c in &params.rtp_parameters.codecs {
                                            if c.capability.mime_type.eq_ignore_ascii_case(&codec.capability.mime_type) {
                                                // Basic match.
                                                // If both have sdp_fmtp_line, check equality.
                                                // If mismatch, skip this codec (unless it's the only one).
                                                if codec.capability.sdp_fmtp_line.is_empty() || c.capability.sdp_fmtp_line == codec.capability.sdp_fmtp_line {
                                                     target_pt = c.payload_type;
                                                     break;
                                                }
                                                // If we have a mime match but fmtp mismatch, store as fallback?
                                                if target_pt == 0 {
                                                    target_pt = c.payload_type;
                                                }
                                            }
                                        }

                                        if target_ssrc != 0 && target_pt != 0 {
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
                                            tracing::info!("Track {} for {} negotiated: SSRC {} PT {} (matches {})", track_id, other_pid, target_ssrc, target_pt, codec.capability.mime_type);
                                            break;
                                        }
                                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                                    }
                                    if target_ssrc == 0 {
                                        tracing::warn!("Could not determine SSRC for track {} -> {}", track_id, other_pid);
                                    }
                                    if target_pt == 0 {
                                        tracing::warn!("Could not determine PT for track {} -> {}", track_id, other_pid);
                                    }

                                    // Subscribe this output track to receive packets with SSRC/PT rewriting
                                    mr.add_subscriber(track_id.clone(), output_track, target_ssrc, target_pt).await;
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
                        let pc_for_route = pc.clone();
                        tokio::spawn(async move {
                            mr.route_track(receiver, pc_for_route, None).await;
                        });
                    })
                }));

                // Logging ICE state changes
                let pid_for_ice = pid.clone();
                pc.on_ice_connection_state_change(Box::new(move |state| {
                    tracing::info!("ICE Connection State for {} has changed: {}", pid_for_ice, state);
                    if state == webrtc::ice_transport::ice_connection_state::RTCIceConnectionState::Failed {
                        tracing::warn!("ICE Connection for {} FAILED. Check network/firewall.", pid_for_ice);
                    }
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

                // pc is already Arc
                // let pc = Arc::new(pc);
                room_manager
                    .set_peer_connection(&room_id, &participant_id, pc.clone())
                    .await;

                // Subscribe to existing tracks from other participants
                let existing_tracks = room_manager
                    .get_all_published_tracks_except(&room_id, &participant_id)
                    .await;
                for (other_pid, track_id, codec) in existing_tracks {
                    let output_track = Arc::new(
                        webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP::new(
                            codec.capability.clone(),
                            track_id.clone(),
                            format!("stream-{}", other_pid),
                        )
                    );

                    match pc.add_track(output_track.clone()).await {
                        Ok(sender) => {
                            tracing::info!(
                                "Subscribing new participant {} to existing track {} from {}",
                                participant_id,
                                track_id,
                                other_pid
                            );
                            // Wait for SSRC and PT to be available (needed for rewriting)
                            let mut target_ssrc = 0;
                            let mut target_pt = 0;
                            // Poll for parameters (simple retry)
                            for _ in 0..10 {
                                let params = sender.get_parameters().await;
                                if let Some(encoding) = params.encodings.first() {
                                    target_ssrc = encoding.ssrc;
                                }

                                // Find PT that matches the Publisher's codec MimeType AND (if
                                // possible) FMTP parameters
                                for c in &params.rtp_parameters.codecs {
                                    if c.capability
                                        .mime_type
                                        .eq_ignore_ascii_case(&codec.capability.mime_type)
                                    {
                                        // Basic match.
                                        // If both have sdp_fmtp_line, check equality.
                                        // If mismatch, skip this codec (unless it's the only one).
                                        if codec.capability.sdp_fmtp_line.is_empty()
                                            || c.capability.sdp_fmtp_line
                                                == codec.capability.sdp_fmtp_line
                                        {
                                            target_pt = c.payload_type;
                                            break;
                                        }
                                        // If we have a mime match but fmtp mismatch, store as
                                        // fallback?
                                        // For now, let's strictly prefer exact match or break on
                                        // first match.
                                        // Actually, if we skip here, we might find a better one
                                        // later.
                                        // But if we don't find *any* better one, we should probably
                                        // fall back to this one.
                                        if target_pt == 0 {
                                            target_pt = c.payload_type;
                                        }
                                    }
                                }

                                if target_ssrc != 0 && target_pt != 0 {
                                    tracing::info!(
                                        "Track {} for {} negotiated: SSRC {} PT {} (matches {})",
                                        track_id,
                                        other_pid,
                                        target_ssrc,
                                        target_pt,
                                        codec.capability.mime_type
                                    );
                                    break;
                                }
                                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                            }

                            if target_ssrc == 0 {
                                tracing::warn!(
                                    "Could not determine SSRC for track {} -> {}",
                                    track_id,
                                    other_pid
                                );
                            }

                            media_router
                                .add_subscriber(
                                    track_id.clone(),
                                    output_track,
                                    target_ssrc,
                                    target_pt,
                                )
                                .await;

                            // Spawn a task to forward PLI requests from the subscriber to the
                            // publisher
                            let media_router = media_router.clone();
                            let track_id = track_id.clone();
                            tokio::spawn(async move {
                                while let Ok((packets, _)) = sender.read_rtcp().await {
                                    for packet in packets {
                                        if let Some(_pli) =
                                            packet.as_any().downcast_ref::<PictureLossIndication>()
                                        {
                                            media_router.request_keyframe(&track_id).await;
                                        } else if let Some(_fir) =
                                            packet.as_any().downcast_ref::<FullIntraRequest>()
                                        {
                                            media_router.request_keyframe(&track_id).await;
                                        }
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to subscribe new participant to existing track: {}",
                                e
                            );
                        }
                    }
                }

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
