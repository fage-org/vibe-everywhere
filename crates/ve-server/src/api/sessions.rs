//! Session API Handlers
//!
//! Session CRUD, message, control, and lifecycle endpoints.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::models::{CreateSessionRequest, Session, SessionMessage};
use ve_shared::proto::{DaemonMessage, SessionControlAction};
use ve_shared::types::{Paginated, SessionStatus};

use crate::error::{Result, ServerError};
use crate::state::AppState;
use crate::utils::{self, parse_uuid};
use crate::validation::{validate_title, validate_content};

/// Session list query parameters
#[derive(Debug, Deserialize)]
pub struct SessionListQuery {
    pub host_id: Option<Uuid>,
    #[allow(dead_code)]
    pub status: Option<String>,
}

/// Database record for session
struct SessionRecord {
    session_id: String,
    title: String,
    host_id: String,
    workspace_id: String,
    agent_type: String,
    status: String,
    last_activity_at: Option<String>,
    latest_summary: Option<String>,
    unread_event_count: i64,
    pending_permission_count: i64,
    can_resume_cross_device: i64,
    claude_session_id: Option<String>,
    created_at: String,
    updated_at: String,
}

impl SessionRecord {
    fn to_model(&self) -> Result<Session> {
        Ok(Session {
            session_id: parse_uuid(&self.session_id, "session_id")?,
            title: self.title.clone(),
            host_id: parse_uuid(&self.host_id, "host_id")?,
            workspace_id: parse_uuid(&self.workspace_id, "workspace_id")?,
            agent_type: self.agent_type.clone(),
            status: utils::parse_session_status(&self.status),
            last_activity_at: self.last_activity_at.as_ref().and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|d| d.with_timezone(&chrono::Utc))
            }),
            latest_summary: self.latest_summary.clone(),
            unread_event_count: self.unread_event_count as i32,
            pending_permission_count: self.pending_permission_count as i32,
            can_resume_cross_device: self.can_resume_cross_device != 0,
            claude_session_id: self.claude_session_id.clone(),
            created_at: chrono::DateTime::parse_from_rfc3339(&self.created_at)
                .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&self.updated_at)
                .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
                .with_timezone(&chrono::Utc),
        })
    }
}

/// GET /api/sessions
///
/// List active sessions (excluding archived).
#[allow(clippy::type_complexity)]
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SessionListQuery>,
) -> Result<Json<Vec<Session>>> {
    let rows: Vec<(
        String,
        String,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        i64,
        i64,
        i64,
        Option<String>,
        String,
        String,
    )> = if let Some(host_id) = query.host_id {
        let host_id_str = host_id.to_string();
        sqlx::query_as(
                r#"
                SELECT session_id, title, host_id, workspace_id, agent_type, status,
                       last_activity_at, latest_summary, unread_event_count, pending_permission_count,
                       can_resume_cross_device, claude_session_id, created_at, updated_at
                FROM sessions
                WHERE host_id = ? AND status != 'archived'
                ORDER BY updated_at DESC
                "#
            )
            .bind(host_id_str)
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as(
                r#"
                SELECT session_id, title, host_id, workspace_id, agent_type, status,
                       last_activity_at, latest_summary, unread_event_count, pending_permission_count,
                       can_resume_cross_device, claude_session_id, created_at, updated_at
                FROM sessions
                WHERE status != 'archived'
                ORDER BY updated_at DESC
                "#
            )
            .fetch_all(&state.db)
            .await?
    };

    let sessions: Result<Vec<Session>> = rows
        .into_iter()
        .map(
            |(
                session_id,
                title,
                host_id,
                workspace_id,
                agent_type,
                status,
                last_activity_at,
                latest_summary,
                unread_event_count,
                pending_permission_count,
                can_resume_cross_device,
                claude_session_id,
                created_at,
                updated_at,
            )| {
                SessionRecord {
                    session_id,
                    title,
                    host_id,
                    workspace_id,
                    agent_type,
                    status,
                    last_activity_at,
                    latest_summary,
                    unread_event_count,
                    pending_permission_count,
                    can_resume_cross_device,
                    claude_session_id,
                    created_at,
                    updated_at,
                }
                .to_model()
            },
        )
        .collect();

    Ok(Json(sessions?))
}

