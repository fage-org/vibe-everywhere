use std::collections::{HashMap, HashSet};
use std::time::Duration;

use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::time::{timeout_at, Instant};
use tracing::{debug, info, warn};
use uuid::Uuid;

use ve_shared::proto::{
    AckPayload, ClientMessage, DaemonMessage, DaemonToServer, ErrorPayload, WsEnvelope,
};

use crate::db::DbPool;

/// Error type for `Hub::send_and_wait` responses.
#[derive(Debug, thiserror::Error)]
pub enum HubError {
    #[error("no active daemon connection for host")]
    NoActiveConnection,

    #[error("Request timeout after {0}ms")]
    Timeout(u128),

    #[error("failed to send request to daemon: {0}")]
    SendFailed(String),

    #[error("daemon returned an error: {reason}")]
    RemoteError { reason: String },

    #[error("response channel closed before reply")]
    ChannelClosed,
}

/// Default bounded channel capacity for WebSocket connections
pub const WS_CHANNEL_CAPACITY: usize = 256;

/// Type alias for WebSocket message sender (bounded)
pub type WsSender = mpsc::Sender<WsEnvelope>;

/// Connection metadata for a daemon
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DaemonConnection {
    pub host_id: Uuid,
    pub connection_id: Uuid,
    pub sender: WsSender,
}

/// Connection metadata for a client
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClientConnection {
    pub device_id: Uuid,
    pub sender: WsSender,
}

type PendingResult = std::result::Result<DaemonResponse, String>;

#[derive(Debug)]
struct PendingDaemonResponse {
    host_id: Uuid,
    connection_id: Option<Uuid>,
    tx: oneshot::Sender<PendingResult>,
}

#[derive(Debug, Clone)]
pub enum DaemonResponse {
    Ack(AckPayload),
    Error(ErrorPayload),
    Message(DaemonToServer),
}

/// WebSocket Hub managing all connections and subscriptions
pub struct Hub {
    /// Daemon connections keyed by host_id
    daemon_connections: Mutex<HashMap<Uuid, DaemonConnection>>,

    /// Client connections keyed by device_id
    client_connections: DashMap<Uuid, ClientConnection>,

    /// Session subscribers: session_id -> set of device_ids
    session_subscribers: DashMap<Uuid, HashSet<Uuid>>,

    /// Reverse mapping: device_id -> set of subscribed session_ids
    device_subscriptions: DashMap<Uuid, HashSet<Uuid>>,

    /// Pending requests waiting for response (request_id -> response channel)
    pending_requests: Mutex<HashMap<String, PendingDaemonResponse>>,
}

impl Hub {
    /// Create a new Hub instance
    pub fn new() -> Self {
        Self {
            daemon_connections: Mutex::new(HashMap::new()),
            client_connections: DashMap::new(),
            session_subscribers: DashMap::new(),
            device_subscriptions: DashMap::new(),
            pending_requests: Mutex::new(HashMap::new()),
        }
    }

    /// Register a daemon connection and make it the sole active connection for the host.
    pub async fn register_daemon(&self, host_id: Uuid, sender: WsSender) -> Uuid {
        let connection_id = Uuid::new_v4();
        info!(%host_id, %connection_id, "Registering daemon connection");

        let replaced_connection_id = {
            let mut daemon_connections = self.daemon_connections.lock().await;
            daemon_connections
                .insert(
                    host_id,
                    DaemonConnection {
                        host_id,
                        connection_id,
                        sender,
                    },
                )
                .map(|connection| connection.connection_id)
        };

        if let Some(replaced_connection_id) = replaced_connection_id {
            self.fail_pending_requests_for_connection(
                &host_id,
                replaced_connection_id,
                "Daemon connection replaced during handoff",
            )
            .await;
        }

        connection_id
    }

    /// Unregister a daemon connection only if it is still the active one.
    pub async fn unregister_daemon(&self, host_id: &Uuid, connection_id: Uuid) -> bool {
        let mut daemon_connections = self.daemon_connections.lock().await;
        if daemon_connections
            .get(host_id)
            .map(|connection| connection.connection_id == connection_id)
            .unwrap_or(false)
        {
            info!(%host_id, %connection_id, "Unregistering daemon connection");
            daemon_connections.remove(host_id);
            return true;
        }

        false
    }

