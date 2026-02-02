use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::{TrackLocal, TrackLocalWriter};
use webrtc::util::Marshal;

use crate::recorder::Recorder;

pub struct MediaRouter {
    // Current active subscribers for each track
    subscribers: Arc<RwLock<HashMap<String, Vec<Arc<TrackLocalStaticRTP>>>>>,
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
        }
    }

    pub async fn add_subscriber(&self, track_id: String, output_track: Arc<TrackLocalStaticRTP>) {
        let mut subs = self.subscribers.write().await;
        subs.entry(track_id).or_default().push(output_track);
    }

    /// Select layer based on bandwidth (integration point for simulcast)
    pub fn select_layer(&self, available_bandwidth: u32) -> u8 {
        let available_layers = vec![0, 1, 2]; // Standard layers
        crate::simulcast::SimulcastManager::select_layer(&available_layers, available_bandwidth)
    }

    pub async fn remove_subscriber(&self, track_id: &str, output_track_id: &str) {
        let mut subs: tokio::sync::RwLockWriteGuard<
            '_,
            HashMap<String, Vec<Arc<TrackLocalStaticRTP>>>,
        > = self.subscribers.write().await;
        if let Some(list) = subs.get_mut(track_id) {
            list.retain(|t| t.id() != output_track_id);
        }
    }

    pub async fn route_track(
        &self,
        receiver: Arc<RTCRtpReceiver>,
        recorder: Option<Arc<Recorder>>,
    ) {
        let tracks = receiver.tracks().await;

        if let Some(track) = tracks.first() {
            let track_id = track.id();
            info!(
                "Starting multi-subscriber media routing for track: {}",
                track_id
            );

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

                let subs = self.subscribers.read().await;
                if let Some(output_tracks) = subs.get(&track_id) {
                    for output_track in output_tracks {
                        if let Err(e) = output_track.write_rtp(&packet).await {
                            if !e.to_string().contains("closed") {
                                warn!("Failed to forward RTP packet to subscriber: {}", e);
                            }
                        }
                    }
                }
            }
            info!("Stopped routing for track: {}", track_id);
        } else {
            warn!("No track found in receiver to route");
        }
    }
}
