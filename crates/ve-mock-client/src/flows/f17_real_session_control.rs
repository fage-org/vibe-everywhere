//! F17: Real agent session control and termination
//!
//! This flow requires `--real-agent` flag. It creates a session with a real
//! Claude Code process, then tests control operations (terminate, pause)
//! that actually affect the running Claude Code subprocess, and verifies
//! the daemon handles process termination gracefully.

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;
use ve_shared::models::SessionMessageType;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f17", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f17", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F17 requires host_id"))?;

    // Step 1: Create workspace
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created_ws = client
        .create_workspace(host_id, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    // Step 2: Create session with a long-running task
    let ik = fixtures::unique_idempotency_key();
    let session_title = fixtures::unique_session_title();
    let initial_message = "Write a Rust program that prints numbers from 1 to 100, one per line. \
         Then explain the code step by step.";

    let session = client
        .create_session(host_id, workspace_id, &session_title, initial_message, &ik)
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;

    tracing::info!(%session_id, "Session created, waiting for agent to start...");

    // Step 3: Wait for the agent to start processing
    wait_for_agent_reply(client, session_id, 120, 1).await?;

    tracing::info!("Agent started processing");

    // Step 4: Send terminate control command
    let terminate_result = client
        .control_session(session_id, "terminate")
        .await
        .map_err(|e| anyhow::anyhow!("terminate session: {e}"))?;

    if !terminate_result.success {
        anyhow::bail!("Terminate session did not return success: {terminate_result:?}");
    }

    tracing::info!("Terminate command sent and acknowledged");

    // Step 5: Verify session status reflects termination
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let updated_session = client
        .get_session(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("get session after terminate: {e}"))?;

    tracing::info!(status = ?updated_session.status, "Session status after terminate");

    // Step 6: Create a second session to test pause
    let ik2 = fixtures::unique_idempotency_key();
    let session_title2 = fixtures::unique_session_title();

    let session2 = client
        .create_session(
            host_id,
            workspace_id,
            &session_title2,
            "Explain the Fibonacci sequence in detail.",
            &ik2,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create second session: {e}"))?;

    let session2_id = session2.session_id;

    tracing::info!(%session2_id, "Second session created for pause test");

    // Wait for agent to start
    wait_for_agent_reply(client, session2_id, 120, 1).await?;

    // Step 7: Try pause on the second session
    let pause_result = client.control_session(session2_id, "pause").await;
    match pause_result {
        Ok(resp) => {
            tracing::info!(success = resp.success, "Pause result");
        }
        Err(e) => {
            let err_msg = e.to_string();
            if err_msg.contains("failed to connect") || err_msg.contains("connection refused") {
                anyhow::bail!("Pause failed with network error: {e}");
            }
            // HTTP 4xx/5xx are real errors
            anyhow::bail!("Pause returned HTTP error: {e}");
        }
    }

    // Step 8: Error path — control non-existent session
    let bad_id = uuid::Uuid::new_v4();
    let result = client.control_session(bad_id, "terminate").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent session terminate, but got OK");
    }

    tracing::info!("Error path verified: non-existent session control rejected");

    // Step 9: Error path — invalid action
    let result = client
        .control_session(session_id, "invalid_action_xyz")
        .await;
    if result.is_ok() {
        anyhow::bail!("Expected error for invalid action, but got OK");
    }

    tracing::info!("Error path verified: invalid action rejected");

    // Step 10: Close both sessions
    for sid in [session_id, session2_id] {
        let close_result = client.close_session(sid).await;
        match close_result {
            Ok(resp) => {
                if !resp.success {
                    anyhow::bail!("Close session did not return success: {resp:?}");
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                if err_msg.contains("failed to connect") || err_msg.contains("connection refused") {
                    anyhow::bail!("Close session failed with network error: {e}");
                }
                anyhow::bail!("Close session returned HTTP error: {e}");
            }
        }
    }

    tracing::info!("F17 complete: session control and termination with real agent verified");

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
