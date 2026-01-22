# Rust Video Chat - Chrome Extension

This extension provides a secure P2P video chat interface powered by Rust and WASM.

## Features
- **High-Contrast Dark Mode**: Designed for professional use.
- **WASM-Powered**: WebRTC logic and signaling handled by Rust.
- **Chat**: Real-time messaging via DataChannels.
- **Media Controls**: Toggle mic/camera and share screen.

## Installation for Developers
1. Ensure you have `wasm-pack` installed.
2. Build the WASM module:
   ```bash
   wasm-pack build crates/wasm --target web --out-dir ../../extension/pkg
   ```
3. Open Brave/Chrome and go to `brave://extensions`.
4. Enable "Developer mode".
5. Click "Load unpacked" and select this directory.

## Architecture
- `background.js`: Service worker that initializes the WASM module and persists connection state.
- `popup.html`: The main user interface.
- `content/`: Scripts injected into pages for overlays.
- `pkg/`: compiled WASM assets.
