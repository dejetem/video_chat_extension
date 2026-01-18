//! Error types for the video chat extension.

use thiserror::Error;

/// Result type alias using our custom Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the video chat extension.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid state error
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// WebRTC error
    #[error("WebRTC error: {0}")]
    WebRTC(String),

    /// Signaling error
    #[error("Signaling error: {0}")]
    Signaling(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Generic error with context
    #[error("Error: {0}")]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Create a configuration error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create an invalid state error.
    pub fn invalid_state(msg: impl Into<String>) -> Self {
        Self::InvalidState(msg.into())
    }

    /// Create a network error.
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a WebRTC error.
    pub fn webrtc(msg: impl Into<String>) -> Self {
        Self::WebRTC(msg.into())
    }

    /// Create a signaling error.
    pub fn signaling(msg: impl Into<String>) -> Self {
        Self::Signaling(msg.into())
    }

    /// Create a database error.
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = Error::config("test config error");
        assert!(matches!(err, Error::Config(_)));
        assert_eq!(err.to_string(), "Configuration error: test config error");
    }

    #[test]
    fn test_error_from_serde() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json");
        assert!(json_err.is_err());

        let err: Error = json_err.unwrap_err().into();
        assert!(matches!(err, Error::Serialization(_)));
    }

    #[test]
    fn test_all_error_variants() {
        let errors = vec![
            Error::config("config"),
            Error::invalid_state("state"),
            Error::network("network"),
            Error::webrtc("webrtc"),
            Error::signaling("signaling"),
            Error::database("database"),
        ];

        for err in errors {
            assert!(!err.to_string().is_empty());
        }
    }
}
