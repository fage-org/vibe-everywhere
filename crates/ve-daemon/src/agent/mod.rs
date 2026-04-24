//! Agent Driver 抽象层
//!
//! 定义与 CLI agent 交互的统一接口。

mod claude_code;

use async_trait::async_trait;
use std::sync::Arc;

pub use claude_code::ClaudeCodeDriver;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::SessionControlAction;
use ve_shared::types::{CloseReason, SessionStatus};

use crate::error::DaemonError;
use crate::Result;

/// Agent 启动配置
#[derive(Debug, Clone)]
pub struct DriverConfig {
    pub session_id: Uuid,
    pub workspace_path: String,
    pub agent_type: String,
    pub initial_message: Option<String>,
}

/// Agent 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverEvent {
    /// 会话事件 (日志、工具调用等)
    SessionEvent {
        session_id: Uuid,
        event_type: String,
        data: serde_json::Value,
    },

    /// 权限请求
    PermissionRequest {
        permission_id: Uuid,
        session_id: Uuid,
        risk_type: String,
        summary: String,
        target: Option<String>,
    },

    /// 状态更新
    StatusUpdate {
        session_id: Uuid,
        status: SessionStatus,
        summary: Option<String>,
        close_reason: Option<CloseReason>,
    },

    /// 致命错误
    FatalError { session_id: Uuid, message: String },

    /// Claude session ID received (for --resume support)
    ClaudeSessionId {
        session_id: Uuid,
        claude_session_id: String,
    },
}

/// Agent Driver trait
#[async_trait]
pub trait AgentDriver: Send + Sync {
    /// 启动 agent 会话
    async fn start(&mut self, config: DriverConfig) -> Result<()>;

    /// 发送消息给 agent
    async fn send_message(&mut self, session_id: Uuid, content: &str) -> Result<()>;

    /// 执行控制动作
    async fn control(&mut self, session_id: Uuid, action: SessionControlAction) -> Result<()>;

    /// 响应权限请求
    async fn respond_permission(
        &mut self,
        session_id: Uuid,
        permission_id: Uuid,
        decision: PermissionDecision,
    ) -> Result<()>;

    /// 处理权限超时
    async fn permission_timeout(&mut self, session_id: Uuid, permission_id: Uuid) -> Result<()>;

    /// 关闭 agent
    async fn close(&mut self, session_id: Uuid) -> Result<()>;

    /// 重新运行会话 (使用 --resume 参数)
    async fn rerun(
        &mut self,
        session_id: Uuid,
        workspace_path: &str,
        claude_session_id: &str,
    ) -> Result<()>;
}

/// Mock Agent Driver (用于测试)
pub struct MockDriver {
    event_tx: tokio::sync::broadcast::Sender<DriverEvent>,
}

const MOCK_PERMISSION_TRIGGER: &str = "__VE_MOCK_PERMISSION__";

impl MockDriver {
    pub fn new(event_tx: tokio::sync::broadcast::Sender<DriverEvent>) -> Self {
        Self { event_tx }
    }

    fn maybe_emit_permission_request(&self, session_id: Uuid, content: &str) {
        if !content.contains(MOCK_PERMISSION_TRIGGER) {
            return;
        }

        let _ = self.event_tx.send(DriverEvent::PermissionRequest {
            permission_id: Uuid::new_v4(),
            session_id,
            risk_type: "exec_cmd".to_string(),
            summary: "Mock driver triggered permission request".to_string(),
            target: Some("/tmp/mock-command".to_string()),
        });
    }
}

#[async_trait]
impl AgentDriver for MockDriver {
    async fn start(&mut self, config: DriverConfig) -> Result<()> {
        let _ = self.event_tx.send(DriverEvent::StatusUpdate {
            session_id: config.session_id,
            status: SessionStatus::Running,
            summary: None,
            close_reason: None,
        });
        if let Some(initial_message) = config.initial_message.as_deref() {
            self.maybe_emit_permission_request(config.session_id, initial_message);
        }
        Ok(())
    }

