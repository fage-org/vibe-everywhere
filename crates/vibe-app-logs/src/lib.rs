pub mod config;
pub mod server;

use config::{Config, ConfigError};
use server::ServerError;

#[derive(Debug, thiserror::Error)]
pub enum AppLogsError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Server(#[from] ServerError),
}

pub async fn run() -> Result<(), AppLogsError> {
    let config = Config::from_env_and_args()?;
    server::run(config).await?;
    Ok(())
}