/// POST /api/sessions
///
/// Create a new session with strict idempotency protection.
pub async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<Session>> {
    // Validate inputs
    validate_title(&req.title)?;
    validate_content(&req.initial_message)?;

    let idempotency_key = req.idempotency_key.clone();

    // Check idempotency key
    let existing = sqlx::query!(
        r#"
        SELECT key, session_id FROM idempotency_keys WHERE key = ?
        "#,
        idempotency_key,
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(existing) = existing {
        // Return existing session for duplicate request
        let session_id = parse_uuid(&existing.session_id, "session_id")?;
        let session_id_str = session_id.to_string();

        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                String,
                String,
                String,
                Option<String>,
                Option<String>,
                i64,
                i64,
                i64,
                Option<String>,
                String,
                String,
            ),
        >(
            r#"
            SELECT session_id, title, host_id, workspace_id, agent_type, status,
                   last_activity_at, latest_summary, unread_event_count, pending_permission_count,
                   can_resume_cross_device, claude_session_id, created_at, updated_at
            FROM sessions WHERE session_id = ?
            "#,
        )
        .bind(session_id_str)
        .fetch_optional(&state.db)
        .await?
        .ok_or(ServerError::NotFound(format!("Session {}", session_id)))?;

        let record = SessionRecord {
            session_id: row.0,
            title: row.1,
            host_id: row.2,
            workspace_id: row.3,
            agent_type: row.4,
            status: row.5,
            last_activity_at: row.6,
            latest_summary: row.7,
            unread_event_count: row.8,
            pending_permission_count: row.9,
            can_resume_cross_device: row.10,
            claude_session_id: row.11,
            created_at: row.12,
            updated_at: row.13,
        };

        tracing::info!(%session_id, key = %idempotency_key, "Returning existing session (idempotent)");

        return Ok(Json(record.to_model()?));
    }

    // Create new session
    let session_id = Uuid::new_v4();
    let session_id_str = session_id.to_string();
    let host_id_str = req.host_id.to_string();
    let workspace_id_str = req.workspace_id.to_string();

    // Get workspace path
    let workspace = sqlx::query!(
        r#"
        SELECT path FROM workspaces WHERE workspace_id = ?
        "#,
        workspace_id_str,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!(
        "Workspace {}",
        req.workspace_id
    )))?;

    // Insert session
    sqlx::query!(
        r#"
        INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type)
        VALUES (?, ?, ?, ?, 'claude_code')
        "#,
        session_id_str,
        req.title,
        host_id_str,
        workspace_id_str,
    )
    .execute(&state.db)
    .await?;

    // Insert initial message
    let message_id = Uuid::new_v4();
    let message_id_str = message_id.to_string();
    sqlx::query!(
        r#"
        INSERT INTO session_messages (message_id, session_id, message_type, content)
        VALUES (?, ?, 'user', ?)
        "#,
        message_id_str,
        session_id_str,
        req.initial_message,
    )
    .execute(&state.db)
    .await?;

    // Store idempotency key
    sqlx::query!(
        r#"
        INSERT INTO idempotency_keys (key, session_id) VALUES (?, ?)
        "#,
        idempotency_key,
        session_id_str,
    )
    .execute(&state.db)
    .await?;

    // Send create_session to daemon
    state.hub.send_to_daemon(
        &req.host_id,
        DaemonMessage::CreateSession {
            session_id,
            workspace_path: workspace.path,
            agent_type: "claude_code".to_string(),
            initial_message: req.initial_message,
        },
    );

    tracing::info!(%session_id, %req.host_id, "Session created");

    Ok(Json(Session {
        session_id,
        title: req.title,
        host_id: req.host_id,
        workspace_id: req.workspace_id,
        agent_type: "claude_code".to_string(),
        status: SessionStatus::Running,
        last_activity_at: Some(chrono::Utc::now()),
        latest_summary: None,
        unread_event_count: 0,
        pending_permission_count: 0,
        can_resume_cross_device: true,
        claude_session_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }))
}

