use std::{
    env,
    net::{IpAddr, SocketAddr},
};

use clap::Parser;
use thiserror::Error;

#[derive(Debug, Clone, Parser)]
#[command(name = "vibe-server", about = "Wave 2 minimum spine backend")]
struct Args {
    #[arg(long, env = "VIBE_SERVER_HOST", default_value = "0.0.0.0")]
    host: String,
    #[arg(long, env = "VIBE_SERVER_PORT", default_value_t = 3005)]
    port: u16,
    #[arg(long, env = "VIBE_MASTER_SECRET")]
    master_secret: Option<String>,
    #[arg(long, env = "VIBE_IOS_UP_TO_DATE", default_value = ">=1.4.1")]
    ios_up_to_date: String,
    #[arg(long, env = "VIBE_ANDROID_UP_TO_DATE", default_value = ">=1.4.1")]
    android_up_to_date: String,
    #[arg(
        long,
        env = "VIBE_IOS_STORE_URL",
        default_value = "https://app.vibe.engineering/ios"
    )]
    ios_store_url: String,
    #[arg(
        long,
        env = "VIBE_ANDROID_STORE_URL",
        default_value = "https://app.vibe.engineering/android"
    )]
    android_store_url: String,
    #[arg(
        long,
        env = "VIBE_WEBAPP_URL",
        default_value = "https://app.vibe.engineering"
    )]
    webapp_url: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub master_secret: String,
    pub ios_up_to_date: String,
    pub android_up_to_date: String,
    pub ios_store_url: String,
    pub android_store_url: String,
    pub webapp_url: String,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required env var VIBE_MASTER_SECRET")]
    MissingMasterSecret,
    #[error("invalid host address: {0}")]
    InvalidHost(String),
}

impl Config {
    pub fn from_env_and_args() -> Result<Self, ConfigError> {
        let args = Args::parse();
        Self::from_args_with_fallback(args, env::var("HANDY_MASTER_SECRET").ok())
    }

    #[cfg(test)]
    fn from_args(args: Args) -> Result<Self, ConfigError> {
        Self::from_args_with_fallback(args, None)
    }

    fn from_args_with_fallback(
        args: Args,
        fallback_master_secret: Option<String>,
    ) -> Result<Self, ConfigError> {
        let master_secret = args
            .master_secret
            .or(fallback_master_secret)
            .ok_or(ConfigError::MissingMasterSecret)?;
        let host = args
            .host
            .parse()
            .map_err(|_| ConfigError::InvalidHost(args.host.clone()))?;

        Ok(Self {
            host,
            port: args.port,
            master_secret,
            ios_up_to_date: args.ios_up_to_date,
            android_up_to_date: args.android_up_to_date,
            ios_store_url: args.ios_store_url,
            android_store_url: args.android_store_url,
            webapp_url: args.webapp_url,
        })
    }

    pub fn bind_addr(&self) -> SocketAddr {
        SocketAddr::from((self.host, self.port))
    }
}

#[cfg(test)]
mod tests {
    use super::{Args, Config, ConfigError};

    #[test]
    fn bind_addr_uses_port() {
        let cfg = Config {
            host: "127.0.0.1".parse().unwrap(),
            port: 4123,
            master_secret: "secret".into(),
            ios_up_to_date: ">=1.0.0".into(),
            android_up_to_date: ">=1.0.0".into(),
            ios_store_url: "ios".into(),
            android_store_url: "android".into(),
            webapp_url: "https://app.vibe.engineering".into(),
        };

        assert_eq!(cfg.bind_addr(), "127.0.0.1:4123".parse().unwrap());
    }

    #[test]
    fn from_args_requires_master_secret() {
        let error = Config::from_args(Args {
            host: "127.0.0.1".into(),
            port: 3005,
            master_secret: None,
            ios_up_to_date: ">=1.0.0".into(),
            android_up_to_date: ">=1.0.0".into(),
            ios_store_url: "ios".into(),
            android_store_url: "android".into(),
            webapp_url: "https://app.vibe.engineering".into(),
        })
        .unwrap_err();

        assert!(matches!(error, ConfigError::MissingMasterSecret));
    }

    #[test]
    fn from_args_rejects_invalid_host() {
        let error = Config::from_args(Args {
            host: "not-an-ip".into(),
            port: 3005,
            master_secret: Some("secret".into()),
            ios_up_to_date: ">=1.0.0".into(),
            android_up_to_date: ">=1.0.0".into(),
            ios_store_url: "ios".into(),
            android_store_url: "android".into(),
            webapp_url: "https://app.vibe.engineering".into(),
        })
        .unwrap_err();

        assert!(matches!(error, ConfigError::InvalidHost(host) if host == "not-an-ip"));
    }
}
