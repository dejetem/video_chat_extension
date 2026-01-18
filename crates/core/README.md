# Video Chat Core Crate

Platform-agnostic core functionality for the Video Chat Extension.

## Features

- **Configuration Management**: Type-safe configuration for signaling, SFU, and STUN/TURN servers.
- **Error Handling**: Comprehensive error types using `thiserror`.
- **Common Types**: Shared data structures used across the workspace.

## Usage

```rust
use video_chat_core::{Config, Result};

// Use default configuration (STUN servers, etc.)
let config = Config::default();

// Or create custom configuration
let mut config = Config::new();
config.add_stun_server("stun:custom.server:3478");

// Validate configuration
config.validate()?;
```

## Testing

Run tests for this crate:

```bash
cargo test --package video-chat-core
```
