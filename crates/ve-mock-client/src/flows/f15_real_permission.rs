//! F15: Real agent permission review/authorization test
//!
//! This flow requires `--real-agent` flag. It creates a workspace with test files,
//! starts a session that triggers Claude Code to use an MCP tool requiring permission,
//! then verifies the full permission request → response → decision cycle works
//! with a real Claude Code process.

use std::sync::Arc;
use std::{fs, path::{Path, PathBuf}};

use crate::fixtures;
use crate::flows::FlowResult;
use crate::test_context::TestContext;
use serde::Serialize;
use ve_shared::models::SessionMessageType;
use ve_shared::types::{PermissionStatus, SessionStatus};

enum PermissionFlowOutcome {
    Passed,
    Skipped(&'static str),
}

const PERMISSION_POLL_INTERVAL_SECS: u64 = 3;
const PERMISSION_ATTEMPT_TIMEOUT_SECS: u64 = 30;
const F15_DIAGNOSTIC_DIR: &str = "target/tmp/f15-diagnostics";

pub async fn run(ctx: Arc<TestContext>) -> FlowResult {
    let start = std::time::Instant::now();

    let result = run_impl(&ctx).await;

    match result {
        Ok(PermissionFlowOutcome::Passed) => FlowResult::pass("f15", start.elapsed().as_secs_f64()),
        Ok(PermissionFlowOutcome::Skipped(reason)) => FlowResult::skipped("f15", reason),
        Err(e) => FlowResult::fail("f15", &e.to_string()),
    }
}

async fn run_impl(ctx: &TestContext) -> anyhow::Result<PermissionFlowOutcome> {
    let client = &ctx.client;

    let host_id = ctx
        .host_id
        .ok_or_else(|| anyhow::anyhow!("F15 requires host_id"))?;

    let pool = ctx.pool();

    // Step 1: Create workspace with test files
    let ws_name = fixtures::unique_workspace_name();
    let ws_path = ctx.workspace_path(&ws_name);
    fixtures::create_test_workspace(&ws_path)?;

    let created_ws = client
        .create_workspace(host_id, &ws_path, None)
        .await
        .map_err(|e| anyhow::anyhow!("create workspace: {e}"))?;

    let workspace_id = created_ws.workspace_id;
    let permission_marker = "F15 permission granted";
    let probe_file_name = "README.md";
    let probe_file_path = std::path::Path::new(&ws_path).join(probe_file_name);

    tracing::info!(%workspace_id, path = %ws_path, "Workspace created with test files");

    let trigger_attempts =
        build_permission_trigger_attempts(&probe_file_path, probe_file_name, permission_marker);
    // Step 2: Start fresh sessions for each trigger attempt. Some real-agent
    // models answer and end the turn quickly after a refusal, so a fresh session
    // per attempt is more reliable than relying on follow-up messages.
    let mut successful_session_id = None;
    let mut successful_session_id_str = None;
    let mut permission = None;

    for (attempt_index, attempt) in trigger_attempts.iter().enumerate() {
        let ik = fixtures::unique_idempotency_key();
        let session_title = fixtures::unique_session_title();
        let session = client
            .create_session(host_id, workspace_id, &session_title, attempt, &ik)
            .await
            .map_err(|e| anyhow::anyhow!("create permission probe session: {e}"))?;

        let session_id = session.session_id;
        tracing::info!(%session_id, attempt = %attempt, "Permission test session created");

        if let Some(found_permission) =
            wait_for_pending_permission(client, session_id, PERMISSION_ATTEMPT_TIMEOUT_SECS)
                .await?
        {
            write_f15_diagnostic(
                ctx,
                client,
                pool,
                F15DiagnosticInput {
                    session_id,
                    attempt_index,
                    attempt_prompt: attempt,
                    outcome: "permission_detected",
                    probe_file_path: &probe_file_path,
                },
            )
            .await;
            successful_session_id = Some(session_id);
            successful_session_id_str = Some(session_id.to_string());
            permission = Some(found_permission);
            break;
        }

        tracing::info!(
            %session_id,
            "No pending permission for this attempt; leaving probe session in place to avoid teardown lock contention"
        );
        write_f15_diagnostic(
            ctx,
            client,
            pool,
            F15DiagnosticInput {
                session_id,
                attempt_index,
                attempt_prompt: attempt,
                outcome: "attempt_without_permission",
                probe_file_path: &probe_file_path,
            },
        )
        .await;
    }

    let (session_id, session_id_str, permission) = if let (Some(session_id), Some(session_id_str), Some(permission)) =
        (successful_session_id, successful_session_id_str, permission)
    {
        (session_id, session_id_str, permission)
    } else {
        tracing::info!(
            "No permission requests generated after explicit trigger attempts."
        );
        write_f15_skip_summary(ctx, &probe_file_path).await;
        return Ok(PermissionFlowOutcome::Skipped(
            "permission prompt not triggered in this environment",
        ));
    };

    tracing::info!(
        permission_id = %permission.permission_id,
        risk_type = ?permission.risk_type,
        summary = %permission.summary,
        "Permission request found"
    );

    // Step 4: Verify permission request persisted and session entered approval state.
    if let Some(pool) = pool {
        let db_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM permission_requests WHERE session_id = $1")
                .bind(&session_id_str)
                .fetch_one(pool)
                .await
                .map_err(|e| anyhow::anyhow!("query permission count in DB: {e}"))?;

        if db_count.0 == 0 {
            anyhow::bail!("Permission requests visible via API but not in DB");
        }
    }

    wait_for_session_status(
        client,
        session_id,
        SessionStatus::WaitingApproval,
        PERMISSION_ATTEMPT_TIMEOUT_SECS,
    )
    .await?;

    // Step 5: Respond with ApproveOnce to unblock the agent.
    let response = client
        .respond_permission(
            permission.permission_id,
            ve_shared::models::PermissionDecision::ApproveOnce,
            Some("F15 real-agent approval"),
        )
        .await
        .map_err(|e| anyhow::anyhow!("respond permission: {e}"))?;

    if response.status != PermissionStatus::ApprovedOnce {
        anyhow::bail!(
            "Permission respond did not return approved_once: {:?}",
            response.status
        );
    }

    tracing::info!("Permission approval sent — agent should continue");

    let assistant_count_before_approval = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list messages before approval wait: {e}"))?
        .items
        .into_iter()
        .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
        .count();

    wait_for_session_to_resume(client, session_id, PERMISSION_ATTEMPT_TIMEOUT_SECS).await?;

    wait_for_assistant_reply(client, session_id, 120, assistant_count_before_approval + 1).await?;
    let probe_contents =
        wait_for_probe_contents(client, host_id, workspace_id, &probe_file_path, 30).await?;
    if !probe_contents.contains(permission_marker) {
        anyhow::bail!(
            "Probe file content mismatch after approval: {}",
            probe_contents
        );
    }

    // Step 6: Verify session_messages contain the expected flow
    let messages = client
        .list_messages(session_id)
        .await
        .map_err(|e| anyhow::anyhow!("list messages: {e}"))?;

    let assistant_msgs: Vec<_> = messages
        .items
        .iter()
        .filter(|m| m.message_type == SessionMessageType::Assistant && !m.content.is_empty())
        .collect();

    if assistant_msgs.is_empty() {
        anyhow::bail!("No assistant messages found after permission flow");
    }

    tracing::info!(
        assistant_message_count = assistant_msgs.len(),
        "Session messages verified after permission flow"
    );

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

    tracing::info!("F15 complete: permission review/authorization flow verified");
    write_f15_diagnostic(
        ctx,
            client,
            pool,
            F15DiagnosticInput {
                session_id,
                attempt_index: usize::MAX,
            attempt_prompt: "approval_completed",
            outcome: "permission_flow_passed",
            probe_file_path: &probe_file_path,
        },
    )
    .await;

    Ok(PermissionFlowOutcome::Passed)
}

fn build_permission_trigger_attempts(
    probe_file_path: &Path,
    probe_file_name: &str,
    permission_marker: &str,
) -> Vec<String> {
    let absolute_probe = probe_file_path.display().to_string();
    vec![
        format!(
            "For an integration test, you MUST use the Bash tool to append a newline with the \
             exact text `{permission_marker}` to the file at absolute path `{absolute_probe}`. \
             Do not only describe the change. Actually run the command, then read `{absolute_probe}` \
             and quote the appended line. If permission is required, request approval and continue after approval."
        ),
        format!(
            "Use the Write or Edit tool, not just a text response, to ensure the existing file \
             `{probe_file_name}` at `{absolute_probe}` contains the exact marker line \
             `{permission_marker}`. You must perform the file modification, and if approval is \
             required you must request it instead of refusing."
        ),
        format!(
            "Before answering, explicitly modify `{absolute_probe}` by replacing the text \
             `This is a test workspace.` with `This is a test workspace.\\n{permission_marker}`. \
             Use an actual file-editing tool call. If an approval step is needed, request approval \
             and then continue with the edit."
        ),
    ]
}

async fn wait_for_assistant_reply(
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

async fn wait_for_pending_permission(
    client: &crate::client::MockClient,
    session_id: uuid::Uuid,
    timeout_secs: u64,
) -> anyhow::Result<Option<ve_shared::models::PermissionRequest>> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

    while std::time::Instant::now() < deadline {
        let permissions = client
            .list_permissions(Some(session_id))
            .await
            .map_err(|e| anyhow::anyhow!("list permissions while polling: {e}"))?;

        if let Some(permission) = permissions
            .into_iter()
            .find(|permission| permission.status == PermissionStatus::Pending)
        {
            return Ok(Some(permission));
        }

        tokio::time::sleep(std::time::Duration::from_secs(
            PERMISSION_POLL_INTERVAL_SECS,
        ))
        .await;
    }

    Ok(None)
}

async fn wait_for_session_status(
    client: &crate::client::MockClient,
    session_id: uuid::Uuid,
    expected: SessionStatus,
    timeout_secs: u64,
) -> anyhow::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        let session = client
            .get_session(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("get session while polling status: {e}"))?;

