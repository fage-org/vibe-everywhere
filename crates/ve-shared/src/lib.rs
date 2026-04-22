//! Vibe Everywhere Shared Library
//!
//! This crate contains shared types, models, and protocol definitions
//! used by both the server (`ve-server`) and daemon (`ve-daemon`).

pub mod jwt;
pub mod models;
pub mod pairing_proof;
pub mod proto;
pub mod types;

pub use jwt::*;
pub use models::*;
pub use pairing_proof::*;
pub use proto::*;
pub use types::*;

// Re-export ts-rs for external usage
pub use ts_rs::TS;

// Re-export commonly used types
pub use models::SessionMessageType;
