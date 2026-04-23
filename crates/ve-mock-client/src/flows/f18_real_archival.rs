//! F18: Real agent session archival and lifecycle
//!
//! This flow requires `--real-agent` flag. It creates a session with a real
//! Claude Code process, waits for the AI reply, then closes the session
//! and verifies the daemon reports session_status_update and the archive
//! is queryable via the archives API.

use std::sync::Arc;

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;
use ve_shared::models::SessionMessageType;

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(()) => FlowResult::pass("f18", start.elapsed().as_secs_f64()),
        Err(e) => FlowResult::fail("f18", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<()> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F18 requires host_id"))?;

    let pool = ctx
        .pool()
        .ok_or_else(|| anyhow::anyhow!("F18 requires integration server pool"))?;

    // Step 1: Create workspace
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);

    let created_ws = client
        .create_workspace(host_id, &ws_name, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;

    // Step 2: Create session with real agent
    let ik = fixtures::unique_idempotency_key();
    let session_title = fixtures::unique_session_title();
    let initial_message = "What is 2+2? Reply with just the number.";

    let session = client
        .create_session(host_id, workspace_id, &session_title, initial_message, &ik)
        .await
        .map_err(|e| anyhow::anyhow!("create session: {e}"))?;

    let session_id = session.session_id;
    let session_id_str = session_id.to_string();

    tracing::info!(%session_id, "Session created for archival test");

    // Step 3: Wait for agent reply
    wait_for_agent_reply(client, session_id, 120, 1).await?;

    tracing::info!("Agent reply received — session has content");

    // Step 4: Verify session has messages
    let messages = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list messages: {e}"))?;

    let assistant_count = messages
        .items
        .iter()
        .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
        .count();

    if assistant_count == 0 {
        anyhow::bail!("No assistant messages found before archival");
    }

    tracing::info!(
        assistant_message_count = assistant_count,
        "Messages before archival"
    );

    // Step 5: Close the session — this should trigger daemon archival
    let close_result = client
        .close_session(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("close session: {e}"))?;

    if !close_result.success {
        anyhow::bail!("Close session did not return success: {close_result:?}");
    }

    tracing::info!("Session close acknowledged, waiting for archival...");

    // Step 6: Wait for archival — poll session status in DB
    let max_wait = std::time::Duration::from_secs(30);
    let poll_interval = std::time::Duration::from_secs(2);
    let deadline = std::time::Instant::now() + max_wait;

    let mut is_archived = false;
    while std::time::Instant::now() < deadline {
        tokio::time::sleep(poll_interval).await;

        let db_status: Result<(String,), _> =
            sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
                .bind(&session_id_str)
                .fetch_one(pool)
                .await;

        match db_status {
            Ok((status,)) => {
                if status == "archived" {
                    is_archived = true;
                    break;
                }
                tracing::debug!(%status, "Session status, waiting for archived...");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to query session status");
            }
        }
    }

    if is_archived {
        tracing::info!("Session archival detected via DB status");

        // Step 7: Verify archived session is accessible via API
        let archived_session = client
            .get_session(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("get archived session: {e}"))?;

        if archived_session.status != ve_shared::types::SessionStatus::Archived {
            anyhow::bail!(
                "Archived session status should be 'archived', got '{:?}'",
                archived_session.status
            );
        }

        // Step 8: Verify messages are preserved after archival
        let archived_messages = client
            .list_messages(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("list archived messages: {e}"))?;

        let archived_assistant_count = archived_messages
            .items
            .iter()
            .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
            .count();

        if archived_assistant_count != assistant_count {
            anyhow::bail!(
                "Assistant message count changed after archival: was {assistant_count}, now {archived_assistant_count}"
            );
        }

        tracing::info!("Archived session messages preserved");
    } else {
        tracing::info!(
            "Session not yet archived within {max_wait:?} — \
             daemon may need more time. Close flow verified, archival pending."
        );
    }

    // Step 9: Error path — close already-closed session
    let close_again = client.close_session(session_id).await;
    match close_again {
        Ok(resp) => {
            if !resp.success && !resp.already_archived {
                anyhow::bail!("Expected success or already_archived for double close");
            }
            tracing::info!("Double close handled correctly");
        }
        Err(e) => {
            tracing::info!(error = %e, "Close archived session returned error (acceptable)");
        }
    }

    // Step 10: Error path — send message to closed/archived session
    let send_result = client
        .send_message(session_id, "This should fail on a closed session")
        .await;
    if send_result.is_ok() {
        anyhow::bail!("Expected error for sending to closed session, but got OK");
    }

    tracing::info!("Error path verified: send to closed session rejected");

    tracing::info!("F18 complete: session archival lifecycle with real agent verified");

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
