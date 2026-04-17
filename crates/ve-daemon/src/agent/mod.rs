//! Agent Driver 抽象层
//!
//! 定义与 CLI agent 交互的统一接口。

mod claude_code;

use async_trait::async_trait;
use std::sync::Arc;
use tracing::warn;

pub use claude_code::ClaudeCodeDriver;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use ve_shared::proto::SessionControlAction;
use ve_shared::types::SessionStatus;
use ve_shared::models::PermissionDecision;

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
    },

    /// 致命错误
    FatalError {
        session_id: Uuid,
        message: String,
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

    /// 关闭 agent
    async fn close(&mut self, session_id: Uuid) -> Result<()>;
}

/// Mock Agent Driver (用于测试)
pub struct MockDriver {
    event_tx: tokio::sync::mpsc::Sender<DriverEvent>,
}

impl MockDriver {
    pub fn new(event_tx: tokio::sync::mpsc::Sender<DriverEvent>) -> Self {
        Self { event_tx }
    }
}

#[async_trait]
impl AgentDriver for MockDriver {
    async fn start(&mut self, config: DriverConfig) -> Result<()> {
        self.event_tx.send(DriverEvent::StatusUpdate {
            session_id: config.session_id,
            status: SessionStatus::Running,
            summary: None,
        }).await.ok();
        Ok(())
    }

    async fn send_message(&mut self, session_id: Uuid, content: &str) -> Result<()> {
        // 模拟回复
        self.event_tx.send(DriverEvent::SessionEvent {
            session_id,
            event_type: "user_message".to_string(),
            data: serde_json::json!({ "content": content }),
        }).await.ok();
        Ok(())
    }

    async fn control(&mut self, session_id: Uuid, action: SessionControlAction) -> Result<()> {
        let status = match action {
            SessionControlAction::Pause => SessionStatus::Paused,
            SessionControlAction::Terminate => SessionStatus::Archived,
            _ => SessionStatus::Running,
        };
        self.event_tx.send(DriverEvent::StatusUpdate {
            session_id,
            status,
            summary: None,
        }).await.ok();
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

    async fn close(&mut self, session_id: Uuid) -> Result<()> {
        self.event_tx.send(DriverEvent::StatusUpdate {
            session_id,
            status: SessionStatus::Archived,
            summary: Some("Session closed".to_string()),
        }).await.ok();
        Ok(())
    }
}

/// Create an Agent Driver based on the specified type
///
/// # Arguments
/// * `agent_type` - The type of agent driver to create ("claude_code" or other)
/// * `config` - Daemon configuration reference
/// * `event_tx` - Channel for sending driver events
///
/// # Returns
/// A boxed driver implementing `AgentDriver`
pub fn create_driver(
    agent_type: &str,
    config: Arc<crate::config::Config>,
    event_tx: tokio::sync::mpsc::Sender<DriverEvent>,
) -> Box<dyn AgentDriver> {
    match agent_type {
        "claude_code" => Box::new(ClaudeCodeDriver::new(config, event_tx)),
        _ => {
            warn!(agent_type, "Unknown agent type, using mock driver");
            Box::new(MockDriver::new(event_tx))
        }
    }
}