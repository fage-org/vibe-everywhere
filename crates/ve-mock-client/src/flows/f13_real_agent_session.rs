//! F13: Real agent session — end-to-end Claude Code integration test
//!
//! This flow requires `--real-agent` flag. It creates a workspace and session,
//! sends a real message to Claude Code CLI, and verifies the AI reply appears
//! in session_messages.

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;
use ve_shared::models::SessionMessageType;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f13", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f13", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F13 requires host_id"))?;

    // Step 1: Create workspace
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created_ws = client
        .create_workspace(host_id, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    tracing::info!(%workspace_id, path = %ws_path, "Workspace created");

    // Step 2: Create a session with a simple question
    let ik = fixtures::unique_idempotency_key();
    let session_title = fixtures::unique_session_title();
    let initial_message = "What is 2+2? Reply with just the number.";

    let session = client
        .create_session(host_id, workspace_id, &session_title, initial_message, &ik)
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;

    tracing::info!(%session_id, "Session created, waiting for agent to process...");

    // Step 3: Wait for the agent to produce a reply.
    // The real Claude Code agent processes the initial_message asynchronously.
    // Poll for session_messages until we see an Assistant-type message.
    let max_wait = std::time::Duration::from_secs(120);
    let poll_interval = std::time::Duration::from_secs(5);
    let deadline = std::time::Instant::now() + max_wait;

    let mut got_reply = false;
    while std::time::Instant::now() < deadline {
        tokio::time::sleep(poll_interval).await;

        let messages = client
            .list_messages(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("list messages: {e}"))?;

        // Look for a message from the assistant (not the user's initial message)
        let agent_messages: Vec<_> = messages
            .items
            .iter()
            .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
            .collect();

        if !agent_messages.is_empty() {
            tracing::info!(count = agent_messages.len(), "Agent messages found");
            got_reply = true;
            break;
        }

        tracing::debug!("No agent reply yet, polling again...");
    }

    if !got_reply {
        // Check session status for clues
        let s = client
            .get_session(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("get session: {e}"))?;

        anyhow::bail!(
            "No agent reply received within {max_wait:?}. Session status: {:?}",
            s.status
        );
    }

    tracing::info!("F13 complete: real Claude Code session verified");

    Ok(())
}
