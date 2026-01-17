# Video Chat Extension

A production-ready Chrome extension for group video calls built with Rust, WebAssembly, and WebRTC.

## Features

- **P2P Mesh Architecture**: Direct peer-to-peer connections for up to ~10 participants
- **WebRTC DataChannels**: Low-latency chat messaging without central relay
- **SQLite-WASM**: Local persistent storage using Origin Private File System (OPFS)
- **Cloudflare Tunnel**: Secure external access via `cloudflared`
- **Auto-Delete Messages**: Transient chat logs cleared on call end
- **Admin Interface**: Separate web-based UI with persistent logs
- **Dark Mode UI**: High-contrast off-white on black theme
- **Manifest V3**: Modern Chrome extension architecture with service workers

## Architecture

This project uses a **Cargo workspace** with multiple crates:

```
video-chat-extension/
├── crates/
│   ├── core/       # Core business logic (platform-agnostic)
│   ├── signaling/  # WebRTC signaling protocol
│   ├── media/      # Media stream & DataChannel management
│   ├── wasm/       # WASM bindings and Chrome API
│   └── server/     # Local HTTP server for signaling
├── extension/      # Chrome extension files (HTML/CSS/JS)
├── admin/          # Web-based admin interface
└── tests/          # Integration tests
```

### Key Technologies

- **Rust** Compiled to WASM for browser execution
- **WebRTC** P2P video/audio + DataChannels for chat
- **SQLite-WASM** Local database with OPFS
- **STUN** NAT traversal (Google's stun.l.google.com:19302)
- **Cloudflare Tunnel** Expose local signaling server

## Quick Start

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- wasm-pack (`cargo install wasm-pack`)
- Node.js 18+ (for extension development)
- cloudflared (for tunnel setup)

### Development Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd video_chat_extension
   ```

2. **Build the project**
   ```bash
   cargo build --workspace
   ```

3. **Run tests**
   ```bash
   cargo test --workspace
   ```

4. **Build WASM module** (after Stage 2)
   ```bash
   cd crates/wasm
   wasm-pack build --target web
   ```

5. **Load extension in Chrome**
   - Navigate to `chrome://extensions/`
   - Enable "Developer mode"
   - Click "Load unpacked"
   - Select the `extension/` directory

## Development Stages

This project is developed in **7 stages**, each with complete tests:

| Stage | Focus | Status |
|-------|-------|--------|
| **0** | Project initialization | In Progress |
| **1** | Core foundation & signaling | Pending |
| **2** | WASM bridge & SQLite | Pending |
| **3** | Media streams & DataChannels | Pending |
| **4** | P2P mesh & group calls | Pending |
| **5** | UI & admin interface | Pending |
| **6** | Cloudflare Tunnel & production | Pending |

See [implementation_plan.md](docs/implementation_plan.md) for detailed stage breakdown.

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with coverage
cargo tarpaulin --out Html --workspace

# Run clippy
cargo clippy --all-targets --all-features --workspace -- -D warnings

# Format code
cargo fmt --all

# WASM tests (after Stage 2)
cd crates/wasm && wasm-pack test --headless --firefox
```

## Build Commands

```bash
# Development build
cargo build --workspace

# Release build (optimized)
cargo build --workspace --release

# WASM build
cd crates/wasm && wasm-pack build --release --target web

# Run benchmarks
cargo bench
```

## Documentation

- [Implementation Plan](docs/implementation_plan.md) - Detailed development roadmap
- [Architecture](docs/architecture.md) - System design and P2P mesh topology
- [API Documentation](docs/api.md) - Rust API reference
- [Deployment Guide](docs/deployment.md) - Production deployment steps
- [Cloudflare Tunnel Setup](docs/cloudflare-tunnel.md) - Tunnel configuration

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

### Code Standards

- **Formatting**: `cargo fmt` (enforced by CI)
- **Linting**: `cargo clippy` with pedantic warnings
- **Testing**: >80% code coverage required
- **Documentation**: All public APIs must be documented

## License

This project is licensed under MIT OR Apache-2.0.

## Security

- No unsafe Rust code allowed
- All dependencies audited with `cargo audit`
- Transient message storage (auto-delete on call end)
- Optional persistent logs for admin only

## Browser Support

- Chrome 90+ (Manifest V3 required)
- Edge 90+ (Chromium-based)

## Support

For issues and questions, please open a GitHub issue.

---

**Built with love using Rust and WebAssembly**
