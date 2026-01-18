//! WebRTC signaling protocol for SFU architecture.
//!
//! This crate provides signaling message types and protocol handling for
//! establishing WebRTC connections with an SFU (Selective Forwarding Unit).

pub mod error;
pub mod messages;
pub mod protocol;
pub mod websocket;

pub use error::{Error, Result};
pub use messages::{IceCandidate, Message, MessageType, SdpType, SessionDescription};
pub use protocol::{Protocol, ProtocolVersion};
pub use websocket::WebSocketSignaling;
