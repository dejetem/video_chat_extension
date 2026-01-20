# Video Chat Media Crate

This crate handles the WebRTC media stack for the video chat extension.

## Features

- **PeerConnectionManager**: Wraps `RtcPeerConnection` and handles SDP negotiation (Offer/Answer).
- **StreamManager**: Manages local and remote `MediaStream` and tracks.
- **DataChannelManager**: Manages `RtcDataChannel` for chat and control messages.
- **IceCandidateHandler**: Handles ICE candidates with correct signaling.
- **TransientStorage**: In-memory storage for chat history.

## Architecture

This crate is designed to compile to WASM and run in the browser (or extension service worker).
It relies on `web-sys` for WebRTC APIs.

## Usage

```rust
use video_chat_media::{PeerConnectionManager, StunConfig};

let config = StunConfig::default();
let pc_manager = PeerConnectionManager::new(&config).expect("Failed to create PC");
```

## Testing

Unit tests run via `cargo test`.
Integration tests require `wasm-pack test --headless --firefox`.
