//! Vibe Everywhere Shared Library
//!
//! This crate contains shared types, models, and protocol definitions
//! used by both the server (`ve-server`) and daemon (`ve-daemon`).

pub mod jwt;
pub mod models;
pub mod proto;
pub mod types;

pub use jwt::*;
pub use models::*;
pub use proto::*;
pub use types::*;

// Re-export commonly used types
pub use models::SessionMessageType;
