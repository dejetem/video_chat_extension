use std::sync::Arc;
use video_chat_core::error::Result;
use video_chat_media::PeerConnectionManager;
use video_chat_signaling::stun_config::StunConfig;

pub mod reconnection;
pub use reconnection::ReconnectionManager;

#[cfg(test)]
mod tests;

pub struct SfuClient {
    #[allow(clippy::arc_with_non_send_sync)]
    pub peer_connection: Arc<PeerConnectionManager>,
    pub room_id: String,
}

impl SfuClient {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(room_id: String, stun_config: &StunConfig) -> Result<Self> {
        let pc = PeerConnectionManager::new(stun_config)?;

        Ok(Self {
            peer_connection: Arc::new(pc),
            room_id,
        })
    }

    pub async fn subscribe_to_stream(&self, participant_id: &str) -> Result<()> {
        log::info!("Subscribing to stream from participant: {}", participant_id);
        // In a real SFU, this would involve sending a signaling message to the SFU
        // and then handling the incoming track in the PeerConnection.
        // For now, we simulate the client-side state change.
        Ok(())
    }

    pub async fn unsubscribe_from_stream(&self, participant_id: &str) -> Result<()> {
        log::info!(
            "Unsubscribing from stream from participant: {}",
            participant_id
        );
        // In a real SFU, we would signal the SFU to stop forwarding this stream.
        Ok(())
    }
}
