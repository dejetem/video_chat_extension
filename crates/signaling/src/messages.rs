//! Signaling message types for SFU architecture.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// SDP (Session Description Protocol) type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SdpType {
    /// Offer SDP
    Offer,
    /// Answer SDP
    Answer,
}

/// ICE candidate information.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IceCandidate {
    /// Candidate string
    pub candidate: String,
    /// SDP media line index
    pub sdp_mid: Option<String>,
    /// SDP media line index number
    pub sdp_m_line_index: Option<u16>,
}

/// Session Description Protocol message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionDescription {
    /// SDP type (offer or answer)
    #[serde(rename = "type")]
    pub sdp_type: SdpType,
    /// SDP content
    pub sdp: String,
}

/// Message types for SFU signaling.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageType {
    /// Join a room
    Join {
        /// Room ID
        room_id: String,
        /// Participant ID
        participant_id: String,
    },

    /// Leave a room
    Leave {
        /// Room ID
        room_id: String,
        /// Participant ID
        participant_id: String,
    },

    /// Offer SDP to SFU
    Offer {
        /// Session description
        sdp: SessionDescription,
        /// Participant ID
        participant_id: String,
    },

    /// Answer SDP from SFU
    Answer {
        /// Session description
        sdp: SessionDescription,
        /// Participant ID
        participant_id: String,
    },

    /// ICE candidate
    IceCandidate {
        /// Candidate information
        candidate: IceCandidate,
        /// Participant ID
        participant_id: String,
    },

    /// Subscribe to a participant's stream
    Subscribe {
        /// Target participant ID
        target_id: String,
        /// Requesting participant ID
        participant_id: String,
    },

    /// Unsubscribe from a participant's stream
    Unsubscribe {
        /// Target participant ID
        target_id: String,
        /// Requesting participant ID
        participant_id: String,
    },

    /// Chat message to be relayed to all participants in the room
    Chat {
        /// Room ID
        room_id: String,
        /// Sender participant ID
        participant_id: String,
        /// Message text
        text: String,
    },

    /// Error message
    Error {
        /// Error code
        code: u16,
        /// Error message
        message: String,
    },

    /// Keepalive pong
    Pong,
}

/// Complete signaling message with metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Protocol version
    pub version: String,
    /// Message ID for tracking
    pub id: String,
    /// Timestamp (milliseconds since epoch)
    pub timestamp: u64,
    /// Message payload
    #[serde(flatten)]
    pub payload: MessageType,
}

impl Message {
    /// Create a new message with the given payload.
    pub fn new(id: impl Into<String>, payload: MessageType) -> Self {
        Self {
            version: "1.0".to_string(),
            id: id.into(),
            timestamp: Self::current_timestamp(),
            payload,
        }
    }

    /// Get current timestamp in milliseconds.
    fn current_timestamp() -> u64 {
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::now() as u64
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        }
    }

    /// Serialize message to JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(Error::from)
    }

    /// Deserialize message from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Error::from)
    }

    /// Validate the message.
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::invalid_message("Message ID cannot be empty"));
        }

        if self.version.is_empty() {
            return Err(Error::invalid_message("Version cannot be empty"));
        }

        match &self.payload {
            MessageType::Offer { sdp, .. } | MessageType::Answer { sdp, .. } => {
                if sdp.sdp.is_empty() {
                    return Err(Error::invalid_sdp("SDP content cannot be empty"));
                }
            }
            MessageType::IceCandidate { candidate, .. } => {
                if candidate.candidate.is_empty() {
                    return Err(Error::invalid_ice_candidate(
                        "Candidate string cannot be empty",
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new(
            "test-id",
            MessageType::Join {
                room_id: "room1".to_string(),
                participant_id: "user1".to_string(),
            },
        );

        assert_eq!(msg.id, "test-id");
        assert_eq!(msg.version, "1.0");
        assert!(msg.timestamp > 0);
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new(
            "test-id",
            MessageType::Join {
                room_id: "room1".to_string(),
                participant_id: "user1".to_string(),
            },
        );

        let json = msg.to_json().unwrap();
        let deserialized = Message::from_json(&json).unwrap();

        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.version, deserialized.version);
    }

    #[test]
    fn test_offer_message() {
        let sdp = SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: "v=0\r\no=- 123 456 IN IP4 0.0.0.0\r\n".to_string(),
        };

        let msg = Message::new(
            "offer-1",
            MessageType::Offer {
                sdp: sdp.clone(),
                participant_id: "user1".to_string(),
            },
        );

        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_ice_candidate_message() {
        let candidate = IceCandidate {
            candidate: "candidate:1 1 UDP 2130706431 192.168.1.1 54321 typ host".to_string(),
            sdp_mid: Some("0".to_string()),
            sdp_m_line_index: Some(0),
        };

        let msg = Message::new(
            "ice-1",
            MessageType::IceCandidate {
                candidate,
                participant_id: "user1".to_string(),
            },
        );

        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_subscribe_message() {
        let msg = Message::new(
            "sub-1",
            MessageType::Subscribe {
                target_id: "user2".to_string(),
                participant_id: "user1".to_string(),
            },
        );

        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_message_validation() {
        let mut msg = Message::new(
            "",
            MessageType::Join {
                room_id: "room1".to_string(),
                participant_id: "user1".to_string(),
            },
        );

        assert!(msg.validate().is_err());

        msg.id = "valid-id".to_string();
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_empty_sdp_validation() {
        let sdp = SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: String::new(),
        };

        let msg = Message::new(
            "offer-1",
            MessageType::Offer {
                sdp,
                participant_id: "user1".to_string(),
            },
        );

        assert!(msg.validate().is_err());
    }
}
