# WASM Crate

This crate provides the WebAssembly bindings and Chrome Extension integration for the video chat extension.

## Features

- **Chrome API Bindings**: Safe wrappers for `chrome.runtime`, `chrome.tabs`, `chrome.storage`.
- **SQLite Integration**: In-memory SQLite database (via `rusqlite` bundled) for chat logs. 
  - *Note: OPFS persistence will be enabled in future iterations via VFS shim.*
- **JS Interop**: Type-safe conversion between Rust and JavaScript values.

## Build

To build the WASM module for the extension:

```bash
wasm-pack build --target web --release
```

## Testing

Run unit tests (in headless browser):

```bash
wasm-pack test --headless --firefox
```

## Structure

- `src/lib.rs`: Entry point and initialization.
- `src/chrome_api.rs`: `extern "C"` bindings for Chrome APIs.
- `src/sqlite.rs`: Database connection and logic.
- `src/schema.rs`: Database schema definitions.
- `src/js_interop.rs`: JSON/JSValue conversion helpers.
