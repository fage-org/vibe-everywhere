use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use thiserror::Error;
use url::Url;

pub const DEFAULT_SERVER_URL: &str = "https://api.cluster-fluster.com";
pub const DEFAULT_WEBAPP_URL: &str = "https://app.vibe.engineering";
pub const DEFAULT_CLAUDE_BIN: &str = "claude";
pub const DEFAULT_CODEX_BIN: &str = "codex";
pub const DEFAULT_GEMINI_BIN: &str = "gemini";
pub const DEFAULT_OPENCLAW_BIN: &str = "openclaw";
pub const DEFAULT_ACP_BIN: &str = "acp";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server_url: String,
    pub webapp_url: String,
    pub home_dir: PathBuf,
    pub credential_path: PathBuf,
    pub settings_path: PathBuf,
    pub session_state_dir: PathBuf,
    pub daemon_state_path: PathBuf,
    pub daemon_registry_path: PathBuf,
    pub daemon_install_path: PathBuf,
    pub daemon_launcher_path: PathBuf,
    pub claude_bin: String,
    pub codex_bin: String,
    pub gemini_bin: String,
    pub openclaw_bin: String,
    pub acp_bin: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to resolve the user's home directory")]
    MissingHomeDir,
    #[error("invalid server URL: {0}")]
    InvalidServerUrl(String),
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        Self::from_sources(
            env::var("VIBE_SERVER_URL").ok(),
            env::var("VIBE_WEBAPP_URL").ok(),
            env::var_os("VIBE_HOME_DIR"),
            env::var("VIBE_CLAUDE_BIN").ok(),
        )
    }

    pub fn from_sources(
        server_url: Option<String>,
        webapp_url: Option<String>,
        home_dir: Option<OsString>,
        claude_bin: Option<String>,
    ) -> Result<Self, ConfigError> {
        let server_url =
            normalize_server_url(server_url.unwrap_or_else(|| DEFAULT_SERVER_URL.into()))?;
        let webapp_url =
            normalize_server_url(webapp_url.unwrap_or_else(|| DEFAULT_WEBAPP_URL.into()))?;
        let home_dir = match home_dir {
            Some(home_dir) => PathBuf::from(home_dir),
            None => default_home_dir()?,
        };

        Ok(Self {
            server_url,
            webapp_url,
            credential_path: home_dir.join("access.key"),
            settings_path: home_dir.join("cli-settings.json"),
            session_state_dir: home_dir.join("cli-sessions"),
            daemon_state_path: home_dir.join("daemon").join("vibe-cli-daemon.json"),
            daemon_registry_path: home_dir.join("daemon").join("vibe-cli-registry.json"),
            daemon_install_path: home_dir.join("daemon").join("vibe-cli-install.json"),
            daemon_launcher_path: home_dir.join("bin").join("vibe-daemon"),
            home_dir,
            claude_bin: claude_bin.unwrap_or_else(|| DEFAULT_CLAUDE_BIN.into()),
            codex_bin: env::var("VIBE_CODEX_BIN").unwrap_or_else(|_| DEFAULT_CODEX_BIN.into()),
            gemini_bin: env::var("VIBE_GEMINI_BIN").unwrap_or_else(|_| DEFAULT_GEMINI_BIN.into()),
            openclaw_bin: env::var("VIBE_OPENCLAW_BIN")
                .unwrap_or_else(|_| DEFAULT_OPENCLAW_BIN.into()),
            acp_bin: env::var("VIBE_ACP_BIN").unwrap_or_else(|_| DEFAULT_ACP_BIN.into()),
        })
    }

    pub fn socket_url(&self) -> String {
        format!("{}/v1/updates", self.server_url)
    }

    pub fn home_dir(&self) -> &Path {
        &self.home_dir
    }
}

fn normalize_server_url(raw: String) -> Result<String, ConfigError> {
    let trimmed = raw.trim_end_matches('/').to_string();
    Url::parse(&trimmed).map_err(|_| ConfigError::InvalidServerUrl(raw))?;
    Ok(trimmed)
}

fn default_home_dir() -> Result<PathBuf, ConfigError> {
    let dirs = BaseDirs::new().ok_or(ConfigError::MissingHomeDir)?;
    Ok(dirs.home_dir().join(".vibe"))
}

#[cfg(test)]
mod tests {
    use super::{
        Config, DEFAULT_CLAUDE_BIN, DEFAULT_CODEX_BIN, DEFAULT_SERVER_URL, DEFAULT_WEBAPP_URL,
    };

    #[test]
    fn default_config_uses_vibe_home_and_access_key() {
        let config = Config::from_sources(None, None, None, None).unwrap();

        assert_eq!(config.server_url, DEFAULT_SERVER_URL);
        assert_eq!(config.webapp_url, DEFAULT_WEBAPP_URL);
        assert_eq!(config.claude_bin, DEFAULT_CLAUDE_BIN);
        assert_eq!(config.codex_bin, DEFAULT_CODEX_BIN);
        assert!(config.home_dir.ends_with(".vibe"));
        assert!(config.credential_path.ends_with(".vibe/access.key"));
        assert!(config.settings_path.ends_with(".vibe/cli-settings.json"));
    }

    #[test]
    fn env_overrides_are_applied() {
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005///".into()),
            Some("https://app.vibe.example///".into()),
            Some("/tmp/vibe-cli".into()),
            Some("/custom/claude".into()),
        )
        .unwrap();

        assert_eq!(config.server_url, "http://127.0.0.1:3005");
        assert_eq!(config.webapp_url, "https://app.vibe.example");
        assert_eq!(config.home_dir, PathBuf::from("/tmp/vibe-cli"));
        assert_eq!(config.claude_bin, "/custom/claude");
        assert_eq!(config.codex_bin, DEFAULT_CODEX_BIN);
        assert_eq!(config.socket_url(), "http://127.0.0.1:3005/v1/updates");
    }

    use std::path::PathBuf;

    #[test]
    fn invalid_server_url_is_rejected() {
        let error = Config::from_sources(Some("not-a-url".into()), None, None, None).unwrap_err();
        assert_eq!(error.to_string(), "invalid server URL: not-a-url");
    }
}
