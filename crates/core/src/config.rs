//! Configuration management for the video chat extension.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Main configuration for the video chat extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// STUN server URLs
    pub stun_servers: Vec<String>,

    /// TURN server URLs (optional)
    pub turn_servers: Vec<TurnServer>,

    /// SFU server URL
    pub sfu_server_url: String,

    /// Signaling server URL
    pub signaling_server_url: String,

    /// Maximum number of participants
    pub max_participants: usize,

    /// Enable auto-delete for chat messages
    pub auto_delete_messages: bool,

    /// Admin mode (enables persistent logs)
    pub admin_mode: bool,
}

/// TURN server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnServer {
    /// TURN server URL
    pub url: String,

    /// Username for authentication
    pub username: Option<String>,

    /// Credential for authentication
    pub credential: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            stun_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            turn_servers: Vec::new(),
            sfu_server_url: "ws://localhost:8080".to_string(),
            signaling_server_url: "ws://localhost:8081".to_string(),
            max_participants: 100,
            auto_delete_messages: true,
            admin_mode: false,
        }
    }
}

impl Config {
    /// Create a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from JSON string.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Error::from)
    }

    /// Serialize configuration to JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Error::from)
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<()> {
        if self.stun_servers.is_empty() && self.turn_servers.is_empty() {
            return Err(Error::config(
                "At least one STUN or TURN server must be configured",
            ));
        }

        if self.sfu_server_url.is_empty() {
            return Err(Error::config("SFU server URL cannot be empty"));
        }

        if self.signaling_server_url.is_empty() {
            return Err(Error::config("Signaling server URL cannot be empty"));
        }

        if self.max_participants == 0 {
            return Err(Error::config("Max participants must be greater than 0"));
        }

        Ok(())
    }

    /// Add a STUN server.
    pub fn add_stun_server(&mut self, url: impl Into<String>) {
        self.stun_servers.push(url.into());
    }

    /// Add a TURN server.
    pub fn add_turn_server(&mut self, server: TurnServer) {
        self.turn_servers.push(server);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.stun_servers.is_empty());
        assert_eq!(config.max_participants, 100);
        assert!(config.auto_delete_messages);
        assert!(!config.admin_mode);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // Test empty STUN servers (should fail if no TURN servers)
        config.stun_servers.clear();
        assert!(config.validate().is_err());

        // Add TURN server and try again (should pass)
        config.add_turn_server(TurnServer {
            url: "turn:server.com".to_string(),
            username: None,
            credential: None,
        });
        assert!(config.validate().is_ok());

        // Test empty SFU URL
        config.sfu_server_url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = config.to_json().unwrap();
        let deserialized = Config::from_json(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_add_servers() {
        let mut config = Config::default();
        let initial_stun_count = config.stun_servers.len();

        config.add_stun_server("stun:custom.server.com:3478");
        assert_eq!(config.stun_servers.len(), initial_stun_count + 1);

        config.add_turn_server(TurnServer {
            url: "turn:turn.server.com:3478".to_string(),
            username: Some("user".to_string()),
            credential: Some("pass".to_string()),
        });
        assert_eq!(config.turn_servers.len(), 1);
    }

    #[test]
    fn test_max_participants_validation() {
        let mut config = Config::default();
        config.max_participants = 0;
        assert!(config.validate().is_err());

        config.max_participants = 1;
        assert!(config.validate().is_ok());
    }
}
