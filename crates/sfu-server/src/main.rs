use anyhow::Result;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use video_chat_core::config::Config;
use video_chat_signaling::websocket;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Rust Video Chat SFU Server...");

    // Initial dummy configuration
    let config = Config::default();

    // Build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "SFU Server is running" }))
        // Future: WS signaling endpoint
        // .route("/ws", get(websocket_handler))
        ;

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