/// GET /api/sessions/:id
///
/// Get session details.
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Session>> {
    let session_id_str = id.to_string();

    let row = sqlx::query!(
        r#"
        SELECT session_id, title, host_id, workspace_id, agent_type, status,
               last_activity_at, latest_summary, unread_event_count, pending_permission_count,
               can_resume_cross_device, claude_session_id, created_at, updated_at
        FROM sessions WHERE session_id = ?
        "#,
        session_id_str,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    Ok(Json(Session {
        session_id: id,
        title: row.title,
        host_id: parse_uuid(&row.host_id, "host_id")?,
        workspace_id: parse_uuid(&row.workspace_id, "workspace_id")?,
        agent_type: row.agent_type,
        status: utils::parse_session_status(&row.status),
        last_activity_at: row.last_activity_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        latest_summary: row.latest_summary,
        unread_event_count: row.unread_event_count as i32,
        pending_permission_count: row.pending_permission_count as i32,
        can_resume_cross_device: row.can_resume_cross_device != 0,
        claude_session_id: row.claude_session_id,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.updated_at)
            .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&chrono::Utc),
    }))
}

/// Send message request
#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

/// POST /api/sessions/:id/messages
///
/// Send a message to the session.
pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>> {
    // Validate content
    validate_content(&req.content)?;

    let session_id_str = id.to_string();

    // Check session is not archived
    let session = sqlx::query!(
        r#"
        SELECT status, host_id FROM sessions WHERE session_id = ?
        "#,
        session_id_str,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    if session.status == "archived" {
        return Err(ServerError::SessionArchived);
    }

    let host_id = parse_uuid(&session.host_id, "host_id")?;

    // Insert message
    let message_id = Uuid::new_v4();
    let message_id_str = message_id.to_string();
    sqlx::query!(
        r#"
        INSERT INTO session_messages (message_id, session_id, message_type, content)
        VALUES (?, ?, 'user', ?)
        "#,
        message_id_str,
        session_id_str,
        req.content,
    )
    .execute(&state.db)
    .await?;

    // Update session activity
    sqlx::query!(
        r#"
        UPDATE sessions SET last_activity_at = datetime('now'), updated_at = datetime('now')
        WHERE session_id = ?
        "#,
        session_id_str,
    )
    .execute(&state.db)
    .await?;

    // Forward to daemon
    state.hub.send_to_daemon(
        &host_id,
        DaemonMessage::SendMessage {
            session_id: id,
            content: req.content,
        },
    );

    Ok(Json(
        serde_json::json!({ "success": true, "message_id": message_id }),
    ))
}

/// Message list query parameters
#[derive(Debug, Deserialize)]
pub struct MessageListQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// GET /api/sessions/:id/messages
///
/// List session messages.
pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Query(query): Query<MessageListQuery>,
) -> Result<Json<Paginated<SessionMessage>>> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;
    let session_id_str = id.to_string();
    let limit_i32 = limit as i32;
    let offset_i32 = offset as i32;

    let rows = sqlx::query!(
        r#"
        SELECT message_id, session_id, message_type, content, created_at
        FROM session_messages
        WHERE session_id = ?
        ORDER BY created_at ASC
        LIMIT ? OFFSET ?
        "#,
        session_id_str,
        limit_i32,
        offset_i32,
    )
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM session_messages WHERE session_id = ?
        "#,
        session_id_str,
    )
    .fetch_one(&state.db)
    .await?
    .count as u64;

    let messages: Result<Vec<SessionMessage>> = rows
        .into_iter()
        .map(|row| {
            Ok(SessionMessage {
                message_id: parse_uuid(&row.message_id, "message_id")?,
                session_id: id,
                message_type: utils::parse_message_type(&row.message_type),
                content: row.content,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.created_at)
                    .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                    .with_timezone(&chrono::Utc),
            })
        })
        .collect();

    Ok(Json(Paginated::new(messages?, total, page, limit)))
}