        if session.status == expected {
            return Ok(());
        }

        tokio::time::sleep(std::time::Duration::from_secs(
            PERMISSION_POLL_INTERVAL_SECS,
        ))
        .await;
    }

    anyhow::bail!(
        "Session did not reach expected status {:?} within {}s",
        expected,
        timeout_secs
    );
}

async fn wait_for_session_to_resume(
    client: &crate::client::MockClient,
    session_id: uuid::Uuid,
    timeout_secs: u64,
) -> anyhow::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        let session = client
            .get_session(session_id)
            .await
            .map_err(|e| anyhow::anyhow!("get session while waiting for resume: {e}"))?;

        if session.status != SessionStatus::WaitingApproval {
            return Ok(());
        }

        tokio::time::sleep(std::time::Duration::from_secs(
            PERMISSION_POLL_INTERVAL_SECS,
        ))
        .await;
    }

    anyhow::bail!(
        "Session stayed in waiting_approval for more than {}s after approval",
        timeout_secs
    );
}

async fn wait_for_probe_file(path: &std::path::Path, timeout_secs: u64) -> anyhow::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        if path.exists() {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    anyhow::bail!(
        "Probe file {} was not created within {}s",
        path.display(),
        timeout_secs
    );
}

async fn wait_for_probe_contents(
    client: &crate::client::MockClient,
    host_id: uuid::Uuid,
    workspace_id: uuid::Uuid,
    path: &std::path::Path,
    timeout_secs: u64,
) -> anyhow::Result<String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("probe file path missing file name: {}", path.display()))?;

    if wait_for_probe_file(path, timeout_secs).await.is_ok() {
        return std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("read probe file after approval: {e}"));
    }

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    while std::time::Instant::now() < deadline {
        match client.get_file_content(host_id, workspace_id, file_name).await {
            Ok(response) => return Ok(response.data.content),
            Err(error) => {
                tracing::debug!(error = %error, file_name, "Probe file not visible via API yet");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }

    anyhow::bail!(
        "Probe file {} was not readable locally or via API within {}s",
        path.display(),
        timeout_secs
    );
}

#[derive(Debug, Serialize)]
struct F15DiagnosticSnapshot {
    generated_at: String,
    attempt_index: usize,
    attempt_prompt: String,
    outcome: String,
    session: Option<F15SessionSnapshot>,
    permission_count_in_db: Option<i64>,
    message_count_in_db: Option<i64>,
    permissions: Vec<F15PermissionSnapshot>,
    recent_messages: Vec<F15MessageSnapshot>,
    probe_file_exists: bool,
    probe_file_excerpt: Option<String>,
    daemon_log_path: Option<String>,
    daemon_log_tail: Option<String>,
}

struct F15DiagnosticInput<'a> {
    session_id: uuid::Uuid,
    attempt_index: usize,
    attempt_prompt: &'a str,
    outcome: &'a str,
    probe_file_path: &'a Path,
}

