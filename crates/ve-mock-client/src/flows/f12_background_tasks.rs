//! F12: Background tasks — permission expiry and idempotency cleanup
//!
//! This flow starts a real integration server with background tasks enabled,
//! inserts stale rows, and waits for the scheduler ticks to converge DB state.

use std::sync::Arc;
use std::time::{Duration, Instant};

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
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("insert idempotency key: {e}"))?;

    let count_before: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM idempotency_keys WHERE key = $1")
            .bind(&ik_key)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("count idempotency key before scheduler: {e}"))?;

    if count_before.0 != 1 {
        anyhow::bail!("Idempotency key not found after insert");
    }

    tracing::info!("Idempotency key inserted, waiting for scheduler cleanup");

    wait_until(Duration::from_secs(10), || async {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idempotency_keys WHERE key = $1")
            .bind(&ik_key)
            .fetch_one(pool)
            .await?;
        Ok::<bool, sqlx::Error>(count.0 == 0)
    })
    .await
    .map_err(|e| anyhow::anyhow!("waiting for idempotency cleanup tick: {e}"))?;

    tracing::info!("Expired idempotency key verified deleted by scheduler");

    // ---- Part 2: Permission expiry ----
    let workspace_id = uuid::Uuid::new_v4();
    let session_id = uuid::Uuid::new_v4();
    let permission_id = uuid::Uuid::new_v4();

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

    tracing::info!("Stale permission request created, waiting for scheduler expiry");

    wait_until(Duration::from_secs(10), || async {
        let status: (String,) =
            sqlx::query_as("SELECT status FROM permission_requests WHERE permission_id = $1")
                .bind(permission_id.to_string())
                .fetch_one(pool)
                .await?;
        let pending_count: (i64,) =
            sqlx::query_as("SELECT pending_permission_count FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(pool)
                .await?;
        Ok::<bool, sqlx::Error>(status.0 == "expired" && pending_count.0 == 0)
    })
    .await
    .map_err(|e| anyhow::anyhow!("waiting for permission expiry tick: {e}"))?;

    tracing::info!(
        "Permission expiry verified: stale permission expired and session count updated by scheduler"
    );

    Ok(())
}

async fn wait_until<F, Fut, E>(timeout: Duration, mut check: F) -> anyhow::Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<bool, E>>,
    E: std::fmt::Display + Send + Sync + 'static,
{
    let deadline = Instant::now() + timeout;
    loop {
        if check().await.map_err(|err| anyhow::anyhow!("{err}"))? {
            return Ok(());
        }
        if Instant::now() > deadline {
            anyhow::bail!("condition not met within {timeout:?}");
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
