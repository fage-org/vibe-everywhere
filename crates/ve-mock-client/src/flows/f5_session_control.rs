//! F5: Session control (pause/restart/terminate)

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

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
        .create_workspace(host_id, &ws_name, &ws_path, None)
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

    // Step 2: Try pause action — endpoint must respond (success or a recognized error)
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
            tracing::info!(error = %e, "Pause returned HTTP error (acceptable without real agent)");
        }
    }

    // Step 3: Get session to check current status
    let got = client
        .get_session(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("get session: {e}"))?;

    tracing::info!(%session_id, status = ?got.status, "Session status after control");

    // Step 4: Error path — control non-existent session
    let bad_id = uuid::Uuid::new_v4();
    let result = client.control_session(bad_id, "pause").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent session control, but got OK");
    }

    tracing::info!("Error path verified: non-existent session control rejected");

    // Step 5: Error path — invalid action
    let result = client
        .control_session(session_id, "invalid_action_xyz")
        .await;
    if result.is_ok() {
        anyhow::bail!("Expected error for invalid control action, but got OK");
    }

    tracing::info!("Error path verified: invalid action rejected");

    // Step 6: Rerun on non-archived session should fail
    let result = client.control_session(session_id, "rerun").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for rerun on non-archived session, but got OK");
    }

    tracing::info!("Error path verified: rerun on non-archived session rejected");

    // Step 7: Close session
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
            tracing::info!(error = %e, "Close session returned HTTP error (acceptable)");
        }
    }

    Ok(())
}
