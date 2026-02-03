# Deployment Guide

This guide explains how to deploy the Rust Video Chat SFU and Signaling servers.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Cloudflared](https://developers.cloudflare.com/cloudflare-one/connections/connect-apps/install-and-setup/installation/) (for tunneling)

## Setup SFU & Signaling Server (Unified)

1.  Navigate to the SFU server directory:
    ```bash
    cd crates/sfu-server
    ```
2.  Run the server:
    ```bash
    cargo run -p video-chat-sfu-server
    ```
    The server listens on `127.0.0.1:8080` and handles both:
    - **HTTP**: Default health check at `/`
    - **WebSocket Signaling**: Signaling endpoint at `/ws`

## Cloudflare Tunnel Configuration

To expose your servers to the public internet securely, use a Cloudflare Tunnel.

1.  Create a `tunnel_config.yml`:
    ```yaml
    tunnel: <YOUR_TUNNEL_ID>
    credentials-file: /root/.cloudflared/<YOUR_TUNNEL_ID>.json

    ingress:
      - hostname: sfu.yourdomain.com
        service: http://localhost:8080
      - service: http_status:404
    ```
2.  Run the tunnel:
    ```bash
    cloudflared tunnel run <YOUR_TUNNEL_NAME>
    ```

## Production Considerations

- **Logging**: Use a log management service or structured logging to a file.
- **Security**: Always use HTTPS via Cloudflare.
- **Resource Limits**: Monitor CPU and memory usage, as video routing can be intensive.
