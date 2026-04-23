//! F19: Real agent error handling and exception paths
//!
//! This flow requires `--real-agent` flag. It tests various error scenarios
//! with a real Claude Code process: invalid API usage, empty messages,
//! and verifies the server/daemon handle errors gracefully without crashing.

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;
use ve_shared::models::SessionMessageType;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f19", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f19", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F19 requires host_id"))?;

    // Step 1: Create workspace
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created_ws = client
        .create_workspace(host_id, &ws_name, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    // Step 2: Create a valid session first to verify daemon is healthy
    let ik = fixtures::unique_idempotency_key();
    let session_title = fixtures::unique_session_title();
    let initial_message = "Say hello and nothing else.";

    let session = client
        .create_session(host_id, workspace_id, &session_title, initial_message, &ik)
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;

    tracing::info!(%session_id, "Baseline session created");

    // Step 3: Wait for baseline reply — confirms daemon is healthy
    wait_for_agent_reply(client, session_id, 120, 1).await?;

    tracing::info!("Baseline agent reply received — daemon is healthy");

    // Step 4: Error path — send message to non-existent session
    let bad_session_id = uuid::Uuid::new_v4();
    let result = client.send_message(bad_session_id, "Hello").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent session message, but got OK");
    }
    tracing::info!("Error path verified: non-existent session message rejected");

    // Step 5: Error path — create session with empty initial message
    let ik2 = fixtures::unique_idempotency_key();
    let result = client
        .create_session(host_id, workspace_id, "empty message test", "", &ik2)
        .await;
    // The server may accept or reject empty messages — either behavior is valid
    match result {
        Ok(s) => {
            tracing::info!(session_id = %s.session_id, "Empty message session accepted (server allows)");
        }
        Err(e) => {
            tracing::info!(error = %e, "Empty message session rejected (server validates)");
        }
    }

    // Step 6: Error path — create session with non-existent workspace
    let ik3 = fixtures::unique_idempotency_key();
    let bad_workspace_id = uuid::Uuid::new_v4();
    let result = client
        .create_session(host_id, bad_workspace_id, "bad workspace", "test", &ik3)
        .await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent workspace, but got OK");
    }
    tracing::info!("Error path verified: non-existent workspace rejected");

    // Step 7: Error path — control non-existent session
    let result = client.control_session(bad_session_id, "pause").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent session control, but got OK");
    }
    tracing::info!("Error path verified: non-existent session control rejected");

    // Step 8: Error path — close non-existent session
    let result = client.close_session(bad_session_id).await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent session close, but got OK");
    }
    tracing::info!("Error path verified: non-existent session close rejected");

    // Step 9: Error path — list messages for non-existent session
    let result = client.list_messages(bad_session_id).await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent session messages, but got OK");
    }
    tracing::info!("Error path verified: non-existent session messages rejected");

    // Step 10: Verify the baseline session is still functional after error barrage
    let send_result = client
        .send_message(session_id, "What is 1+1? Reply with just the number.")
        .await
        .map_err(|e| anyhow::anyhow!("send follow-up message: {e}"))?;

    tracing::info!(message_id = %send_result.message_id, "Follow-up message sent");

    // Wait for the reply to confirm daemon hasn't crashed
    wait_for_agent_reply(client, session_id, 120, 2).await?;

    let messages = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list messages after error tests: {e}"))?;

    let assistant_count = messages
        .items
        .iter()
        .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
        .count();

    if assistant_count < 2 {
        anyhow::bail!(
            "Expected at least 2 assistant messages after error tests, got {assistant_count}"
        );
    }

    tracing::info!(
        assistant_message_count = assistant_count,
        "Daemon survived error barrage — still responding"
    );

    // Step 11: Clean up — close the baseline session
    let close_result = client.close_session(session_id).await;
    match close_result {
        Ok(resp) => {
            if !resp.success {
                tracing::warn!("Close session did not return success: {resp:?}");
            }
        }
        Err(e) => {
            tracing::info!(error = %e, "Close session returned HTTP error (acceptable)");
        }
    }

    tracing::info!("F19 complete: error handling and exception paths with real agent verified");

    Ok(())
}

async fn wait_for_agent_reply(
    client: &crate::client::MockClient,
    session_id: uuid::Uuid,
    timeout_secs: u64,
    min_count: usize,
) -> anyhow::Result<()> {
    let max_wait = std::time::Duration::from_secs(timeout_secs);
    let poll_interval = std::time::Duration::from_secs(5);
    let deadline = std::time::Instant::now() + max_wait;

    while std::time::Instant::now() < deadline {
        tokio::time::sleep(poll_interval).await;

        let messages = client
            .list_messages(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("list messages during poll: {e}"))?;

        let reply_count = messages
            .items
            .iter()
            .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
            .count();

        if reply_count >= min_count {
            return Ok(());
        }

        tracing::debug!(reply_count, "No agent reply yet, polling again...");
    }

    let s = client
        .get_session(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("get session: {e}"))?;

    anyhow::bail!(
        "No agent reply within {max_wait:?}. Session status: {:?}",
        s.status
    );
}
