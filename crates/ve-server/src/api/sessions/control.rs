//! Session control and archive handlers
//!
//! Control commands (pause, terminate, interrupt, restart, rerun) and session archiving.

use super::commands::{
    ensure_command_acked, mark_session_error, persist_control_success, send_daemon_command_and_wait,
};
use super::{
    extract_request_id, generate_request_id, parse_uuid, require_host_access, utils, AppState,
    ArchiveMetadata, ArchiveStatistics, DaemonMessage, Result, ServerError, SessionAccess,
    SessionControlAction,
};
use crate::authz::{require_client_device_id, require_session_access};
use axum::http::HeaderMap;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use ve_shared::jwt::Claims;
use ve_shared::types::CloseReason;

#[cfg(debug_assertions)]
use super::test_support;

/// Control request
#[derive(Debug, Deserialize)]
pub struct ControlRequest {
    pub action: String,
}

/// Validate whether a control action is allowed for the current persisted session status.
pub(crate) fn validate_live_control_action(
    status: &str,
    action: SessionControlAction,
) -> Result<()> {
    if status == "archived" {
        return Err(ServerError::SessionArchived);
    }

    if action == SessionControlAction::Rerun {
        return Err(ServerError::Conflict(
            "rerun is only supported for archived sessions; use restart for live sessions"
                .to_string(),
        ));
    }

    Ok(())
}

pub async fn control_session_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ControlRequest>,
) -> Result<Json<serde_json::Value>> {
    control_session_for_id(state, headers, access.device_id, access.session_id, req).await
}

pub async fn control_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<ControlRequest>,
) -> Result<Json<serde_json::Value>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    control_session_for_id(state, headers, device_id, id, req).await
}

