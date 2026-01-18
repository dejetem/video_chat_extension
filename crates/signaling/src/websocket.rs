//! WebSocket signaling implementation.

use crate::error::Result;
use crate::messages::Message;
use crate::protocol::Protocol;

/// WebSocket signaling client using web-sys (browser API).
/// Note: This is a placeholder structure for now. In a real WASM implementation,
/// this would wrap a web_sys::WebSocket.
#[derive(Debug)]
pub struct WebSocketSignaling {
    /// Signaling URL
    url: String,
    /// Protocol handler
    protocol: Protocol,
}

impl WebSocketSignaling {
    /// Create a new WebSocket signaling client.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
            protocol: Protocol::new(),
        }
    }

    /// Get the signaling URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Connect to the signaling server (mock implementation).
    pub async fn connect(&self) -> Result<()> {
        // In a real implementation:
        // 1. Create WebSocket connection
        // 2. Set up event handlers
        // 3. Wait for open event
        Ok(())
    }

    /// Send a message (mock implementation).
    pub async fn send(&self, message: Message) -> Result<()> {
        self.protocol.validate_message(&message)?;
        let _json = message.to_json()?;
        // In real implementation: ws.send_with_str(&json)
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::MessageType;

    #[tokio::test]
    async fn test_new_client() {
        let client = WebSocketSignaling::new("ws://localhost:8080");
        assert_eq!(client.url(), "ws://localhost:8080");
    }

    #[tokio::test]
    async fn test_connect() {
        let client = WebSocketSignaling::new("ws://localhost:8080");
        assert!(client.connect().await.is_ok());
    }

    #[tokio::test]
    async fn test_send_valid_message() {
        let client = WebSocketSignaling::new("ws://localhost:8080");
        let msg = Message::new(
            "test-id",
            MessageType::Join {
                room_id: "room1".to_string(),
                participant_id: "user1".to_string(),
            },
        );
        assert!(client.send(msg).await.is_ok());
    }
}
