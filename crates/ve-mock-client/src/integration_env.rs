//! Integration test environment — server + daemon + temp directory

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use uuid::Uuid;
use ve_shared::jwt::JwtManager;

use crate::client::MockClient;
use crate::daemon::IntegrationDaemon;
use crate::server::IntegrationServer;

use ve_server::db;

/// Integration-mode environment: temp directory, server, and daemon subprocess.
pub struct IntegrationEnv {
    pub temp_dir: tempfile::TempDir,
    pub server: IntegrationServer,
    pub daemon: IntegrationDaemon,
    pub jwt_manager: Arc<JwtManager>,
    pub pair_code: String,
    pub host_id: Uuid,
    pub device_id: Uuid,
    pub client: MockClient,
}

impl IntegrationEnv {
    /// Create a full integration environment: temp dir + server + daemon + pairing.
    /// When `mock_mode` is false, the daemon uses real Claude Code instead of MockDriver.
    pub async fn new(mock_mode: bool) -> Result<Self> {
        let temp_dir = tempfile::tempdir().context("creating temp directory")?;
        let temp_path = temp_dir.path();

        // Start server
        let server = IntegrationServer::start(temp_path)
            .await
            .context("starting test server")?;

        let server_url = server.server_url.clone();
        let jwt_manager = Arc::clone(&server.jwt_manager);
        let pool = server.pool.clone();

        // Start daemon (without credentials → enters pairing mode)
        let daemon = IntegrationDaemon::spawn(&server_url, temp_path, mock_mode)
            .await
            .context("starting test daemon")?;

        // Step 1: Wait for daemon-hello to create host record in DB
        let (host_id, pair_code) = wait_for_pending_host(&pool, Duration::from_secs(15))
            .await
            .context("waiting for daemon hello")?;

        tracing::info!(%host_id, %pair_code, "Daemon hello received, pair_code available");

        // Step 2: Register a test device and get bootstrap token
        let device_id = Uuid::new_v4();
        let bootstrap_token = jwt_manager
            .create_client_bootstrap_token(device_id, "mock-client")
            .context("creating bootstrap JWT")?;

        // Insert device record (normally done by register-device endpoint)
        let device_id_str = device_id.to_string();
        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(&device_id_str)
        .bind("mock-client")
        .bind("desktop")
        .bind(&server_url)
        .execute(&pool)
        .await
        .context("inserting test device")?;

        // Step 3: Complete pairing by calling POST /api/auth/pair
        let client_token = complete_pairing(&server_url, &pair_code, &bootstrap_token)
            .await
            .context("completing pairing")?;

        tracing::info!("Pairing completed, client token obtained");

        // Step 4: Wait for daemon WS connection
        wait_for_daemon_connected(&server, Duration::from_secs(10))
            .await
            .context("waiting for daemon WS connection")?;

        // Get the actual host_id from connected daemons
        let actual_host_id = server
            .hub
            .connected_daemons()
            .await
            .into_iter()
            .next()
            .unwrap_or(host_id);

        let client = MockClient::new(server_url.clone(), client_token);

        tracing::info!(
            "Integration environment ready: server={}, host_id={:?}",
            server_url,
            actual_host_id
        );

        Ok(Self {
            temp_dir,
            server,
            daemon,
            jwt_manager,
            pair_code,
            host_id: actual_host_id,
            device_id,
            client,
        })
    }

    pub fn pool(&self) -> &db::DbPool {
        &self.server.pool
    }

    pub fn hub(&self) -> &Arc<ve_server::hub::Hub> {
        &self.server.hub
    }

    pub fn daemon_pid(&self) -> u32 {
        self.daemon.process.id()
    }

    pub fn temp_dir_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    pub fn workspace_path(&self, name: &str) -> String {
        self.temp_dir
            .path()
            .join(name)
            .to_string_lossy()
            .to_string()
    }

    pub fn server_url(&self) -> &str {
        &self.server.server_url
    }
}

/// Wait for the daemon to create a pending host record via daemon-hello
async fn wait_for_pending_host(pool: &sqlx::AnyPool, timeout: Duration) -> Result<(Uuid, String)> {
    let deadline = std::time::Instant::now() + timeout;

    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("No pending host found within {timeout:?}");
        }

        let row: Option<(String, String, String)> = sqlx::query_as(
            "SELECT host_id, pair_code, pair_status FROM hosts WHERE pair_status = 'pending' ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(pool)
        .await?;

        if let Some((host_id, pair_code, status)) = row {
            if status == "pending" && !pair_code.is_empty() {
                let host_id = Uuid::parse_str(&host_id)?;
                return Ok((host_id, pair_code));
            }
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/// Complete the pairing flow by calling POST /api/auth/pair
async fn complete_pairing(
    server_url: &str,
    pair_code: &str,
    bootstrap_token: &str,
) -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/auth/pair", server_url.trim_end_matches('/'));

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {bootstrap_token}"))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "pair_code": pair_code }))
        .send()
        .await
        .context("sending pair request")?;

    let status = response.status();
    let body = response.text().await.context("reading pair response")?;

    if !status.is_success() {
        anyhow::bail!("Pair request failed with status {status}: {body}");
    }

    let json: serde_json::Value = serde_json::from_str(&body).context("parsing pair response")?;

    json.get("token")
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("Missing 'token' in pair response: {body}"))
}

/// Wait for the daemon to establish a WebSocket connection
async fn wait_for_daemon_connected(server: &IntegrationServer, timeout: Duration) -> Result<()> {
    let deadline = std::time::Instant::now() + timeout;

    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("Daemon WS connection not established within {timeout:?}");
        }

        let daemons = server.hub.connected_daemons().await;
        if !daemons.is_empty() {
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