async fn control_session_for_id(
    state: Arc<AppState>,
    headers: HeaderMap,
    device_id: Uuid,
    id: Uuid,
    req: ControlRequest,
) -> Result<Json<serde_json::Value>> {
    // Extract trace_id for correlation
    let trace_id = extract_request_id(&headers);

    let session_id_str = id.to_string();

    let session: (String, String, String, String, String, Option<String>) = sqlx::query_as(
        r#"
        SELECT status, host_id, workspace_id, title, agent_type, claude_session_id
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    let host_id = parse_uuid(&session.1, "host_id")?;
    let action = utils::parse_control_action(&req.action)?;

    if session.0 == "archived" {
        require_host_access(&state, device_id, host_id).await?;
        return handle_archived_rerun(&state, &trace_id, device_id, id, &req.action, session).await;
    }

    validate_live_control_action(&session.0, action)?;

    let request_id = generate_request_id();
    let request = DaemonMessage::SessionControl {
        request_id: request_id.clone(),
        session_id: id,
        action,
    };

    ensure_command_acked(
        send_daemon_command_and_wait(&state, host_id, request_id, request).await?,
    )?;
    persist_control_success(&state, id, action).await?;

    tracing::debug!(trace_id = %trace_id, session_id = %id, action = ?action, "Control command sent");

    Ok(Json(serde_json::json!({ "success": true })))
}

pub(crate) async fn handle_archived_rerun(
    state: &AppState,
    trace_id: &str,
    requester_device_id: Uuid,
    archived_session_id: Uuid,
    action_raw: &str,
    session: (String, String, String, String, String, Option<String>),
) -> Result<Json<serde_json::Value>> {
    if action_raw != "rerun" {
        return Err(ServerError::SessionArchived);
    }

    let (_status, host_id_raw, workspace_id_raw, title, agent_type, live_claude_session_id) =
        session;

    let host_id = parse_uuid(&host_id_raw, "host_id")?;
    let host_id_str = host_id.to_string();
    let workspace_id = parse_uuid(&workspace_id_raw, "workspace_id")?;
    let workspace_id_str = workspace_id.to_string();
    let archived_session_id_str = archived_session_id.to_string();
    let requester_device_id_str = requester_device_id.to_string();

    let workspace: (String,) = sqlx::query_as(
        r#"
        SELECT path FROM workspaces WHERE workspace_id = $1 AND host_id = $2
        "#,
    )
    .bind(&workspace_id_str)
    .bind(&host_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Workspace {}", workspace_id)))?;

    let archived_claude_session_id = if let Some(claude_session_id) = live_claude_session_id {
        claude_session_id
    } else {
        let metadata_json = sqlx::query_as::<_, (Option<String>,)>(
            r#"
            SELECT metadata_json FROM session_archives WHERE session_id = $1
            ORDER BY closed_at DESC LIMIT 1
            "#,
        )
        .bind(&archived_session_id_str)
        .fetch_optional(&state.db)
        .await?;

        let archive_metadata = metadata_json
            .and_then(|(json,)| json)
            .map(|json| {
                serde_json::from_str::<ve_shared::models::ArchiveMetadata>(&json).map_err(|error| {
                    ServerError::Internal(format!(
                        "Failed to parse archive metadata for session {}: {}",
                        archived_session_id, error
                    ))
                })
            })
            .transpose()?;

        archive_metadata
            .and_then(|metadata| metadata.claude_session_id)
            .ok_or_else(|| {
                ServerError::Conflict(
                    "Archived session has no Claude session ID to resume".to_string(),
                )
            })?
    };

    if let Some(existing_session_id) =
        find_reusable_rerun_session_id(state, &archived_session_id_str).await?
    {
        ensure_device_session_access(state, &requester_device_id_str, &existing_session_id).await?;

        tracing::info!(
            trace_id = %trace_id,
            archived_session_id = %archived_session_id,
            existing_session_id = %existing_session_id,
            "Archived session rerun reused existing live session"
        );

        return Ok(Json(serde_json::json!({
            "success": true,
            "session_id": existing_session_id,
            "resumed_from_session_id": archived_session_id,
        })));
    }

    if has_dispatching_rerun(state, &archived_session_id_str).await? {
        return Err(ServerError::Conflict(
            "Archived session rerun is still dispatching; retry shortly".to_string(),
        ));
    }

    #[cfg(debug_assertions)]
    test_support::wait_for_archived_rerun_race(&archived_session_id_str).await;

    let new_session_id = Uuid::new_v4();
    let new_session_id_str = new_session_id.to_string();
    let mut tx = state.db.begin().await?;

    let insert_result = sqlx::query(
        r#"
        INSERT INTO sessions (
            session_id, title, host_id, workspace_id, agent_type, status, claude_session_id,
            rerun_from_session_id, can_resume_cross_device
        )
        VALUES ($1, $2, $3, $4, $5, 'dispatching', $6, $7, TRUE)
        "#,
    )
    .bind(&new_session_id_str)
    .bind(&title)
    .bind(&host_id_str)
    .bind(&workspace_id_str)
    .bind(&agent_type)
    .bind(&archived_claude_session_id)
    .bind(&archived_session_id_str)
    .execute(&mut *tx)
    .await;

    match insert_result {
        Ok(_) => {
            sqlx::query(
                r#"
                INSERT INTO device_session_access (device_id, session_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(&requester_device_id_str)
            .bind(&new_session_id_str)
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
        }
        Err(error)
            if error.to_string().contains("UNIQUE constraint")
                || error.to_string().contains("duplicate key") =>
        {
            tx.rollback().await?;

            if let Some(existing_session_id) =
                find_reusable_rerun_session_id(state, &archived_session_id_str).await?
            {
                ensure_device_session_access(state, &requester_device_id_str, &existing_session_id)
                    .await?;

                tracing::info!(
                    trace_id = %trace_id,
                    archived_session_id = %archived_session_id,
                    existing_session_id = %existing_session_id,
                    "Archived session rerun reused concurrent live session"
                );

                return Ok(Json(serde_json::json!({
                    "success": true,
                    "session_id": existing_session_id,
                    "resumed_from_session_id": archived_session_id,
                })));
            }

            if has_dispatching_rerun(state, &archived_session_id_str).await? {
                return Err(ServerError::Conflict(
                    "Archived session rerun is still dispatching; retry shortly".to_string(),
                ));
            }

            return Err(ServerError::Internal(
                "Active rerun disappeared after uniqueness conflict".to_string(),
            ));
        }
        Err(error) => return Err(error.into()),
    }

    let request_id = generate_request_id();
    match send_daemon_command_and_wait(
        state,
        host_id,
        request_id.clone(),
        DaemonMessage::RerunSession {
            request_id,
            session_id: new_session_id,
            workspace_path: workspace.0,
            agent_type: agent_type.clone(),
            claude_session_id: archived_claude_session_id.clone(),
        },
    )
    .await
    {
        Ok(response) => {
            if let Err(error) = ensure_command_acked(response) {
                mark_session_error(
                    state,
                    &new_session_id_str,
                    "Failed to queue rerun request to daemon",
                )
                .await?;
                return Err(error);
            }
        }
        Err(error) => {
            mark_session_error(
                state,
                &new_session_id_str,
                "Failed to queue rerun request to daemon",
            )
            .await?;

            tracing::warn!(
                trace_id = %trace_id,
                archived_session_id = %archived_session_id,
                new_session_id = %new_session_id,
                error = %error,
                "Archived session rerun failed before daemon ack"
            );

            return Err(ServerError::Conflict(
                "Failed to queue rerun request to daemon".to_string(),
            ));
        }
    }

    sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'pending', updated_at = CURRENT_TIMESTAMP
        WHERE session_id = $1 AND status = 'dispatching'
        "#,
    )
    .bind(&new_session_id_str)
    .execute(&state.db)
    .await?;

    tracing::info!(
        trace_id = %trace_id,
        archived_session_id = %archived_session_id,
        new_session_id = %new_session_id,
        claude_session_id = %archived_claude_session_id,
        "Archived session rerun requested"
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "session_id": new_session_id,
        "resumed_from_session_id": archived_session_id,
    })))
}

