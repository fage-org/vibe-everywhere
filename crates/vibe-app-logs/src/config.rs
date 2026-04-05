use std::{
    env,
    ffi::OsString,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
};

use clap::Parser;
use directories::BaseDirs;
use thiserror::Error;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "vibe-app-logs",
    about = "Remote console log receiver for the Vibe app"
)]
struct Args {
    #[arg(long, env = "VIBE_APP_LOGS_HOST", default_value = "0.0.0.0")]
    host: String,
    #[arg(long, env = "VIBE_APP_LOGS_PORT")]
    port: Option<u16>,
    #[arg(long, env = "VIBE_HOME_DIR")]
    home_dir: Option<OsString>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub home_dir: PathBuf,
    pub logs_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to resolve the user's home directory")]
    MissingHomeDir,
    #[error("invalid host address: {0}")]
    InvalidHost(String),
    #[error("invalid port value: {0}")]
    InvalidPort(String),
}

impl Config {
    pub fn from_env_and_args() -> Result<Self, ConfigError> {
        let args = Args::parse();
        Self::from_sources(args.host, args.port, env::var("PORT").ok(), args.home_dir)
    }

    #[cfg(test)]
    pub fn from_test_sources(
        host: impl Into<String>,
        port: Option<u16>,
        port_fallback: Option<String>,
        home_dir: Option<OsString>,
    ) -> Result<Self, ConfigError> {
        Self::from_sources(host.into(), port, port_fallback, home_dir)
    }

    fn from_sources(
        host: String,
        port: Option<u16>,
        port_fallback: Option<String>,
        home_dir: Option<OsString>,
    ) -> Result<Self, ConfigError> {
        let host = host
            .parse()
            .map_err(|_| ConfigError::InvalidHost(host.clone()))?;
        let port = match port {
            Some(port) => port,
            None => match port_fallback {
                Some(raw) => raw
                    .parse()
                    .map_err(|_| ConfigError::InvalidPort(raw.clone()))?,
                None => 8787,
            },
        };
        let home_dir = resolve_home_dir(home_dir)?;
        let logs_dir = home_dir.join("app-logs");

        Ok(Self {
            host,
            port,
            home_dir,
            logs_dir,
        })
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::from((self.host, self.port))
    }
}

fn resolve_home_dir(home_dir: Option<OsString>) -> Result<PathBuf, ConfigError> {
    match home_dir {
        Some(home_dir) => expand_home_dir(home_dir),
        None => default_home_dir(),
    }
}

fn expand_home_dir(home_dir: OsString) -> Result<PathBuf, ConfigError> {
    let raw = home_dir.to_string_lossy();

    if raw == "~" {
        return user_home_dir();
    }

    if let Some(stripped) = raw.strip_prefix("~/") {
        return Ok(user_home_dir()?.join(stripped));
    }

    Ok(PathBuf::from(home_dir))
}

fn default_home_dir() -> Result<PathBuf, ConfigError> {
    Ok(user_home_dir()?.join(".vibe"))
}

fn user_home_dir() -> Result<PathBuf, ConfigError> {
    let dirs = BaseDirs::new().ok_or(ConfigError::MissingHomeDir)?;
    Ok(dirs.home_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use directories::BaseDirs;

    use super::Config;

    #[test]
    fn default_config_uses_vibe_home_and_default_port() {
        let config =
            Config::from_test_sources("127.0.0.1", None, None, Some("/tmp/vibe-home".into()))
                .unwrap();

        assert_eq!(config.port, 8787);
        assert_eq!(config.home_dir, std::path::PathBuf::from("/tmp/vibe-home"));
        assert_eq!(
            config.logs_dir,
            std::path::PathBuf::from("/tmp/vibe-home/app-logs")
        );
    }

    #[test]
    fn port_falls_back_to_port_env_compatibility() {
        let config = Config::from_test_sources(
            "127.0.0.1",
            None,
            Some("9001".into()),
            Some("/tmp/vibe-home".into()),
        )
        .unwrap();

        assert_eq!(config.port, 9001);
    }

    #[test]
    fn explicit_port_takes_priority_over_port_env_fallback() {
        let config = Config::from_test_sources(
            "127.0.0.1",
            Some(8788),
            Some("9001".into()),
            Some("/tmp/vibe-home".into()),
        )
        .unwrap();

        assert_eq!(config.port, 8788);
    }

    #[test]
    fn tilde_home_dir_expands_to_the_users_home_directory() {
        let expected_home_dir = BaseDirs::new().unwrap().home_dir().join(".vibe-dev");
        let config =
            Config::from_test_sources("127.0.0.1", None, None, Some("~/.vibe-dev".into())).unwrap();

        assert_eq!(config.home_dir, expected_home_dir);
        assert_eq!(config.logs_dir, expected_home_dir.join("app-logs"));
    }
}
