# Video Chat Extension

A production-ready Chrome extension for group video calls built with Rust, WebAssembly, and WebRTC.

## Features

- **SFU Architecture**: Scalable Selective Forwarding Unit supporting 100+ participants
- **WebRTC DataChannels**: Low-latency chat messaging without central relay
- **SQLite-WASM**: Local persistent storage using Origin Private File System (OPFS)
- **Cloudflare Tunnel**: Secure external access via `cloudflared`
- **Auto-Delete Messages**: Transient chat logs cleared on call end
- **Admin Interface**: Separate web-based UI with persistent logs
- **Dark Mode UI**: High-contrast off-white on black theme
- **Manifest V3**: Modern Chrome extension architecture with service workers
- **Simulcast Support**: Multiple quality layers for adaptive streaming

## Architecture

This project uses a **Cargo workspace** with multiple crates:

```
video-chat-extension/
├── crates/
│   ├── core/       # Core business logic (platform-agnostic)
│   ├── signaling/  # WebRTC signaling protocol
│   ├── media/      # Media stream & DataChannel management
│   ├── wasm/       # WASM bindings and Chrome API
│   ├── sfu-client/ # SFU client logic
│   └── sfu-server/ # SFU server implementation
├── extension/      # Chrome extension files (HTML/CSS/JS)
├── admin/          # Web-based admin interface
└── tests/          # Integration tests
```

### Key Technologies

- **Rust** Compiled to WASM for browser execution
- **WebRTC** SFU-based video/audio + DataChannels for chat
- **SQLite-WASM** Local database with OPFS
- **STUN/TURN** NAT traversal and relay servers
- **SFU Server** Self-hosted media router for scalable streaming
- **Cloudflare Tunnel** Expose SFU server and signaling

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
| **0** | Project initialization | Complete |
| **1** | Core foundation & SFU signaling | Complete |
| **2** | WASM bridge & SQLite | Complete |
| **3** | Media streams & SFU connection | Complete |
| **4** | SFU client & room management | Complete |
| **5** | UI & admin interface | Pending |
| **6** | SFU server & production | Pending |

See [implementation_plan.md](docs/implementation_plan.md) for detailed stage breakdown.

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with coverage
cargo tarpaulin --out Html --workspace
chmod +x scripts/coverage.sh && ./scripts/coverage.sh

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

# WASM build (for extension)
wasm-pack build crates/wasm --target web --out-dir ../../extension/pkg

# Run benchmarks
cargo bench
```

## Project Status

- [x] **Stage 1**: Core Foundation & WebRTC Signaling
- [x] **Stage 2**: WASM Bridge & Chrome Extension Interface
- [x] **Stage 3**: Media Stream Management & Peer Connections
- [x] **Stage 4**: Rust-to-JS Bridge & UI Utilities
- [x] **Stage 5**: UI Components & Admin Interface
- [x] **Stage 6**: Production Readiness (SFU Server, SQLite Persistence, Telemetry)

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
