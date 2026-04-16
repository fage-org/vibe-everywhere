//! WebSocket Protocol Definitions
//!
//! Message types for client↔server and daemon↔server communication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::models::PermissionDecision;
use crate::types::{DaemonStatus, OnlineStatus, RiskType, SessionStatus};

/// WebSocket message envelope
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct WsEnvelope {
    pub r#type: String,
    #[ts(type = "unknown")]
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl WsEnvelope {
    /// Create a new envelope with current timestamp
    pub fn new(message_type: impl Into<String>, payload: impl Serialize) -> Self {
        Self {
            r#type: message_type.into(),
            payload: serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null),
            timestamp: Utc::now(),
            request_id: None,
        }
    }

    /// Add a request ID for request-response patterns
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

// ============================================================================
// Client → Server Messages
// ============================================================================

/// Client-to-server message types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ClientToServer {
    SubscribeSession { session_id: Uuid },
    UnsubscribeSession { session_id: Uuid },
    Ping,
}

// ============================================================================
// Server → Client Messages
// ============================================================================

/// Server-to-client message types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ServerToClient {
    SessionEvent {
        session_id: Uuid,
        event_type: String,
        #[ts(type = "unknown")]
        data: serde_json::Value,
    },
    PermissionRequest {
        permission_id: Uuid,
        session_id: Uuid,
        risk_type: RiskType,
        summary: String,
        target: Option<String>,
    },
    SessionStatusChanged {
        session_id: Uuid,
        old_status: SessionStatus,
        new_status: SessionStatus,
    },
    HostStatusChanged {
        host_id: Uuid,
        online_status: OnlineStatus,
        daemon_status: DaemonStatus,
    },
    Notification {
        notification_type: String,
        title: String,
        body: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<Uuid>,
    },
    Pong,
    /// Acknowledgment for server-initiated commands
    Ack {
        request_id: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
}

/// Alias for server-to-client messages
pub type ClientMessage = ServerToClient;

impl From<ClientMessage> for WsEnvelope {
    fn from(msg: ClientMessage) -> Self {
        let msg_type = match &msg {
            ClientMessage::SessionEvent { .. } => "session_event",
            ClientMessage::PermissionRequest { .. } => "permission_request",
            ClientMessage::SessionStatusChanged { .. } => "session_status_changed",
            ClientMessage::HostStatusChanged { .. } => "host_status_changed",
            ClientMessage::Notification { .. } => "notification",
            ClientMessage::Pong => "pong",
            ClientMessage::Ack { .. } => "ack",
        };
        WsEnvelope::new(msg_type, &msg)
    }
}

// ============================================================================
// Daemon → Server Messages
// ============================================================================

/// Daemon-to-server message types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DaemonToServer {
    DaemonHello {
        host_id: Uuid,
        host_name: String,
        platform: String,
    },
    DaemonHeartbeat {
        host_id: Uuid,
        active_sessions: Vec<Uuid>,
    },
    SessionEvent {
        session_id: Uuid,
        event_type: String,
        #[ts(type = "unknown")]
        data: serde_json::Value,
    },
    PermissionRequest {
        session_id: Uuid,
        risk_type: RiskType,
        summary: String,
        target: Option<String>,
    },
    SessionStatusUpdate {
        session_id: Uuid,
        status: SessionStatus,
        summary: Option<String>,
    },
    FileTreeResponse {
        request_id: String,
        session_id: Uuid,
        #[ts(type = "unknown")]
        tree: serde_json::Value,
    },
    FileContentResponse {
        request_id: String,
        file_path: String,
        content: String,
        file_type: String,
    },
}

// ============================================================================
// Server → Daemon Messages
// ============================================================================

/// Server-to-daemon message types
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ServerToDaemon {
    CreateSession {
        /// Unique request ID for correlation and acknowledgment
        request_id: String,
        session_id: Uuid,
        workspace_path: String,
        agent_type: String,
        initial_message: String,
    },
    SendMessage {
        /// Unique request ID for correlation and acknowledgment
        request_id: String,
        session_id: Uuid,
        content: String,
    },
    SessionControl {
        /// Unique request ID for correlation and acknowledgment
        request_id: String,
        session_id: Uuid,
        action: SessionControlAction,
    },
    CloseSession {
        /// Unique request ID for correlation and acknowledgment
        request_id: String,
        session_id: Uuid,
    },
    PermissionResponse {
        permission_id: Uuid,
        session_id: Uuid,
        decision: PermissionDecision,
    },
    FileTreeRequest {
        request_id: String,
        session_id: Uuid,
        workspace_path: String,
    },
    FileContentRequest {
        request_id: String,
        file_path: String,
    },
    Pong,
    /// Pairing success notification (WS delivery mode)
    Paired {
        host_id: Uuid,
        daemon_token: String,
    },
}

/// Alias for server-to-daemon messages
pub type DaemonMessage = ServerToDaemon;

impl From<DaemonMessage> for WsEnvelope {
    fn from(msg: DaemonMessage) -> Self {
        let (msg_type, request_id) = match &msg {
            DaemonMessage::CreateSession { request_id, .. } => ("create_session", Some(request_id.clone())),
            DaemonMessage::SendMessage { request_id, .. } => ("send_message", Some(request_id.clone())),
            DaemonMessage::SessionControl { request_id, .. } => ("session_control", Some(request_id.clone())),
            DaemonMessage::CloseSession { request_id, .. } => ("close_session", Some(request_id.clone())),
            DaemonMessage::PermissionResponse { .. } => ("permission_response", None),
            DaemonMessage::FileTreeRequest { request_id, .. } => ("file_tree_request", Some(request_id.clone())),
            DaemonMessage::FileContentRequest { request_id, .. } => ("file_content_request", Some(request_id.clone())),
            DaemonMessage::Pong => ("pong", None),
            DaemonMessage::Paired { .. } => ("paired", None),
        };

        let mut envelope = WsEnvelope::new(msg_type, &msg);
        if let Some(id) = request_id {
            envelope = envelope.with_request_id(id);
        }
        envelope
    }
}

/// Session control action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SessionControlAction {
    Pause,
    Terminate,
    Interrupt,
    Rerun,
}

// ============================================================================
// Ack Message for Server → Daemon commands
// ============================================================================

/// Acknowledgment payload for daemon responses
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AckPayload {
    pub request_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Error payload for failed operations
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ErrorPayload {
    pub request_id: String,
    pub error_code: String,
    pub error_message: String,
}