    async fn send_message(&mut self, session_id: Uuid, content: &str) -> Result<()> {
        let _ = self.event_tx.send(DriverEvent::SessionEvent {
            session_id,
            event_type: "user_message".to_string(),
            data: serde_json::json!({ "content": content }),
        });
        self.maybe_emit_permission_request(session_id, content);
        Ok(())
    }

    async fn control(&mut self, session_id: Uuid, action: SessionControlAction) -> Result<()> {
        let status = match action {
            SessionControlAction::Pause => SessionStatus::Paused,
            SessionControlAction::Terminate => SessionStatus::Archived,
            _ => SessionStatus::Running,
        };
        let _ = self.event_tx.send(DriverEvent::StatusUpdate {
            session_id,
            status,
            summary: None,
            close_reason: match action {
                SessionControlAction::Terminate => Some(CloseReason::Terminated),
                _ => None,
            },
        });
        Ok(())
    }

    async fn respond_permission(
        &mut self,
        _session_id: Uuid,
        _permission_id: Uuid,
        _decision: PermissionDecision,
    ) -> Result<()> {
        Ok(())
    }

    async fn permission_timeout(&mut self, _session_id: Uuid, _permission_id: Uuid) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self, session_id: Uuid) -> Result<()> {
        let _ = self.event_tx.send(DriverEvent::StatusUpdate {
            session_id,
            status: SessionStatus::Archived,
            summary: Some("Session closed".to_string()),
            close_reason: Some(CloseReason::UserClosed),
        });
        Ok(())
    }

    async fn rerun(
        &mut self,
        session_id: Uuid,
        _workspace_path: &str,
        _claude_session_id: &str,
    ) -> Result<()> {
        let _ = self.event_tx.send(DriverEvent::StatusUpdate {
            session_id,
            status: SessionStatus::Running,
            summary: Some("Session resumed".to_string()),
            close_reason: None,
        });
        Ok(())
    }
}

/// Create an Agent Driver based on the specified type
pub fn create_driver(
    agent_type: &str,
    config: Arc<crate::config::Config>,
    event_tx: tokio::sync::broadcast::Sender<DriverEvent>,
) -> Result<Box<dyn AgentDriver>> {
    if config.mock_mode {
        return Ok(Box::new(MockDriver::new(event_tx)));
    }
    match agent_type {
        "claude_code" => Ok(Box::new(ClaudeCodeDriver::new(config, event_tx))),
        _ => Err(DaemonError::AgentUnsupported {
            agent_type: agent_type.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_driver_emits_permission_request_for_trigger_in_initial_message() {
        let (event_tx, mut event_rx) = tokio::sync::broadcast::channel(8);
        let session_id = Uuid::new_v4();
        let mut driver = MockDriver::new(event_tx);

        driver
            .start(DriverConfig {
                session_id,
                workspace_path: "/tmp".to_string(),
                agent_type: "claude_code".to_string(),
                initial_message: Some(MOCK_PERMISSION_TRIGGER.to_string()),
            })
            .await
            .expect("mock driver start should succeed");

        let first = event_rx.recv().await.expect("status update event");
        let second = event_rx.recv().await.expect("permission request event");

        assert!(matches!(first, DriverEvent::StatusUpdate { .. }));
        assert!(matches!(
            second,
            DriverEvent::PermissionRequest {
                session_id: emitted_session_id,
                ..
            } if emitted_session_id == session_id
        ));
    }

    #[tokio::test]
    async fn mock_driver_emits_permission_request_for_trigger_in_followup_message() {
        let (event_tx, mut event_rx) = tokio::sync::broadcast::channel(8);
        let session_id = Uuid::new_v4();
        let mut driver = MockDriver::new(event_tx);

        driver
            .send_message(session_id, MOCK_PERMISSION_TRIGGER)
            .await
            .expect("mock driver send_message should succeed");

        let first = event_rx.recv().await.expect("user message event");
        let second = event_rx.recv().await.expect("permission request event");

        assert!(matches!(first, DriverEvent::SessionEvent { .. }));
        assert!(matches!(
            second,
            DriverEvent::PermissionRequest {
                session_id: emitted_session_id,
                ..
            } if emitted_session_id == session_id
        ));
    }
}
