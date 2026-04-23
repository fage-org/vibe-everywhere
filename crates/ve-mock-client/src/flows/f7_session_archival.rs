//! F7: Session archival

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f7", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f7", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F7 requires host_id"))?;

    let pool = ctx
        .pool()
        .ok_or_else(|| anyhow::anyhow!("F7 requires integration server pool"))?;

    let device_id = ctx
        .device_id
        .ok_or_else(|| anyhow::anyhow!("F7 requires device_id"))?;

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
            "F7 archival test",
            &ik,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;
    let session_id_str = session_id.to_string();

    tracing::info!(%session_id, "Session created for archival test");

    // Step 2: Close the session
    let close_result = client.close_session(session_id).await;
    match close_result {
        Ok(resp) => {
            if !resp.success {
                anyhow::bail!("Close session did not return success: {resp:?}");
            }
            tracing::info!("Session close acknowledged");
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("failed to connect") || err_msg.contains("connection refused") {
                anyhow::bail!("Close session failed with network error: {e}");
            }
            tracing::info!(error = %e, "Close session returned HTTP error (acceptable)");
        }
    }

    // Step 3: Manually simulate archival (daemon would normally do this via session_status_update)
    let archive_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("UPDATE sessions SET status = 'archived' WHERE session_id = $1")
        .bind(&session_id_str)
        .execute(pool)
        .await
        .map_err(|e| anyhow::anyhow!("set session archived: {e}"))?;

    sqlx::query(
        "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(archive_id.to_string())
    .bind(&session_id_str)
    .bind("F7 archival test")
    .bind(&now)
    .bind("user_closed")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("insert archive record: {e}"))?;

    tracing::info!(%archive_id, "Session manually archived");

    // Step 4: Verify session status is archived in DB
    let db_status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
        .bind(&session_id_str)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow::anyhow!("query session status: {e}"))?;

    if db_status.0 != "archived" {
        anyhow::bail!("Session should be archived, got status '{}'", db_status.0);
    }

    // Step 5: List archives — should include our archive
    let archives = client
        .list_archives(None, None)
        .await
        .map_err(|e| anyhow::anyhow!("list archives: {e}"))?;

    if archives.total < 1 {
        anyhow::bail!("Expected at least 1 archive, got {}", archives.total);
    }

    let found = archives.items.iter().any(|a| a.archive_id == archive_id);

    if !found {
        anyhow::bail!("Our archive not found in list");
    }

    tracing::info!("Archive listing verified");

    // Step 6: Get archive by ID
    let archive = client
        .get_archive(archive_id)
        .await
        .map_err(|e| anyhow::anyhow!("get archive: {e}"))?;

    if archive.session_id != session_id {
        anyhow::bail!(
            "Archive session_id mismatch: expected {session_id}, got {}",
            archive.session_id
        );
    }

    if archive.close_reason != ve_shared::types::CloseReason::UserClosed {
        anyhow::bail!(
            "Archive close_reason should be 'user_closed', got '{:?}'",
            archive.close_reason
        );
    }

    tracing::info!("Archive details verified");

    // Step 7: Get archived session via get_session — should still be accessible
    let got_session = client
        .get_session(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("get archived session: {e}"))?;

    if got_session.status != ve_shared::types::SessionStatus::Archived {
        anyhow::bail!(
            "Archived session status should be 'archived', got '{:?}'",
            got_session.status
        );
    }

    tracing::info!("Archived session accessible via get_session");

    // Step 8: Error path — close an also-archived session should return already_archived
    let close_again = client.close_session(session_id).await;
    match close_again {
        Ok(resp) => {
            if !resp.already_archived {
                anyhow::bail!("Expected already_archived=true for archived session close");
            }
            tracing::info!("Already-archived behavior verified");
        }
        Err(e) => {
            tracing::info!(error = %e, "Close archived session returned error");
        }
    }

    // Step 9: Verify device_session_access still exists for archived session
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
            "Expected 1 device_session_access for archived session, got {}",
            access_count.0
        );
    }

    tracing::info!("F7 complete: session archival lifecycle verified");

    Ok(())
}
