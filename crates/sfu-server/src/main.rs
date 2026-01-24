use anyhow::Result;
use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use video_chat_core::config::Config;

mod bandwidth;
mod negotiation;
mod recorder;
mod room_manager;
mod router;
mod simulcast;

use crate::recorder::Recorder;
use crate::room_manager::RoomManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Rust Video Chat SFU Server...");

    // Initial configuration
    let _config = Config::default();
    let _room_manager = RoomManager::new();
    let _recorder = Recorder::new();

    // Build our application routes
    let app = Router::new()
        .route("/", get(|| async { "SFU Server is running" }))
        // Future: Unified signaling + SFU websocket
        // .route("/ws", get(signaling_handler))
        ;

    // Run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
