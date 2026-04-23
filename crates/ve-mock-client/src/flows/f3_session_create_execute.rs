//! F3: Session create & execute

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f3", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f3", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F3 requires host_id (integration mode not set up)"))?;

    let pool = ctx
        .pool()
        .ok_or_else(|| anyhow::anyhow!("F3 requires integration server pool"))?;

    let device_id = ctx
        .device_id
        .ok_or_else(|| anyhow::anyhow!("F3 requires device_id"))?;

    // Step 1: Create a workspace for the session
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created_ws = client
        .create_workspace(host_id, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    // Step 2: Create a session
    let session_title = fixtures::unique_session_title();
    let ik = fixtures::unique_idempotency_key();

    let session = client
        .create_session(
            host_id,
            workspace_id,
            &session_title,
            "Hello from integration test",
            &ik,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;
    let session_id_str = session_id.to_string();

    let status = format!("{:?}", session.status);

    tracing::info!(%session_id, %status, "Session created");

    // Step 3: Verify session is in DB
    let db_status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
        .bind(&session_id_str)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow::anyhow!("query session status: {e}"))?;

    tracing::info!(%session_id, db_status = %db_status.0, "Session DB status verified");

    // Step 4: Verify device_session_access was created
    let access_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM device_session_access WHERE session_id = $1 AND device_id = $2",
    )
    .bind(&session_id_str)
    .bind(device_id.to_string())
    .fetch_one(pool)
    .await
    .map_err(|e| anyhow::anyhow!("query session access: {e}"))?;

    if access_count.0 != 1 {
        anyhow::bail!(
            "Expected 1 device_session_access row, got {}",
            access_count.0
        );
    }

    // Step 5: Idempotency test — same key should return same session
    let session_dup = client
        .create_session(
            host_id,
            workspace_id,
            &session_title,
            "Hello from integration test",
            &ik,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create session (idempotent): {e}"))?;

    if session_dup.session_id != session_id {
        anyhow::bail!(
            "Idempotency failed: expected same session_id {session_id}, got {}",
            session_dup.session_id
        );
    }

    tracing::info!("Session idempotency verified: same key returns same session");

    // Step 6: List sessions — should include our session
    let sessions = client
        .list_sessions()
        .await
        .map_err(|e| anyhow::anyhow!("list sessions: {e}"))?;

    if sessions.is_empty() {
        anyhow::bail!("Expected at least 1 session, got none");
    }

    let found = sessions.iter().any(|s| s.session_id == session_id);

    if !found {
        anyhow::bail!("Our session not found in session list");
    }

    // Step 7: Error path — create session with non-existent workspace
    let bad_workspace_id = uuid::Uuid::new_v4();
    let ik2 = fixtures::unique_idempotency_key();
    let result = client
        .create_session(
            host_id,
            bad_workspace_id,
            "bad workspace",
            "test message",
            &ik2,
        )
        .await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent workspace, but got OK");
    }

    tracing::info!("Error path verified: non-existent workspace rejected");

    Ok(())
}
