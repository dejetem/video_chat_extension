# SFU Setup Guide

The Selective Forwarding Unit (SFU) is the backbone of multi-party video conferencing in this project.

## Server Requirements

- **CPU**: 2+ cores for production (forwarding can be CPU-intensive).
- **Network**: Low-latency, high-bandwidth connection (100Mbps+ recommended).
- **Operating System**: Linux or macOS (Rust-compatible).

## Installation

1.  **Clone the Repository**:
    ```bash
    git clone https://github.com/dejetem/video_chat_extension.git
    cd video_chat_extension
    ```
2.  **Build the SFU**:
    ```bash
    cargo build -p video-chat-sfu-server --release
    ```

## Starting the Server

The SFU requires the signaling server to be running or configured correctly in the client.

```bash
# Run the SFU
./target/release/video-chat-sfu-server
```

## Configuration

The SFU server currently uses environment-based configuration (via `video-chat-core`).

| Environment Variable | Description | Default |
| :--- | :--- | :--- |
| `SFU_PORT` | Port for the SFU to listen on | `3000` |
| `SFU_LOG_LEVEL` | Logging level (info, debug, trace) | `info` |

## WebRTC Specifics

- **UDP Ports**: The SFU uses UDP for media transport. Ensure your firewall allows incoming UDP traffic on the ports allocated by the WebRTC stack (usually a dynamic range).
- **ICE Servers**: In production, you should configure STUN/TURN servers in the `crates/core/src/config.rs` to help bypass symmetric NATs.

## Monitoring

The SFU includes basic telemetry. Monitor the logs for:
- "Starting media routing for track": Indicates a new stream is being forwarded.
- "Creating new room": Indicates a participant started a new session.
