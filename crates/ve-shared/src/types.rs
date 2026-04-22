//! Shared Types and Enums
//!
//! Common enumerations and type aliases used across the Vibe Everywhere system.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Device type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum DeviceType {
    Mobile,
    Desktop,
}

/// Host platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Platform {
    Linux,
    Macos,
    Windows,
    Wsl,
}

/// Host online status
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum OnlineStatus {
    Online,
    Offline,
    #[default]
    Unknown,
}

/// Daemon connection status
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum DaemonStatus {
    Healthy,
    Connecting,
    #[default]
    Disconnected,
    Error,
}

/// Host pairing status
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum PairStatus {
    Paired,
    #[default]
    Pending,
    Failed,
}

/// Session status
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum SessionStatus {
    #[default]
    Running,
    Pending,
    Dispatching,
    WaitingApproval,
    Paused,
    Error,
    Closing,
    Archived,
}

impl SessionStatus {
    /// Check if the session is active (can receive events)
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Archived)
    }

    /// Check if the session can be closed
    pub fn can_close(&self) -> bool {
        matches!(
            self,
            Self::Running
                | Self::Pending
                | Self::Dispatching
                | Self::WaitingApproval
                | Self::Paused
                | Self::Error
        )
    }
}

/// Archive close reason
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum CloseReason {
    UserClosed,
    Completed,
    Failed,
    Terminated,
}

/// Permission risk type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RiskType {
    WriteFs,
    ExecCmd,
    Network,
}

/// Permission request status
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum PermissionStatus {
    #[default]
    Pending,
    ApprovedOnce,
    DeniedOnce,
    ApprovedSession,
    Expired,
}

impl PermissionStatus {
    /// Check if the permission has been responded to
    pub fn is_responded(&self) -> bool {
        !matches!(self, Self::Pending | Self::Expired)
    }

    /// Check if the permission is approved
    pub fn is_approved(&self) -> bool {
        matches!(self, Self::ApprovedOnce | Self::ApprovedSession)
    }
}

/// Connection status for server config
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ConnectionStatus {
    Idle,
    Testing,
    Connected,
    Failed,
}

/// Pagination parameters
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export)]
pub struct Pagination {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: default_page(),
            limit: default_limit(),
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Paginated<T: TS> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
}

impl<T: TS> Paginated<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, limit: u32) -> Self {
        let has_more = (page * limit) < total as u32;
        Self {
            items,
            total,
            page,
            limit,
            has_more,
        }
    }
}
