# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Changed
- Signaling URL now uses the page's own host/protocol (supports Cloudflare tunnel).

---

## [1.1.0] - 2026-02-19

### Fixed
- **ICE Renegotiation Race Condition** (`ICE Agent can not be restarted when gathering`):
  Audio and video tracks from a joining participant were delivered ~700 ms apart by
  `webrtc-rs`, causing two concurrent renegotiation offers. The second offer failed because
  ICE was still gathering from the first. Fixed by:
  1. Removing `ice_restart: true` from renegotiation offers — ICE is already `connected`
     when adding new tracks and a restart is not required.
  2. Replacing the naive 200 ms debounce with a per-PC `AtomicU64` generation counter
     (1.5 s window). Only the latest `on_track` generation fires `create_and_send_offer`,
     ensuring audio + video are covered by a single renegotiation.
- **Ghost Card (Placeholder Participant)**: The first joiner's browser received an `ontrack`
  event from the server's initial Answer audio placeholder transceiver with a UUID stream
  ID. Fixed by dropping `ontrack` events in the WASM client where the participant ID cannot
  be resolved (stream ID does not start with `stream-` and track ID has no `user-` prefix).
- **SDP Parsing Error** (`Invalid SDP line`): Removed `inject_msid_attributes` which was
  corrupting SDP line endings (`\r\n` → `\n`), causing Chrome to reject
  `setRemoteDescription`.
- **SDP Direction Mismatch**: `sanitize_sdp` now converts `a=recvonly` → `a=sendrecv` for
  the **audio** media section only. Video retains `a=recvonly` in the initial Answer,
  preventing premature video `ontrack` events before real tracks arrive via renegotiation.
- Fixed pre-existing test errors in `signaling/src/messages.rs` where `MessageType::Offer`
  test constructors were missing the required `room_id` field.

### Changed
- Renegotiation offers no longer restart ICE; they reuse the existing connected ICE
  transport for efficiency.
- `sanitize_sdp` tracks audio and video `m=` sections separately instead of applying the
  same direction rewrite to all sections.

---

## [1.0.2] - 2026-02-10

### Fixed
- **Blank Video**: Resolved blank video on subscriber side caused by incorrect codec
  payload type mapping. SSRC and PT are now correctly negotiated per track.
- **Audio Codec Mismatch**: SFU answer now correctly selects Opus payload type 111
  and injects proper SSRC/MSID attributes into the Answer SDP.
- **Duplicate Video Elements**: Fixed JavaScript race condition that rendered two video
  cards for the same participant when `ontrack` fired twice.
- Code quality issues: resolved Clippy warnings across the workspace; fixed test
  compilation failures introduced by the real-SFU refactor.

---

## [1.0.1] - 2026-02-02

### Added
- **TURN Server Support**: Load ICE server configuration (1 STUN + 4 TURN servers)
  from environment variables (`TURN_URL_1`–`TURN_URL_4`). Falls back to default STUN
  if env vars are absent.
- `.example.env` template for ICE server configuration.
- **WebSocket Keepalive**: Ping/pong heartbeat every 30 s with a 60 s timeout to detect
  dead connections; added `Pong` message type to the signaling protocol.
- Unique participant ID generation (`user-{timestamp}`) per client session.

### Fixed
- Chat message attribution: messages now display as **You** for the sender and
  **Participant** for others. WASM layer sends `sender_id`; JS compares it with the
  local `participant_id`.
- Participant label shown incorrectly as "Participant" for both parties.
- Resolved `Failed to handle offer: invalid turn server credentials` error.
- Unused imports and variables cleaned up with `cargo fix`.

### Changed
- WASM layer parses `sender:text` format from chat events for proper attribution.

---

## [1.0.0] - 2026-01-25

### Added
- **SFU Server**: High-performance Rust-based Selective Forwarding Unit using
  `webrtc-rs` and `axum`. Handles offer/answer negotiation, track routing, and
  room management.
- **SQLite WASM**: Real persistent storage for chat logs and room metadata in the
  Chrome extension via `sqlite-wasm-rs`.
