# Video Chat Signaling Crate

WebRTC signaling protocol for SFU-based video calling.

## Features

- **Protocol Versioning**: Robust protocol version handling with compatibility checks.
- **Message Types**: Strongly typed messages for:
  - Join/Leave
  - Offer/Answer (SDP)
  - ICE Candidates
  - Subscribe/Unsubscribe
- **WebSocket Signaling**: Client implementation for signaling server communication.
- **Validation**: Strict validation for all message types.

## Protocol

This crate implements version `1.0` of the signaling protocol.

### Message Format

```json
{
  "version": "1.0",
  "id": "msg-uuid",
  "timestamp": 1234567890,
  "type": "offer",
  "sdp": {
    "type": "offer",
    "sdp": "v=0..."
  },
  "participant_id": "user-123"
}
```

## Usage

```rust
use video_chat_signaling::{WebSocketSignaling, Message, MessageType};

// Initialize signaling client
let client = WebSocketSignaling::new("wss://signaling.example.com");
client.connect().await?;

// Send a join message
let join_msg = Message::new(
    "msg-1",
    MessageType::Join {
        room_id: "room-1".to_string(),
        participant_id: "user-1".to_string(),
    }
);
client.send(join_msg).await?;
```

## Testing

Run tests for this crate:

```bash
cargo test --package video-chat-signaling
```
