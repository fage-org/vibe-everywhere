//! F14: Real agent multi-turn conversation
//!
//! This flow requires `--real-agent` flag. It creates a workspace and session,
//! verifies the initial AI reply, then sends additional messages via the
//! `send_message` API and verifies each reply appears in session_messages.

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;
use ve_shared::models::SessionMessageType;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f14", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f14", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F14 requires host_id"))?;

    // Step 1: Create workspace
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created_ws = client
        .create_workspace(host_id, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    // Step 2: Create session with first question
    let ik = fixtures::unique_idempotency_key();
    let session_title = fixtures::unique_session_title();
    let initial_message = "What is 2+2? Reply with just the number.";

    let session = client
        .create_session(host_id, workspace_id, &session_title, initial_message, &ik)
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;

    tracing::info!(%session_id, "Session created, waiting for first agent reply...");

    // Step 3: Wait for first reply
    wait_for_agent_reply(client, session_id, 120, 1).await?;

    let messages_after_first = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list messages after first reply: {e}"))?;

    let first_reply_count = messages_after_first
        .items
        .iter()
        .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
        .count();

    tracing::info!(count = first_reply_count, "First reply verified");

    // Step 4: Send second message (multi-turn)
    let second_message = "What is 3*5? Reply with just the number.";
    tracing::info!("Sending second message");

    client
        .send_message(session_id, second_message)
        .await
        .map_err(|e| anyhow::anyhow!("send second message: {e}"))?;

    // Step 5: Wait for second reply
    wait_for_agent_reply(client, session_id, 120, 2).await?;

    let messages_after_second = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list messages after second reply: {e}"))?;

    let second_reply_count = messages_after_second
        .items
        .iter()
        .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
        .count();

    if second_reply_count <= first_reply_count {
        anyhow::bail!(
            "Expected more assistant messages after second turn (was {first_reply_count}, now {second_reply_count})"
        );
    }

    tracing::info!(
        new_count = second_reply_count,
        "Second reply verified — multi-turn conversation works"
    );

    // Step 6: Verify user messages are also stored
    let user_messages: Vec<_> = messages_after_second
        .items
        .iter()
        .filter(|m| m.message_type == SessionMessageType::User)
        .collect();

    if user_messages.len() < 2 {
        anyhow::bail!(
            "Expected at least 2 user messages, got {}",
            user_messages.len()
        );
    }

    tracing::info!(
        user_message_count = user_messages.len(),
        "User messages verified"
    );

    // Step 7: Send third message to further confirm stability
    let third_message = "What is 10-7? Reply with just the number.";
    tracing::info!("Sending third message");

    client
        .send_message(session_id, third_message)
        .await
        .map_err(|e| anyhow::anyhow!("send third message: {e}"))?;

    wait_for_agent_reply(client, session_id, 120, 3).await?;

    let messages_after_third = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list messages after third reply: {e}"))?;

    let third_reply_count = messages_after_third
        .items
        .iter()
        .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
        .count();

    if third_reply_count <= second_reply_count {
        anyhow::bail!(
            "Expected more assistant messages after third turn (was {second_reply_count}, now {third_reply_count})"
        );
    }

    tracing::info!(
        total_assistant_messages = third_reply_count,
        "Third reply verified — 3-turn conversation stable"
    );

    tracing::info!("F14 complete: multi-turn conversation with real agent verified");

    Ok(())
}

/// Poll session_messages until at least `min_count` non-empty Assistant messages appear.
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

        tracing::debug!(reply_count, "No new agent reply yet, polling again...");
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
