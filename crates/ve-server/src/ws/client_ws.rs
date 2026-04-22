//! Client WebSocket Handler
//!
//! WebSocket endpoint for client connections.

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

use ve_shared::jwt::JwtManager;
use ve_shared::proto::WsEnvelope;

use crate::authz::{decode_ws_claims, require_client_device_id, require_session_access};
use crate::error::ServerError;
use crate::hub::WS_CHANNEL_CAPACITY;
use crate::state::AppState;

/// WebSocket authentication query parameters
#[derive(Debug, Deserialize)]
pub struct WsAuthQuery {
    pub token: String,
}

/// GET /ws/client?token=<jwt>
///
/// WebSocket upgrade handler for client connections.
pub async fn ws_client_handler(
    ws: WebSocketUpgrade,
    Query(auth): Query<WsAuthQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Response, ServerError> {
    let jwt_manager = JwtManager::new(&state.config.jwt_secret, state.config.jwt_expiration());
    let claims = decode_ws_claims(&jwt_manager, &auth.token)?;
    let device_id = require_client_device_id(&claims)?;

    tracing::info!(%device_id, "Client WebSocket connection request");

    Ok(ws.on_upgrade(move |socket| handle_client_socket(socket, state, device_id)))
}

/// Handle WebSocket connection
async fn handle_client_socket(socket: WebSocket, state: Arc<AppState>, device_id: Uuid) {
    let (mut sender, mut receiver) = socket.split();

    // Create bounded channel for sending messages
    let (tx, mut rx) = tokio::sync::mpsc::channel::<WsEnvelope>(WS_CHANNEL_CAPACITY);

    // Register connection
    state.hub.register_client(device_id, tx.clone());

    tracing::info!(%device_id, "Client WebSocket connected");

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
                if let Err(e) = handle_client_message(&state, device_id, &text).await {
                    tracing::warn!(%device_id, error = %e, "Failed to handle client message");
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!(%device_id, "Client WebSocket closed by client");
                break;
            }
            Ok(Message::Ping(_)) => {
                // Send pong (try_send for bounded channel)
                let _ = tx.try_send(WsEnvelope::new("pong", serde_json::json!({})));
            }
            _ => {}
        }
    }

    // Cleanup
    state.hub.unregister_client(&device_id);
    send_task.abort();

    tracing::info!(%device_id, "Client WebSocket disconnected");
}

/// Handle incoming client message
async fn handle_client_message(
    state: &AppState,
    device_id: Uuid,
    text: &str,
) -> Result<(), ServerError> {
    let envelope: WsEnvelope = serde_json::from_str(text)?;

    match envelope.r#type.as_str() {
        "subscribe_session" => {
            let session_id_str = envelope
                .payload
                .get("session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::BadRequest("Missing session_id".to_string()))?;
            let session_id = Uuid::parse_str(session_id_str)
                .map_err(|_| ServerError::BadRequest("Invalid session_id".to_string()))?;
            require_session_access(state, device_id, session_id).await?;
            state.hub.subscribe_session(device_id, session_id);
            tracing::debug!(%device_id, %session_id, "Client subscribed to session");
        }
        "unsubscribe_session" => {
            let session_id_str = envelope
                .payload
                .get("session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ServerError::BadRequest("Missing session_id".to_string()))?;
            let session_id = Uuid::parse_str(session_id_str)
                .map_err(|_| ServerError::BadRequest("Invalid session_id".to_string()))?;
            state.hub.unsubscribe_session(&device_id, &session_id);
            tracing::debug!(%device_id, %session_id, "Client unsubscribed from session");
        }
        "ping" => {
            // Already handled above, but send pong just in case
        }
        _ => {
            tracing::warn!(%device_id, type = %envelope.r#type, "Unknown client message type");
        }
    }

    Ok(())
}
