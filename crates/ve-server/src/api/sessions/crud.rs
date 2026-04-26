//! Session CRUD handlers
//!
//! List, create, and get sessions.

use super::commands::{ensure_command_acked, mark_session_error, send_daemon_command_and_wait};
use super::IdempotencyKeyStore;
use super::{
    authorize_session_create, extract_request_id, generate_request_id, idempotency_duplicate_error,
    load_existing_idempotent_session, load_session_for_idempotency, parse_uuid, sqlite_busy_error,
    utils, validate_content, validate_idempotency_key, validate_title,
    wait_for_existing_idempotent_session, AppState, ClientAccess, CreateSessionRequest,
    DaemonMessage, Result, ServerError, Session, SessionAccess, SessionCollectionAccess,
    SessionRecord,
};
use crate::authz::{require_client_device_id, require_session_access};
use axum::http::HeaderMap;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use ve_shared::jwt::Claims;

#[cfg(debug_assertions)]
use super::test_support;

pub async fn list_sessions(
    access: SessionCollectionAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Session>>> {
    let rows: Vec<SessionRecord> = if let Some(host_id) = access.host_id {
        sqlx::query_as(
            r#"
                SELECT sessions.session_id, sessions.title, sessions.host_id, sessions.workspace_id,
                       sessions.agent_type, sessions.status, CAST(sessions.last_activity_at AS TEXT) AS last_activity_at,
                       sessions.latest_summary, sessions.unread_event_count,
                       sessions.pending_permission_count,
                       CASE WHEN sessions.can_resume_cross_device THEN 1 ELSE 0 END AS can_resume_cross_device,
                       sessions.claude_session_id, CAST(sessions.created_at AS TEXT) AS created_at,
                       CAST(sessions.updated_at AS TEXT) AS updated_at
                FROM sessions
                INNER JOIN device_session_access
                    ON device_session_access.session_id = sessions.session_id
                WHERE sessions.host_id = $1
                  AND device_session_access.device_id = $2
                  AND sessions.status != 'archived'
                ORDER BY sessions.updated_at DESC
                "#,
        )
        .bind(host_id.to_string())
        .bind(access.device_id.to_string())
        .fetch_all(&state.db)
        .await?
    } else {
        sqlx::query_as(
            r#"
                SELECT sessions.session_id, sessions.title, sessions.host_id, sessions.workspace_id,
                       sessions.agent_type, sessions.status, CAST(sessions.last_activity_at AS TEXT) AS last_activity_at,
                       sessions.latest_summary, sessions.unread_event_count,
                       sessions.pending_permission_count,
                       CASE WHEN sessions.can_resume_cross_device THEN 1 ELSE 0 END AS can_resume_cross_device,
                       sessions.claude_session_id, CAST(sessions.created_at AS TEXT) AS created_at,
                       CAST(sessions.updated_at AS TEXT) AS updated_at
                FROM sessions
                INNER JOIN device_session_access
                    ON device_session_access.session_id = sessions.session_id
                WHERE device_session_access.device_id = $1
                  AND sessions.status != 'archived'
                ORDER BY sessions.updated_at DESC
                "#,
        )
        .bind(access.device_id.to_string())
        .fetch_all(&state.db)
        .await?
    };

    let sessions: Result<Vec<Session>> = rows
        .into_iter()
        .map(|record| record.to_model())
        .collect();

    Ok(Json(sessions?))
}

/// POST /api/sessions
///
/// Create a new session with strict idempotency protection.
pub async fn create_session(
    client: ClientAccess,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<Session>> {
    let device_id = client.device_id;
    let trace_id = extract_request_id(&headers);

    validate_title(&req.title)?;
    validate_content(&req.initial_message)?;
    validate_idempotency_key(&req.idempotency_key)?;
    authorize_session_create(&state, device_id, &req).await?;

    let idempotency_key = req.idempotency_key.clone();
    let request_hash = IdempotencyKeyStore::compute_hash(
        &serde_json::to_string(&serde_json::json!({
            "host_id": req.host_id,
            "workspace_id": req.workspace_id,
            "title": req.title,
            "initial_message": req.initial_message,
        }))
        .map_err(|error| {
            ServerError::Internal(format!("Failed to serialize request hash: {}", error))
        })?,
    );
    let store = IdempotencyKeyStore::new(state.db.clone(), state.config.idempotency_ttl_secs);

    if let Some(existing) = store.get(&idempotency_key).await? {
        return load_existing_idempotent_session(
            &state,
            &trace_id,
            &idempotency_key,
            &request_hash,
            existing,
        )
        .await;
    }

    #[cfg(debug_assertions)]
    test_support::wait_for_create_session_race(&idempotency_key).await;

    let session_id = Uuid::new_v4();
    let session_id_str = session_id.to_string();
    let host_id_str = req.host_id.to_string();
    let workspace_id_str = req.workspace_id.to_string();
    let workspace: (String,) = sqlx::query_as(
        r#"
        SELECT path FROM workspaces WHERE workspace_id = $1 AND host_id = $2
        "#,
    )
    .bind(&workspace_id_str)
    .bind(&host_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!(
        "Workspace {}",
        req.workspace_id
    )))?;

    let message_id = Uuid::new_v4();
    let message_id_str = message_id.to_string();

    let tx_result: Result<()> = async {
        let mut tx = state.db.begin().await?;

        if sqlx::query_scalar::<_, String>(
            r#"
            SELECT session_id
            FROM idempotency_keys
            WHERE key = $1
            "#,
        )
        .bind(&idempotency_key)
        .fetch_optional(&mut *tx)
        .await?
        .is_some()
        {
            tx.rollback().await?;
            return Err(ServerError::Conflict("idempotency-existing".to_string()));
        }

        sqlx::query(
            r#"
            INSERT INTO sessions (
                session_id, title, host_id, workspace_id, agent_type
            )
            VALUES ($1, $2, $3, $4, 'claude_code')
            "#,
        )
        .bind(&session_id_str)
        .bind(&req.title)
        .bind(&host_id_str)
        .bind(&workspace_id_str)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO session_messages (message_id, session_id, message_type, content)
            VALUES ($1, $2, 'user', $3)
            "#,
        )
        .bind(&message_id_str)
        .bind(&session_id_str)
        .bind(&req.initial_message)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO idempotency_keys (key, request_hash, session_id, result_type)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(&idempotency_key)
        .bind(&request_hash)
        .bind(&session_id_str)
        .bind("session")
        .execute(&mut *tx)
        .await
        .map_err(|error| {
            if idempotency_duplicate_error(&error) {
                ServerError::Conflict("idempotency-conflict".to_string())
            } else {
                error.into()
            }
        })?;

        sqlx::query(
            r#"
            INSERT INTO device_session_access (device_id, session_id)
            SELECT device_host_access.device_id, $2
            FROM device_host_access
            WHERE device_host_access.host_id = $1
              AND NOT EXISTS (
                  SELECT 1
                  FROM device_session_access
                  WHERE device_id = device_host_access.device_id AND session_id = $2
              )
            "#,
        )
        .bind(&host_id_str)
        .bind(&session_id_str)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }
    .await;

    let tx_result = match tx_result {
        Err(ServerError::Database(error)) if sqlite_busy_error(&error) => {
            if let Some(existing) = wait_for_existing_idempotent_session(
                &state,
                &trace_id,
                &idempotency_key,
                &request_hash,
                10,
                Duration::from_millis(10),
            )
            .await?
            {
                return Ok(existing);
            }
            Err(ServerError::Database(error))
        }
        other => other,
    };

    if let Err(ServerError::Conflict(message)) = &tx_result {
        if message == "idempotency-conflict" || message == "idempotency-existing" {
            if let Some(existing) = wait_for_existing_idempotent_session(
                &state,
                &trace_id,
                &idempotency_key,
                &request_hash,
                10,
                Duration::from_millis(10),
            )
            .await?
            {
                return Ok(existing);
            }
            return Err(ServerError::Internal(
                "Idempotency key disappeared after conflict".into(),
            ));
        }
    }
    tx_result?;

    let request_id = generate_request_id();
    let command = DaemonMessage::CreateSession {
        request_id: request_id.clone(),
        session_id,
        workspace_path: workspace.0,
        agent_type: "claude_code".to_string(),
        initial_message: req.initial_message,
    };

    match send_daemon_command_and_wait(&state, req.host_id, request_id, command).await {
        Ok(response) => {
            if let Err(error) = ensure_command_acked(response) {
                mark_session_error(
                    &state,
                    &session_id_str,
                    "Failed to queue create_session request to daemon",
                )
                .await?;
                return Err(error);
            }
        }
        Err(error) => {
            mark_session_error(
                &state,
                &session_id_str,
                "Failed to queue create_session request to daemon",
            )
            .await?;
            return Err(error);
        }
    }

    tracing::info!(trace_id = %trace_id, session_id = %session_id, host_id = %req.host_id, "Session created");

    load_session_for_idempotency(&state, session_id).await
}

