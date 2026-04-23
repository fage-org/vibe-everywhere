//! Test context — orchestrates server, daemon, and client

use std::sync::Arc;

use anyhow::{Context, Result};
use uuid::Uuid;
use ve_shared::jwt::JwtManager;

use crate::client::MockClient;
use crate::integration_env::IntegrationEnv;

// Re-export db module type
use ve_server::db;

#[derive(Debug, Clone, PartialEq)]
pub enum TestMode {
    /// Real server + daemon subprocess
    Integration,
    /// Connect to existing server
    Remote,
}

/// Test context that manages the lifecycle of server, daemon, and client
pub struct TestContext {
    pub mode: TestMode,
    pub server_url: String,
    pub host_id: Option<Uuid>,
    pub device_id: Option<Uuid>,
    pub client: MockClient,
    pub jwt_manager: Option<Arc<JwtManager>>,
    pub pair_code: Option<String>,

    // Integration mode resources (delegated to IntegrationEnv)
    env: Option<IntegrationEnv>,
}

impl TestContext {
    /// Create a new integration test context with real server and daemon.
    /// Performs full pairing setup: daemon-hello → register-device → pair → WS connect.
    pub async fn new_integration() -> Result<Self> {
        let mut env = IntegrationEnv::new()
            .await
            .context("creating integration environment")?;

        let server_url = env.server_url().to_string();
        let host_id = env.host_id;
        let device_id = env.device_id;
        let client = std::mem::replace(
            &mut env.client,
            MockClient::new(String::new(), String::new()),
        );
        let jwt_manager = Arc::clone(&env.jwt_manager);
        let pair_code = env.pair_code.clone();

        Ok(Self {
            mode: TestMode::Integration,
            server_url,
            host_id: Some(host_id),
            device_id: Some(device_id),
            client,
            jwt_manager: Some(jwt_manager),
            pair_code: Some(pair_code),
            env: Some(env),
        })
    }

    /// Create a remote test context connecting to existing server
    pub fn new_remote(
        server_url: String,
        _host_name: String,
        daemon_token: String,
        host_id: Option<Uuid>,
    ) -> Result<Self> {
        let client = MockClient::new(server_url.clone(), daemon_token);

        Ok(Self {
            mode: TestMode::Remote,
            server_url,
            host_id,
            device_id: None,
            client,
            jwt_manager: None,
            pair_code: None,
            env: None,
        })
    }

    /// Create a daemon JWT token (for integration testing)
    pub fn create_daemon_token(&self, host_id: Uuid) -> Result<String> {
        let jwt = self
            .jwt_manager
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no JWT manager available"))?;
        jwt.create_daemon_token(host_id, "test-host")
            .map_err(|e| anyhow::anyhow!("creating daemon JWT token: {e}"))
    }

    /// Get the database pool (integration mode only)
    pub fn pool(&self) -> Option<&db::DbPool> {
        self.env.as_ref().map(|e| e.pool())
    }

    /// Get the daemon PID (integration mode only)
    pub fn daemon_pid(&self) -> Option<u32> {
        self.env.as_ref().map(|e| e.daemon_pid())
    }

    /// Get the Hub reference (integration mode only)
    pub fn hub(&self) -> Option<&Arc<ve_server::hub::Hub>> {
        self.env.as_ref().map(|e| e.hub())
    }

    /// Get the temp directory path (integration mode only)
    pub fn temp_dir_path(&self) -> Option<&std::path::Path> {
        self.env.as_ref().map(|e| e.temp_dir_path())
    }

    /// Generate a unique workspace path under the temp directory (integration mode only).
    /// Falls back to `/tmp/` in remote mode.
    pub fn workspace_path(&self, name: &str) -> String {
        match self.temp_dir_path() {
            Some(p) => p.join(name).to_string_lossy().to_string(),
            None => format!("/tmp/{name}"),
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if let Some(mut env) = self.env.take() {
            // Terminate daemon subprocess
            if let Err(e) = env.daemon.terminate() {
                tracing::warn!("Failed to terminate daemon: {e}");
            }
            // Abort server task
            env.server.abort();
            // temp_dir is dropped here (TempDir auto-cleans on Drop)
        }

        tracing::info!("Test context cleaned up");
    }
}
