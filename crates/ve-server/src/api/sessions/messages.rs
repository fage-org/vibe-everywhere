//! Session message handlers
//!
//! Send messages and list message history.

use super::commands::{ensure_command_acked, send_daemon_command_and_wait};
use super::{extract_request_id, generate_request_id, parse_uuid, utils, validate_content};
use super::{AppState, DaemonMessage, Result, ServerError, SessionAccess, SessionMessage};
use crate::authz::{require_client_device_id, require_session_access};
use axum::http::HeaderMap;
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;
use ve_shared::jwt::Claims;
use ve_shared::types::Paginated;

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
}

/// POST /api/sessions/:id/messages
///
/// Send a message to the session.
pub async fn send_message_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<ve_shared::models::SendMessageResponse>> {
    send_message_for_session(state, headers, access.session_id, req).await
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<ve_shared::models::SendMessageResponse>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    send_message_for_session(state, headers, id, req).await
}

async fn send_message_for_session(
    state: Arc<AppState>,
    headers: HeaderMap,
    id: Uuid,
    req: SendMessageRequest,
) -> Result<Json<ve_shared::models::SendMessageResponse>> {
    // Extract trace_id for correlation
    let trace_id = extract_request_id(&headers);

    // Validate content
    validate_content(&req.content)?;

    let session_id_str = id.to_string();

    // Check session is not archived
    let session: (String, String) = sqlx::query_as(
        r#"
        SELECT status, host_id FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    if session.0 == "archived" {
        return Err(ServerError::SessionArchived);
    }

    let host_id = parse_uuid(&session.1, "host_id")?;

    let message_id = Uuid::new_v4();
    let message_id_str = message_id.to_string();

    let request_id = generate_request_id();
    let request = DaemonMessage::SendMessage {
        request_id: request_id.clone(),
        session_id: id,
        content: req.content.clone(),
    };

    ensure_command_acked(
        send_daemon_command_and_wait(&state, host_id, request_id, request).await?,
    )?;

    sqlx::query(
        r#"
        INSERT INTO session_messages (message_id, session_id, message_type, content)
        VALUES ($1, $2, 'user', $3)
        "#,
    )
    .bind(&message_id_str)
    .bind(&session_id_str)
    .bind(&req.content)
    .execute(&state.db)
    .await?;

    sqlx::query(
        r#"
        UPDATE sessions SET last_activity_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
        WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .execute(&state.db)
    .await?;

    tracing::debug!(trace_id = %trace_id, session_id = %id, message_id = %message_id, "Message sent");

    Ok(Json(ve_shared::models::SendMessageResponse {
        success: true,
        message_id,
    }))
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
pub async fn list_messages_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
    Query(query): Query<MessageListQuery>,
) -> Result<Json<Paginated<SessionMessage>>> {
    list_messages_for_session(state, access.session_id, query).await
}

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Query(query): Query<MessageListQuery>,
) -> Result<Json<Paginated<SessionMessage>>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    list_messages_for_session(state, id, query).await
}

async fn list_messages_for_session(
    state: Arc<AppState>,
    id: Uuid,
    query: MessageListQuery,
) -> Result<Json<Paginated<SessionMessage>>> {
    let page = query.page.unwrap_or(1);
    if page == 0 {
        return Err(ServerError::BadRequest(
            "page must be greater than 0".to_string(),
        ));
    }
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = (page - 1) * limit;
    let session_id_str = id.to_string();
    let limit_i32 = limit as i32;
    let offset_i32 = offset as i32;

    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        r#"
        SELECT message_id, session_id, message_type, content, CAST(created_at AS TEXT)
        FROM session_messages
        WHERE session_id = $1
        ORDER BY created_at ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(&session_id_str)
    .bind(limit_i32)
    .bind(offset_i32)
    .fetch_all(&state.db)
    .await?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) as count FROM session_messages WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_one(&state.db)
    .await?;
    let total = total.0 as u64;

    let messages: Result<Vec<SessionMessage>> = rows
        .into_iter()
        .map(|row| {
            Ok(SessionMessage {
                message_id: parse_uuid(&row.0, "message_id")?,
                session_id: id,
                message_type: utils::parse_message_type(&row.2),
                content: row.3,
                created_at: utils::parse_sqlite_timestamp(&row.4)
                    .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
                    .with_timezone(&chrono::Utc),
            })
        })
        .collect();

    Ok(Json(Paginated::new(messages?, total, page, limit)))
}