/// GET /api/sessions/:id
///
/// Get session details.
pub async fn get_session_route(
    access: SessionAccess,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Session>> {
    get_session_by_id(state, access.session_id).await
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
) -> Result<Json<Session>> {
    let device_id = require_client_device_id(&claims)?;
    require_session_access(&state, device_id, id).await?;
    get_session_by_id(state, id).await
}

async fn get_session_by_id(state: Arc<AppState>, id: Uuid) -> Result<Json<Session>> {
    let session_id_str = id.to_string();

    let row: SessionRecord = sqlx::query_as(
        r#"
        SELECT session_id, title, host_id, workspace_id, agent_type, status,
               CAST(last_activity_at AS TEXT) AS last_activity_at, latest_summary, unread_event_count,
               pending_permission_count, CASE WHEN can_resume_cross_device THEN 1 ELSE 0 END AS can_resume_cross_device,
               claude_session_id,
               CAST(created_at AS TEXT) AS created_at, CAST(updated_at AS TEXT) AS updated_at
        FROM sessions WHERE session_id = $1
        "#,
    )
    .bind(&session_id_str)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ServerError::NotFound(format!("Session {}", id)))?;

    Ok(Json(Session {
        session_id: id,
        title: row.title.clone(),
        host_id: parse_uuid(&row.host_id, "host_id")?,
        workspace_id: parse_uuid(&row.workspace_id, "workspace_id")?,
        agent_type: row.agent_type.clone(),
        status: utils::parse_session_status(&row.status)?,
        last_activity_at: row.last_activity_at.as_ref().and_then(|s| {
            utils::parse_sqlite_timestamp(s)
                .ok()
                .map(|d| d.with_timezone(&chrono::Utc))
        }),
        latest_summary: row.latest_summary.clone(),
        unread_event_count: row.unread_event_count as i32,
        pending_permission_count: row.pending_permission_count as i32,
        can_resume_cross_device: row.can_resume_cross_device != 0,
        claude_session_id: row.claude_session_id.clone(),
        created_at: utils::parse_sqlite_timestamp(&row.created_at)
            .map_err(|e| ServerError::Internal(format!("Invalid created_at: {}", e)))?
            .with_timezone(&chrono::Utc),
        updated_at: utils::parse_sqlite_timestamp(&row.updated_at)
            .map_err(|e| ServerError::Internal(format!("Invalid updated_at: {}", e)))?
            .with_timezone(&chrono::Utc),
    }))
}
