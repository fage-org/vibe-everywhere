//! Session close handler

use super::commands::{ensure_command_acked, send_daemon_command_and_wait};
use super::{extract_request_id, generate_request_id, parse_uuid};
use super::{AppState, DaemonMessage, Result, ServerError, SessionAccess};
use crate::authz::{require_client_device_id, require_session_access};
use axum::http::HeaderMap;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use std::sync::Arc;
use uuid::Uuid;
use ve_shared::jwt::Claims;

pub async fn close_session_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>> {
    close_session_for_id(state, headers, access.session_id).await
}

#[allow(clippy::type_complexity)]
pub async fn close_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    close_session_for_id(state, headers, id).await
}

async fn close_session_for_id(
    state: Arc<AppState>,
    headers: HeaderMap,
    id: Uuid,
) -> Result<Json<serde_json::Value>> {
    let trace_id = extract_request_id(&headers);

    let session_id_str = id.to_string();

    let session_row: Option<(String, String)> = sqlx::query_as(
        r#"
        SELECT status, host_id
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?;

    let session = session_row.ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    if session.0 == "archived" {
        let archive: Option<(String,)> = sqlx::query_as(
            r#"
            SELECT archive_id FROM session_archives WHERE session_id = $1
            "#,
        )
        .bind(&session_id_str)
        .fetch_optional(&state.db)
        .await?;

        return match archive {
            Some((archive_id,)) => Ok(Json(serde_json::json!({
                "success": true,
                "already_archived": true,
                "archive_id": archive_id,
            }))),
            None => Err(ServerError::Conflict(format!(
                "Session {} is archived without an archive record",
                id
            ))),
        };
    }

    let host_id = parse_uuid(&session.1, "host_id")?;
    let request_id = generate_request_id();
    let request = DaemonMessage::CloseSession {
        request_id: request_id.clone(),
        session_id: id,
    };

    ensure_command_acked(
        send_daemon_command_and_wait(&state, host_id, request_id, request).await?,
    )?;

    tracing::info!(trace_id = %trace_id, session_id = %id, "Session close acknowledged by daemon");

    Ok(Json(serde_json::json!({
        "success": true,
        "close_requested": true,
    })))
}
