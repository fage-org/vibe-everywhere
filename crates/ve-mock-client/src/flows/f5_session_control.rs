//! F5: Session control (pause/restart/terminate)

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;
use ve_shared::types::SessionStatus;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f5", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f5", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F5 requires host_id"))?;

    let _pool = ctx
        .pool()
        .ok_or_else(|| anyhow::anyhow!("F5 requires integration server pool"))?;

    // Step 1: Create workspace and session
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created_ws = client
        .create_workspace(host_id, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    let ik = fixtures::unique_idempotency_key();
    let session = client
        .create_session(
            host_id,
            workspace_id,
            &fixtures::unique_session_title(),
            "F5 control test",
            &ik,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;

    tracing::info!(%session_id, status = ?session.status, "Session created for control test");

    // Step 2: Pause the session
    let pause_result = client.control_session(session_id, "pause").await;

    match pause_result {
        Ok(resp) => {
            tracing::info!(success = resp.success, "Pause action result");
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("failed to connect") || err_msg.contains("connection refused") {
                anyhow::bail!("Pause control failed with network error (server unreachable): {e}");
            }
            // HTTP 4xx/5xx are real errors — the endpoint should respond
            anyhow::bail!("Pause control returned HTTP error: {e}");
        }
    }

    // Step 3: Session should now be paused
    let paused = client
        .get_session(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("get session: {e}"))?;
    ensure_status("pause", paused.status, SessionStatus::Paused)?;

    tracing::info!(%session_id, status = ?paused.status, "Session paused");

    // Step 4: Restart should either resume the session or fail explicitly while
    // leaving the session paused when no resumable Claude session ID exists.
    let restart_result = client.control_session(session_id, "restart").await;

    match restart_result {
        Ok(resp) => {
            tracing::info!(success = resp.success, "Restart action result");

            let restarted = client
                .get_session(session_id)
                .await
                .map_err(|e| anyhow::anyhow!("get session after restart: {e}"))?;
            ensure_status("restart", restarted.status, SessionStatus::Running)?;
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("failed to connect") || err_msg.contains("connection refused") {
                anyhow::bail!(
                    "Restart control failed with network error (server unreachable): {e}"
                );
            }

            let still_paused = client.get_session(session_id).await.map_err(|fetch_err| {
                anyhow::anyhow!("get session after restart failure: {fetch_err}")
            })?;
            ensure_status(
                "restart failure",
                still_paused.status,
                SessionStatus::Paused,
            )?;
            tracing::info!("Restart rejected explicitly: {e}");
        }
    }

    // Step 5: Error path — control non-existent session
    let bad_id = uuid::Uuid::new_v4();
    let result = client.control_session(bad_id, "pause").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent session control, but got OK");
    }

    tracing::info!("Error path verified: non-existent session control rejected");

    // Step 6: Error path — invalid action
    let result = client
        .control_session(session_id, "invalid_action_xyz")
        .await;
    if result.is_ok() {
        anyhow::bail!("Expected error for invalid control action, but got OK");
    }

    tracing::info!("Error path verified: invalid action rejected");

    // Step 7: Rerun on non-archived session should fail
    let result = client.control_session(session_id, "rerun").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for rerun on non-archived session, but got OK");
    }

    tracing::info!("Error path verified: rerun on non-archived session rejected");

    // Step 8: Close session
    let close_result = client.close_session(session_id).await;
    match close_result {
        Ok(resp) => {
            tracing::info!(success = resp.success, "Close session result");
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("failed to connect") || err_msg.contains("connection refused") {
                anyhow::bail!("Close session failed with network error: {e}");
            }
            // HTTP 4xx/5xx are real errors — the endpoint should respond
            anyhow::bail!("Close session returned HTTP error: {e}");
        }
    }

    wait_for_status(
        client,
        session_id,
        SessionStatus::Archived,
        Duration::from_secs(5),
    )
    .await?;

    Ok(())
}

fn ensure_status(
    action: &str,
    actual: SessionStatus,
    expected: SessionStatus,
) -> anyhow::Result<()> {
    if actual != expected {
        anyhow::bail!(
            "session status after {action} should be {:?}, got {:?}",
            expected,
            actual
        );
    }
    Ok(())
}

async fn wait_for_status(
    client: &crate::client::MockClient,
    session_id: uuid::Uuid,
    expected: SessionStatus,
    timeout: Duration,
) -> anyhow::Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        let session = client
            .get_session(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("get session while waiting for status: {e}"))?;
        if session.status == expected {
            return Ok(());
        }
        if Instant::now() >= deadline {
            anyhow::bail!(
                "session status after close should be {:?}, got {:?}",
                expected,
                session.status
            );
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_status_accepts_matching_state() {
        assert!(ensure_status("pause", SessionStatus::Paused, SessionStatus::Paused).is_ok());
    }

    #[test]
    fn ensure_status_rejects_unexpected_state() {
        let error = ensure_status("pause", SessionStatus::Running, SessionStatus::Paused)
            .unwrap_err()
            .to_string();
        assert!(error.contains("should be"));
        assert!(error.contains("Paused"));
        assert!(error.contains("Running"));
    }
}