/// Control request
#[derive(Debug, Deserialize)]
pub struct ControlRequest {
    pub action: String,
}

/// POST /api/sessions/:id/control
///
/// Send a control command (pause, terminate, interrupt, rerun).
pub async fn control_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ControlRequest>,
) -> Result<Json<serde_json::Value>> {
    let session_id_str = id.to_string();

    let session = sqlx::query!(
        r#"
        SELECT status, host_id FROM sessions WHERE session_id = ?
        "#,
        session_id_str,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    if session.status == "archived" {
        return Err(ServerError::SessionArchived);
    }

    let host_id = parse_uuid(&session.host_id, "host_id")?;
    let action = utils::parse_control_action(&req.action);

    // Update session status for pause
    if action == SessionControlAction::Pause {
        sqlx::query!(
            r#"
            UPDATE sessions SET status = 'paused', updated_at = datetime('now')
            WHERE session_id = ?
            "#,
            session_id_str,
        )
        .execute(&state.db)
        .await?;
    }

    state.hub.send_to_daemon(
        &host_id,
        DaemonMessage::SessionControl {
            session_id: id,
            action,
        },
    );

    Ok(Json(serde_json::json!({ "success": true })))
}

/// POST /api/sessions/:id/close
///
/// Close and archive a session.
pub async fn close_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let session_id_str = id.to_string();

    let session = sqlx::query!(
        r#"
        SELECT session_id, title, host_id, workspace_id, status
        FROM sessions WHERE session_id = ?
        "#,
        session_id_str,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    // Check if already archived (idempotent - return existing archive)
    if session.status == "archived" {
        // Find existing archive
        let archive = sqlx::query!(
            r#"
            SELECT archive_id FROM session_archives WHERE session_id = ?
            "#,
            session_id_str,
        )
        .fetch_optional(&state.db)
        .await?;

        return Ok(Json(serde_json::json!({
            "success": true,
            "already_archived": true,
            "archive_id": archive.map(|a| a.archive_id)
        })));
    }

    let host_id = parse_uuid(&session.host_id, "host_id")?;
    let workspace_id = parse_uuid(&session.workspace_id, "workspace_id")?;
    let host_id_str = host_id.to_string();
    let workspace_id_str = workspace_id.to_string();

    // Create archive
    let archive_id = Uuid::new_v4();
    let archive_id_str = archive_id.to_string();
    sqlx::query!(
        r#"
        INSERT INTO session_archives (archive_id, session_id, title, closed_at, close_reason, host_id, workspace_id)
        VALUES (?, ?, ?, datetime('now'), 'user_closed', ?, ?)
        "#,
        archive_id_str,
        session_id_str,
        session.title,
        host_id_str,
        workspace_id_str,
    )
    .execute(&state.db)
    .await?;

    // Update session status to archived
    sqlx::query!(
        r#"
        UPDATE sessions SET status = 'archived', updated_at = datetime('now')
        WHERE session_id = ?
        "#,
        session_id_str,
    )
    .execute(&state.db)
    .await?;

    // Notify daemon
    state
        .hub
        .send_to_daemon(&host_id, DaemonMessage::CloseSession { session_id: id });

    tracing::info!(%id, %archive_id, "Session archived");

    Ok(Json(
        serde_json::json!({ "success": true, "archive_id": archive_id }),
    ))
}