    /// Check whether a daemon connection is still the active one for a host.
    pub async fn is_active_daemon_connection(&self, host_id: &Uuid, connection_id: Uuid) -> bool {
        self.daemon_connections
            .lock()
            .await
            .get(host_id)
            .map(|connection| connection.connection_id == connection_id)
            .unwrap_or(false)
    }

    /// Register a client connection
    pub fn register_client(&self, device_id: Uuid, sender: WsSender) {
        info!(%device_id, "Registering client connection");
        self.client_connections
            .insert(device_id, ClientConnection { device_id, sender });
    }

    /// Unregister a client connection and clean up subscriptions
    pub fn unregister_client(&self, device_id: &Uuid) {
        info!(%device_id, "Unregistering client connection");
        self.client_connections.remove(device_id);

        if let Some((_, sessions)) = self.device_subscriptions.remove(device_id) {
            for session_id in sessions {
                if let Some(mut subscribers) = self.session_subscribers.get_mut(&session_id) {
                    subscribers.remove(device_id);
                }
            }
        }
    }

    /// Subscribe a client to a session's events
    pub fn subscribe_session(&self, device_id: Uuid, session_id: Uuid) {
        debug!(%device_id, %session_id, "Subscribing to session");
        self.session_subscribers
            .entry(session_id)
            .or_default()
            .insert(device_id);
        self.device_subscriptions
            .entry(device_id)
            .or_default()
            .insert(session_id);
    }

    /// Unsubscribe a client from a session
    pub fn unsubscribe_session(&self, device_id: &Uuid, session_id: &Uuid) {
        debug!(%device_id, %session_id, "Unsubscribing from session");

        let remove_session_entry =
            if let Some(mut subscribers) = self.session_subscribers.get_mut(session_id) {
                subscribers.remove(device_id);
                subscribers.is_empty()
            } else {
                false
            };

        if remove_session_entry {
            self.session_subscribers.remove(session_id);
        }

        let remove_device_entry =
            if let Some(mut sessions) = self.device_subscriptions.get_mut(device_id) {
                sessions.remove(session_id);
                sessions.is_empty()
            } else {
                false
            };

        if remove_device_entry {
            self.device_subscriptions.remove(device_id);
        }
    }

    async fn try_send_to_active_daemon(
        &self,
        host_id: &Uuid,
        message: DaemonMessage,
    ) -> Result<(), mpsc::error::TrySendError<WsEnvelope>> {
        let mut daemon_connections = self.daemon_connections.lock().await;
        let connection = daemon_connections
            .get_mut(host_id)
            .ok_or_else(|| mpsc::error::TrySendError::Closed(WsEnvelope::from(message.clone())))?;
        let envelope = WsEnvelope::from(message);
        connection.sender.try_send(envelope)
    }

    /// Send a message to a specific daemon
    ///
    /// Uses try_send for non-blocking behavior with bounded channels.
    /// Returns false if channel is full or closed.
    pub async fn send_to_daemon(&self, host_id: &Uuid, message: DaemonMessage) -> bool {
        match self.try_send_to_active_daemon(host_id, message).await {
            Ok(()) => {
                debug!(%host_id, "Sent message to daemon");
                true
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!(%host_id, "Daemon channel full, dropping message");
                false
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                warn!(%host_id, "Daemon channel closed");
                false
            }
        }
    }

    /// Send a message to a specific client
    ///
    /// Uses try_send for non-blocking behavior with bounded channels.
    /// Returns false if channel is full or closed.
    #[allow(dead_code)]
    pub fn send_to_client(&self, device_id: &Uuid, message: ClientMessage) -> bool {
        if let Some(conn) = self.client_connections.get(device_id) {
            let envelope = WsEnvelope::from(message);
            match conn.sender.try_send(envelope) {
                Ok(()) => {
                    debug!(%device_id, "Sent message to client");
                    true
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    warn!(%device_id, "Client channel full, dropping message");
                    false
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    warn!(%device_id, "Client channel closed");
                    false
                }
            }
        } else {
            debug!(%device_id, "Client not connected");
            false
        }
    }

