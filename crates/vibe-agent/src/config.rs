use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use directories::BaseDirs;
use thiserror::Error;
use url::Url;

pub const DEFAULT_SERVER_URL: &str = "https://api.cluster-fluster.com";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server_url: String,
    pub home_dir: PathBuf,
    pub credential_path: PathBuf,
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
            env::var_os("VIBE_HOME_DIR"),
        )
    }

    pub fn from_sources(
        server_url: Option<String>,
        home_dir: Option<OsString>,
    ) -> Result<Self, ConfigError> {
        let server_url =
            normalize_server_url(server_url.unwrap_or_else(|| DEFAULT_SERVER_URL.into()))?;
        let home_dir = match home_dir {
            Some(home_dir) => PathBuf::from(home_dir),
            None => default_home_dir()?,
        };
        let credential_path = home_dir.join("agent.key");

        Ok(Self {
            server_url,
            home_dir,
            credential_path,
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
    use super::{Config, DEFAULT_SERVER_URL};

    #[test]
    fn default_config_uses_vibe_home_and_server_defaults() {
        let config = Config::from_sources(None, None).unwrap();

        assert_eq!(config.server_url, DEFAULT_SERVER_URL);
        assert!(config.home_dir.ends_with(".vibe"));
        assert!(config.credential_path.ends_with(".vibe/agent.key"));
    }

    #[test]
    fn env_overrides_are_applied() {
        let config = Config::from_sources(
            Some("http://127.0.0.1:3005///".into()),
            Some("/tmp/vibe-home".into()),
        )
        .unwrap();

        assert_eq!(config.server_url, "http://127.0.0.1:3005");
        assert_eq!(config.home_dir, std::path::PathBuf::from("/tmp/vibe-home"));
        assert_eq!(
            config.credential_path,
            std::path::PathBuf::from("/tmp/vibe-home/agent.key")
        );
    }

    #[test]
    fn invalid_server_url_is_rejected() {
        let error = Config::from_sources(Some("not-a-url".into()), None).unwrap_err();
        assert_eq!(error.to_string(), "invalid server URL: not-a-url");
    }
}
