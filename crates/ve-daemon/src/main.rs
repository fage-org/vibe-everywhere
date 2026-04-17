//! Vibe Everywhere Daemon
//!
//! Remote host daemon that manages CLI agent sessions.
//!
//! ## Overview
//!
//! The daemon runs on the host machine and:
//! - Maintains WebSocket connection to the coordination server
//! - Runs CLI agent processes (claude-code)
//! - Handles file system access within workspace boundaries
//! - Manages permission requests and responses
//!
//! ## Startup
//!
//! 1. Load configuration from `~/.config/vibe-daemon/config.toml` and environment
//! 2. Initialize logging (JSON in production, pretty in development)
//! 3. Check for existing credentials
//! 4. Enter pairing mode (no credentials) or connection mode (credentials exist)
//! 5. Wait for shutdown signal (Ctrl+C, SIGTERM)

use std::sync::Arc;

use tokio::signal;
use tracing::{error, info, warn};

use ve_daemon::config::Config;
use ve_daemon::credentials::Credentials;
use ve_daemon::DaemonError;

#[tokio::main]
async fn main() {
    // Initialize and run, handling any errors
    if let Err(e) = run().await {
        error!(error = %e, "Daemon failed");
        std::process::exit(1);
    }
}

/// Main daemon entry point
async fn run() -> Result<(), DaemonError> {
    // 1. Load configuration
    let config = Config::load()?;
    let config = Arc::new(config);

    // 2. Initialize tracing
    init_tracing(&config);

    info!(
        server_url = %config.server_url,
        host_name = %config.host_name,
        platform = %config.platform,
        "Starting Vibe Everywhere daemon"
    );

    // 3. Check for existing credentials
    let credentials = load_credentials(&config)?;

    // 4. Validate credentials file permissions
    if let Some(ref creds) = credentials {
        let path = config.credentials_path();
        if let Some(warning) = Credentials::check_permissions(&path)? {
            warn!("{}", warning);
        }
        info!(host_id = %creds.host_id, "Found existing credentials, entering connection mode");
        // TODO: WebSocket client connection (next phase)
        // ws_client::connect(&config, creds, shutdown_tx.subscribe()).await?;
    } else {
        info!("No credentials found, entering pairing mode");
        // TODO: Pairing flow (next phase)
        // pairing::start_pairing(&config, shutdown_tx.subscribe()).await?;
    }

    // 5. Wait for shutdown signal
    wait_for_shutdown().await;

    info!("Daemon shutdown complete");
    Ok(())
}

/// Initialize tracing subscriber
///
/// Uses JSON format in production (log_format = "json"),
/// pretty format in development (log_format = "pretty").
fn init_tracing(config: &Config) {
    let level = config.log_level();

    if config.is_json_logging() {
        tracing_subscriber::fmt()
            .json()
            .with_max_level(level)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(true)
            .with_file(true)
            .with_line_number(true)
            .pretty()
            .init();
    }
}

/// Load credentials from the configured path
fn load_credentials(config: &Config) -> Result<Option<Credentials>, DaemonError> {
    let path = config.credentials_path();
    Credentials::load(&path)
}

/// Wait for shutdown signals (Ctrl+C or SIGTERM)
async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down"),
        _ = terminate => info!("Received SIGTERM, shutting down"),
    }
}
