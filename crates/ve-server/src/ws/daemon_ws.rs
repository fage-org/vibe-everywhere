//! Daemon WebSocket Handler
//!
//! WebSocket endpoint for daemon connections.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use ve_shared::jwt::{JwtManager, TokenType};
use ve_shared::proto::WsEnvelope;
use ve_shared::types::{DaemonStatus, OnlineStatus, RiskType};

use crate::error::ServerError;
use crate::hub::WS_CHANNEL_CAPACITY;
use crate::state::AppState;
use crate::utils;

/// WebSocket authentication query parameters
#[derive(Debug, Deserialize)]
pub struct WsAuthQuery {
    pub token: String,
}

/// GET /ws/daemon?token=<jwt>
///
/// WebSocket upgrade handler for daemon connections.
pub async fn ws_daemon_handler(
    ws: WebSocketUpgrade,
    Query(auth): Query<WsAuthQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, ServerError> {
    // Verify JWT
    let jwt_manager = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration());

    let claims = jwt_manager.decode(&auth.token)?;

    if claims.r#type != TokenType::Daemon {
        return Err(ServerError::InvalidToken);
    }

    let host_id = claims
        .subject_uuid()
        .map_err(|_| ServerError::InvalidToken)?;

    tracing::info!(%host_id, "Daemon WebSocket connection request");

    Ok(ws.on_upgrade(move |socket| handle_daemon_socket(socket, state, host_id)))
}

/// Handle WebSocket connection
async fn handle_daemon_socket(socket: WebSocket, state: Arc<AppState>, host_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();

    // Create bounded channel for sending messages
    let (tx, mut rx) = tokio::sync::mpsc::channel::<WsEnvelope>(WS_CHANNEL_CAPACITY);

    // Register connection
    state.hub.register_daemon(host_id, tx.clone());

    // Update host status to online
    let _ = update_host_status(&state, host_id, OnlineStatus::Online, DaemonStatus::Healthy).await;

    tracing::info!(%host_id, "Daemon WebSocket connected");

    // Spawn task to send messages
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_daemon_message(&state, host_id, &text).await {
                    tracing::warn!(%host_id, error = %e, "Failed to handle daemon message");
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!(%host_id, "Daemon WebSocket closed by daemon");
                break;
            }
            Ok(Message::Ping(_)) => {
                // Send pong (try_send for bounded channel)
                let _ = tx.try_send(WsEnvelope::new("pong", serde_json::json!({})));
            }
            _ => {}
        }
    }

    // Cleanup - update host status to offline
    let _ = update_host_status(
        &state,
        host_id,
        OnlineStatus::Offline,
        DaemonStatus::Disconnected,
    )
    .await;
    state.hub.unregister_daemon(&host_id);
    send_task.abort();

    tracing::info!(%host_id, "Daemon WebSocket disconnected");
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
    sqlx::query!(
        r#"
        UPDATE hosts
        SET online_status = $1, daemon_status = $2, updated_at = $4
        WHERE host_id = $3
        "#,
        online_str,
        daemon_str,
        host_id_str,
        updated_at,
    )
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

    match envelope.r#type.as_str() {
        "daemon_heartbeat" => {
            // Update last_active_at
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query!(
                r#"
                UPDATE hosts
                SET last_active_at = $2, updated_at = $2
                WHERE host_id = $1
                "#,
                host_id_str,
                now,
            )
            .execute(&state.db)
            .await?;
        }
        "permission_request" => {
            // Insert permission request and broadcast to clients
            handle_permission_request(state, envelope.payload).await?;
        }
        "session_event" => {
            // Broadcast to subscribers
            let payload = envelope.payload.clone();
            if let Some(session_id_str) = payload.get("session_id").and_then(|v| v.as_str()) {
                if let Ok(session_id) = Uuid::parse_str(session_id_str) {
                    state.hub.broadcast_to_session(
                        &session_id,
                        ve_shared::proto::ClientMessage::SessionEvent {
                            session_id,
                            event_type: payload
                                .get("event_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            data: payload,
                        },
                    );
                }
            }
        }
        "session_status_update" => {
            // Update session status
            handle_session_status_update(state, envelope.payload).await?;
        }
        _ => {
            tracing::warn!(%host_id, type = %envelope.r#type, "Unknown daemon message type");
        }
    }

    Ok(())
}

/// Handle permission request from daemon
async fn handle_permission_request(
    state: &AppState,
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

    // Insert permission request
    let permission_id = Uuid::new_v4();
    let permission_id_str = permission_id.to_string();

    sqlx::query!(
        r#"
        INSERT INTO permission_requests (permission_id, session_id, risk_type, summary, target)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        permission_id_str,
        session_id_str,
        risk_type_db,
        summary,
        target,
    )
    .execute(&state.db)
    .await?;

    // Increment pending permission count in session
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query!(
        r#"
        UPDATE sessions
        SET pending_permission_count = pending_permission_count + 1, updated_at = $2
        WHERE session_id = $1
        "#,
        session_id_str,
        now,
    )
    .execute(&state.db)
    .await?;

    // Broadcast to subscribed clients
    state.hub.broadcast_to_session(
        &session_id,
        ve_shared::proto::ClientMessage::PermissionRequest {
            permission_id,
            session_id,
            risk_type,
            summary,
            target,
        },
    );

    tracing::info!(%permission_id, %session_id, ?risk_type, "Permission request received");

    Ok(())
}

/// Handle session status update from daemon
async fn handle_session_status_update(
    state: &AppState,
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

    // Check if session is archived - reject updates
    let session = sqlx::query!(
        r#"
        SELECT status FROM sessions WHERE session_id = $1
        "#,
        session_id_str_db,
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ServerError::NotFound(format!("Session {}", session_id)))?;

    if session.status == "archived" {
        tracing::warn!(%session_id, "Ignoring status update for archived session");
        return Ok(());
    }

    // Update session status
    let summary = payload
        .get("summary")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let updated_at = chrono::Utc::now().to_rfc3339();

    sqlx::query!(
        r#"
        UPDATE sessions
        SET status = $1, latest_summary = COALESCE($2, latest_summary), updated_at = $4
        WHERE session_id = $3
        "#,
        status,
        summary,
        session_id_str_db,
        updated_at,
    )
    .execute(&state.db)
    .await?;

    // Broadcast to subscribed clients
    state.hub.broadcast_to_session(
        &session_id,
        ve_shared::proto::ClientMessage::SessionStatusChanged {
            session_id,
            old_status: ve_shared::types::SessionStatus::Running,
            new_status: utils::parse_session_status(status),
        },
    );

    tracing::debug!(%session_id, %status, "Session status updated");

    Ok(())
}
