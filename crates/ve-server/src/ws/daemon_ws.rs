//! Daemon WebSocket Handler
//!
//! WebSocket endpoint for daemon connections.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::jwt::JwtManager;
use ve_shared::proto::{DaemonToServer, ErrorPayload, WsEnvelope};
use ve_shared::types::{DaemonStatus, OnlineStatus, RiskType};

use crate::api::sessions::archive_session_with_metadata;
use crate::authz::{decode_ws_claims, require_daemon_host_id};
use crate::error::ServerError;
use crate::hub::WS_CHANNEL_CAPACITY;
use crate::state::AppState;
use crate::utils;

/// GET /ws/daemon with Authorization: Bearer <jwt> header
///
/// WebSocket upgrade handler for daemon connections.
pub async fn ws_daemon_handler(
    ws: WebSocketUpgrade,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, ServerError> {
    // Verify JWT
    let jwt_manager = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration());

    let claims = decode_ws_claims(&jwt_manager, auth.token())?;
    let host_id = require_daemon_host_id(&claims)?;

    tracing::info!(%host_id, "Daemon WebSocket connection request");

    Ok(ws.on_upgrade(move |socket| handle_daemon_socket(socket, state, host_id)))
}

/// Handle WebSocket connection
async fn handle_daemon_socket(socket: WebSocket, state: Arc<AppState>, host_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();

    // Create bounded channel for sending messages
    let (tx, mut rx) = tokio::sync::mpsc::channel::<WsEnvelope>(WS_CHANNEL_CAPACITY);

    // Register connection
    let connection_id = state.hub.register_daemon(host_id, tx.clone()).await;

    // Update host status to online
    let _ = update_host_status(&state, host_id, OnlineStatus::Online, DaemonStatus::Healthy).await;

    tracing::info!(%host_id, %connection_id, "Daemon WebSocket connected");

    // Spawn task to send messages
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = match serde_json::to_string(&msg) {
                Ok(j) => j,
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to serialize daemon message, dropping");
                    continue;
                }
            };
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match handle_daemon_message_if_active(&state, host_id, connection_id, &text).await {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(e) => {
                        tracing::warn!(%host_id, %connection_id, error = %e, "Failed to handle daemon message");
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!(%host_id, %connection_id, "Daemon WebSocket closed by daemon");
                break;
            }
            Ok(Message::Ping(_)) => {
                if !state
                    .hub
                    .is_active_daemon_connection(&host_id, connection_id)
                    .await
                {
                    tracing::info!(%host_id, %connection_id, "Closing stale daemon WebSocket");
                    break;
                }

                // Send pong (try_send for bounded channel)
                let _ = tx.try_send(WsEnvelope::new("pong", serde_json::json!({})));
            }
            _ => {}
        }
    }

    let was_active = state.hub.unregister_daemon(&host_id, connection_id).await;

    // Cleanup - update host status to offline only for the active connection
    if was_active {
        state
            .hub
            .fail_pending_requests_for_connection(
                &host_id,
                connection_id,
                "Daemon disconnected before responding",
            )
            .await;
        let _ = update_host_status(
            &state,
            host_id,
            OnlineStatus::Offline,
            DaemonStatus::Disconnected,
        )
        .await;
    }
    send_task.abort();

    tracing::info!(%host_id, %connection_id, active = was_active, "Daemon WebSocket disconnected");
}

/// Update host online and daemon status
async fn update_host_status(
    state: &AppState,
    host_id: Uuid,
    online_status: OnlineStatus,
    daemon_status: DaemonStatus,
) -> Result<(), sqlx::Error> {
    let host_id_str = host_id.to_string();
    let online_str = match online_status {
        OnlineStatus::Online => "online",
        OnlineStatus::Offline => "offline",
        OnlineStatus::Unknown => "unknown",
    };

    let daemon_str = match daemon_status {
        DaemonStatus::Healthy => "healthy",
        DaemonStatus::Connecting => "connecting",
        DaemonStatus::Disconnected => "disconnected",
        DaemonStatus::Error => "error",
    };

    let updated_at = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE hosts
        SET online_status = $1, daemon_status = $2, updated_at = $4
        WHERE host_id = $3
        "#,
    )
    .bind(online_str)
    .bind(daemon_str)
    .bind(&host_id_str)
    .bind(&updated_at)
    .execute(&state.db)
    .await?;

    Ok(())
}

