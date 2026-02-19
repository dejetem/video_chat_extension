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

                // Explicitly register audio codecs (Opus is the primary audio codec for WebRTC)
                // This is CRITICAL - without this, the server silently drops audio packets!
                m.register_codec(
                    webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecParameters {
                        capability: webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability {
                            mime_type: "audio/opus".to_owned(),
                            clock_rate: 48000,
                            channels: 2,
                            sdp_fmtp_line: "minptime=10;useinbandfec=1".to_owned(),
                            rtcp_feedback: vec![webrtc::rtp_transceiver::RTCPFeedback {
                                typ: "transport-cc".to_owned(),
                                parameter: "".to_owned(),
                            }],
                        },
                        payload_type: 111,
                        ..Default::default()
                    },
                    webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Audio,
                )?;

                // Explicitly register video codecs (VP8 and H264)
                // We need to register these explicitly because mixing manual and default codec
                // registration can lead to unexpected behavior or missing codecs.

                // VP8
                m.register_codec(
                    webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecParameters {
                        capability: webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability {
                            mime_type: "video/VP8".to_owned(),
                            clock_rate: 90000,
                            channels: 0,
                            sdp_fmtp_line: "".to_owned(),
                            rtcp_feedback: vec![
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "goog-remb".to_owned(),
                                    parameter: "".to_owned(),
                                },
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "transport-cc".to_owned(),
                                    parameter: "".to_owned(),
                                },
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "ccm".to_owned(),
                                    parameter: "fir".to_owned(),
                                },
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "nack".to_owned(),
                                    parameter: "".to_owned(),
                                },
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "nack".to_owned(),
                                    parameter: "pli".to_owned(),
                                },
                            ],
                        },
                        payload_type: 96,
                        ..Default::default()
                    },
                    webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Video,
                )?;

                // H264
                m.register_codec(
                    webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecParameters {
                        capability: webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability {
                            mime_type: "video/H264".to_owned(),
                            clock_rate: 90000,
                            channels: 0,
                            sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;\
                                            profile-level-id=42e01f"
                                .to_owned(),
                            rtcp_feedback: vec![
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "goog-remb".to_owned(),
                                    parameter: "".to_owned(),
                                },
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "transport-cc".to_owned(),
                                    parameter: "".to_owned(),
                                },
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "ccm".to_owned(),
                                    parameter: "fir".to_owned(),
                                },
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "nack".to_owned(),
                                    parameter: "".to_owned(),
                                },
                                webrtc::rtp_transceiver::RTCPFeedback {
                                    typ: "nack".to_owned(),
                                    parameter: "pli".to_owned(),
                                },
                            ],
                        },
                        payload_type: 102,
                        ..Default::default()
                    },
                    webrtc::rtp_transceiver::rtp_codec::RTPCodecType::Video,
                )?;

                tracing::info!(
                    "MediaEngine created with Opus, VP8, and H264 codecs explicitly registered"
                );

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

                // Per-PC generation counter for renegotiation debouncing.
                // Each new incoming track increments this counter, sleeps 1.5s,
                // then only renegotiates if no newer track has arrived.
                // This handles audio+video arriving up to ~1s apart without
                // triggering two concurrent renegotiations.
                let renego_gen: Arc<std::sync::atomic::AtomicU64> =
                    Arc::new(std::sync::atomic::AtomicU64::new(0));

                let pid_for_ontrack = pid.clone();
                pc.on_track(Box::new(move |track, receiver, _transceiver| {
                    let mr = mr.clone();
                    let pc = pc_clone.clone();
                    let rm = rm.clone();
                    let rid = rid.clone();
                    let pid = pid_for_ontrack.clone();
                    let renego_gen = renego_gen.clone();

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
                            // We prefix the track ID with the participant ID to ensure the client can always
                            // identify the owner, even if Stream ID signaling (MSID) fails or is inconsistent.
                            // Format: "user-{id}_{original_track_id}"
                            let output_track_id = format!("{}_{}", pid, track_id);

                            let output_track = Arc::new(
                                webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP::new(
                                    codec.capability.clone(),
                                    output_track_id.clone(),
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
                                    // Poll for parameters (increased retries to avoid SSRC=0 race)
                                    for _ in 0..50 {
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
                                    let renego_gen_clone = renego_gen.clone();

                                     tokio::spawn(async move {
                                         use std::sync::atomic::Ordering;
                                         // Increment generation: this task "owns" renegotiation.
                                         // If another track arrives before we fire, it increments
                                         // again and we will see our generation is stale → skip.
                                         let my_gen = renego_gen_clone.fetch_add(1, Ordering::SeqCst) + 1;

                                         // Wait 1.5s — longer than the observed 703ms gap between
                                         // audio and video on_track events — so both tracks are
                                         // added to the PC before one renegotiation covers both.
                                         tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

                                         // Only renegotiate if no newer track has superseded us
                                         if renego_gen_clone.load(Ordering::SeqCst) != my_gen {
                                             tracing::info!(
                                                 "Skipping stale renegotiation for {} (gen {})",
                                                 other_pid_clone,
                                                 my_gen
                                             );
                                             return;
                                         }

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
                                        tracing::error!("ABORTING: Could not determine SSRC for track {} -> {}. Preventing blank video.", track_id, other_pid);
                                        return;
                                    }
                                    if target_pt == 0 {
                                         tracing::error!("ABORTING: Could not determine PT for track {} -> {}. Preventing blank video.", track_id, other_pid);
                                         return;
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
                        let receiver_clone = receiver.clone();
                        let track_id_rtcp = track_id.clone();
                        let pid_rtcp = pid.clone();

                        tokio::spawn(async move {
                            mr.route_track(receiver, pc_for_route, None).await;
                        });

                        // Monitor RTCP packets from the publisher to track PLI/FIR responses
                        tokio::spawn(async move {
                            while let Ok((rtcp_packets, _)) = receiver_clone.read_rtcp().await {
                                for rtcp_packet in rtcp_packets {
                                    // Log all RTCP packet types for debugging
                                    let packet_type = format!("{:?}", rtcp_packet);
                                    if packet_type.contains("PictureLossIndication") {
                                        tracing::info!(
                                            "Received PLI from publisher {} for track {}",
                                            pid_rtcp,
                                            track_id_rtcp
                                        );
                                    } else if packet_type.contains("FullIntraRequest") {
                                        tracing::info!(
                                            "Received FIR from publisher {} for track {}",
                                            pid_rtcp,
                                            track_id_rtcp
                                        );
                                    } else if packet_type.contains("SenderReport") {
                                        tracing::debug!(
                                            "Received Sender Report from publisher {} for track {}",
                                            pid_rtcp,
                                            track_id_rtcp
                                        );
                                    } else {
                                        tracing::debug!(
                                            "Received RTCP from publisher {} for track {}: {}",
                                            pid_rtcp,
                                            track_id_rtcp,
                                            packet_type
                                        );
                                    }
                                }
                            }
                            tracing::warn!(
                                "RTCP monitoring stopped for track {} from publisher {}",
                                track_id_rtcp,
                                pid_rtcp
                            );
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
                    // Prefix the track ID with the participant ID to ensure robust identification
                    // Format: "user-{id}_{original_track_id}"
                    let output_track_id = format!("{}_{}", other_pid, track_id);

                    let output_track = Arc::new(
                        webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP::new(
                            codec.capability.clone(),
                            output_track_id.clone(),
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
                            // Poll for parameters (increased retries to avoid SSRC=0 race)
                            for _ in 0..50 {
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
                                tracing::error!(
                                    "ABORTING TRACK: Could not determine SSRC for track {} -> {}. \
                                     Preventing blank video.",
                                    track_id,
                                    other_pid
                                );
                                continue;
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

        // Sanitize the SDP to remove simulcast-related attributes and fix direction
        // This is a workaround for the webrtc-rs library automatically adding simulcast support,
        // and also converts a=recvonly → a=sendrecv for all media sections so Chrome accepts it.
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

        let mut in_audio_section = false;
        let mut result_lines = Vec::new();

        for line in lines {
            // Track when we enter/exit audio media section
            if line.starts_with("m=audio") {
                in_audio_section = true;
            } else if line.starts_with("m=") {
                in_audio_section = false;
            }

            // Filter out simulcast and RID attributes
            if line.starts_with("a=simulcast:") || line.starts_with("a=rid:") {
                continue;
            }

            // Change recvonly to sendrecv for AUDIO ONLY.
            // For audio: the client sends audio, the server must echo it back (sendrecv)
            // so Chrome doesn't silence the microphone track.
            // For video: keep a=recvonly as-is. This prevents Chrome from firing
            // placeholder ontrack events (which cause Ghost Cards). Real video tracks
            // arrive via renegotiation with proper stream-user-* IDs.
            if in_audio_section && line == "a=recvonly" {
                result_lines.push("a=sendrecv");
            } else {
                result_lines.push(line);
            }
        }

        // Rejoin with \r\n and ensure we end with \r\n if original did
        let result = result_lines.join("\r\n");
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

    // Create a plain renegotiation offer WITHOUT ice_restart.
    // ICE is already connected; we are just adding new tracks to the existing
    // connection. Using ice_restart = true causes "ICE Agent can not be restarted
    // when gathering" if two tracks arrive close together (audio/video gap ~700ms)
    // because both would try to restart ICE simultaneously.
    let offer: RTCSessionDescription = pc.create_offer(None).await?;
    pc.set_local_description(offer.clone()).await?;

    // Sanitize SDP
    let sanitized_sdp = NegotiationManager::sanitize_sdp(&offer.sdp);

    tracing::info!("Renegotiation Offer SDP (Sanitized):\n{}", sanitized_sdp);

    // Send offer to participant
    let offer_msg = Message::new(
        format!(
            "offer-renegotiate-{}",
            chrono::Utc::now().timestamp_millis()
        ),
        MessageType::Offer {
            room_id: room_id.to_string(),
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
