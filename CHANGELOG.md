# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2026-01-23

### Added
- **SFU Server**: High-performance Rust-based media router using `webrtc-rs` and `axum`.
- **SQLite WASM**: Real persistent storage for chat logs and room metadata in the Chrome extension.
- **Production Telemetry**: Integrated logging, performance, and error tracking system in `crates/core`.
- **System Documentation**: Comprehensive guides for architecture, deployment, API, and setup.
- **Media Crate**: Shared logic for WebRTC peer connection management across client and server.
- **WASM Bridge**: Robust bridge between Rust logic and Vanilla JS extension UI.

### Fixed
- Build errors on macOS related to `sqlite-wasm-rs` by implementing platform-specific mocks.
- Proper handling of private fields in `webrtc-rs` structs.
- Duplicate module definitions and redundant imports in the `sfu-server`.

### Changed
- Refactored `MediaRouter` and `RoomManager` for better scalability on the server-side.
- Updated `webrtc` dependencies to version `0.11` for improved stability.

## [0.5.0] - 2026-01-20
- Initial internal release of the P2P prototype.
- Basic signaling server implementation.
- Core UI components for the Chrome extension popup.