/// Handle incoming daemon message
async fn handle_daemon_message(
    state: &AppState,
    host_id: Uuid,
    text: &str,
) -> Result<(), ServerError> {
    let envelope: WsEnvelope = serde_json::from_str(text)?;
    let host_id_str = host_id.to_string();

    if try_handle_pending_response(state, &envelope).await? {
        return Ok(());
    }

    match envelope.r#type.as_str() {
        "daemon_hello" => {
            tracing::debug!(%host_id, "Ignoring post-connect daemon_hello message");
        }
        "daemon_heartbeat" => {
            // Update last_active_at
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                r#"
                UPDATE hosts
                SET last_active_at = $2, updated_at = $2
                WHERE host_id = $1
                "#,
            )
            .bind(&host_id_str)
            .bind(&now)
            .execute(&state.db)
            .await?;
        }
        "permission_request" => {
            // Insert permission request and broadcast to clients
            handle_permission_request(state, host_id, envelope.payload).await?;
        }
        "session_event" => {
            handle_session_event(state, host_id, envelope.payload).await?;
        }
        "session_status_update" => {
            // Update session status
            handle_session_status_update(state, host_id, envelope.payload).await?;
        }
        _ => {
            tracing::warn!(%host_id, type = %envelope.r#type, "Unknown daemon message type");
        }
    }

    Ok(())
}

async fn try_handle_pending_response(
    state: &AppState,
    envelope: &WsEnvelope,
) -> Result<bool, ServerError> {
    let response = match envelope.r#type.as_str() {
        "file_tree_response" | "file_content_response" => Some(serde_json::from_value::<
            DaemonToServer,
        >(envelope.payload.clone())?),
        "ack" => {
            let ack =
                serde_json::from_value::<ve_shared::proto::AckPayload>(envelope.payload.clone())?;
            state.hub.complete_with_ack(ack).await;
            return Ok(true);
        }
        "error" => {
            let error = serde_json::from_value::<ErrorPayload>(envelope.payload.clone())?;
            state.hub.complete_with_error(error.clone()).await;
            Some(DaemonToServer::Error {
                request_id: error.request_id,
                error_code: error.error_code,
                error_message: error.error_message,
            })
        }
        _ => None,
    };

    if let Some(response) = response {
        state.hub.handle_response(response).await;
        return Ok(true);
    }

    Ok(false)
}

async fn handle_daemon_message_if_active(
    state: &AppState,
    host_id: Uuid,
    connection_id: Uuid,
    text: &str,
) -> Result<bool, ServerError> {
    if !state
        .hub
        .is_active_daemon_connection(&host_id, connection_id)
        .await
    {
        tracing::warn!(%host_id, %connection_id, "Ignoring message from stale daemon connection");
        return Ok(false);
    }

    handle_daemon_message(state, host_id, text).await?;
    Ok(true)
}

enum SessionAvailability {
    Active,
    Archived,
}

async fn load_host_session_status_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Any>,
    host_id: Uuid,
    session_id: Uuid,
) -> Result<Option<String>, ServerError> {
    let session = sqlx::query_as(
        r#"
        SELECT status
        FROM sessions
        WHERE session_id = $1 AND host_id = $2
        "#,
    )
    .bind(session_id.to_string())
    .bind(host_id.to_string())
    .fetch_optional(&mut **tx)
    .await?;

    Ok(session.map(|(status,)| status))
}

