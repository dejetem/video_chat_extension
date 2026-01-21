//! Media stream management and WebRTC handling.

pub mod bandwidth;
pub mod data_channel;
pub mod ice_handler;
pub mod message;
pub mod mocks;
pub mod peer_connection;
pub mod quality;
pub mod stream_manager;
pub mod transient_storage;

#[cfg(test)]
mod tests;

pub use bandwidth::BandwidthStats;
pub use data_channel::DataChannelManager;
pub use ice_handler::IceCandidateHandler;
pub use message::DataChannelMessage;
pub use mocks::{MockSfu, SfuConnection};
pub use peer_connection::PeerConnectionManager;
pub use quality::QualityLayer;
pub use stream_manager::StreamManager;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MediaError {
    #[error("WebRTC Error: {0}")]
    WebRtc(String),
    #[error("DataChannel Error: {0}")]
    DataChannel(String),
    #[error("Stream Error: {0}")]
    Stream(String),
    #[error("Serialization Error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Internal Error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, MediaError>;

impl From<MediaError> for video_chat_core::Error {
    fn from(err: MediaError) -> Self {
        video_chat_core::Error::Media(err.to_string())
    }
}