async fn find_reusable_rerun_session_id(
    state: &AppState,
    archived_session_id: &str,
) -> Result<Option<Uuid>> {
    let session_id = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT session_id
        FROM sessions
        WHERE rerun_from_session_id = $1
          AND status NOT IN ('dispatching', 'archived', 'error')
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(archived_session_id)
    .fetch_optional(&state.db)
    .await?
    .map(|(session_id,)| parse_uuid(&session_id, "session_id"))
    .transpose()?;

    Ok(session_id)
}

async fn has_dispatching_rerun(state: &AppState, archived_session_id: &str) -> Result<bool> {
    let row = sqlx::query_as::<_, (i64,)>(
        r#"
        SELECT COUNT(*)
        FROM sessions
        WHERE rerun_from_session_id = $1
          AND status = 'dispatching'
        "#,
    )
    .bind(archived_session_id)
    .fetch_one(&state.db)
    .await?;

    Ok(row.0 > 0)
}

async fn ensure_device_session_access(
    state: &AppState,
    device_id: &str,
    session_id: &Uuid,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO device_session_access (device_id, session_id)
        VALUES ($1, $2)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(device_id)
    .bind(session_id.to_string())
    .execute(&state.db)
    .await?;

    Ok(())
}

fn archive_closed_by(close_reason: CloseReason) -> &'static str {
    match close_reason {
        CloseReason::Terminated => "daemon",
        _ => "server",
    }
}