async fn guard_active_host_session_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Any>,
    host_id: Uuid,
    session_id: Uuid,
) -> Result<SessionAvailability, ServerError> {
    let lock_result = sqlx::query(
        r#"
        UPDATE sessions
        SET updated_at = updated_at
        WHERE session_id = $1 AND host_id = $2 AND status != 'archived'
        "#,
    )
    .bind(session_id.to_string())
    .bind(host_id.to_string())
    .execute(&mut **tx)
    .await?;

    if lock_result.rows_affected() > 0 {
        return Ok(SessionAvailability::Active);
    }

    match load_host_session_status_tx(tx, host_id, session_id).await? {
        Some(status) if status == "archived" => Ok(SessionAvailability::Archived),
        Some(_) => Err(ServerError::Conflict(format!(
            "Failed to acquire active session {}",
            session_id
        ))),
        None => Err(ServerError::NotFound(format!("Session {}", session_id))),
    }
}

async fn handle_session_event(
    state: &AppState,
    host_id: Uuid,
    payload: serde_json::Value,
) -> Result<(), ServerError> {
    let session_id_str = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::BadRequest("Missing session_id".to_string()))?;

    let session_id = Uuid::parse_str(session_id_str)
        .map_err(|_| ServerError::BadRequest("Invalid session_id".to_string()))?;
    let event_type = payload
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let mut tx = state.db.begin().await?;
    match guard_active_host_session_tx(&mut tx, host_id, session_id).await? {
        SessionAvailability::Archived => {
            tracing::warn!(%session_id, %event_type, "Ignoring session event for archived session");
            tx.commit().await?;
            return Ok(());
        }
        SessionAvailability::Active => {}
    }

    if event_type == "claude_session_id" {
        if let Some(claude_session_id) = payload
            .get("data")
            .and_then(|v| v.get("claude_session_id"))
            .and_then(|v| v.as_str())
        {
            let updated_at = chrono::Utc::now().to_rfc3339();
            sqlx::query(
                r#"
                UPDATE sessions
                SET claude_session_id = $2,
                    can_resume_cross_device = $3,
                    updated_at = $4
                WHERE session_id = $1 AND host_id = $5 AND status != 'archived'
                "#,
            )
            .bind(session_id.to_string())
            .bind(claude_session_id)
            .bind(true)
            .bind(&updated_at)
            .bind(host_id.to_string())
            .execute(&mut *tx)
            .await?;
        }
    }

    if event_type == "agent_reply" {
        if let Some(content) = payload
            .get("data")
            .and_then(|v| v.get("content"))
            .and_then(|v| v.as_str())
        {
            let message_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                r#"
                INSERT INTO session_messages (message_id, session_id, message_type, content)
                VALUES ($1, $2, 'assistant', $3)
                "#,
            )
            .bind(&message_id)
            .bind(session_id.to_string())
            .bind(content)
            .execute(&mut *tx)
            .await?;
            tracing::info!(%session_id, message_id, "Agent reply saved to session_messages");
        }
    }

    tx.commit().await?;

    state
        .hub
        .broadcast_to_session(
            &state.db,
            &session_id,
            ve_shared::proto::ClientMessage::SessionEvent {
                session_id,
                event_type,
                data: payload,
            },
        )
        .await;

    Ok(())
}

