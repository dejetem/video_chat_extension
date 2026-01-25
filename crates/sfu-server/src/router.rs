use std::sync::Arc;
use tracing::info;
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;

pub struct MediaRouter {
    // Current active subscribers for each track
    subscribers: std::sync::Arc<
        tokio::sync::RwLock<
            std::collections::HashMap<String, Vec<std::sync::Arc<TrackLocalStaticRTP>>>,
        >,
    >,
}

impl MediaRouter {
    pub fn new() -> Self {
        Self {
            subscribers: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    pub async fn add_subscriber(
        &self,
        track_id: String,
        output_track: std::sync::Arc<TrackLocalStaticRTP>,
    ) {
        let mut subs = self.subscribers.write().await;
        subs.entry(track_id).or_default().push(output_track);
    }

    pub async fn remove_subscriber(&self, track_id: &str, output_track_id: &str) {
        let mut subs: tokio::sync::RwLockWriteGuard<
            '_,
            HashMap<String, Vec<Arc<TrackLocalStaticRTP>>>,
        > = self.subscribers.write().await;
        if let Some(list) = subs.get_mut(track_id) {
            list.retain(|t: &Arc<TrackLocalStaticRTP>| t.id() != output_track_id);
        }
    }

    pub async fn route_track(
        &self,
        mut receiver: std::sync::Arc<RTCRtpReceiver>,
        output_track: std::sync::Arc<TrackLocalStaticRTP>,
    ) {
        let track_id = receiver.track().await.unwrap().id();
        info!("Starting media routing for track: {}", track_id);

        // Add minimal buffer for packet handling
        let mut buffer = vec![0u8; 1500];

        while let Ok((n, _)) = receiver.read(&mut buffer).await {
            let packet_data = &buffer[..n];

            // Forward packet to the specific output track
            // In a full implementation, we would multicast to all subscribers here
            if let Err(e) = output_track.write(packet_data).await {
                if e.to_string().contains("closed") {
                    break;
                }
                tracing::warn!("Failed to forward RTP packet: {}", e);
            }
        }

        info!("Stopped routing for track: {}", track_id);
    }
}
