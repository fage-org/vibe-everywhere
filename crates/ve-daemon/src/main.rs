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
use uuid::Uuid;

use ve_daemon::config::Config;
use ve_daemon::credentials::Credentials;
use ve_daemon::pairing::Pairing;
use ve_daemon::session_registry::SessionRegistry;
use ve_daemon::ws_client::WsClient;
use ve_daemon::DaemonError;

fn build_ws_client(config: Arc<Config>, host_id: Uuid, token: String) -> WsClient {
    // Use bounded mpsc channel for driver events (single producer, single consumer).
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(2048);
    let registry = Arc::new(SessionRegistry::new(config.clone(), event_tx.clone()));
    WsClient::with_registry(config, host_id, token, registry, event_tx, event_rx)
}

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

    // 3. Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);

    // 4. Spawn signal handler
    let shutdown_tx_clone = shutdown_tx.clone();
    tokio::spawn(async move {
        wait_for_shutdown().await;
        let _ = shutdown_tx_clone.send(());
    });

    // 5. Check for existing credentials
    let credentials = load_credentials(&config)?;

    // 6. Run either pairing or connection mode
    if let Some(creds) = credentials {
        // Validate credentials file permissions
        let path = config.credentials_path();
        if let Some(warning) = Credentials::check_permissions(&path)? {
            warn!("{}", warning);
        }

        creds.validate_server_url(&config.server_url)?;

        info!(host_id = %creds.host_id, "Found existing credentials, entering connection mode");

        // Parse host_id UUID
        let host_id = Uuid::parse_str(&creds.host_id).map_err(|_| DaemonError::TokenParse)?;

        // Create and run WebSocket client
        let client = build_ws_client(config, host_id, creds.expose_token().to_string());
        client.run(shutdown_rx).await?;
    } else {
        info!("No credentials found, entering pairing mode");

        // Run pairing flow
        let pairing = Pairing::new(config.clone());
        let creds = pairing.start(shutdown_rx).await?;
        pairing.save_credentials(&creds)?;

        // After pairing, start WebSocket client
        let host_id = Uuid::parse_str(&creds.host_id).map_err(|_| DaemonError::TokenParse)?;
        let client = build_ws_client(config, host_id, creds.expose_token().to_string());
        client.run(shutdown_tx.subscribe()).await?;
    }

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
