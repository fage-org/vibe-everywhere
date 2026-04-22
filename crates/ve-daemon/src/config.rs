//! Daemon Configuration
//!
//! Configuration structures and loading logic for the Vibe Everywhere daemon.

use std::path::PathBuf;

use serde::Deserialize;

use crate::error::DaemonError;

/// Result type for config operations
type Result<T> = std::result::Result<T, DaemonError>;

/// Daemon configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Server URL (e.g., https://example.com)
    pub server_url: String,

    /// Host name for display
    pub host_name: String,

    /// Platform identifier (linux, macos, windows)
    pub platform: String,

    /// Configuration directory path
    #[serde(default = "default_config_dir")]
    pub config_dir: PathBuf,

    /// Heartbeat interval in seconds
    #[serde(default = "default_heartbeat_interval")]
    pub heartbeat_interval_secs: u64,

    /// Heartbeat timeout in seconds
    #[serde(default = "default_heartbeat_timeout")]
    pub heartbeat_timeout_secs: u64,

    /// Ack timeout in seconds
    #[serde(default = "default_ack_timeout")]
    pub ack_timeout_secs: u64,

    /// Permission wait timeout in seconds
    #[serde(default = "default_permission_timeout")]
    pub permission_timeout_secs: u64,

    /// Minimum reconnect backoff in milliseconds
    #[serde(default = "default_reconnect_backoff_min")]
    pub reconnect_backoff_min_ms: u64,

    /// Maximum reconnect backoff in milliseconds
    #[serde(default = "default_reconnect_backoff_max")]
    pub reconnect_backoff_max_ms: u64,

    /// Maximum parallel sessions
    #[serde(default = "default_max_parallel_sessions")]
    pub max_parallel_sessions: usize,

    /// File read text limit in bytes
    #[serde(default = "default_file_read_limit")]
    pub file_read_text_limit_bytes: u64,

    /// Maximum file tree nodes
    #[serde(default = "default_file_tree_max_nodes")]
    pub file_tree_max_nodes: usize,

    /// Claude Code CLI command path
    #[serde(default = "default_claude_command")]
    pub claude_command: String,

    /// Default model
    #[serde(default = "default_model")]
    pub default_model: String,

    /// Log format (json/pretty)
    #[serde(default = "default_log_format")]
    pub log_format: String,

    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vibe-daemon")
}

fn default_heartbeat_interval() -> u64 {
    30
}

fn default_heartbeat_timeout() -> u64 {
    90
}

fn default_ack_timeout() -> u64 {
    30
}

fn default_permission_timeout() -> u64 {
    60
}

fn default_reconnect_backoff_min() -> u64 {
    1000
}

fn default_reconnect_backoff_max() -> u64 {
    30000
}

fn default_max_parallel_sessions() -> usize {
    4
}

fn default_file_read_limit() -> u64 {
    262_144 // 256KB
}

fn default_file_tree_max_nodes() -> usize {
    20_000
}

fn default_claude_command() -> String {
    "claude".to_string()
}