async fn handle_permission_request(
    state: &AppState,
    host_id: Uuid,
    payload: serde_json::Value,
) -> Result<(), ServerError> {
    let session_id_str = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::BadRequest("Missing session_id".to_string()))?;

    let session_id = Uuid::parse_str(session_id_str)
        .map_err(|_| ServerError::BadRequest("Invalid session_id".to_string()))?;
    let session_id_str = session_id.to_string();

    let risk_type_str = payload
        .get("risk_type")
        .and_then(|v| v.as_str())
        .unwrap_or("write_fs");

    let risk_type = match risk_type_str {
        "write_fs" => RiskType::WriteFs,
        "exec_cmd" => RiskType::ExecCmd,
        "network" => RiskType::Network,
        _ => RiskType::WriteFs,
    };

    let summary = payload
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let target = payload
        .get("target")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let risk_type_db = match risk_type {
        RiskType::WriteFs => "write_fs",
        RiskType::ExecCmd => "exec_cmd",
        RiskType::Network => "network",
    };

    let permission_id_str = payload
        .get("permission_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::BadRequest("Missing permission_id".to_string()))?;
    let permission_id = Uuid::parse_str(permission_id_str)
        .map_err(|_| ServerError::BadRequest("Invalid permission_id".to_string()))?;
    let permission_id_str = permission_id.to_string();

    let mut tx = state.db.begin().await?;
    match guard_active_host_session_tx(&mut tx, host_id, session_id).await? {
        SessionAvailability::Archived => {
            return Err(ServerError::BadRequest(
                "Cannot create permission request for archived session".to_string(),
            ));
        }
        SessionAvailability::Active => {}
    }
    let created_at = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, target, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&permission_id_str)
    .bind(&session_id_str)
    .bind(risk_type_db)
    .bind(&summary)
    .bind(&target)
    .bind(&created_at)
    .execute(&mut *tx)
    .await?;

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        UPDATE sessions
        SET pending_permission_count = pending_permission_count + 1, updated_at = $2
        WHERE session_id = $1 AND host_id = $3 AND status != 'archived'
        "#,
    )
    .bind(&session_id_str)
    .bind(&now)
    .bind(host_id.to_string())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    state
        .hub
        .broadcast_to_session(
            &state.db,
            &session_id,
            ve_shared::proto::ClientMessage::PermissionRequest {
                permission_id,
                session_id,
                risk_type,
                summary,
                target,
            },
        )
        .await;

    tracing::info!(%permission_id, %session_id, ?risk_type, "Permission request received");

    Ok(())
}