    /// Broadcast a message to all currently authorized clients subscribed to a session.
    ///
    /// Uses try_send for non-blocking behavior with bounded channels.
    /// Messages may be dropped if client channels are full.
    pub async fn broadcast_to_session(
        &self,
        db: &DbPool,
        session_id: &Uuid,
        message: ClientMessage,
    ) {
        let Some(subscribers) = self.session_subscribers.get(session_id) else {
            return;
        };

        let device_ids: Vec<Uuid> = subscribers.iter().copied().collect();
        drop(subscribers);

        if device_ids.is_empty() {
            return;
        }

        let session_id_str = session_id.to_string();

        // Batch authorization check: single query instead of N queries
        let device_id_strs: Vec<String> = device_ids.iter().map(|id| id.to_string()).collect();
        // $1 = session_id, $2..$N = device_ids
        let placeholders: Vec<String> = (2..=device_id_strs.len() + 1)
            .map(|i| format!("${}", i))
            .collect();
        let sql = format!(
            "SELECT device_id FROM device_session_access WHERE session_id = $1 AND device_id IN ({})",
            placeholders.join(", ")
        );

        let mut query = sqlx::query_scalar::<_, String>(&sql).bind(&session_id_str);
        for did in &device_id_strs {
            query = query.bind(did);
        }

        let authorized_ids: std::collections::HashSet<String> = match query.fetch_all(db).await {
            Ok(rows) => rows.into_iter().collect(),
            Err(error) => {
                warn!(%session_id, error = %error, "Broadcast auth check failed, dropping all messages");
                return;
            }
        };

        let envelope = WsEnvelope::from(message);
        let mut delivered = 0usize;

        for device_id in &device_ids {
            let device_id_str = device_id.to_string();
            if !authorized_ids.contains(&device_id_str) {
                self.unsubscribe_session(device_id, session_id);
                continue;
            }

            if let Some(conn) = self.client_connections.get(device_id) {
                match conn.sender.try_send(envelope.clone()) {
                    Ok(()) => {
                        delivered += 1;
                    }
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        warn!(%device_id, "Client channel full, dropping broadcast");
                    }
                    Err(mpsc::error::TrySendError::Closed(_)) => {
                        warn!(%device_id, "Client channel closed");
                        self.unsubscribe_session(device_id, session_id);
                    }
                }
            }
        }

