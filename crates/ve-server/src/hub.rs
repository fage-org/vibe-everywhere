//! WebSocket Hub
//!
//! Event routing center: manages connection pools, subscriptions, and message distribution.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::{debug, info, warn};
use uuid::Uuid;

use ve_shared::proto::{ClientMessage, DaemonMessage, DaemonToServer, WsEnvelope};

/// Default bounded channel capacity for WebSocket connections
pub const WS_CHANNEL_CAPACITY: usize = 256;

/// Type alias for WebSocket message sender (bounded)
pub type WsSender = mpsc::Sender<WsEnvelope>;

/// Connection metadata for a daemon
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DaemonConnection {
    pub host_id: Uuid,
    pub sender: WsSender,
}

/// Connection metadata for a client
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClientConnection {
    pub device_id: Uuid,
    pub sender: WsSender,
}

/// WebSocket Hub managing all connections and subscriptions
pub struct Hub {
    /// Daemon connections keyed by host_id
    daemon_connections: DashMap<Uuid, DaemonConnection>,

    /// Client connections keyed by device_id
    client_connections: DashMap<Uuid, ClientConnection>,

    /// Session subscribers: session_id -> set of device_ids
    session_subscribers: DashMap<Uuid, HashSet<Uuid>>,

    /// Reverse mapping: device_id -> set of subscribed session_ids
    device_subscriptions: DashMap<Uuid, HashSet<Uuid>>,

    /// Pending requests waiting for response (request_id -> response channel)
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<DaemonToServer>>>>,
}

impl Hub {
    /// Create a new Hub instance
    pub fn new() -> Self {
        Self {
            daemon_connections: DashMap::new(),
            client_connections: DashMap::new(),
            session_subscribers: DashMap::new(),
            device_subscriptions: DashMap::new(),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a daemon connection
    pub fn register_daemon(&self, host_id: Uuid, sender: WsSender) {
        info!(%host_id, "Registering daemon connection");
        self.daemon_connections
            .insert(host_id, DaemonConnection { host_id, sender });
    }

    /// Unregister a daemon connection
    pub fn unregister_daemon(&self, host_id: &Uuid) {
        info!(%host_id, "Unregistering daemon connection");
        self.daemon_connections.remove(host_id);
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

        // Clean up all subscriptions for this device
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

        // Add to session subscribers
        self.session_subscribers
            .entry(session_id)
            .or_default()
            .insert(device_id);

        // Add to device subscriptions
        self.device_subscriptions
            .entry(device_id)
            .or_default()
            .insert(session_id);
    }

    /// Unsubscribe a client from a session
    pub fn unsubscribe_session(&self, device_id: &Uuid, session_id: &Uuid) {
        debug!(%device_id, %session_id, "Unsubscribing from session");

        if let Some(mut subscribers) = self.session_subscribers.get_mut(session_id) {
            subscribers.remove(device_id);
        }

        if let Some(mut sessions) = self.device_subscriptions.get_mut(device_id) {
            sessions.remove(session_id);
        }
    }

    /// Send a message to a specific daemon
    ///
    /// Uses try_send for non-blocking behavior with bounded channels.
    /// Returns false if channel is full or closed.
    pub fn send_to_daemon(&self, host_id: &Uuid, message: DaemonMessage) -> bool {
        if let Some(conn) = self.daemon_connections.get(host_id) {
            let envelope = WsEnvelope::from(message);
            match conn.sender.try_send(envelope) {
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
        } else {
            debug!(%host_id, "Daemon not connected");
            false
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

    /// Broadcast a message to all clients subscribed to a session
    ///
    /// Uses try_send for non-blocking behavior with bounded channels.
    /// Messages may be dropped if client channels are full.
    pub fn broadcast_to_session(&self, session_id: &Uuid, message: ClientMessage) {
        if let Some(subscribers) = self.session_subscribers.get(session_id) {
            let envelope = WsEnvelope::from(message.clone());
            for device_id in subscribers.iter() {
                if let Some(conn) = self.client_connections.get(device_id) {
                    match conn.sender.try_send(envelope.clone()) {
                        Ok(()) => {}
                        Err(mpsc::error::TrySendError::Full(_)) => {
                            warn!(%device_id, "Client channel full, dropping broadcast");
                        }
                        Err(mpsc::error::TrySendError::Closed(_)) => {
                            warn!(%device_id, "Client channel closed");
                        }
                    }
                }
            }
            debug!(%session_id, count = subscribers.len(), "Broadcast to session subscribers");
        }
    }

    /// Check if a daemon is connected
    #[allow(dead_code)]
    pub fn is_daemon_connected(&self, host_id: &Uuid) -> bool {
        self.daemon_connections.contains_key(host_id)
    }

    /// Check if a client is connected
    #[allow(dead_code)]
    pub fn is_client_connected(&self, device_id: &Uuid) -> bool {
        self.client_connections.contains_key(device_id)
    }

    /// Get all connected daemon host IDs
    #[allow(dead_code)]
    pub fn connected_daemons(&self) -> Vec<Uuid> {
        self.daemon_connections.iter().map(|e| *e.key()).collect()
    }

    /// Get all connected client device IDs
    #[allow(dead_code)]
    pub fn connected_clients(&self) -> Vec<Uuid> {
        self.client_connections.iter().map(|e| *e.key()).collect()
    }

    /// Get the sender for a daemon connection
    pub async fn get_daemon_sender(&self, host_id: &Uuid) -> Option<WsSender> {
        self.daemon_connections.get(host_id).map(|c| c.sender.clone())
    }

    /// Send a request to daemon and wait for response
    pub async fn send_and_wait(
        &self,
        _host_id: &Uuid,
        sender: WsSender,
        request: DaemonMessage,
        request_id: String,
        timeout: Duration,
    ) -> std::result::Result<DaemonToServer, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();

        // Store pending request
        self.pending_requests.lock().await.insert(request_id.clone(), tx);

        // Send request
        let envelope = WsEnvelope::from(request);
        match sender.send(envelope).await {
            Ok(()) => {}
            Err(e) => {
                // Remove pending request on send failure
                self.pending_requests.lock().await.remove(&request_id);
                return Err(Box::new(e));
            }
        }

        // Wait for response with timeout
        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                // Channel closed
                self.pending_requests.lock().await.remove(&request_id);
                Err("Response channel closed".into())
            }
            Err(_) => {
                // Timeout
                self.pending_requests.lock().await.remove(&request_id);
                Err("Request timeout".into())
            }
        }
    }

    /// Handle incoming response and complete pending request
    #[allow(dead_code)]
    pub async fn handle_response(&self, response: DaemonToServer) {
        let request_id = match &response {
            DaemonToServer::FileTreeResponse { request_id, .. } => request_id.clone(),
            DaemonToServer::FileContentResponse { request_id, .. } => request_id.clone(),
            _ => return,
        };

        if let Some(tx) = self.pending_requests.lock().await.remove(&request_id) {
            let _ = tx.send(response);
        }
    }
}

impl Default for Hub {
    fn default() -> Self {
        Self::new()
    }
}
