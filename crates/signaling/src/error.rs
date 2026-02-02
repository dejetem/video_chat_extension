//! Signaling-specific error types.

use thiserror::Error;

/// Result type alias for signaling operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Signaling error types.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Unsupported protocol version
    #[error("Unsupported protocol version: {0}")]
    UnsupportedVersion(String),

    /// Invalid SDP
    #[error("Invalid SDP: {0}")]
    InvalidSdp(String),

    /// Invalid ICE candidate
    #[error("Invalid ICE candidate: {0}")]
    InvalidIceCandidate(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Network or WebSocket error
    #[error("Network error: {0}")]
    Network(String),

    /// Core error
    #[error(transparent)]
    Core(#[from] video_chat_core::Error),
}

impl Error {
    /// Create an invalid message error.
    pub fn invalid_message(msg: impl Into<String>) -> Self {
        Self::InvalidMessage(msg.into())
    }

    /// Create an unsupported version error.
    pub fn unsupported_version(version: impl Into<String>) -> Self {
        Self::UnsupportedVersion(version.into())
    }

    /// Create an invalid SDP error.
    pub fn invalid_sdp(msg: impl Into<String>) -> Self {
        Self::InvalidSdp(msg.into())
    }

    /// Create an invalid ICE candidate error.
    pub fn invalid_ice_candidate(msg: impl Into<String>) -> Self {
        Self::InvalidIceCandidate(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::invalid_message("test");
        assert!(matches!(err, Error::InvalidMessage(_)));
        assert_eq!(err.to_string(), "Invalid message format: test");
    }

    #[test]
    fn test_error_from_serde() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid");
        let err: Error = json_err.unwrap_err().into();
        assert!(matches!(err, Error::Serialization(_)));
    }
}