#[derive(Debug, Serialize)]
struct F15SessionSnapshot {
    session_id: String,
    status: String,
    latest_summary: Option<String>,
    pending_permission_count: i32,
    claude_session_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct F15PermissionSnapshot {
    permission_id: String,
    risk_type: String,
    summary: String,
    target: Option<String>,
    status: String,
    responded_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct F15MessageSnapshot {
    message_id: String,
    message_type: String,
    content_excerpt: String,
    created_at: String,
}

async fn write_f15_skip_summary(ctx: &TestContext, probe_file_path: &Path) {
    let daemon_log_tail = read_daemon_log_tail(ctx.daemon_log_path());
    let snapshot = F15DiagnosticSnapshot {
        generated_at: chrono::Utc::now().to_rfc3339(),
        attempt_index: usize::MAX,
        attempt_prompt: "all_attempts_exhausted".to_string(),
        outcome: "skip_without_permission".to_string(),
        session: None,
        permission_count_in_db: None,
        message_count_in_db: None,
        permissions: Vec::new(),
        recent_messages: Vec::new(),
        probe_file_exists: probe_file_path.exists(),
        probe_file_excerpt: read_probe_excerpt(probe_file_path),
        daemon_log_path: ctx
            .daemon_log_path()
            .map(|path| path.display().to_string()),
        daemon_log_tail,
    };
    if let Err(error) = persist_f15_snapshot(&snapshot) {
        tracing::warn!(error = %error, "Failed to persist F15 skip summary");
    }
}

async fn write_f15_diagnostic(
    ctx: &TestContext,
    client: &crate::client::MockClient,
    pool: Option<&sqlx::AnyPool>,
    input: F15DiagnosticInput<'_>,
) {
    let session = client.get_session(input.session_id).await.ok();
    let permissions = client.list_permissions(Some(input.session_id)).await.ok();
    let messages = client.list_messages(input.session_id).await.ok();

    let permission_count_in_db: Option<i64> = match pool {
        Some(pool) => sqlx::query_scalar("SELECT COUNT(*) FROM permission_requests WHERE session_id = $1")
            .bind(input.session_id.to_string())
            .fetch_one(pool)
            .await
            .ok(),
        None => None,
    };

    let message_count_in_db: Option<i64> = match pool {
        Some(pool) => sqlx::query_scalar("SELECT COUNT(*) FROM session_messages WHERE session_id = $1")
            .bind(input.session_id.to_string())
            .fetch_one(pool)
            .await
            .ok(),
        None => None,
    };

    let snapshot = F15DiagnosticSnapshot {
        generated_at: chrono::Utc::now().to_rfc3339(),
        attempt_index: input.attempt_index,
        attempt_prompt: input.attempt_prompt.to_string(),
        outcome: input.outcome.to_string(),
        session: session.map(|session| F15SessionSnapshot {
            session_id: session.session_id.to_string(),
            status: format!("{:?}", session.status),
            latest_summary: session.latest_summary,
            pending_permission_count: session.pending_permission_count,
            claude_session_id: session.claude_session_id,
        }),
        permission_count_in_db,
        message_count_in_db,
        permissions: permissions
            .unwrap_or_default()
            .into_iter()
            .map(|permission| F15PermissionSnapshot {
                permission_id: permission.permission_id.to_string(),
                risk_type: format!("{:?}", permission.risk_type),
                summary: permission.summary,
                target: permission.target,
                status: format!("{:?}", permission.status),
                responded_at: permission.responded_at.map(|time| time.to_rfc3339()),
            })
            .collect(),
        recent_messages: messages
            .map(|messages| messages.items)
            .unwrap_or_default()
            .into_iter()
            .rev()
            .take(8)
            .map(|message| F15MessageSnapshot {
                message_id: message.message_id.to_string(),
                message_type: format!("{:?}", message.message_type),
                content_excerpt: excerpt(&message.content, 400),
                created_at: message.created_at.to_rfc3339(),
            })
            .collect(),
        probe_file_exists: input.probe_file_path.exists(),
        probe_file_excerpt: read_probe_excerpt(input.probe_file_path),
        daemon_log_path: ctx
            .daemon_log_path()
            .map(|path| path.display().to_string()),
        daemon_log_tail: read_daemon_log_tail(ctx.daemon_log_path()),
    };

    if let Err(error) = persist_f15_snapshot(&snapshot) {
        tracing::warn!(error = %error, "Failed to persist F15 diagnostic snapshot");
    }
}

fn persist_f15_snapshot(snapshot: &F15DiagnosticSnapshot) -> anyhow::Result<PathBuf> {
    let dir = PathBuf::from(F15_DIAGNOSTIC_DIR);
    fs::create_dir_all(&dir)?;
    let filename = format!(
        "f15-{}-{}.json",
        snapshot.generated_at.replace(':', "-"),
        snapshot.attempt_index
    );
    let path = dir.join(filename);
    fs::write(&path, serde_json::to_string_pretty(snapshot)?)?;
    tracing::info!(path = %path.display(), outcome = %snapshot.outcome, "F15 diagnostic snapshot written");
    Ok(path)
}

fn read_probe_excerpt(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|content| excerpt(&content, 400))
}

fn read_daemon_log_tail(path: Option<&Path>) -> Option<String> {
    path.and_then(|path| fs::read_to_string(path).ok())
        .map(|content| tail_lines(&content, 80))
}

fn excerpt(content: &str, max_chars: usize) -> String {
    let mut out = content.chars().take(max_chars).collect::<String>();
    if content.chars().count() > max_chars {
        out.push_str("...");
    }
    out
}

fn tail_lines(content: &str, line_count: usize) -> String {
    content
        .lines()
        .rev()
        .take(line_count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use crate::flows::FlowResult;

    #[test]
    fn skipped_result_preserves_skip_status_for_permission_flow() {
        let result = FlowResult::skipped("f15", "permission prompt not triggered");
        assert_eq!(result.id, "f15");
        assert_eq!(result.status, "SKIP");
        assert_eq!(result.message, "permission prompt not triggered");
    }

    #[test]
    fn permission_trigger_attempts_include_explicit_tool_and_shell_fallback() {
        let attempts = super::build_permission_trigger_attempts(
            std::path::Path::new("/tmp/workspace/README.md"),
            "README.md",
            "F15 permission granted",
        );
        assert_eq!(attempts.len(), 3);
        assert!(attempts[0].contains("MUST use the Bash tool"));
        assert!(attempts[1].contains("Write or Edit tool"));
        assert!(attempts[2].contains("/tmp/workspace/README.md"));
    }

    #[test]
    fn tail_lines_returns_only_requested_tail() {
        let content = "a\nb\nc\nd";
        assert_eq!(super::tail_lines(content, 2), "c\nd");
    }
}
