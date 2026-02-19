use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};
use webrtc::util::Marshal;

use crate::recorder::Recorder;

type Subscribers = Arc<RwLock<HashMap<String, Vec<(Arc<TrackLocalStaticRTP>, u32, u8)>>>>;

#[derive(Clone)]
pub struct MediaRouter {
    // Current active subscribers for each track: (OutputTrack, TargetSSRC, TargetPT)
    subscribers: Subscribers,
    // Channels to signal PLI requests to the publisher handling task
    pli_senders: Arc<RwLock<HashMap<String, mpsc::Sender<()>>>>,
}

impl Default for MediaRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl MediaRouter {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            pli_senders: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_subscriber(
        &self,
        track_id: String,
        output_track: Arc<TrackLocalStaticRTP>,
        target_ssrc: u32,
        target_pt: u8,
    ) {
        let mut subs = self.subscribers.write().await;
        subs.entry(track_id.clone())
            .or_default()
            .push((output_track, target_ssrc, target_pt));
        drop(subs); // Release lock to avoid overlap with request_keyframe logic

        // Request a keyframe immediately and retry a few times to ensure it's received
        // This is critical for new subscribers to start getting video (need I-frame)
        let _self_clone = self.clone(); // In a real implementation you'd need to clone the Arc<Self> or similar...
                                        // Wait, MediaRouter is likely Arc-wrapped or cheap to clone?
                                        // Actually, MediaRouter struct fields are Arcs, but the struct itself isn't Clone by
                                        // default unless derived. Let's implement Clone for
                                        // MediaRouter or just pass the sender logic.

        let track_id_clone = track_id.clone();
        let senders = self.pli_senders.clone();

        tokio::spawn(async move {
            for i in 0..5 {
                let s = senders.read().await;
                if let Some(sender) = s.get(&track_id_clone) {
                    let _ = sender.try_send(());
                    info!(
                        "Requested keyframe for track {} (attempt {})",
                        track_id_clone,
                        i + 1
                    );
                }
                drop(s);
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        });
    }

    pub async fn request_keyframe(&self, track_id: &str) {
        let senders = self.pli_senders.read().await;
        if let Some(sender) = senders.get(track_id) {
            // Use try_send to avoid waiting/blocking if channel is full
            match sender.try_send(()) {
                Ok(_) => info!("Requested keyframe for track {}", track_id),
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    info!(
                        "Keyframe request for track {} skipped (channel full)",
                        track_id
                    )
                }
                Err(_) => warn!("Keyframe request for track {} failed", track_id),
            }
        } else {
            // This is the race condition: subscriber added before route_track registers pli_sender
            warn!(
                "Could not request keyframe for track {}: No PLI sender registered yet",
                track_id
            );
        }
    }

    /// Select layer based on bandwidth (integration point for simulcast)
    pub fn select_layer(&self, available_bandwidth: u32) -> u8 {
        let available_layers = vec![0, 1, 2]; // Standard layers
        crate::simulcast::SimulcastManager::select_layer(&available_layers, available_bandwidth)
    }

    pub async fn remove_subscriber(&self, track_id: &str, output_track_id: &str) {
        let mut subs = self.subscribers.write().await;
        if let Some(list) = subs.get_mut(track_id) {
            list.retain(|(t, _, _)| t.id() != output_track_id);
        }
    }

    pub async fn route_track(
        &self,
        receiver: Arc<RTCRtpReceiver>,
        pc: Arc<RTCPeerConnection>,
        recorder: Option<Arc<Recorder>>,
    ) {
        let tracks = receiver.tracks().await;

        if let Some(track) = tracks.first() {
            let track_id = track.id();
            let ssrc = track.ssrc();
            info!(
                "Starting multi-subscriber media routing for track: {}",
                track_id
            );

            // Register PLI handler (Increased buffer size to avoid drops)
            let (pli_tx, mut pli_rx) = mpsc::channel::<()>(10);
            self.pli_senders
                .write()
                .await
                .insert(track_id.clone(), pli_tx.clone());

            // Spawn PLI handler task
            let pc_clone = pc.clone();
            let track_id_clone = track_id.clone();
            tokio::spawn(async move {
                while pli_rx.recv().await.is_some() {
                    info!(
                        "Sending PLI (Keyframe Request) to publisher for track {}",
                        track_id_clone
                    );
                    let pli = PictureLossIndication {
                        sender_ssrc: 0,
                        media_ssrc: ssrc,
                    };
                    if let Err(e) = pc_clone.write_rtcp(&[Box::new(pli)]).await {
                        warn!("Failed to write PLI for track {}: {}", track_id_clone, e);
                    }
                }
            });

            // Trigger an INITIAL keyframe request now that the handler is ready
            // This fixes the race where early subscribers' requests were lost
            let _ = pli_tx.send(()).await;
            info!("Triggered initial keyframe request for track {}", track_id);

            // Start recording if recorder is provided
            if let Some(rec) = &recorder {
                let _ = rec.start_recording(&track_id).await;
            }

            // Read RTP packets from the remote track
            while let Ok((packet, _)) = track.read_rtp().await {
                // Record the packet
                if let Some(rec) = &recorder {
                    let mut buf = Vec::new();
                    if packet.marshal_to(&mut buf).is_ok() {
                        rec.record_packet(&track_id, &buf).await;
                    }
                }

                // Log every 100 packets to confirm flow
                if packet.header.sequence_number % 100 == 0 {
                    info!(
                        "Routing packet seq={} for track {}",
                        packet.header.sequence_number, track_id
                    );
                }

                let subs = self.subscribers.read().await;
                if let Some(output_tracks) = subs.get(&track_id) {
                    for (output_track, target_ssrc, target_pt) in output_tracks {
                        // Forward packet with rewritten SSRC and Payload Type
                        let mut packet_clone = packet.clone();
                        let _old_ssrc = packet_clone.header.ssrc;
                        let _old_pt = packet_clone.header.payload_type;

                        packet_clone.header.ssrc = *target_ssrc;
                        packet_clone.header.payload_type = *target_pt;

                        // CRITICAL FIX: Strip header extensions (like MID/RID)
                        // The publisher's extensions don't match the subscriber's SDP
                        // This was causing "Failed to set remote description" errors
                        packet_clone.header.extensions.clear();
                        packet_clone.header.extension = false;

                        if let Err(e) = output_track.write_rtp(&packet_clone).await {
                            if !e.to_string().contains("closed") {
                                warn!("Failed to forward RTP packet to subscriber: {}", e);
                            }
                        }
                    }
                }
            }
            info!("Stopped routing for track: {}", track_id);

            // Cleanup PLI sender
            self.pli_senders.write().await.remove(&track_id);
        } else {
            warn!("No track found in receiver to route");
        }
    }
}