- **Production Telemetry**: Integrated logging, metrics, and error tracking in
  `crates/core`.
- **System Documentation**: Comprehensive architecture, deployment, API, SFU setup,
  and user guides.
- **Benchmarks**: SFU server, client, and WASM performance benchmarks.
- **Cloudflare Tunnel** configuration for public access without port forwarding.
- `CHANGELOG.md` introduced.

### Fixed
- Build errors on macOS related to `sqlite-wasm-rs` with platform-specific mocks.
- Proper handling of private fields in `webrtc-rs` structs.
- Duplicate module definitions and redundant imports in `sfu-server`.

### Changed
- `MediaRouter` and `RoomManager` refactored for better scalability.
- `webrtc` dependency updated to `0.11` for improved stability.
- WASM bundle optimised (`opt-level=z`, LTO enabled).

---

## [0.9.0] - 2026-01-25

### Added
- Comprehensive test suite: 45 tests passing across the workspace.
  - Platform-agnostic reconnection manager tests for `sfu-client`.
  - 6 room manager tests for `sfu-server`.
  - Signaling integration tests (`tests/signaling_integration_tests.rs`).
  - WASM integration tests (`tests/wasm_integration_tests.rs`).
- Deployment checklist artifact.

---

## [0.8.0] - 2026-01-22

### Added
- **Stage 5 — UI**: Full participant video grid, admin interface, and expanded WASM API.
- Chrome extension popup UI, icons, and build assets for sideloading.
- Screen share capability stub.

### Fixed
- Clippy lint in `sfu-client` stage-4 code.
- `allow(dead_code)` for `from_js_value` utility in WASM crate.

---

## [0.7.0] - 2026-01-21

### Added
- **Stage 4 — SFU Client**: Full `SfuClient` implementation with room management,
  reconnection manager, and >80 % test coverage.

### Changed
- README updated and Stage 4 artifacts archived.

---

## [0.6.0] - 2026-01-20

### Added
- **Stage 3 — Media**: Media stream management, simulcast support, and mock-SFU
  integration test suite.
- Media integration tests finalized.

### Fixed
- Clippy lints and WASM build failures in CI for stage-3 code.

---

## [0.5.0] - 2026-01-19

### Added
- **Stage 2 — WASM Bridge**: WASM module with Chrome API bindings, extension
  Manifest V3, service worker, SQLite-WASM integration, and JS interop utilities.
- WASM integration tests (`crates/wasm/tests/integration.rs`).

### Fixed
- CI WASM build failure: replaced deprecated `JsStatic` with `thread_local_v2`
  in `chrome_api.rs`; removed unused `FileSystem` web-sys feature.
- Disabled `wasm-opt` in `Cargo.toml` metadata to fix CI opt-level 1 failures.
- Clippy lints and rustfmt issues: `Display`/`FromStr` for `ProtocolVersion`,
  removed placeholder `main` functions from library crates.
- Unused imports in `sqlite.rs` and deprecated constants in `chrome_api.rs`.

---

## [0.4.0] - 2026-01-18

### Added
- **Stage 1 — Core + Signaling**:
  - `crates/core`: error types (`thiserror`), configuration management,
    STUN/TURN config, SFU URL handling.
  - `crates/signaling`: versioned signaling protocol, message types
    (`Join`, `Offer`, `Answer`, `IceCandidate`, `Subscribe`), mock WebSocket
    client, and unit tests.
  - Integration tests for the signaling handshake flow.

### Changed
- Architecture changed from **P2P mesh** to **SFU** for scalability to 100+
  participants, lower client bandwidth, and server-side quality adaptation.

---

## [0.1.0] - 2026-01-18

### Added
- Initialized Cargo workspace with `rustfmt` and `clippy` configuration.
- CI/CD pipeline: test, lint, format, and coverage jobs.
- MIT license.
- Project README, CONTRIBUTING guide, and pre-commit quality hook.
- WASM build optimization configuration.
