//! F6: Permission request/response
//!
//! This flow drives the mock daemon permission path end-to-end:
//! mock driver -> daemon event bus -> daemon WS -> server DB/API -> permission response.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

const MOCK_PERMISSION_TRIGGER: &str = "__VE_MOCK_PERMISSION__";

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f6", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f6", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F6 requires host_id"))?;

    let pool = ctx
        .pool()
        .ok_or_else(|| anyhow::anyhow!("F6 requires integration server pool"))?;

    let _device_id = ctx
        .device_id
        .ok_or_else(|| anyhow::anyhow!("F6 requires device_id"))?;

    // Step 1: Create workspace and session, then trigger permissions through follow-up
    // messages so the session is fully established before the daemon writes back.
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
            "F6 permission test",
            &ik,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;
    let session_id_str = session_id.to_string();

    // Step 2: Trigger the first permission through the daemon write path.
    let permission_count_before = count_permissions(client, session_id).await?;
    client
        .send_message(session_id, MOCK_PERMISSION_TRIGGER)
        .await
        .map_err(|e| anyhow::anyhow!("send first permission trigger message: {e}"))?;

    // Step 3: Wait for the daemon -> WS -> server write chain to persist a permission request.
    let permission_id =
        wait_for_pending_permission(client, session_id, permission_count_before, &[]).await?;
    tracing::info!(%permission_id, "First permission request arrived through the real write path");

    let pending_count: (i64,) =
        sqlx::query_as("SELECT pending_permission_count FROM sessions WHERE session_id = $1")
            .bind(&session_id_str)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("query pending count after write path: {e}"))?;

    if pending_count.0 < 1 {
        anyhow::bail!(
            "Expected pending_permission_count >= 1 after daemon write path, got {}",
            pending_count.0
        );
    }

    // Step 4: List permissions — should include our daemon-originated request.
    let permissions = client
        .list_permissions(Some(session_id))
        .await
        .map_err(|e| anyhow::anyhow!("list permissions: {e}"))?;

    if permissions.is_empty() {
        anyhow::bail!("Expected at least 1 permission, got none");
    }

    let our_perm = permissions
        .iter()
        .find(|permission| permission.permission_id == permission_id)
        .ok_or_else(|| anyhow::anyhow!("Our daemon-originated permission not found in list"))?;

    if our_perm.summary.is_empty() {
        anyhow::bail!("Permission summary should be populated");
    }

    tracing::info!("Permission list verified");

    // Step 5: Respond with approve_once.
    let response = client
        .respond_permission(
            permission_id,
            ve_shared::models::PermissionDecision::ApproveOnce,
            Some("F6 integration test approval"),
        )
        .await
        .map_err(|e| anyhow::anyhow!("respond permission: {e}"))?;

    if response.status != ve_shared::types::PermissionStatus::ApprovedOnce {
        anyhow::bail!(
            "Permission respond did not return approved_once status: {:?}",
            response.status
        );
    }

    let status_after: (String,) =
        sqlx::query_as("SELECT status FROM permission_requests WHERE permission_id = $1")
            .bind(permission_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("query permission status after respond: {e}"))?;

    if status_after.0 != "approved_once" {
        anyhow::bail!(
            "Permission status should be 'approved_once', got '{}'",
            status_after.0
        );
    }

    tracing::info!("Permission status changed to 'approved_once'");

    // Step 6: Trigger a second permission through the same daemon path, then deny it.
    let permission_count_before = count_permissions(client, session_id).await?;
    client
        .send_message(session_id, MOCK_PERMISSION_TRIGGER)
        .await
        .map_err(|e| anyhow::anyhow!("send permission trigger message: {e}"))?;

    let deny_permission_id = wait_for_pending_permission(
        client,
        session_id,
        permission_count_before,
        &[permission_id],
    )
    .await?;
    tracing::info!(%deny_permission_id, "Second permission request arrived through the real write path");

    let deny_response = client
        .respond_permission(
            deny_permission_id,
            ve_shared::models::PermissionDecision::DenyOnce,
            Some("F6 integration test denial"),
        )
        .await
        .map_err(|e| anyhow::anyhow!("respond deny: {e}"))?;

    if deny_response.status != ve_shared::types::PermissionStatus::DeniedOnce {
        anyhow::bail!(
            "Deny respond did not return denied status: {:?}",
            deny_response.status
        );
    }

    let deny_status_after: (String,) =
        sqlx::query_as("SELECT status FROM permission_requests WHERE permission_id = $1")
            .bind(deny_permission_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("query deny status after respond: {e}"))?;

    if deny_status_after.0 != "denied_once" {
        anyhow::bail!(
            "Permission status should be 'denied_once', got '{}'",
            deny_status_after.0
        );
    }

    tracing::info!("Permission DenyOnce happy path verified");

    // Step 7: Error path — respond to non-existent permission.
    let bad_id = uuid::Uuid::new_v4();
    let result = client
        .respond_permission(
            bad_id,
            ve_shared::models::PermissionDecision::DenyOnce,
            None,
        )
        .await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent permission respond, but got OK");
    }

    tracing::info!("Error path verified: non-existent permission rejected");

    // Step 8: Idempotent re-respond — responding again returns current state.
    let re_result = client
        .respond_permission(
            permission_id,
            ve_shared::models::PermissionDecision::DenyOnce,
            None,
        )
        .await
        .map_err(|e| anyhow::anyhow!("re-respond already-responded permission: {e}"))?;
    assert_eq!(
        re_result.status,
        ve_shared::types::PermissionStatus::ApprovedOnce
    );

    tracing::info!(
        "Idempotent re-respond verified: returns original state regardless of new decision"
    );

    Ok(())
}

async fn wait_for_pending_permission(
    client: &crate::client::MockClient,
    session_id: uuid::Uuid,
    count_before: usize,
    exclude_ids: &[uuid::Uuid],
) -> anyhow::Result<uuid::Uuid> {
    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        if Instant::now() > deadline {
            anyhow::bail!("Timed out waiting for daemon-originated permission request");
        }

        let permissions = client
            .list_permissions(Some(session_id))
            .await
            .map_err(|e| anyhow::anyhow!("list permissions while polling: {e}"))?;

        if permissions.len() > count_before + 1 {
            anyhow::bail!(
                "Expected exactly one new permission request, but count grew from {} to {}",
                count_before,
                permissions.len()
            );
        }

        if let Some(permission) = permissions.iter().find(|permission| {
            permission.status == ve_shared::types::PermissionStatus::Pending
                && !exclude_ids.contains(&permission.permission_id)
        }) {
            if permissions.len() != count_before + 1 {
                anyhow::bail!(
                    "Permission arrived before list count stabilized: before={} after={}",
                    count_before,
                    permissions.len()
                );
            }
            return Ok(permission.permission_id);
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn count_permissions(
    client: &crate::client::MockClient,
    session_id: uuid::Uuid,
) -> anyhow::Result<usize> {
    client
        .list_permissions(Some(session_id))
        .await
        .map(|permissions| permissions.len())
        .map_err(|e| anyhow::anyhow!("count permissions: {e}"))
}
