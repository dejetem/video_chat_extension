use std::sync::Arc;
use tracing::info;
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;

pub struct MediaRouter {
    // Mapping from track ID to list of downstream subscribers
}

impl MediaRouter {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn route_track(
        &self,
        _receiver: Arc<RTCRtpReceiver>,
        _output_track: Arc<TrackLocalStaticRTP>,
    ) {
        info!("Setting up media routing for track");
        // Logic to read from receiver and broadcast to output_track
        // while let Ok((packet, _)) = receiver.read_rtp().await {
        //     output_track.write_rtp(&packet).await.ok();
        // }
    }
}
