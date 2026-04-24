//! F9: Archive browse & delete
//!
//! NOTE: This flow is marked as `integration-read-path`. It creates test fixtures
//! directly in the database to verify the read-side API path (browse/delete archives).
//! It does NOT exercise the full daemon → WS → server → DB write chain.

use std::sync::Arc;

use crate::flows::FlowResult;
use crate::test_context::TestContext;
use ve_server::config::DatabaseBackend;

struct ArchiveFixture {
    device_id: uuid::Uuid,
    host_id: uuid::Uuid,
    workspace_id: uuid::Uuid,
    session_id: uuid::Uuid,
    archive_id: uuid::Uuid,
    title: &'static str,
}

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f9", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f9", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F9 requires host_id (integration mode not set up)"))?;

    let device_id = ctx
        .device_id
        .ok_or_else(|| anyhow::anyhow!("F9 requires device_id (integration mode not set up)"))?;

    let pool = ctx.pool().ok_or_else(|| {
        anyhow::anyhow!("F9 requires integration server pool (integration mode not set up)")
    })?;
    let backend = ctx
        .database_backend()
        .ok_or_else(|| anyhow::anyhow!("F9 requires integration server backend"))?;

    // Step 1: List archives — should be empty initially
    let list = client
        .list_archives(None, None)
        .await
        .map_err(|e| anyhow::anyhow!("list_archives (empty): {e}"))?;

    if list.total != 0 {
        anyhow::bail!("Expected 0 archives initially, got {}", list.total);
    }

    tracing::info!("Archive list empty as expected");

    // Step 2: Create archive fixtures via DB (simulating archived sessions)
    let workspace_id = uuid::Uuid::new_v4();
    let session_id_1 = uuid::Uuid::new_v4();
    let archive_id_1 = uuid::Uuid::new_v4();
    let session_id_2 = uuid::Uuid::new_v4();
    let archive_id_2 = uuid::Uuid::new_v4();
    let workspace_path = format!("/tmp/archive-test-{workspace_id}");

    sqlx::query(
        "INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(workspace_id.to_string())
    .bind(host_id.to_string())
    .bind(&workspace_path)
    .bind("archive-test")
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("insert shared workspace fixture: {e}"))?;

    insert_archive(
        pool,
        backend,
        ArchiveFixture {
            device_id,
            host_id,
            workspace_id,
            session_id: session_id_1,
            archive_id: archive_id_1,
            title: "archive-test-session-1",
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!("insert archive fixture 1: {e}"))?;

    insert_archive(
        pool,
        backend,
        ArchiveFixture {
            device_id,
            host_id,
            workspace_id,
            session_id: session_id_2,
            archive_id: archive_id_2,
            title: "archive-test-session-2",
        },
    )
    .await
    .map_err(|e| anyhow::anyhow!("insert archive fixture 2: {e}"))?;

    tracing::info!("Created 2 archive fixtures");

    // Step 3: List archives — should now have 2
    let list = client
        .list_archives(None, None)
        .await
        .map_err(|e| anyhow::anyhow!("list_archives (after insert): {e}"))?;

    if list.total != 2 {
        anyhow::bail!("Expected 2 archives, got {}", list.total);
    }

    if list.items.len() != 2 {
        anyhow::bail!("Expected 2 items in page, got {}", list.items.len());
    }

    tracing::info!("List archives returns 2 items with correct total");

    // Step 4: Pagination test — limit=1
    let list = client
        .list_archives(Some(1), Some(1))
        .await
        .map_err(|e| anyhow::anyhow!("list_archives (page=1,limit=1): {e}"))?;

    if list.items.len() != 1 {
        anyhow::bail!("Expected 1 item with limit=1, got {}", list.items.len());
    }

    tracing::info!("Pagination verified: limit=1 returns 1 item");

    // Step 5: Get archive by ID
    let archive = client
        .get_archive(archive_id_1)
        .await
        .map_err(|e| anyhow::anyhow!("get_archive: {e}"))?;

    if archive.archive_id != archive_id_1 {
        anyhow::bail!(
            "get_archive returned wrong archive_id: {}",
            archive.archive_id
        );
    }

    tracing::info!("Get archive by ID verified");

    // Step 6: Get archive by non-existent ID — should 404
    let bad_id = uuid::Uuid::new_v4();
    let result = client.get_archive(bad_id).await;
    if result.is_ok() {
        anyhow::bail!("Expected 404 for non-existent archive, but got OK");
    }

    tracing::info!("Error path verified: non-existent archive returns 404");

    // Step 7: Batch delete — delete one archive
    let delete_resp = client
        .batch_delete_archives(vec![archive_id_1])
        .await
        .map_err(|e| anyhow::anyhow!("batch_delete: {e}"))?;

    if delete_resp.deleted_count != 1 {
        anyhow::bail!(
            "Expected deleted_count=1, got {}",
            delete_resp.deleted_count
        );
    }

    if !delete_resp.failed_ids.is_empty() {
        anyhow::bail!(
            "Expected no failed deletions, got: {:?}",
            delete_resp.failed_ids
        );
    }

    // Step 8: Verify archive was deleted
    let result = client.get_archive(archive_id_1).await;
    if result.is_ok() {
        anyhow::bail!("Archive should be deleted but still accessible");
    }

    // Step 9: Batch delete with non-existent ID — should fail gracefully
    let bad_id = uuid::Uuid::new_v4();
    let delete_resp = client
        .batch_delete_archives(vec![bad_id])
        .await
        .map_err(|e| anyhow::anyhow!("batch_delete (bad id): {e}"))?;

    if delete_resp.deleted_count != 0 {
        anyhow::bail!(
            "Expected deleted_count=0 for non-existent ID, got {}",
            delete_resp.deleted_count
        );
    }

    tracing::info!("Batch delete with non-existent ID returns 0 deleted");

    // Step 10: Verify remaining archive still exists
    let list = client
        .list_archives(None, None)
        .await
        .map_err(|e| anyhow::anyhow!("list_archives (after delete): {e}"))?;

    if list.total != 1 {
        anyhow::bail!("Expected 1 archive remaining, got {}", list.total);
    }

    tracing::info!("F9 complete: archive browse, pagination, and batch delete verified");

    Ok(())
}

/// Insert an archive fixture with all required referential integrity records
async fn insert_archive(
    pool: &sqlx::AnyPool,
    backend: DatabaseBackend,
    fixture: ArchiveFixture,
) -> anyhow::Result<()> {
    let now_expr = match backend {
        DatabaseBackend::Postgres => "CURRENT_TIMESTAMP",
        DatabaseBackend::Sqlite => "datetime('now')",
    };

    // Create session (archived status)
    sqlx::query(&format!(
        "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'archived', {now_expr}, {now_expr})",
    ))
    .bind(fixture.session_id.to_string())
    .bind(fixture.title)
    .bind(fixture.host_id.to_string())
    .bind(fixture.workspace_id.to_string())
    .bind("claude_code")
    .execute(pool)
    .await?;

    // Create device_session_access
    sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
        .bind(fixture.device_id.to_string())
        .bind(fixture.session_id.to_string())
        .execute(pool)
        .await?;

    // Create archive record
    sqlx::query(&format!(
        "INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id, created_at) VALUES ($1, $2, $3, {now_expr}, $4, $5, $6, {now_expr})",
    ))
    .bind(fixture.archive_id.to_string())
    .bind(fixture.session_id.to_string())
    .bind(fixture.title)
    .bind("user_closed")
    .bind(fixture.host_id.to_string())
    .bind(fixture.workspace_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}
