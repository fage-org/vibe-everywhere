//! F6: Permission request/response

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

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

    // Step 1: Create workspace and session (prerequisites)
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
            "F6 permission test",
            &ik,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;
    let session_id_str = session_id.to_string();

    // Step 2: Insert a permission request fixture (simulating daemon-sent permission)
    let permission_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, target, status, created_at) VALUES ($1, $2, $3, $4, $5, 'pending', $6)",
    )
    .bind(permission_id.to_string())
    .bind(&session_id_str)
    .bind("exec_cmd")
    .bind("F6 integration test permission")
    .bind(Option::<String>::None)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("insert permission fixture: {e}"))?;

    // Note: pending_permission_count is set during session creation but our manual
    // insert may not be reflected. That's OK — we verify the respond flow works.
    let _pending_count: (i64,) =
        sqlx::query_as("SELECT pending_permission_count FROM sessions WHERE session_id = $1")
            .bind(&session_id_str)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("query pending count: {e}"))?;

    // Initial count is set to 1 during session creation, but we manually
    // inserted a permission after. The count might not reflect our manual insert.
    // That's OK — we just need to verify the respond flow works.

    tracing::info!(%permission_id, "Permission fixture inserted");

    // Step 3: List permissions — should include our fixture
    let permissions = client
        .list_permissions(Some(session_id))
        .await
        .map_err(|e| anyhow::anyhow!("list permissions: {e}"))?;

    if permissions.is_empty() {
        anyhow::bail!("Expected at least 1 permission, got none");
    }

    let found = permissions.iter().any(|p| p.permission_id == permission_id);

    if !found {
        anyhow::bail!("Our permission not found in list");
    }

    tracing::info!("Permission list verified");

    // Step 4: Get permission by ID
    let all_perms = client
        .list_permissions(None)
        .await
        .map_err(|e| anyhow::anyhow!("list all permissions: {e}"))?;

    let our_perm = all_perms.iter().find(|p| p.permission_id == permission_id);

    if our_perm.is_none() {
        anyhow::bail!("Our permission not found in global list");
    }

    // Step 5: Respond with approve_once
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

    // Step 6: Verify status changed in DB
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

    // Step 6b: Test DenyOnce happy path — create and deny a second permission
    let deny_permission_id = uuid::Uuid::new_v4();
    let now2 = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, target, status, created_at) VALUES ($1, $2, $3, $4, $5, 'pending', $6)",
    )
    .bind(deny_permission_id.to_string())
    .bind(&session_id_str)
    .bind("exec_cmd")
    .bind("F6 deny test permission")
    .bind(Option::<String>::None)
    .bind(&now2)
    .execute(pool)
    .await
    .map_err(|e| anyhow::anyhow!("insert deny permission: {e}"))?;

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

    // Step 7: Error path — respond to non-existent permission
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

    // Step 8: Idempotent re-respond — responding again returns current state
    let re_result = client
        .respond_permission(
            permission_id,
            ve_shared::models::PermissionDecision::DenyOnce,
            None,
        )
        .await
        .map_err(|e| anyhow::anyhow!("re-respond already-responded permission: {e}"))?;
    // Idempotent: re-responding returns the original state (ApprovedOnce), not the new decision
    assert_eq!(re_result.status, ve_shared::types::PermissionStatus::ApprovedOnce);

    tracing::info!("Idempotent re-respond verified: returns original state regardless of new decision");

    Ok(())
}
