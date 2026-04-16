//! WebSocket Protocol Definitions
//!
//! Message types for client↔server and daemon↔server communication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::PermissionDecision;
use crate::types::{DaemonStatus, OnlineStatus, RiskType, SessionStatus};

/// WebSocket message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEnvelope {
    pub r#type: String,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
pub enum ClientToServer {
    SubscribeSession { session_id: Uuid },
    UnsubscribeSession { session_id: Uuid },
    Ping,
}

// ============================================================================
// Server → Client Messages
// ============================================================================

/// Server-to-client message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
pub enum ServerToClient {
    SessionEvent {
        session_id: Uuid,
        event_type: String,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
pub enum ServerToDaemon {
    CreateSession {
        session_id: Uuid,
        workspace_path: String,
        agent_type: String,
        initial_message: String,
    },
    SendMessage {
        session_id: Uuid,
        content: String,
    },
    SessionControl {
        session_id: Uuid,
        action: SessionControlAction,
    },
    CloseSession {
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
        let msg_type = match &msg {
            DaemonMessage::CreateSession { .. } => "create_session",
            DaemonMessage::SendMessage { .. } => "send_message",
            DaemonMessage::SessionControl { .. } => "session_control",
            DaemonMessage::CloseSession { .. } => "close_session",
            DaemonMessage::PermissionResponse { .. } => "permission_response",
            DaemonMessage::FileTreeRequest { .. } => "file_tree_request",
            DaemonMessage::FileContentRequest { .. } => "file_content_request",
            DaemonMessage::Pong => "pong",
            DaemonMessage::Paired { .. } => "paired",
        };
        WsEnvelope::new(msg_type, &msg)
    }
}

/// Session control action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckPayload {
    pub request_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Error payload for failed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub request_id: String,
    pub error_code: String,
    pub error_message: String,
}