        debug!(%session_id, delivered, "Broadcast to authorized session subscribers");
    }

    /// Check if a daemon is connected
    #[allow(dead_code)]
    pub async fn is_daemon_connected(&self, host_id: &Uuid) -> bool {
        self.daemon_connections.lock().await.contains_key(host_id)
    }

    /// Check if a client is connected
    #[allow(dead_code)]
    pub fn is_client_connected(&self, device_id: &Uuid) -> bool {
        self.client_connections.contains_key(device_id)
    }

    /// Get all connected daemon host IDs
    #[allow(dead_code)]
    pub async fn connected_daemons(&self) -> Vec<Uuid> {
        self.daemon_connections
            .lock()
            .await
            .keys()
            .copied()
            .collect()
    }

    /// Get all connected client device IDs
    #[allow(dead_code)]
    pub fn connected_clients(&self) -> Vec<Uuid> {
        self.client_connections
            .iter()
            .map(|entry| *entry.key())
            .collect()
    }

    async fn active_connection_id_for_host(&self, host_id: &Uuid) -> Option<Uuid> {
        self.daemon_connections
            .lock()
            .await
            .get(host_id)
            .map(|connection| connection.connection_id)
    }

    async fn active_daemon_for_host(&self, host_id: &Uuid) -> Option<(Uuid, WsSender)> {
        self.daemon_connections
            .lock()
            .await
            .get(host_id)
            .map(|connection| (connection.connection_id, connection.sender.clone()))
    }

    pub async fn fail_pending_requests_for_connection(
        &self,
        host_id: &Uuid,
        connection_id: Uuid,
        reason: &str,
    ) {
        let mut pending_requests = self.pending_requests.lock().await;
        let request_ids: Vec<String> = pending_requests
            .iter()
            .filter(|(_, pending)| {
                pending.host_id == *host_id && pending.connection_id == Some(connection_id)
            })
            .map(|(request_id, _)| request_id.clone())
            .collect();

        for request_id in request_ids {
            if let Some(pending) = pending_requests.remove(&request_id) {
                let _ = pending.tx.send(Err(reason.to_string()));
            }
        }
    }

    /// Send a request to daemon and wait for response
    pub async fn send_and_wait(
        &self,
        host_id: &Uuid,
        request: DaemonMessage,
        request_id: String,
        timeout: Duration,
    ) -> std::result::Result<DaemonResponse, HubError> {
        let initial_connection_id = self.active_connection_id_for_host(host_id).await;
        let (tx, rx) = oneshot::channel();
        self.pending_requests.lock().await.insert(
            request_id.clone(),
            PendingDaemonResponse {
                host_id: *host_id,
                connection_id: initial_connection_id,
                tx,
            },
        );

        let (active_connection_id, sender) = match self.active_daemon_for_host(host_id).await {
            Some(active_daemon) => active_daemon,
            None => {
                self.pending_requests.lock().await.remove(&request_id);
                return Err(HubError::NoActiveConnection);
            }
        };

        if initial_connection_id != Some(active_connection_id) {
            if let Some(pending) = self.pending_requests.lock().await.get_mut(&request_id) {
                pending.connection_id = Some(active_connection_id);
            }
        }

        let deadline = Instant::now() + timeout;

        if timeout_at(deadline, sender.send(WsEnvelope::from(request)))
            .await
            .map_err(|_| HubError::Timeout(timeout.as_millis()))?
            .is_err()
        {
            self.pending_requests.lock().await.remove(&request_id);
            return Err(HubError::SendFailed("response channel closed during send".to_string()));
        }

        match timeout_at(deadline, rx).await {
            Ok(Ok(Ok(response))) => {
                // Defensive cleanup: pending entry is normally removed by the response
                // handler (complete_with_ack/handle_response), but guard against leaks.
                self.pending_requests.lock().await.remove(&request_id);
                Ok(response)
            }
            Ok(Ok(Err(reason))) => Err(HubError::RemoteError { reason }),
            Ok(Err(_)) => {
                self.pending_requests.lock().await.remove(&request_id);
                Err(HubError::ChannelClosed)
            }
            Err(_) => {
                self.pending_requests.lock().await.remove(&request_id);
                Err(HubError::Timeout(timeout.as_millis()))
            }
        }
    }

    pub async fn complete_with_ack(&self, ack: AckPayload) {
        if let Some(pending) = self.pending_requests.lock().await.remove(&ack.request_id) {
            let _ = pending.tx.send(Ok(DaemonResponse::Ack(ack)));
        }
    }

    pub async fn complete_with_error(&self, error: ErrorPayload) {
        if let Some(pending) = self.pending_requests.lock().await.remove(&error.request_id) {
            let _ = pending.tx.send(Ok(DaemonResponse::Error(error)));
        }
    }

    /// Handle incoming response and complete pending request
    #[allow(dead_code)]
    pub async fn handle_response(&self, response: DaemonToServer) {
        let request_id = match &response {
            DaemonToServer::FileTreeResponse { request_id, .. } => request_id.clone(),
            DaemonToServer::FileContentResponse { request_id, .. } => request_id.clone(),
            DaemonToServer::Error { request_id, .. } => request_id.clone(),
            _ => return,
        };

        if let Some(pending) = self.pending_requests.lock().await.remove(&request_id) {
            let _ = pending.tx.send(Ok(DaemonResponse::Message(response)));
        }
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use futures::FutureExt;

    use super::*;

    #[tokio::test]
    async fn unregister_daemon_ignores_stale_connection_id() {
        let hub = Hub::new();
        let (tx1, _rx1) = mpsc::channel(1);
        let (tx2, _rx2) = mpsc::channel(1);
        let host_id = Uuid::new_v4();

        let first = hub.register_daemon(host_id, tx1).await;
        let second = hub.register_daemon(host_id, tx2).await;

        assert!(!hub.unregister_daemon(&host_id, first).await);
        assert!(hub.is_active_daemon_connection(&host_id, second).await);
        assert!(hub.unregister_daemon(&host_id, second).await);
        assert!(!hub.is_active_daemon_connection(&host_id, second).await);
    }

    #[tokio::test]
    async fn send_and_wait_fails_when_active_connection_disconnects() {
        let hub = Arc::new(Hub::new());
        let host_id = Uuid::new_v4();
        let request_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::channel(1);
        let connection_id = hub.register_daemon(host_id, tx).await;

        let pending_hub = hub.clone();
        let pending_request_id = request_id.clone();
        let pending_host_id = host_id;
        let pending = tokio::spawn(async move {
            pending_hub
                .send_and_wait(
                    &pending_host_id,
                    DaemonMessage::FileTreeRequest {
                        request_id: pending_request_id.clone(),
                        session_id: Uuid::nil(),
                        workspace_path: "/tmp".to_string(),
                        relative_path: None,
                    },
                    pending_request_id,
                    Duration::from_secs(5),
                )
                .await
        });

        let _ = rx.recv().await.expect("daemon should receive request");
        assert!(hub.unregister_daemon(&host_id, connection_id).await);
        hub.fail_pending_requests_for_connection(
            &host_id,
            connection_id,
            "Daemon disconnected before responding",
        )
        .await;

        let error = pending.await.unwrap().unwrap_err().to_string();
        assert!(error.contains("Daemon disconnected before responding"));
    }

    #[tokio::test]
    async fn send_and_wait_times_out_when_daemon_channel_stays_full() {
        let hub = Arc::new(Hub::new());
        let host_id = Uuid::new_v4();
        let request_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::channel(1);
        hub.register_daemon(host_id, tx).await;

        rx.recv().now_or_never();
        hub.send_to_daemon(
            &host_id,
            DaemonMessage::FileTreeRequest {
                request_id: Uuid::new_v4().to_string(),
                session_id: Uuid::nil(),
                workspace_path: "/tmp".to_string(),
                relative_path: None,
            },
        )
        .await;

        let error = hub
            .send_and_wait(
                &host_id,
                DaemonMessage::FileTreeRequest {
                    request_id: request_id.clone(),
                    session_id: Uuid::nil(),
                    workspace_path: "/tmp".to_string(),
                    relative_path: None,
                },
                request_id,
                Duration::from_millis(50),
            )
            .await
            .unwrap_err()
            .to_string();

        assert!(error.contains("Request timeout"));
    }

    #[tokio::test]
    async fn broadcast_to_session_prunes_revoked_subscribers() {
        let temp_db = std::env::temp_dir().join(format!("ve-hub-test-{}.db", Uuid::new_v4()));
        let database_url = format!("sqlite:{}?mode=rwc", temp_db.display());
        crate::db::install_drivers();
        let pool = crate::db::DbPool::connect(&database_url).await.unwrap();
        crate::db::run_migrations(&pool, crate::config::DatabaseBackend::Sqlite)
            .await
            .unwrap();

        let hub = Hub::new();
        let device_id = Uuid::new_v4();
        let host_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let (tx, mut rx) = mpsc::channel(1);
        let now = chrono::Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO client_devices (device_id, device_name, device_type, server_url) VALUES ($1, $2, $3, $4)",
        )
        .bind(device_id.to_string())
        .bind("device")
        .bind("desktop")
        .bind("http://localhost")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO hosts (host_id, host_name, platform, pair_status, created_at, updated_at) VALUES ($1, $2, $3, 'paired', $4, $4)",
        )
        .bind(host_id.to_string())
        .bind("host")
        .bind("linux")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO workspaces (workspace_id, host_id, path, display_name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $5)",
        )
        .bind(workspace_id.to_string())
        .bind(host_id.to_string())
        .bind("/tmp/project")
        .bind("project")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO sessions (session_id, title, host_id, workspace_id, agent_type, status, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'running', $6, $6)",
        )
        .bind(session_id.to_string())
        .bind("session")
        .bind(host_id.to_string())
        .bind(workspace_id.to_string())
        .bind("claude_code")
        .bind(&now)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO device_session_access (device_id, session_id) VALUES ($1, $2)")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&pool)
            .await
            .unwrap();

        hub.register_client(device_id, tx);
        hub.subscribe_session(device_id, session_id);

        sqlx::query("DELETE FROM device_session_access WHERE device_id = $1 AND session_id = $2")
            .bind(device_id.to_string())
            .bind(session_id.to_string())
            .execute(&pool)
            .await
            .unwrap();

        hub.broadcast_to_session(
            &pool,
            &session_id,
            ClientMessage::SessionStatusChanged {
                session_id,
                new_status: ve_shared::types::SessionStatus::Running,
                close_reason: None,
            },
        )
        .await;

        assert!(rx.try_recv().is_err());
        assert!(hub.device_subscriptions.get(&device_id).is_none());
    }
}
