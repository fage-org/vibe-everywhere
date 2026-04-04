#![forbid(unsafe_code)]

pub mod api;
pub mod auth;
pub mod config;
pub mod context;
pub mod events;
pub mod machines;
pub mod monitoring;
pub mod presence;
pub mod sessions;
pub mod storage;
pub mod version;

use thiserror::Error;

use crate::{config::Config, context::AppContext};

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub async fn run() -> Result<(), ServerError> {
    let config = Config::from_env_and_args()?;
    run_with_config(config).await
}

pub async fn run_with_config(config: Config) -> Result<(), ServerError> {
    let context = AppContext::new(config);
    let app = api::build_router(context.clone());
    let listener = tokio::net::TcpListener::bind(context.config().bind_addr()).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(context.clone()))
        .await?;
    context.shutdown().await;
    Ok(())
}

async fn shutdown_signal(context: AppContext) {
    let _ = tokio::signal::ctrl_c().await;
    context.shutdown().await;
}
