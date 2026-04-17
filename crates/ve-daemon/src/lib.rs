//! Vibe Everywhere Daemon
//!
//! Local agent runner and WebSocket bridge for Vibe Everywhere.
//!
//! ## Architecture
//!
//! The daemon runs on the host machine and:
//! - Maintains WebSocket connection to the coordination server
//! - Runs CLI agent processes (claude-code)
//! - Handles file system access within workspace boundaries
//! - Manages permission requests and responses
//!
//! ## Error Handling
//!
//! All errors use [`DaemonError`] for categorization and handling.
//! Critical errors that need to be reported back use [`AckError`].

pub mod config;
pub mod credentials;
pub mod error;

pub use config::Config;
pub use credentials::Credentials;
pub use error::{AckError, DaemonError};

/// Result type alias for daemon operations
pub type Result<T> = std::result::Result<T, DaemonError>;
