//! Shared Data Models
//!
//! Core domain models used across the Vibe Everywhere system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::types::*;

/// Client device information
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ClientDevice {
    pub device_id: Uuid,
    pub device_name: String,
    pub device_type: DeviceType,
    pub authorized_at: DateTime<Utc>,
    pub server_url: String,
    pub token_hash: Option<String>,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Request to register a new client device
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct RegisterDeviceRequest {
    pub device_name: String,
    pub device_type: DeviceType,
    pub server_url: String,
}

/// Response after device registration
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct RegisterDeviceResponse {
    pub device_id: Uuid,
    pub token: String,
}

/// Response after a successful pairing
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct PairResponse {
    pub host_id: Uuid,
    pub host_name: String,
    pub token: String,
}

/// Remote host information
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Host {
    pub host_id: Uuid,
    pub host_name: String,
    pub platform: Platform,
    pub online_status: OnlineStatus,
    pub daemon_status: DaemonStatus,
    pub last_active_at: Option<DateTime<Utc>>,
    pub pair_status: PairStatus,
    pub pair_code: Option<String>,
    pub qr_payload: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Workspace information
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Workspace {
    pub workspace_id: Uuid,
    pub host_id: Uuid,
    pub path: String,
    pub display_name: String,
    pub is_favorited: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub exists_on_host: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new workspace
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct CreateWorkspaceRequest {
    pub host_id: Uuid,
    pub path: String,
    pub display_name: Option<String>,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Session {
    pub session_id: Uuid,
    pub title: String,
    pub host_id: Uuid,
    pub workspace_id: Uuid,
    pub agent_type: String,
    pub status: SessionStatus,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub latest_summary: Option<String>,
    pub unread_event_count: i32,
    pub pending_permission_count: i32,
    pub can_resume_cross_device: bool,
    pub claude_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request to create a new session
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct CreateSessionRequest {
    /// Idempotency key for duplicate request protection
    pub idempotency_key: String,
    pub host_id: Uuid,
    pub workspace_id: Uuid,
    pub title: String,
    pub initial_message: String,
}

/// Session archive information
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionArchive {
    pub archive_id: Uuid,
    pub session_id: Uuid,
    pub title: String,
    pub closed_at: DateTime<Utc>,
    pub close_reason: CloseReason,
    pub host_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<Utc>,
    /// Metadata captured at archive time
    pub metadata: Option<ArchiveMetadata>,
}

/// Archive metadata captured when session is closed
///
/// Contains session snapshot information for archive details display
/// and historical tracing.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ArchiveMetadata {
    /// Workspace path where the session ran
    pub workspace_path: String,
    /// Display name of the workspace
    pub workspace_display_name: Option<String>,
    /// Agent type (claude_code, acp)
    pub agent_type: String,
    /// Who triggered the close: device_id or 'daemon'
    pub closed_by: String,
    /// Latest session summary at close time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_summary: Option<String>,
    /// Claude Code CLI's internal session-id if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claude_session_id: Option<String>,
    /// Session statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<ArchiveStatistics>,
    /// Git commit SHA at close time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit_sha: Option<String>,
    /// Git commit message at close time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit_message: Option<String>,
}

/// Session statistics captured at archive time
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ArchiveStatistics {
    /// Total message count
    pub message_count: u32,
    /// Total event count
    pub event_count: u32,
    /// Total permission requests count
    pub permission_count: u32,
    /// Session duration in seconds
    pub duration_seconds: u32,
}

/// Permission request information
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PermissionRequest {
    pub permission_id: Uuid,
    pub session_id: Uuid,
    pub risk_type: RiskType,
    pub summary: String,
    pub target: Option<String>,
    pub created_at: DateTime<Utc>,
    pub status: PermissionStatus,
    pub responded_at: Option<DateTime<Utc>>,
}

/// Request to respond to a permission request
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct PermissionResponseRequest {
    pub decision: PermissionDecision,
}

/// Permission decision types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PermissionDecision {
    ApproveOnce,
    DenyOnce,
    ApproveSession,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NotificationPreference {
    pub device_id: Uuid,
    pub enabled: bool,
    pub permission_request_enabled: bool,
    pub task_completed_enabled: bool,
    pub task_failed_enabled: bool,
    pub session_error_enabled: bool,
}

impl Default for NotificationPreference {
    fn default() -> Self {
        Self {
            device_id: Uuid::nil(),
            enabled: true,
            permission_request_enabled: true,
            task_completed_enabled: true,
            task_failed_enabled: true,
            session_error_enabled: true,
        }
    }
}

/// Session message (log entry, assistant output, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SessionMessage {
    pub message_id: Uuid,
    pub session_id: Uuid,
    pub message_type: SessionMessageType,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// Type of session message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SessionMessageType {
    User,
    Assistant,
    System,
    Tool,
    Error,
    Permission,
}

// ============================================================================
// File Tree Types
// ============================================================================

/// Node in file tree
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileTreeNode {
    /// File or directory name
    pub name: String,
    /// Full path relative to workspace root
    pub path: String,
    /// Whether this is a directory
    pub is_dir: bool,
    /// File type classification
    pub file_type: FileType,
    /// File size in bytes (None for directories)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// Children (for directories)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<FileTreeNode>>,
}

/// File type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Text,
    Binary,
    Unknown,
}

/// File content response
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileContent {
    /// File path
    pub path: String,
    /// File content (text only)
    pub content: String,
    /// File type
    pub file_type: FileType,
    /// Whether content was truncated
    pub truncated: bool,
    /// Total file size in bytes
    pub total_size: u64,
}
