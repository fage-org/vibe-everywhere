//! Daemon Error Types
//!
//! Comprehensive error definitions for the Vibe Everywhere Daemon.
//! Errors are organized by layer for clear categorization and handling.

use thiserror::Error;

/// Daemon top-level error type
///
/// Errors are organized by source layer:
/// - Configuration: startup config loading failures
/// - Authentication: token validation and pairing failures
/// - Protocol: WebSocket message parsing and version issues
/// - Network: connection and heartbeat failures
/// - Session: session state and CLI process issues
/// - File: file access, path validation, and permission issues
/// - Agent: CLI execution and I/O failures
#[derive(Debug, Error)]
pub enum DaemonError {
    // ========== Configuration Layer ==========
    #[error("配置文件读取失败: {0}")]
    ConfigRead(#[source] std::io::Error),

    #[error("配置解析失败: {0}")]
    ConfigParse(#[source] toml::de::Error),

    #[error("配置项缺失或无效: {0}")]
    ConfigInvalid(String),

    #[error("配置目录权限不足: {path}")]
    ConfigPermission { path: String },

    // ========== Authentication Layer ==========
    #[error("Token 文件损坏或缺失")]
    TokenMissing,

    #[error("Token 解析失败")]
    TokenParse,

    #[error("Token 验证失败: {reason}")]
    TokenInvalid { reason: String },

    #[error("Token 已过期")]
    TokenExpired,

    #[error("配对失败: {reason}")]
    PairingFailed { reason: String },

    #[error("配对码无效或已过期")]
    PairCodeInvalid,

    #[error("配对超时")]
    PairingTimeout,

    // ========== Protocol Layer ==========
    #[error("WebSocket 消息解析失败: {0}")]
    WsMessageParse(#[source] serde_json::Error),

    #[error("未知的消息类型: {0}")]
    WsUnknownMessageType(String),

    #[error("消息 payload 缺失必要字段: {0}")]
    WsPayloadMissing(String),

    #[error("协议版本不兼容: 期望 {expected}, 收到 {received}")]
    ProtocolVersionMismatch { expected: u32, received: u32 },

    #[error("请求 ID 缺失")]
    RequestIdMissing,

    // ========== Network Layer ==========
    #[error("WebSocket 连接失败: {0}")]
    WsConnect(#[source] Box<tokio_tungstenite::tungstenite::Error>),

    #[error("WebSocket 连接已断开")]
    WsDisconnected,

    #[error("连接超时")]
    ConnectionTimeout,

    #[error("重连次数超限: 已尝试 {attempts} 次")]
    ReconnectLimitExceeded { attempts: u32 },

    #[error("心跳超时: 超过 {timeout_seconds} 秒未收到响应")]
    HeartbeatTimeout { timeout_seconds: u32 },

    // ========== Session Layer ==========
    #[error("Session 不存在: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Session 状态非法: 当前 {current}, 期望 {expected}")]
    SessionInvalidStatus { current: String, expected: String },

    #[error("Session 已归档: {session_id}")]
    SessionArchived { session_id: String },

    #[error("Session 创建失败: {reason}")]
    SessionCreateFailed { reason: String },

    #[error("Session 关闭失败: {reason}")]
    SessionCloseFailed { reason: String },

    #[error("Session rerun 失败: {reason}")]
    SessionRerunFailed { reason: String },

    #[error("Workspace 路径不存在: {path}")]
    WorkspaceNotFound { path: String },

    #[error("Workspace 路径非法: {path}")]
    WorkspaceInvalid { path: String },

    #[error("已达到最大并发会话数: {max}")]
    MaxSessionsReached { max: usize },

    // ========== File Layer ==========
    #[error("文件不存在: {path}")]
    FileNotFound { path: String },

    #[error("路径越界: {path} 不在 workspace {workspace} 范围内")]
    FileAccessDenied { path: String, workspace: String },

    #[error("文件读取失败: {path}")]
    FileReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("文件过大: {size} 字节, 上限 {limit} 字节")]
    FileTooLarge { size: u64, limit: u64 },

    #[error("文件树节点数超限: {count} > {limit}")]
    FileTreeLimitExceeded { count: usize, limit: usize },

    #[error("符号链接不允许: {path}")]
    SymlinkNotAllowed { path: String },

    #[error("非文本文件不允许读取内容: {path}")]
    FileNotText { path: String },

    // ========== Agent Layer ==========
    #[error("CLI 可执行文件未找到: {command}")]
    CliNotFound { command: String },

    #[error("CLI 启动失败: {reason}")]
    CliStartFailed { reason: String },

    #[error("CLI 进程异常退出: 退出码 {code}")]
    CliExitError { code: i32 },

    #[error("CLI 进程被信号终止: {signal}")]
    CliKilled { signal: String },

    #[error("CLI stdin 写入失败")]
    CliStdinWriteFailed,

    #[error("CLI stdout 解析失败: {reason}")]
    CliStdoutParseFailed { reason: String },

    #[error("流式 JSON 解析错误: {line}")]
    StreamJsonParseError { line: String },

    #[error("权限请求处理失败: {permission_id}")]
    PermissionHandleFailed { permission_id: String },

    #[error("权限等待超时")]
    PermissionWaitTimeout,

    #[error("权限响应已过期")]
    PermissionExpired,

    // ========== Internal Errors ==========
    #[error("内部任务错误: {0}")]
    InternalTaskError(String),

    #[error("通道发送失败: {0}")]
    ChannelSendFailed(String),

    #[error("未知错误: {0}")]
    Unknown(String),
}

/// Acknowledgment error type for daemon → server error messages
///
/// These errors are sent back to the server when a command cannot be processed.
#[derive(Debug, Error)]
pub enum AckError {
    #[error("Session 不存在")]
    SessionNotFound,

    #[error("Session 状态不允许此操作")]
    SessionInvalidState,

    #[error("Session 已归档")]
    SessionArchived,

    #[error("Workspace 无效")]
    WorkspaceInvalid,

    #[error("内部错误")]
    InternalError,

    #[error("CLI 未运行")]
    CliNotRunning,
}

impl AckError {
    /// Convert to error code string for protocol messages
    pub fn as_error_code(&self) -> &'static str {
        match self {
            Self::SessionNotFound => "SESSION_NOT_FOUND",
            Self::SessionInvalidState => "SESSION_INVALID_STATE",
            Self::SessionArchived => "SESSION_ARCHIVED",
            Self::WorkspaceInvalid => "WORKSPACE_INVALID",
            Self::InternalError => "INTERNAL_ERROR",
            Self::CliNotRunning => "CLI_NOT_RUNNING",
        }
    }
}

impl DaemonError {
    /// Convert to WebSocket error code for protocol messages
    pub fn to_ws_error_code(&self) -> &'static str {
        match self {
            Self::SessionArchived { .. } => "UNPROCESSABLE_STATE",
            Self::SessionNotFound { .. } => "NOT_FOUND",
            Self::SessionInvalidStatus { .. } => "UNPROCESSABLE_STATE",
            Self::FileAccessDenied { .. } => "FORBIDDEN",
            Self::FileTooLarge { .. } => "INVALID_INPUT",
            Self::CliNotFound { .. }
            | Self::CliStartFailed { .. }
            | Self::CliExitError { .. }
            | Self::CliKilled { .. } => "SERVICE_UNAVAILABLE",
            _ => "INTERNAL_ERROR",
        }
    }

    /// Convert to AckError for sending to server
    pub fn to_ack_error(&self) -> AckError {
        match self {
            Self::SessionNotFound { .. } => AckError::SessionNotFound,
            Self::SessionInvalidStatus { .. } => AckError::SessionInvalidState,
            Self::SessionArchived { .. } => AckError::SessionArchived,
            Self::WorkspaceNotFound { .. } | Self::WorkspaceInvalid { .. } => AckError::WorkspaceInvalid,
            Self::CliNotFound { .. }
            | Self::CliStartFailed { .. }
            | Self::CliExitError { .. }
            | Self::CliKilled { .. } => AckError::CliNotRunning,
            _ => AckError::InternalError,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::ConnectionTimeout
                | Self::HeartbeatTimeout { .. }
                | Self::ReconnectLimitExceeded { .. }
                | Self::WsDisconnected
        )
    }

    /// Check if this error should cause session closure
    pub fn should_close_session(&self) -> bool {
        matches!(
            self,
            Self::SessionArchived { .. }
                | Self::CliExitError { .. }
                | Self::CliKilled { .. }
                | Self::CliStartFailed { .. }
        )
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for DaemonError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        DaemonError::WsConnect(Box::new(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_mapping() {
        let err = DaemonError::SessionArchived {
            session_id: "test".to_string(),
        };
        assert_eq!(err.to_ws_error_code(), "UNPROCESSABLE_STATE");

        let err = DaemonError::FileAccessDenied {
            path: "/etc/passwd".to_string(),
            workspace: "/home/user".to_string(),
        };
        assert_eq!(err.to_ws_error_code(), "FORBIDDEN");
    }

    #[test]
    fn test_retryable_detection() {
        let err = DaemonError::ConnectionTimeout;
        assert!(err.is_retryable());

        let err = DaemonError::TokenExpired;
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_session_close_detection() {
        let err = DaemonError::CliExitError { code: 1 };
        assert!(err.should_close_session());

        let err = DaemonError::FileNotFound {
            path: "/tmp/test".to_string(),
        };
        assert!(!err.should_close_session());
    }

    #[test]
    fn test_ack_error_display() {
        let err = AckError::SessionArchived;
        assert!(err.to_string().contains("归档"));
    }

    #[test]
    fn test_ack_error_code_mapping() {
        assert_eq!(AckError::SessionNotFound.as_error_code(), "SESSION_NOT_FOUND");
        assert_eq!(AckError::SessionInvalidState.as_error_code(), "SESSION_INVALID_STATE");
        assert_eq!(AckError::SessionArchived.as_error_code(), "SESSION_ARCHIVED");
        assert_eq!(AckError::WorkspaceInvalid.as_error_code(), "WORKSPACE_INVALID");
        assert_eq!(AckError::InternalError.as_error_code(), "INTERNAL_ERROR");
        assert_eq!(AckError::CliNotRunning.as_error_code(), "CLI_NOT_RUNNING");
    }

    #[test]
    fn test_daemon_error_to_ack_error() {
        let err = DaemonError::SessionNotFound {
            session_id: "test".to_string(),
        };
        assert!(matches!(err.to_ack_error(), AckError::SessionNotFound));

        let err = DaemonError::SessionArchived {
            session_id: "test".to_string(),
        };
        assert!(matches!(err.to_ack_error(), AckError::SessionArchived));

        let err = DaemonError::CliExitError { code: 1 };
        assert!(matches!(err.to_ack_error(), AckError::CliNotRunning));

        let err = DaemonError::TokenExpired;
        assert!(matches!(err.to_ack_error(), AckError::InternalError));
    }
}
