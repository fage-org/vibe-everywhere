//! Real ve-daemon subprocess management for integration testing

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result};
use tokio::time::{sleep, Duration};

/// Integration test daemon subprocess
pub struct IntegrationDaemon {
    pub process: Child,
    pub log_path: PathBuf,
    pub config_path: PathBuf,
}

impl IntegrationDaemon {
    /// Spawn a ve-daemon subprocess pointing at the test server.
    /// When `mock_mode` is false, the daemon runs with real Claude Code (no VIBE_DAEMON__MOCK_MODE env var).
    pub async fn spawn(server_url: &str, temp_dir: &Path, mock_mode: bool) -> Result<Self> {
        let config_path = temp_dir.join("daemon-config.toml");
        // Use a persistent path for logs so they survive temp dir cleanup
        let log_path = PathBuf::from("/tmp/ve-mock-daemon.log");

        // Write daemon config for reference (daemon reads from env vars)
        let config_toml = format!(
            r#"
server_url = "{server_url}"
host_name = "test-host"
platform = "linux"
log_level = "debug"
reconnect_min_secs = 1
reconnect_max_secs = 2
"#
        );
        std::fs::write(&config_path, &config_toml).context("writing daemon config")?;

        // Clear previous log
        let _ = std::fs::remove_file(&log_path);
        // Create log file
        let log_file = std::fs::File::create(&log_path).context("creating log file")?;

        // Find ve-daemon binary
        let daemon_bin = find_daemon_binary()?;

        // Spawn subprocess with environment variables (daemon reads VIBE_DAEMON__* env vars).
        // The config_dir is set to temp_dir so credentials also land there.
        // VIBE_DAEMON__MOCK_MODE is only set when mock_mode is true, allowing real Claude Code testing.
        let mut cmd = Command::new(&daemon_bin);
        cmd.env("VIBE_DAEMON__SERVER_URL", server_url)
            .env("VIBE_DAEMON__HOST_NAME", "test-host")
            .env("VIBE_DAEMON__PLATFORM", "linux")
            .env("VIBE_DAEMON__LOG_LEVEL", "debug")
            .env("VIBE_DAEMON__RECONNECT_BACKOFF_MIN_MS", "1000")
            .env("VIBE_DAEMON__RECONNECT_BACKOFF_MAX_MS", "2000")
            .env(
                "VIBE_DAEMON__CONFIG_DIR",
                temp_dir.to_string_lossy().as_ref(),
            )
            // Override default model for test environments where Anthropic models may be unavailable
            .env("VIBE_DAEMON__DEFAULT_MODEL", "sonnet");
        if mock_mode {
            cmd.env("VIBE_DAEMON__MOCK_MODE", "true");
        }
        let process = cmd
            .stdout(Stdio::from(log_file.try_clone()?))
            .stderr(Stdio::from(log_file))
            .spawn()
            .context("spawning ve-daemon subprocess")?;

        tracing::info!("Daemon subprocess started (PID: {:?})", process.id());

        let daemon = Self {
            process,
            log_path: log_path.clone(),
            config_path,
        };

        // Wait for daemon to connect by checking the log file.
        // This is a first-pass check; TestContext additionally verifies via
        // Hub.connected_daemons() after this returns.
        wait_for_daemon_hello(&log_path, Duration::from_secs(15)).await?;

        Ok(daemon)
    }

    /// Terminate the daemon subprocess
    pub fn terminate(&mut self) -> Result<()> {
        #[cfg(unix)]
        {
            nix::sys::signal::kill(
                nix::unistd::Pid::from_raw(self.process.id() as i32),
                nix::sys::signal::Signal::SIGTERM,
            )
            .ok();
        }

        #[cfg(not(unix))]
        {
            let _ = self.process.kill();
        }

        // Wait up to 5 seconds
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            match self.process.try_wait() {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => break,
            }
        }

        // Force kill
        let _ = self.process.kill();
        let _ = self.process.wait();
        Ok(())
    }
}

impl Drop for IntegrationDaemon {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}

/// Find the ve-daemon binary
fn find_daemon_binary() -> Result<PathBuf> {
    // Allow override via environment variable (useful for CI with custom target dirs)
    if let Ok(path) = std::env::var("VE_DAEMON_BIN") {
        let p = PathBuf::from(&path);
        if p.exists() {
            return Ok(p);
        }
    }

    let workspace_root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/ve-daemon");

    if workspace_root.exists() {
        return Ok(workspace_root);
    }

    let release_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/ve-daemon");
    if release_path.exists() {
        return Ok(release_path);
    }

    tracing::warn!("ve-daemon binary not found, building now...");
    let status = Command::new("cargo")
        .args(["build", "-p", "ve-daemon"])
        .status()
        .context("building ve-daemon")?;

    if !status.success() {
        anyhow::bail!("Failed to build ve-daemon");
    }

    Ok(workspace_root)
}

/// Wait for the daemon to log a successful connection
async fn wait_for_daemon_hello(log_path: &Path, timeout: Duration) -> Result<()> {
    let deadline = std::time::Instant::now() + timeout;

    loop {
        if std::time::Instant::now() > deadline {
            let log_content = std::fs::read_to_string(log_path).unwrap_or_default();
            anyhow::bail!(
                "Daemon did not connect within {:?}. Last log:\n{}",
                timeout,
                last_n_lines(&log_content, 20)
            );
        }

        if let Ok(content) = std::fs::read_to_string(log_path) {
            if content.contains("PAIRING REQUIRED")
                || content.contains("daemon_hello")
                || content.contains("daemon hello")
                || content.contains("entering pairing mode")
            {
                tracing::info!("Daemon connected to server");
                return Ok(());
            }
            if content.contains("connection refused")
                || content.contains("invalid token")
                || content.contains("authentication failed")
            {
                anyhow::bail!(
                    "Daemon connection failed. Log:\n{}",
                    last_n_lines(&content, 20)
                );
            }
        }

        sleep(Duration::from_millis(200)).await;
    }
}

fn last_n_lines(content: &str, n: usize) -> String {
    content
        .lines()
        .rev()
        .take(n)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}
