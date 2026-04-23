//! F2: Host & Workspace CRUD

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f2", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f2", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    // Require host_id from integration setup
    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F2 requires a host_id (integration mode not set up)"))?;

    // Step 1: List hosts — should include the paired daemon host
    let hosts = client
        .list_hosts()
        .await
        .map_err(|e| anyhow::anyhow!("list_hosts: {e}"))?;

    if hosts.hosts.is_empty() {
        anyhow::bail!("Expected at least one paired host, got none");
    }

    tracing::info!("Found {} paired host(s)", hosts.hosts.len());

    // Verify our host is in the list
    let found = hosts.hosts.iter().any(|h| h.host_id == host_id);
    if !found {
        anyhow::bail!("Our host_id {host_id} not found in host list");
    }

    // Step 2: Create workspace
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created = client
        .create_workspace(host_id, &ws_path, Some(&ws_name))
        .await
        .map_err(|e| anyhow::anyhow!("create_workspace: {e}"))?;

    let workspace_id = created.workspace_id;

    tracing::info!("Created workspace {workspace_id} at {ws_path}");

    // Step 3: List workspaces — should include the new one
    let workspaces = client
        .list_workspaces(host_id)
        .await
        .map_err(|e| anyhow::anyhow!("list_workspaces: {e}"))?;

    if workspaces.is_empty() {
        anyhow::bail!("Expected at least one workspace, got none");
    }

    // Verify our workspace is in the list
    let found = workspaces.iter().any(|w| w.workspace_id == workspace_id);
    if !found {
        anyhow::bail!("Our workspace_id {workspace_id} not found in workspace list");
    }

    tracing::info!("Workspace verified in list_workspaces");

    // Step 4: Get workspace by ID
    let got = client
        .get_workspace(workspace_id)
        .await
        .map_err(|e| anyhow::anyhow!("get_workspace: {e}"))?;

    if got.path != ws_path {
        anyhow::bail!(
            "get_workspace path mismatch: expected {ws_path}, got {}",
            got.path
        );
    }

    // Step 5: Update workspace
    let updated_name = format!("updated-{ws_name}");
    let updated = client
        .update_workspace(workspace_id, &updated_name, &ws_path)
        .await
        .map_err(|e| anyhow::anyhow!("update_workspace: {e}"))?;

    if updated.display_name != updated_name {
        anyhow::bail!(
            "update_workspace display_name mismatch: expected {updated_name}, got {}",
            updated.display_name
        );
    }

    // Step 6: Delete workspace
    client
        .delete_workspace(workspace_id)
        .await
        .map_err(|e| anyhow::anyhow!("delete_workspace: {e}"))?;

    // Step 7: Verify 404 after delete
    let result = client.get_workspace(workspace_id).await;
    if result.is_ok() {
        anyhow::bail!("Expected 404 after workspace delete, but got OK");
    }

    tracing::info!("Workspace CRUD verified: create → read → update → delete → 404");

    // Step 8: Error path — create workspace with invalid host_id
    let bad_host_id = uuid::Uuid::new_v4();
    let result = client
        .create_workspace(bad_host_id, "/tmp/bad", Some("bad-host"))
        .await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent host_id, but got OK");
    }

    tracing::info!("Error path verified: invalid host_id rejected");

    Ok(())
}
