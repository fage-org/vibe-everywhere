//! F11: Daemon reconnection

use std::sync::Arc;

use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f11", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f11", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    if ctx.mode != crate::test_context::TestMode::Integration {
        anyhow::bail!("F11 requires integration mode");
    }

    let hub = ctx
        .hub()
        .ok_or_else(|| anyhow::anyhow!("F11 requires Hub reference"))?;

    let temp_dir_path = ctx
        .temp_dir_path()
        .ok_or_else(|| anyhow::anyhow!("F11 requires temp_dir path"))?;

    let old_pid = ctx
        .daemon_pid()
        .ok_or_else(|| anyhow::anyhow!("F11 requires daemon PID"))?;

    // Step 1: Verify daemon is currently connected
    let hosts_before = ctx
        .client
        .list_hosts()
        .await
        .map_err(|e| anyhow::anyhow!("list_hosts before disconnect: {e}"))?;

    if hosts_before.hosts.is_empty() {
        anyhow::bail!("Expected at least one paired host before disconnect");
    }

    let daemon_count = hub.connected_daemons().await.len();
    tracing::info!(
        daemon_count,
        old_pid,
        "Daemon is connected before disconnect"
    );

    // Step 2: SIGTERM the daemon
    #[cfg(unix)]
    {
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(old_pid as i32),
            nix::sys::signal::Signal::SIGTERM,
        )
        .map_err(|e| anyhow::anyhow!("SIGTERM daemon: {e}"))?;
    }

    #[cfg(not(unix))]
    {
        anyhow::bail!("F11 reconnection test requires Unix for SIGTERM");
    }

    // Step 3: Wait for daemon to disconnect from Hub
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("Daemon did not disconnect from Hub within 10s");
        }
        if hub.connected_daemons().await.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    tracing::info!("Daemon disconnected from Hub");

    // Step 4: Restart the daemon
    let _new_daemon = crate::daemon::IntegrationDaemon::spawn(&ctx.server_url, temp_dir_path)
        .await
        .map_err(|e| anyhow::anyhow!("restart daemon: {e}"))?;

    let new_pid = _new_daemon.process.id();

    // Step 5: Wait for daemon reconnection
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("Daemon did not reconnect within 15s");
        }
        if !hub.connected_daemons().await.is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    tracing::info!(new_pid, "Daemon reconnected");

    // Step 6: Verify host is still listed after reconnection
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let hosts_after = ctx
        .client
        .list_hosts()
        .await
        .map_err(|e| anyhow::anyhow!("list_hosts after reconnect: {e}"))?;

    if hosts_after.hosts.is_empty() {
        anyhow::bail!("No hosts listed after daemon reconnection");
    }

    tracing::info!(
        host_count = hosts_after.hosts.len(),
        "Daemon reconnection verified: host still listed"
    );

    // Step 7: Verify server healthz
    let health_url = format!("{}/healthz", ctx.server_url);
    let resp = reqwest::get(&health_url)
        .await
        .map_err(|e| anyhow::anyhow!("healthz request failed: {e}"))?;

    if !resp.status().is_success() {
        anyhow::bail!("Server healthz returned {}", resp.status());
    }

    let body = resp
        .text()
        .await
        .map_err(|e| anyhow::anyhow!("reading healthz body: {e}"))?;

    if body != "OK" {
        anyhow::bail!("Server healthz returned '{body}', expected 'OK'");
    }

    tracing::info!("Server healthz OK after reconnection");

    Ok(())
}
