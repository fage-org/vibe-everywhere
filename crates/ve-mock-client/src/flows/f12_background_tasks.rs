//! F12: Background tasks — permission expiry and idempotency cleanup

use std::sync::Arc;

use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f12", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f12", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let pool = ctx
        .pool()
        .ok_or_else(|| anyhow::anyhow!("F12 requires integration server pool"))?;

    let device_id = ctx
        .device_id
        .ok_or_else(|| anyhow::anyhow!("F12 requires device_id"))?;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F12 requires host_id"))?;

    // ---- Part 1: Idempotency key cleanup ----

    // Step 1: Create an idempotency key directly in the DB
    let ik_key = format!("ik-{}", uuid::Uuid::new_v4());
    let ik_hash = format!("hash-{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO idempotency_keys (key, request_hash, session_id, result_type, created_at, expires_at) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(&ik_key)
    .bind(&ik_hash)
    .bind("fake-session-id")
    .bind("session")
    .bind(&now)
    .bind(&now) // expires_at = now → already expired
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("insert idempotency key: {e}"))?;

    // Verify it exists
    let count_before: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM idempotency_keys WHERE key = $1")
            .bind(&ik_key)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("count idempotency key: {e}"))?;

    if count_before.0 != 1 {
        anyhow::bail!("Idempotency key not found after insert");
    }

    tracing::info!("Idempotency key inserted");

    // Step 2: Run the actual server-side cleanup function
    let deleted = ve_server::tasks::cleanup_expired_keys(pool)
        .await
        .map_err(|e| anyhow::anyhow!("run idempotency cleanup: {e}"))?;

    if deleted != 1 {
        anyhow::bail!("Expected 1 expired key deleted, got {deleted}");
    }

    tracing::info!(deleted, "Idempotency cleanup task ran");

    // Step 3: Verify the expired key was actually removed
    let count_after: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM idempotency_keys WHERE key = $1")
            .bind(&ik_key)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("count idempotency key after cleanup: {e}"))?;

    if count_after.0 != 0 {
        anyhow::bail!(
            "Idempotency key should be deleted, still exists (count={})",
            count_after.0
        );
    }

    tracing::info!("Expired idempotency key verified deleted");

    // ---- Part 2: Permission expiry ----

    // Step 1: Create a session and a stale permission request
    let workspace_id = uuid::Uuid::new_v4();
    let session_id = uuid::Uuid::new_v4();
    let permission_id = uuid::Uuid::new_v4();

    // Ensure prerequisite records exist
    sqlx::query("INSERT OR IGNORE INTO device_host_access (device_id, host_id) VALUES ($1, $2)")
        .bind(device_id.to_string())
        .bind(host_id.to_string())
        .execute(pool)
        .await?;

    sqlx::query(
        "INSERT OR IGNORE INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .bind("/tmp/perm-test")
    .bind("perm-test")
    .execute(pool)
    .await?;

    // Create a session in waiting_approval status
    let old_time = (chrono::Utc::now() - chrono::Duration::hours(2)).to_rfc3339();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, pending_permission_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'waiting_approval', 1, $6, $6)",
    )
    .bind(session_id.to_string())
    .bind("perm-test-session")
    .bind(host_id.to_string())
    .bind(workspace_id.to_string())
    .bind("claude_code")
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("insert session: {e}"))?;

    // Create a stale permission request (2 hours old, TTL is 30 min)
    sqlx::query(
        "INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, status, created_at) VALUES ($1, $2, $3, $4, 'pending', $5)",
    )
    .bind(permission_id.to_string())
    .bind(session_id.to_string())
    .bind("exec_cmd")
    .bind("stale permission test")
    .bind(&old_time)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("insert permission request: {e}"))?;

    // Verify permission is pending
    let status_before: (String,) =
        sqlx::query_as("SELECT status FROM permission_requests WHERE permission_id = $1")
            .bind(permission_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("query permission status: {e}"))?;

    if status_before.0 != "pending" {
        anyhow::bail!("Permission should be pending, got: {}", status_before.0);
    }

    tracing::info!("Stale permission request created");

    // Step 2: Run the actual server-side permission expiry function
    let ttl_secs: u64 = 1800; // 30 minutes
    let expired = ve_server::tasks::expire_stale_permissions(pool, ttl_secs)
        .await
        .map_err(|e| anyhow::anyhow!("run permission expiry: {e}"))?;

    if expired != 1 {
        anyhow::bail!("Expected 1 permission expired, got {expired}");
    }

    // Step 3: Verify permission is now expired
    let status_after: (String,) =
        sqlx::query_as("SELECT status FROM permission_requests WHERE permission_id = $1")
            .bind(permission_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("query permission status after expiry: {e}"))?;

    if status_after.0 != "expired" {
        anyhow::bail!("Permission should be expired, got: {}", status_after.0);
    }

    // Step 4: Verify session pending_permission_count was updated
    let pending_count: (i64,) =
        sqlx::query_as("SELECT pending_permission_count FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("query session pending count: {e}"))?;

    if pending_count.0 != 0 {
        anyhow::bail!(
            "Session pending_permission_count should be 0, got {}",
            pending_count.0
        );
    }

    tracing::info!("Permission expiry verified: stale permission expired, session count updated");

    Ok(())
}
