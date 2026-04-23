//! F4: Session message flow

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f4", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f4", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F4 requires host_id"))?;

    let pool = ctx
        .pool()
        .ok_or_else(|| anyhow::anyhow!("F4 requires integration server pool"))?;

    // Step 1: Create workspace
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created_ws = client
        .create_workspace(host_id, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    // Step 2: Create a session
    let ik = fixtures::unique_idempotency_key();
    let session = client
        .create_session(
            host_id,
            workspace_id,
            &fixtures::unique_session_title(),
            "Initial message for F4",
            &ik,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;
    let session_id_str = session_id.to_string();

    tracing::info!(%session_id, "Session created for message flow test");

    // Step 3: Verify initial message was stored
    let msg_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM session_messages WHERE session_id = $1")
            .bind(&session_id_str)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow::anyhow!("count messages: {e}"))?;

    if msg_count.0 < 1 {
        anyhow::bail!("Expected at least 1 message (initial), got {}", msg_count.0);
    }

    tracing::info!("Initial message stored: {} message(s)", msg_count.0);

    // Step 4: List messages — should contain the initial message
    let messages = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list messages: {e}"))?;

    if messages.items.is_empty() {
        anyhow::bail!("Expected at least 1 message in list, got none");
    }

    // Verify the initial message content
    let has_initial = messages
        .items
        .iter()
        .any(|m| m.content.contains("Initial message for F4"));

    if !has_initial {
        anyhow::bail!("Initial message not found in message list");
    }

    tracing::info!("Message list verified: initial message found");

    // Step 5: Happy path — send_message to running session
    client
        .send_message(session_id, "Follow-up message for F4")
        .await
        .map_err(|e| anyhow::anyhow!("send_message: {e}"))?;

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let messages_after = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list_messages after send: {e}"))?;

    if messages_after.items.len() < 2 {
        anyhow::bail!(
            "Expected at least 2 messages after send, got {}",
            messages_after.items.len()
        );
    }

    let has_followup = messages_after
        .items
        .iter()
        .any(|m| m.content.contains("Follow-up message for F4"));

    if !has_followup {
        anyhow::bail!("Follow-up message not found in message list");
    }

    tracing::info!("send_message happy path verified: follow-up message present");

    // Step 6: Error path — send message to archived session
    // Create a session, manually mark it archived, then try to send
    let ik2 = fixtures::unique_idempotency_key();
    let session2 = client
        .create_session(
            host_id,
            workspace_id,
            &fixtures::unique_session_title(),
            "Message test session 2",
            &ik2,
        )
        .await
        .map_err(|e| anyhow::anyhow!("create session 2: {e}"))?;

    let session2_id = session2.session_id;

    // Manually set status to archived in DB
    sqlx::query("UPDATE sessions SET status = 'archived' WHERE session_id = $1")
        .bind(session2_id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow::anyhow!("set session archived: {e}"))?;

    let result = client.send_message(session2_id, "This should fail").await;
    if result.is_ok() {
        anyhow::bail!("Expected error when sending message to archived session, but got OK");
    }

    tracing::info!("Error path verified: archived session rejects messages");

    // Step 6: Error path — empty content
    let result = client.send_message(session_id, "").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for empty message content, but got OK");
    }

    tracing::info!("Error path verified: empty content rejected");

    // Step 7: Error path — send message to non-existent session
    let bad_session = uuid::Uuid::new_v4();
    let result = client.send_message(bad_session, "should fail").await;
    if result.is_ok() {
        anyhow::bail!("Expected error for non-existent session message, but got OK");
    }

    tracing::info!("Error path verified: non-existent session rejected");

    Ok(())
}