/// Handle session status update from daemon
async fn handle_session_status_update(
    state: &AppState,
    host_id: Uuid,
    payload: serde_json::Value,
) -> Result<(), ServerError> {
    let session_id_str = payload
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ServerError::BadRequest("Missing session_id".to_string()))?;

    let session_id = Uuid::parse_str(session_id_str)
        .map_err(|_| ServerError::BadRequest("Invalid session_id".to_string()))?;
    let session_id_str_db = session_id.to_string();

    let status = payload
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("running");

    let summary = payload
        .get("summary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut tx = state.db.begin().await?;
    match guard_active_host_session_tx(&mut tx, host_id, session_id).await? {
        SessionAvailability::Archived => {
            tracing::warn!(%session_id, "Ignoring status update for archived session");
            tx.commit().await?;
            return Ok(());
        }
        SessionAvailability::Active => {}
    }

    if status == "archived" {
        let close_reason = payload
            .get("close_reason")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ServerError::BadRequest(
                    "Missing close_reason for archived status update".to_string(),
                )
            })?;
        match close_reason {
            "user_closed" | "completed" | "failed" | "terminated" => {}
            _ => {
                return Err(ServerError::BadRequest(
                    "Invalid close_reason for archived status update".to_string(),
                ));
            }
        }

        tx.commit().await?;
        archive_session_with_metadata(state, session_id, close_reason, summary.clone()).await?;

        state
            .hub
            .broadcast_to_session(
                &state.db,
                &session_id,
                ve_shared::proto::ClientMessage::SessionStatusChanged {
                    session_id,
                    new_status: utils::parse_session_status(status),
                    close_reason: Some(utils::parse_close_reason(close_reason)),
                },
            )
            .await;
    } else {
        let updated_at = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE sessions
            SET status = $1, latest_summary = COALESCE($2, latest_summary), updated_at = $4
            WHERE session_id = $3 AND host_id = $5 AND status != 'archived'
            "#,
        )
        .bind(status)
        .bind(&summary)
        .bind(&session_id_str_db)
        .bind(&updated_at)
        .bind(host_id.to_string())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        state
            .hub
            .broadcast_to_session(
                &state.db,
                &session_id,
                ve_shared::proto::ClientMessage::SessionStatusChanged {
                    session_id,
                    new_status: utils::parse_session_status(status),
                    close_reason: None,
                },
            )
            .await;
    }

    tracing::debug!(%session_id, %status, "Session status updated");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseBackend};
    use crate::db::{install_drivers, run_migrations, DbPool};
    use crate::hub::Hub;
    use crate::state::AppState;
    use std::time::Duration;

    fn test_config(database_url: String) -> Config {
        Config {
            listen_addr: "127.0.0.1:3000".parse().unwrap(),
            database_url,
            jwt_secret: "01234567890123456789012345678901".to_string(),
            jwt_expiration_secs: 3600,
            pair_code_ttl_secs: 300,
            heartbeat_interval_secs: 30,
            connection_timeout_secs: 60,
            data_dir: std::path::PathBuf::from("/tmp"),
            cors_origins: Vec::new(),
            ack_timeout_ms: 10000,
            ack_max_retries: 2,
            ack_retry_delay_ms: 500,
            permission_ttl_secs: 1800,
            permission_expiry_check_secs: 60,
            idempotency_ttl_secs: 86400,
            idempotency_cleanup_secs: 3600,
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
        }
    }

    async fn setup_state() -> Arc<AppState> {
        install_drivers();
        let temp_db = std::env::temp_dir().join(format!("ve-daemon-ws-test-{}.db", Uuid::new_v4()));
        let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
        let pool = DbPool::connect(&database_url).await.unwrap();
        run_migrations(&pool, DatabaseBackend::Sqlite)
            .await
            .unwrap();

        Arc::new(AppState::new(pool, Hub::new(), test_config(database_url)))
    }

    #[tokio::test]
    async fn archived_session_event_is_ignored() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO sessions (session_id, title, host_id, workspace_id, status) VALUES ($1, $2, $3, $4, 'archived')")
            .bind(session_id.to_string())
            .bind("test")
            .bind(host_id.to_string())
            .bind(workspace_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_client(device_id, tx);
        state.hub.subscribe_session(device_id, session_id);

        let envelope = WsEnvelope::new(
            "session_event",
            serde_json::json!({
                "session_id": session_id.to_string(),
                "event_type": "agent_reply",
                "data": { "content": "late event" }
            }),
        );

        handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
            .await
            .unwrap();

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn daemon_hello_after_ws_connect_is_ignored_without_error() {
        let state = setup_state().await;
        let host_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        let envelope = WsEnvelope::new(
            "daemon_hello",
            serde_json::json!({
                "host_id": host_id.to_string(),
                "host_name": "host",
                "platform": "linux"
            }),
        );

        handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn daemon_archived_status_creates_archive_record_with_payload_close_reason() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let envelope = WsEnvelope::new(
            "session_status_update",
            serde_json::json!({
                "session_id": session_id.to_string(),
                "status": "archived",
                "close_reason": "terminated",
                "summary": "terminated by control",
            }),
        );

        handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
            .await
            .unwrap();

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "archived");

        let archive: (String, String, Option<String>) = sqlx::query_as(
            "SELECT close_reason, metadata_json, (SELECT latest_summary FROM sessions WHERE session_id = $1) FROM session_archives WHERE session_id = $1",
        )
        .bind(session_id.to_string())
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(archive.0, "terminated");
        let metadata: ve_shared::models::ArchiveMetadata =
            serde_json::from_str(&archive.1).unwrap();
        assert_eq!(
            metadata.final_summary.as_deref(),
            Some("terminated by control")
        );
        assert_eq!(archive.2.as_deref(), Some("terminated by control"));
    }

    #[tokio::test]
    async fn daemon_archived_status_requires_close_reason() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let envelope = WsEnvelope::new(
            "session_status_update",
            serde_json::json!({
                "session_id": session_id.to_string(),
                "status": "archived",
                "summary": "closed without reason",
            }),
        );

        let error =
            handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
                .await
                .unwrap_err();
        assert!(
            matches!(error, ServerError::BadRequest(message) if message == "Missing close_reason for archived status update")
        );

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "running");

        let archive_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count.0, 0);
    }

    #[tokio::test]
    async fn daemon_archived_status_rejects_invalid_close_reason() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let envelope = WsEnvelope::new(
            "session_status_update",
            serde_json::json!({
                "session_id": session_id.to_string(),
                "status": "archived",
                "close_reason": "bogus_reason",
                "summary": "closed with invalid reason",
            }),
        );

        let error =
            handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
                .await
                .unwrap_err();
        assert!(
            matches!(error, ServerError::BadRequest(message) if message == "Invalid close_reason for archived status update")
        );

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "running");

        let archive_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM session_archives WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(archive_count.0, 0);
    }

    #[tokio::test]
    async fn session_event_rejects_session_from_other_host() {
        let state = setup_state().await;
        let host_id = Uuid::new_v4();
        let other_host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(other_host_id.to_string())
            .bind("other-host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(other_host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, status) VALUES ($1, $2, $3, $4, 'running')",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(other_host_id.to_string())
        .bind(workspace_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

        let envelope = WsEnvelope::new(
            "session_event",
            serde_json::json!({
                "session_id": session_id.to_string(),
                "event_type": "agent_reply",
                "data": { "content": "forged" }
            }),
        );

        let error =
            handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
                .await
                .unwrap_err();
        assert!(
            matches!(error, ServerError::NotFound(message) if message == format!("Session {}", session_id))
        );
    }

    #[tokio::test]
    async fn permission_request_rejects_session_from_other_host() {
        let state = setup_state().await;
        let host_id = Uuid::new_v4();
        let other_host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let permission_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(other_host_id.to_string())
            .bind("other-host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(other_host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, status) VALUES ($1, $2, $3, $4, 'running')",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(other_host_id.to_string())
        .bind(workspace_id.to_string())
        .execute(&state.db)
        .await
        .unwrap();

        let envelope = WsEnvelope::new(
            "permission_request",
            serde_json::json!({
                "permission_id": permission_id.to_string(),
                "session_id": session_id.to_string(),
                "risk_type": "write_fs",
                "summary": "forged",
                "target": "/tmp"
            }),
        );

        let error =
            handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
                .await
                .unwrap_err();
        assert!(
            matches!(error, ServerError::NotFound(message) if message == format!("Session {}", session_id))
        );

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM permission_requests WHERE permission_id = $1")
                .bind(permission_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(count.0, 0);
    }

    #[tokio::test]
    async fn session_status_update_rejects_session_from_other_host() {
        let state = setup_state().await;
        let host_id = Uuid::new_v4();
        let other_host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(other_host_id.to_string())
            .bind("other-host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(other_host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(other_host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        let envelope = WsEnvelope::new(
            "session_status_update",
            serde_json::json!({
                "session_id": session_id.to_string(),
                "status": "archived",
                "close_reason": "terminated"
            }),
        );

        let error =
            handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
                .await
                .unwrap_err();
        assert!(
            matches!(error, ServerError::NotFound(message) if message == format!("Session {}", session_id))
        );

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "running");
    }

    #[tokio::test]
    async fn daemon_archived_status_broadcasts_close_reason_to_clients() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_client(device_id, tx);
        state.hub.subscribe_session(device_id, session_id);

        let envelope = WsEnvelope::new(
            "session_status_update",
            serde_json::json!({
                "session_id": session_id.to_string(),
                "status": "archived",
                "close_reason": "terminated",
                "summary": "terminated by control",
            }),
        );

        handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
            .await
            .unwrap();

        let message = rx.try_recv().unwrap();
        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["type"], "session_status_changed");
        assert_eq!(json["payload"]["type"], "session_status_changed");
        assert_eq!(
            json["payload"]["payload"]["session_id"],
            session_id.to_string()
        );
        assert_eq!(json["payload"]["payload"]["new_status"], "archived");
        assert_eq!(json["payload"]["payload"]["close_reason"], "terminated");
    }

    #[tokio::test]
    async fn archived_permission_request_is_rejected() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let permission_id = Uuid::new_v4();
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO sessions (session_id, title, host_id, workspace_id, status) VALUES ($1, $2, $3, $4, 'archived')")
            .bind(session_id.to_string())
            .bind("test")
            .bind(host_id.to_string())
            .bind(workspace_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_client(device_id, tx);
        state.hub.subscribe_session(device_id, session_id);

        let envelope = WsEnvelope::new(
            "permission_request",
            serde_json::json!({
                "permission_id": permission_id.to_string(),
                "session_id": session_id.to_string(),
                "risk_type": "write_fs",
                "summary": "late permission",
                "target": "/tmp"
            }),
        );

        let error =
            handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
                .await
                .unwrap_err();
        assert!(
            matches!(error, ServerError::BadRequest(message) if message == "Cannot create permission request for archived session")
        );

        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM permission_requests WHERE permission_id = $1")
                .bind(permission_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(count.0, 0);

        let pending_count: (i64,) =
            sqlx::query_as("SELECT pending_permission_count FROM sessions WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(pending_count.0, 0);

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn file_content_response_completes_pending_hub_request() {
        let state = setup_state().await;
        let host_id = Uuid::new_v4();
        let request_id = Uuid::new_v4().to_string();
        let file_path = "src/main.rs".to_string();

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        state.hub.register_daemon(host_id, tx).await;

        let wait = tokio::spawn({
            let hub = state.hub.clone();
            let request_id = request_id.clone();
            async move {
                hub.send_and_wait(
                    &host_id,
                    ve_shared::proto::DaemonMessage::Pong,
                    request_id,
                    Duration::from_secs(1),
                )
                .await
                .unwrap()
            }
        });

        let _sent = rx.recv().await.unwrap();
        let response = DaemonToServer::FileContentResponse {
            request_id: request_id.clone(),
            file_path,
            content: "fn main() {}".to_string(),
            file_type: "text".to_string(),
        };
        let envelope = WsEnvelope::new("file_content_response", &response);
        let text = serde_json::to_string(&envelope).unwrap();

        handle_daemon_message(&state, host_id, &text).await.unwrap();

        match wait.await.unwrap() {
            crate::hub::DaemonResponse::Message(DaemonToServer::FileContentResponse {
                request_id: completed,
                ..
            }) => assert_eq!(completed, request_id),
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[tokio::test]
    async fn daemon_error_response_completes_pending_hub_request() {
        let state = setup_state().await;
        let host_id = Uuid::new_v4();
        let request_id = Uuid::new_v4().to_string();

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        state.hub.register_daemon(host_id, tx).await;

        let wait = tokio::spawn({
            let hub = state.hub.clone();
            let request_id = request_id.clone();
            async move {
                hub.send_and_wait(
                    &host_id,
                    ve_shared::proto::DaemonMessage::Pong,
                    request_id,
                    Duration::from_secs(1),
                )
                .await
                .unwrap()
            }
        });

        let _sent = rx.recv().await.unwrap();
        let envelope = WsEnvelope::new(
            "error",
            serde_json::json!({
                "request_id": request_id,
                "error_code": "file_access_denied",
                "error_message": "Access denied"
            }),
        );
        let text = serde_json::to_string(&envelope).unwrap();

        handle_daemon_message(&state, host_id, &text).await.unwrap();

        match wait.await.unwrap() {
            crate::hub::DaemonResponse::Error(error) => {
                assert_eq!(error.error_code, "file_access_denied");
                assert_eq!(error.error_message, "Access denied");
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[tokio::test]
    async fn stale_daemon_connection_messages_are_rejected() {
        let state = setup_state().await;
        let host_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        let (tx1, _rx1) = tokio::sync::mpsc::channel(4);
        let (tx2, _rx2) = tokio::sync::mpsc::channel(4);
        let stale_connection_id = state.hub.register_daemon(host_id, tx1).await;
        let active_connection_id = state.hub.register_daemon(host_id, tx2).await;

        let envelope = WsEnvelope::new("daemon_heartbeat", serde_json::json!({}));
        let handled = handle_daemon_message_if_active(
            &state,
            host_id,
            stale_connection_id,
            &serde_json::to_string(&envelope).unwrap(),
        )
        .await
        .unwrap();

        assert!(!handled);
        assert!(
            state
                .hub
                .is_active_daemon_connection(&host_id, active_connection_id)
                .await
        );
    }

    #[tokio::test]
    async fn stale_daemon_disconnect_does_not_unregister_active_connection() {
        let state = setup_state().await;
        let host_id = Uuid::new_v4();

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        let (tx1, _rx1) = tokio::sync::mpsc::channel(4);
        let (tx2, mut rx2) = tokio::sync::mpsc::channel(4);
        let stale_connection_id = state.hub.register_daemon(host_id, tx1).await;
        let active_connection_id = state.hub.register_daemon(host_id, tx2).await;

        assert!(
            !state
                .hub
                .unregister_daemon(&host_id, stale_connection_id)
                .await
        );
        assert!(
            state
                .hub
                .is_active_daemon_connection(&host_id, active_connection_id)
                .await
        );

        let sent = state
            .hub
            .send_to_daemon(
                &host_id,
                ve_shared::proto::DaemonMessage::SessionControl {
                    request_id: Uuid::new_v4().to_string(),
                    session_id: Uuid::new_v4(),
                    action: ve_shared::proto::SessionControlAction::Pause,
                },
            )
            .await;

        assert!(sent);
        assert!(rx2.try_recv().is_ok());
    }

    #[tokio::test]
    async fn session_status_update_commits_before_broadcasting() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'pending', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("test")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_client(device_id, tx);
        state.hub.subscribe_session(device_id, session_id);

        let envelope = WsEnvelope::new(
            "session_status_update",
            serde_json::json!({
                "session_id": session_id.to_string(),
                "status": "running",
            }),
        );

        handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
            .await
            .unwrap();

        let message = rx.try_recv().unwrap();
        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["type"], "session_status_changed");
        assert_eq!(json["payload"]["payload"]["new_status"], "running");

        let status: (String,) = sqlx::query_as("SELECT status FROM sessions WHERE session_id = $1")
            .bind(session_id.to_string())
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(status.0, "running");
    }

    #[tokio::test]
    async fn permission_request_reuses_daemon_permission_id() {
        let state = setup_state().await;
        let session_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let device_id = Uuid::new_v4();
        let permission_id = Uuid::new_v4();
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);

        sqlx::query("INSERT INTO hosts (host_id, host_name, platform) VALUES ($1, $2, $3)")
            .bind(host_id.to_string())
            .bind("host")
            .bind("linux")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO workspaces (workspace_id, host_id, path, display_name) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id.to_string())
            .bind(host_id.to_string())
            .bind("/tmp")
            .bind("tmp")
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO sessions (session_id, title, host_id, workspace_id, status) VALUES ($1, $2, $3, $4, 'running')")
            .bind(session_id.to_string())
            .bind("test")
            .bind(host_id.to_string())
            .bind(workspace_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&state.db)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&state.db)
            .await
            .unwrap();

        state.hub.register_client(device_id, tx);
        state.hub.subscribe_session(device_id, session_id);

        let envelope = WsEnvelope::new(
            "permission_request",
            serde_json::json!({
                "permission_id": permission_id.to_string(),
                "session_id": session_id.to_string(),
                "risk_type": "write_fs",
                "summary": "need access",
                "target": "/tmp"
            }),
        );

        handle_daemon_message(&state, host_id, &serde_json::to_string(&envelope).unwrap())
            .await
            .unwrap();

        let stored: (String,) =
            sqlx::query_as("SELECT permission_id FROM permission_requests WHERE session_id = $1")
                .bind(session_id.to_string())
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert_eq!(stored.0, permission_id.to_string());

        let message = rx.try_recv().unwrap();
        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["type"], "permission_request");
        assert_eq!(json["payload"]["type"], "permission_request");
        assert_eq!(
            json["payload"]["payload"]["permission_id"],
            permission_id.to_string()
        );
    }
}