pub(crate) async fn archive_session_with_metadata(
    state: &AppState,
    session_id: Uuid,
    close_reason: CloseReason,
    summary_override: Option<String>,
) -> Result<Uuid> {
    let session_id_str = session_id.to_string();
    let session: (
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
    ) = sqlx::query_as(
        r#"
        SELECT session_id, title, host_id, workspace_id, status, agent_type,
               latest_summary, claude_session_id, CAST(created_at AS TEXT)
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", session_id)))?;

    if session.4 == "archived" {
        let archive: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT archive_id FROM session_archives WHERE session_id = $1
            "#,
        )
        .bind(&session_id_str)
        .fetch_optional(&state.db)
        .await?;

        return archive
            .map(|(archive_id,)| parse_uuid(&archive_id, "archive_id"))
            .transpose()?
            .ok_or_else(|| {
                ServerError::Conflict(format!(
                    "Session {} is archived without an archive record",
                    session_id
                ))
            });
    }

    let host_id = parse_uuid(&session.2, "host_id")?;
    let workspace_id = parse_uuid(&session.3, "workspace_id")?;
    let host_id_str = host_id.to_string();
    let workspace_id_str = workspace_id.to_string();

    let workspace_row: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT path, display_name FROM workspaces WHERE workspace_id = $1
        "#,
    )
    .bind(&workspace_id_str)
    .fetch_optional(&state.db)
    .await?;

    let message_count_row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count FROM session_messages WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_one(&state.db)
    .await?;
    let message_count = message_count_row.0 as u32;

    let permission_count_row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count FROM permission_requests WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_one(&state.db)
    .await?;
    let permission_count = permission_count_row.0 as u32;

    let created_at = utils::parse_sqlite_timestamp(&session.8)
        .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?;
    let duration_seconds = (chrono::Utc::now() - created_at.with_timezone(&chrono::Utc))
        .num_seconds()
        .max(0) as u32;

    let metadata = ArchiveMetadata {
        workspace_path: workspace_row
            .as_ref()
            .map(|w| w.0.clone())
            .unwrap_or_default(),
        workspace_display_name: workspace_row.as_ref().map(|w| w.1.clone()),
        agent_type: session.5.clone(),
        closed_by: archive_closed_by(close_reason).to_string(),
        final_summary: summary_override.clone().or(session.6.clone()),
        claude_session_id: session.7.clone(),
        statistics: Some(ArchiveStatistics {
            message_count,
            event_count: 0,
            permission_count,
            duration_seconds,
        }),
        last_commit_sha: None,
        last_commit_message: None,
    };

    let metadata_json = serde_json::to_string(&metadata)
        .map_err(|e| ServerError::Internal(format!("Failed to serialize metadata: {}", e)))?;

    let archive_id = Uuid::new_v4();
    let archive_id_str = archive_id.to_string();
    let mut tx = state.db.begin().await?;

    let update_result = sqlx::query(
        r#"
        UPDATE sessions
        SET status = 'archived', latest_summary = COALESCE($2, latest_summary), updated_at = CURRENT_TIMESTAMP
        WHERE session_id = $1 AND status != 'archived'
        "#,
    )
    .bind(&session_id_str)
    .bind(&summary_override)
    .execute(&mut *tx)
    .await?;

    if update_result.rows_affected() == 0 {
        let archive: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT archive_id FROM session_archives WHERE session_id = $1
            "#,
        )
        .bind(&session_id_str)
        .fetch_optional(&mut *tx)
        .await?;
        tx.commit().await?;

        return archive
            .map(|(existing_archive_id,)| parse_uuid(&existing_archive_id, "archive_id"))
            .transpose()?
            .ok_or_else(|| {
                ServerError::Conflict(format!(
                    "Session {} is archived without an archive record",
                    session_id
                ))
            });
    }

    let metadata_expr = match state.config.database_backend() {
        crate::config::DatabaseBackend::Postgres => "$7::jsonb",
        crate::config::DatabaseBackend::Sqlite => "$7",
    };
    sqlx::query(&format!(
        r#"
        INSERT INTO session_archives (
            archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id,
            metadata_json
        )
        VALUES ($1, $2, $3, CURRENT_TIMESTAMP, $4, $5, $6, {metadata_expr})
        "#,
    ))
    .bind(&archive_id_str)
    .bind(&session_id_str)
    .bind(&session.1)
    .bind(match close_reason {
        CloseReason::UserClosed => "user_closed",
        CloseReason::Completed => "completed",
        CloseReason::Failed => "failed",
        CloseReason::Terminated => "terminated",
    })
    .bind(&host_id_str)
    .bind(&workspace_id_str)
    .bind(&metadata_json)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(archive_id)
}
