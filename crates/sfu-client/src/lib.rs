use std::sync::Arc;
use video_chat_core::error::{Error, Result};
use video_chat_media::PeerConnectionManager;
use video_chat_signaling::stun_config::StunConfig;
use video_chat_signaling::{Message, MessageType};

pub mod reconnection;
pub use reconnection::ReconnectionManager;

#[cfg(test)]
mod tests;

pub struct SfuClient {
    #[allow(clippy::arc_with_non_send_sync)]
    pub peer_connection: Arc<PeerConnectionManager>,
    pub room_id: String,
    subscribed_streams: Arc<tokio::sync::RwLock<std::collections::HashSet<String>>>,
}

impl SfuClient {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new(room_id: String, stun_config: &StunConfig) -> Result<Self> {
        let pc = PeerConnectionManager::new(stun_config)?;

        Ok(Self {
            peer_connection: Arc::new(pc),
            room_id,
            subscribed_streams: Arc::new(
                tokio::sync::RwLock::new(std::collections::HashSet::new()),
            ),
        })
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
        let message = Message {
            message_type: MessageType::Subscribe,
            room_id: self.room_id.clone(),
            participant_id: participant_id.to_string(),
            payload: serde_json::json!({
                "action": "subscribe",
                "target_participant": participant_id
            }),
        };

        // In a real implementation, this would be sent via WebSocket to the SFU
        // For now, we log the intent and maintain local state
        log::info!("Would send subscribe message to SFU: {:?}", message);

        // The SFU would respond by adding this client to the forwarding list
        // and the peer connection would receive the remote track

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
        let message = Message {
            message_type: MessageType::Unsubscribe,
            room_id: self.room_id.clone(),
            participant_id: participant_id.to_string(),
            payload: serde_json::json!({
                "action": "unsubscribe",
                "target_participant": participant_id
            }),
        };

        // In a real implementation, this would be sent via WebSocket to the SFU
        log::info!("Would send unsubscribe message to SFU: {:?}", message);

        // The SFU would respond by removing this client from the forwarding list

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
}
