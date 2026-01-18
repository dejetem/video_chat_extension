//! Protocol versioning and handling.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::messages::Message;

/// Protocol version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolVersion {
    /// Major version
    pub major: u16,
    /// Minor version
    pub minor: u16,
}

impl ProtocolVersion {
    /// Create a new protocol version.
    pub fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    /// Current protocol version (1.0).
    pub fn current() -> Self {
        Self::new(1, 0)
    }

    /// Check if this version is compatible with another version.
    /// Compatible if major versions match and minor version is >= other.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl std::str::FromStr for ProtocolVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 2 {
            return Err(Error::unsupported_version(format!(
                "Invalid version format: {}",
                s
            )));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| Error::unsupported_version(format!("Invalid major version: {}", s)))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| Error::unsupported_version(format!("Invalid minor version: {}", s)))?;

        Ok(Self::new(major, minor))
    }
}

/// Protocol handler for managing signaling messages.
#[derive(Debug)]
pub struct Protocol {
    /// Current protocol version
    version: ProtocolVersion,
}

impl Protocol {
    /// Create a new protocol handler.
    pub fn new() -> Self {
        Self {
            version: ProtocolVersion::current(),
        }
    }

    /// Get the current protocol version.
    pub fn version(&self) -> &ProtocolVersion {
        &self.version
    }

    /// Validate a message against the protocol.
    pub fn validate_message(&self, message: &Message) -> Result<()> {
        // Parse message version
        let msg_version: ProtocolVersion = message.version.parse()?;

        // Check compatibility
        if !self.version.is_compatible_with(&msg_version) {
            return Err(Error::unsupported_version(format!(
                "Message version {} is not compatible with protocol version {}",
                msg_version, self.version
            )));
        }

        // Validate message content
        message.validate()?;

        Ok(())
    }

    /// Parse and validate a JSON message.
    pub fn parse_message(&self, json: &str) -> Result<Message> {
        let message = Message::from_json(json)?;
        self.validate_message(&message)?;
        Ok(message)
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::MessageType;

    #[test]
    fn test_protocol_version() {
        let v1 = ProtocolVersion::new(1, 0);
        let v2 = ProtocolVersion::new(1, 1);
        let v3 = ProtocolVersion::new(2, 0);

        assert!(v2 > v1);
        assert!(v3 > v2);
        assert_eq!(v1.to_string(), "1.0");
    }

    #[test]
    fn test_version_from_str() {
        let v: ProtocolVersion = "1.0".parse().unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);

        assert!("invalid".parse::<ProtocolVersion>().is_err());
        assert!("1".parse::<ProtocolVersion>().is_err());
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0 = ProtocolVersion::new(1, 0);
        let v1_1 = ProtocolVersion::new(1, 1);
        let v2_0 = ProtocolVersion::new(2, 0);

        assert!(v1_1.is_compatible_with(&v1_0));
        assert!(!v1_0.is_compatible_with(&v1_1));
        assert!(!v2_0.is_compatible_with(&v1_0));
    }

    #[test]
    fn test_protocol_validation() {
        let protocol = Protocol::new();

        let msg = Message::new(
            "test-1",
            MessageType::Join {
                room_id: "room1".to_string(),
                participant_id: "user1".to_string(),
            },
        );

        assert!(protocol.validate_message(&msg).is_ok());
    }

    #[test]
    fn test_protocol_parse_message() {
        let protocol = Protocol::new();

        let msg = Message::new(
            "test-1",
            MessageType::Join {
                room_id: "room1".to_string(),
                participant_id: "user1".to_string(),
            },
        );

        let json = msg.to_json().unwrap();
        let parsed = protocol.parse_message(&json).unwrap();

        assert_eq!(parsed.id, msg.id);
    }

    #[test]
    fn test_incompatible_version() {
        let protocol = Protocol::new();

        let mut msg = Message::new(
            "test-1",
            MessageType::Join {
                room_id: "room1".to_string(),
                participant_id: "user1".to_string(),
            },
        );

        msg.version = "0.9".to_string();
        assert!(protocol.validate_message(&msg).is_err());
    }
}