fn default_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    /// Load configuration from file and environment variables
    ///
    /// Priority (highest to lowest):
    /// 1. Environment variables (VIBE_DAEMON_*)
    /// 2. Config file (~/.config/vibe-daemon/config.toml)
    /// 3. Default values
    pub fn load() -> Result<Self> {
        let config_dir = default_config_dir();
        let config_path = config_dir.join("config.toml");

        // Load .env file if present (ignored if file doesn't exist)
        // This is expected behavior: .env files are optional, missing file is not an error
        let _ = dotenvy::dotenv();

        let builder = config::Config::builder();

        // Set defaults
        let builder = builder
            .set_default("config_dir", config_dir.to_string_lossy().to_string())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("heartbeat_interval_secs", default_heartbeat_interval())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("heartbeat_timeout_secs", default_heartbeat_timeout())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("ack_timeout_secs", default_ack_timeout())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("permission_timeout_secs", default_permission_timeout())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("reconnect_backoff_min_ms", default_reconnect_backoff_min())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("reconnect_backoff_max_ms", default_reconnect_backoff_max())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default(
                "max_parallel_sessions",
                default_max_parallel_sessions() as u64,
            )
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("file_read_text_limit_bytes", default_file_read_limit())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("file_tree_max_nodes", default_file_tree_max_nodes() as u64)
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("claude_command", default_claude_command())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("default_model", default_model())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("log_format", default_log_format())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?
            .set_default("log_level", default_log_level())
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?;

        // Try to load from config file
        let builder = if config_path.exists() {
            builder.add_source(config::File::from(config_path.as_path()))
        } else {
            builder
        };

        // Override with environment variables (VIBE_DAEMON_*)
        let builder = builder.add_source(
            config::Environment::with_prefix("VIBE_DAEMON")
                .separator("__")
                .try_parsing(true),
        );

        let config = builder
            .build()
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?;

        let cfg: Config = config
            .try_deserialize()
            .map_err(|e| DaemonError::ConfigInvalid(e.to_string()))?;

        // Validate all configuration values
        cfg.validate()?;

        Ok(cfg)
    }

    /// Validate configuration values
    ///
    /// Checks:
    /// - Required fields are present
    /// - server_url is a valid URL format
    /// - heartbeat_timeout > heartbeat_interval
    /// - reconnect_backoff_max >= reconnect_backoff_min
    pub fn validate(&self) -> Result<()> {
        // Required fields
        if self.server_url.is_empty() {
            return Err(DaemonError::ConfigInvalid(
                "server_url is required".to_string(),
            ));
        }
        if self.host_name.is_empty() {
            return Err(DaemonError::ConfigInvalid(
                "host_name is required".to_string(),
            ));
        }
        if self.platform.is_empty() {
            return Err(DaemonError::ConfigInvalid(
                "platform is required".to_string(),
            ));
        }

        // URL format validation (basic check)
        if !self.server_url.starts_with("http://") && !self.server_url.starts_with("https://") {
            return Err(DaemonError::ConfigInvalid(
                "server_url must be a valid HTTP or HTTPS URL".to_string(),
            ));
        }

        // Heartbeat validation: timeout must be greater than interval
        if self.heartbeat_timeout_secs <= self.heartbeat_interval_secs {
            return Err(DaemonError::ConfigInvalid(format!(
                "heartbeat_timeout_secs ({}) must be greater than heartbeat_interval_secs ({})",
                self.heartbeat_timeout_secs, self.heartbeat_interval_secs
            )));
        }

        // Reconnect backoff validation: max must be >= min
        if self.reconnect_backoff_max_ms < self.reconnect_backoff_min_ms {
            return Err(DaemonError::ConfigInvalid(format!(
                "reconnect_backoff_max_ms ({}) must be >= reconnect_backoff_min_ms ({})",
                self.reconnect_backoff_max_ms, self.reconnect_backoff_min_ms
            )));
        }

        Ok(())
    }

    /// Get credentials file path
    pub fn credentials_path(&self) -> PathBuf {
        self.config_dir.join("credentials.json")
    }

    pub fn installation_path(&self) -> PathBuf {
        self.config_dir.join("installation.json")
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

    /// Get heartbeat interval as Duration
    pub fn heartbeat_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.heartbeat_interval_secs)
    }

    /// Get heartbeat timeout as Duration
    pub fn heartbeat_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.heartbeat_timeout_secs)
    }

    /// Get ack timeout as Duration
    pub fn ack_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.ack_timeout_secs)
    }

    /// Get permission timeout as Duration
    pub fn permission_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.permission_timeout_secs)
    }

    /// Get minimum reconnect backoff as Duration
    pub fn reconnect_backoff_min(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.reconnect_backoff_min_ms)
    }

    /// Get maximum reconnect backoff as Duration
    pub fn reconnect_backoff_max(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.reconnect_backoff_max_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_dir() {
        let dir = default_config_dir();
        assert!(dir.ends_with("vibe-daemon"));
    }

    #[test]
    fn test_default_values() {
        assert_eq!(default_heartbeat_interval(), 30);
        assert_eq!(default_heartbeat_timeout(), 90);
        assert_eq!(default_ack_timeout(), 30);
        assert_eq!(default_permission_timeout(), 60);
        assert_eq!(default_reconnect_backoff_min(), 1000);
        assert_eq!(default_reconnect_backoff_max(), 30000);
        assert_eq!(default_max_parallel_sessions(), 4);
        assert_eq!(default_file_read_limit(), 262_144);
        assert_eq!(default_file_tree_max_nodes(), 20_000);
        assert_eq!(default_claude_command(), "claude");
        assert_eq!(default_model(), "claude-sonnet-4-20250514");
        assert_eq!(default_log_format(), "pretty");
        assert_eq!(default_log_level(), "info");
    }

    #[test]
    fn test_is_json_logging() {
        let mut config = Config {
            server_url: "https://test.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: PathBuf::from("/tmp"),
            log_format: "json".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
        };

        assert!(config.is_json_logging());

        config.log_format = "pretty".to_string();
        assert!(!config.is_json_logging());

        config.log_format = "JSON".to_string();
        assert!(config.is_json_logging());
    }

    #[test]
    fn test_log_level_parsing() {
        let mut config = Config {
            server_url: "https://test.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
        };

        assert_eq!(config.log_level(), tracing::Level::INFO);

        config.log_level = "trace".to_string();
        assert_eq!(config.log_level(), tracing::Level::TRACE);

        config.log_level = "debug".to_string();
        assert_eq!(config.log_level(), tracing::Level::DEBUG);

        config.log_level = "warn".to_string();
        assert_eq!(config.log_level(), tracing::Level::WARN);

        config.log_level = "error".to_string();
        assert_eq!(config.log_level(), tracing::Level::ERROR);

        config.log_level = "unknown".to_string();
        assert_eq!(config.log_level(), tracing::Level::INFO);
    }

    #[test]
    fn test_validate_heartbeat_timeout_must_be_greater_than_interval() {
        // heartbeat_timeout must be > heartbeat_interval
        let config = Config {
            server_url: "https://test.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 90,
            heartbeat_timeout_secs: 30, // Invalid: less than interval
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("heartbeat_timeout"));
    }

    #[test]
    fn test_validate_invalid_server_url() {
        let config = Config {
            server_url: "not-a-valid-url".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("server_url"));
    }

    #[test]
    fn test_validate_valid_config() {
        let config = Config {
            server_url: "https://valid.example.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
        };

        assert!(config.validate().is_ok());
    }
}
