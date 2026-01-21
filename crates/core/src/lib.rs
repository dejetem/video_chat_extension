//! Core types and utilities for the video chat extension.
//!
//! This crate provides platform-agnostic core functionality including:
//! - Error types and handling
//! - Configuration management
//! - Common data structures

pub mod cleanup;
pub mod config;
pub mod error;
pub mod participant;
pub mod room;

pub use cleanup::CleanupManager;
pub use config::Config;
pub use error::{Error, Result};
pub use participant::Participant;
pub use room::Room;
