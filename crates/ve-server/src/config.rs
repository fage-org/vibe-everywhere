//! Server Configuration
//!
//! Configuration structures and loading logic for the Vibe Everywhere server.

use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;

use crate::error::{Result, ServerError};

/// Minimum required length for JWT secret (security requirement)
const JWT_SECRET_MIN_LENGTH: usize = 32;

/// Placeholder patterns that should not be used in production JWT secrets
const JWT_SECRET_PLACEHOLDERS: &[&str] = &[
    "your-secret-key",
    "your_secret_key",
    "secret",
    "jwt_secret",
    "change_me",
    "changeme",
    "example",
    "placeholder",
    "default",
    "test",
    "dev",
    "development",
];

/// Main server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server listen address
    #[serde(default = "default_listen_addr")]
    pub listen_addr: SocketAddr,

    /// Database URL (SQLite or PostgreSQL)
    pub database_url: String,

    /// JWT secret key for signing tokens
    pub jwt_secret: String,

    /// JWT expiration time in seconds (default: 30 days)
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration_secs: u64,

    /// Pairing code TTL in seconds (default: 5 minutes)
    #[serde(default = "default_pair_code_ttl")]
    pub pair_code_ttl_secs: u64,

    /// WebSocket heartbeat interval in seconds (default: 30 seconds)
    #[serde(default = "default_heartbeat_interval")]
    #[allow(dead_code)]
    pub heartbeat_interval_secs: u64,

    /// WebSocket connection timeout in seconds (default: 60 seconds)
    #[serde(default = "default_connection_timeout")]
    #[allow(dead_code)]
    pub connection_timeout_secs: u64,

    /// Data directory for SQLite databases
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// CORS allowed origins (comma-separated, empty = most restrictive, "*" = allow all)
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,
}

fn default_listen_addr() -> SocketAddr {
    "0.0.0.0:3000".parse().unwrap()
}

fn default_jwt_expiration() -> u64 {
    30 * 24 * 60 * 60 // 30 days
}

fn default_pair_code_ttl() -> u64 {
    5 * 60 // 5 minutes
}

fn default_heartbeat_interval() -> u64 {
    30
}

fn default_connection_timeout() -> u64 {
    60
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}

fn default_cors_origins() -> Vec<String> {
    Vec::new() // Empty by default = most restrictive
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let config = config::Config::builder()
            .set_default("listen_addr", default_listen_addr().to_string())
            .map_err(ServerError::Config)?
            .set_default("jwt_expiration_secs", default_jwt_expiration())
            .map_err(ServerError::Config)?
            .set_default("pair_code_ttl_secs", default_pair_code_ttl())
            .map_err(ServerError::Config)?
            .set_default("heartbeat_interval_secs", default_heartbeat_interval())
            .map_err(ServerError::Config)?
            .set_default("connection_timeout_secs", default_connection_timeout())
            .map_err(ServerError::Config)?
            .set_default("data_dir", default_data_dir().to_string_lossy().to_string())
            .map_err(ServerError::Config)?
            .set_default("cors_origins", Vec::<String>::new())
            .map_err(ServerError::Config)?
            .add_source(
                config::Environment::default()
                    .separator("__")
                    .try_parsing(true)
                    .list_separator(","),
            )
            .build()
            .map_err(ServerError::Config)?;

        let mut cfg: Config = config.try_deserialize().map_err(ServerError::Config)?;

        // Trim whitespace from CORS origins
        cfg.cors_origins = cfg
            .cors_origins
            .iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Validate JWT secret
        cfg.validate_jwt_secret()?;

        Ok(cfg)
    }

    /// Validate JWT secret meets security requirements
    fn validate_jwt_secret(&self) -> Result<()> {
        let secret = &self.jwt_secret;

        // Check minimum length
        if secret.len() < JWT_SECRET_MIN_LENGTH {
            return Err(ServerError::InvalidJwtSecret(format!(
                "JWT secret must be at least {} characters, got {}",
                JWT_SECRET_MIN_LENGTH,
                secret.len()
            )));
        }

        // Check for placeholder values
        let lower = secret.to_lowercase();
        for placeholder in JWT_SECRET_PLACEHOLDERS {
            if lower.contains(placeholder) {
                return Err(ServerError::InvalidJwtSecret(format!(
                    "JWT secret contains placeholder value '{}', please use a secure random secret",
                    placeholder
                )));
            }
        }

        Ok(())
    }

    /// JWT expiration as chrono Duration
    pub fn jwt_expiration(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.jwt_expiration_secs as i64)
    }

    /// Pairing code TTL as chrono Duration
    pub fn pair_code_ttl(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.pair_code_ttl_secs as i64)
    }

    /// Heartbeat interval
    #[allow(dead_code)]
    pub fn heartbeat_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.heartbeat_interval_secs)
    }

    /// Connection timeout
    #[allow(dead_code)]
    pub fn connection_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.connection_timeout_secs)
    }
}
