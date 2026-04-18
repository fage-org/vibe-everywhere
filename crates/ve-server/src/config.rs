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
    /// Reserved for future WebSocket keepalive implementation.
    #[serde(default = "default_heartbeat_interval")]
    #[allow(dead_code)]
    pub heartbeat_interval_secs: u64,

    /// WebSocket connection timeout in seconds (default: 60 seconds)
    /// Reserved for future WebSocket timeout handling implementation.
    #[serde(default = "default_connection_timeout")]
    #[allow(dead_code)]
    pub connection_timeout_secs: u64,

    /// Data directory for SQLite databases
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// CORS allowed origins (comma-separated, empty = most restrictive, "*" = allow all)
    #[serde(default = "default_cors_origins")]
    pub cors_origins: Vec<String>,

    /// Ack timeout in milliseconds (default: 10 seconds)
    /// Reserved for daemon message acknowledgment retry logic.
    #[serde(default = "default_ack_timeout_ms")]
    #[allow(dead_code)]
    pub ack_timeout_ms: u64,

    /// Ack max retries for retryable operations (default: 2)
    /// Reserved for daemon message acknowledgment retry logic.
    #[serde(default = "default_ack_max_retries")]
    #[allow(dead_code)]
    pub ack_max_retries: u32,

    /// Ack retry delay in milliseconds (default: 500ms)
    /// Reserved for daemon message acknowledgment retry logic.
    #[serde(default = "default_ack_retry_delay_ms")]
    #[allow(dead_code)]
    pub ack_retry_delay_ms: u64,

    /// Permission request default TTL in seconds (default: 30 minutes)
    /// Used by background task for permission expiry.
    #[serde(default = "default_permission_ttl_secs")]
    #[allow(dead_code)]
    pub permission_ttl_secs: u64,

    /// Permission expiry check interval in seconds (default: 60 seconds)
    /// Used by background task for permission expiry.
    #[serde(default = "default_permission_expiry_check_secs")]
    #[allow(dead_code)]
    pub permission_expiry_check_secs: u64,

    /// Idempotency key default TTL in seconds (default: 24 hours)
    /// Used by background task for idempotency key cleanup.
    #[serde(default = "default_idempotency_ttl_secs")]
    #[allow(dead_code)]
    pub idempotency_ttl_secs: u64,

    /// Idempotency cleanup interval in seconds (default: 1 hour)
    /// Used by background task for idempotency key cleanup.
    #[serde(default = "default_idempotency_cleanup_secs")]
    #[allow(dead_code)]
    pub idempotency_cleanup_secs: u64,

    /// Log format: "pretty" for development, "json" for production
    #[serde(default = "default_log_format")]
    pub log_format: String,

    /// Log level: "trace", "debug", "info", "warn", "error"
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
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

fn default_ack_timeout_ms() -> u64 {
    10000 // 10 seconds
}

fn default_ack_max_retries() -> u32 {
    2
}

fn default_ack_retry_delay_ms() -> u64 {
    500
}

fn default_permission_ttl_secs() -> u64 {
    30 * 60 // 30 minutes
}

fn default_permission_expiry_check_secs() -> u64 {
    60
}

fn default_idempotency_ttl_secs() -> u64 {
    24 * 60 * 60 // 24 hours
}

fn default_idempotency_cleanup_secs() -> u64 {
    60 * 60 // 1 hour
}

/// Database backend type
/// Reserved for future multi-database support (currently auto-detected from DATABASE_URL).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DatabaseBackend {
    Sqlite,
    Postgres,
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
            .set_default("ack_timeout_ms", default_ack_timeout_ms())
            .map_err(ServerError::Config)?
            .set_default("ack_max_retries", default_ack_max_retries())
            .map_err(ServerError::Config)?
            .set_default("ack_retry_delay_ms", default_ack_retry_delay_ms())
            .map_err(ServerError::Config)?
            .set_default("permission_ttl_secs", default_permission_ttl_secs())
            .map_err(ServerError::Config)?
            .set_default("permission_expiry_check_secs", default_permission_expiry_check_secs())
            .map_err(ServerError::Config)?
            .set_default("idempotency_ttl_secs", default_idempotency_ttl_secs())
            .map_err(ServerError::Config)?
            .set_default("idempotency_cleanup_secs", default_idempotency_cleanup_secs())
            .map_err(ServerError::Config)?
            .set_default("log_format", default_log_format())
            .map_err(ServerError::Config)?
            .set_default("log_level", default_log_level())
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

    /// Ack timeout as Duration
    #[allow(dead_code)]
    pub fn ack_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.ack_timeout_ms)
    }

    /// Ack retry delay as Duration
    #[allow(dead_code)]
    pub fn ack_retry_delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.ack_retry_delay_ms)
    }

    /// Permission TTL as chrono Duration
    #[allow(dead_code)]
    pub fn permission_ttl(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.permission_ttl_secs as i64)
    }

    /// Idempotency TTL as chrono Duration
    #[allow(dead_code)]
    pub fn idempotency_ttl(&self) -> chrono::Duration {
        chrono::Duration::seconds(self.idempotency_ttl_secs as i64)
    }

    /// Check if JSON log format is enabled
    pub fn is_json_logging(&self) -> bool {
        self.log_format.to_lowercase() == "json"
    }

    /// Parse log level string to tracing::Level
    pub fn log_level(&self) -> tracing::Level {
        match self.log_level.to_lowercase().as_str() {
            "trace" => tracing::Level::TRACE,
            "debug" => tracing::Level::DEBUG,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        }
    }

    /// Get database backend type from database_url
    /// Reserved for future multi-database support.
    #[allow(dead_code)]
    pub fn database_backend(&self) -> DatabaseBackend {
        if self.database_url.starts_with("postgres://")
            || self.database_url.starts_with("postgresql://")
        {
            DatabaseBackend::Postgres
        } else {
            DatabaseBackend::Sqlite
        }
    }
}
