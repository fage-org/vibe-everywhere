//! Session Runner
//!
//! 管理单个会话的运行态。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::SessionControlAction;
use ve_shared::types::SessionStatus;

use crate::agent::{create_driver, AgentDriver, DriverConfig, DriverEvent};
use crate::config::Config;
use crate::error::DaemonError;
use crate::Result;

/// Runner 内部状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerState {
    Starting,
    Running,
    WaitingApproval,
    Paused,
    Error,
    Closing,
    Closed,
}

/// 会话运行器命令
#[derive(Debug)]
pub enum RunnerCommand {
    /// 发送消息
    SendMessage {
        content: String,
    },
    /// 执行控制动作
    Control {
        action: SessionControlAction,
    },
    /// 注册权限请求（存储元数据用于 session 级授权缓存）
    RegisterPermission {
        permission_id: Uuid,
        risk_type: String,
        target: Option<String>,
        summary: String,
    },
    /// 响应权限请求
    PermissionResponse {
        permission_id: Uuid,
        decision: PermissionDecision,
    },
    /// 关闭会话
    Close,
}

/// 会话运行器句柄
#[derive(Clone)]
pub struct SessionRunnerHandle {
    /// 命令发送通道
    pub command_tx: mpsc::Sender<RunnerCommand>,
    /// 当前状态
    pub state: RunnerState,
    /// 会话 ID
    pub session_id: Uuid,
}

/// 会话运行器
#[allow(dead_code)]
pub struct SessionRunner {
    /// 会话 ID
    session_id: Uuid,
    /// Workspace 路径
    workspace_path: String,
    /// Agent 类型
    agent_type: String,
    /// 配置引用
    config: Arc<Config>,

    /// 当前状态
    state: RunnerState,
    /// Agent driver
    driver: Box<dyn AgentDriver>,

    /// 命令接收通道
    command_rx: mpsc::Receiver<RunnerCommand>,
    /// 事件发送通道 (给 WS 客户端)
    event_tx: mpsc::Sender<DriverEvent>,

    /// 等待中的权限请求（存储元数据用于 session 级授权缓存）
    pending_permissions: HashMap<Uuid, PendingPermission>,

    /// 权限请求超时时间点
    permission_timeouts: HashMap<Uuid, Instant>,

    /// 本会话授权缓存
    approval_cache: Vec<ApprovalRule>,
}

/// 授权规则
#[derive(Debug, Clone)]
pub struct ApprovalRule {
    pub risk_type: String,
    pub target_pattern: String,
}

/// Check if a target matches a pattern with wildcard support
///
/// Pattern rules:
/// - `*` matches any sequence of characters
/// - `?` matches any single character
/// - literal characters match themselves
fn matches_pattern(pattern: &str, target: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    fn match_helper(pattern: &[char], target: &[char]) -> bool {
        match (pattern.first(), target.first()) {
            (None, None) => true,
            (Some('*'), _) => {
                // * matches zero or more characters
                match_helper(&pattern[1..], target)
                    || (!target.is_empty() && match_helper(pattern, &target[1..]))
            }
            (Some('?'), Some(_)) => {
                // ? matches exactly one character
                match_helper(&pattern[1..], &target[1..])
            }
            (Some(p), Some(t)) if *p == *t => {
                match_helper(&pattern[1..], &target[1..])
            }
            _ => false,
        }
    }

    match_helper(&pattern_chars, &target_chars)
}

impl SessionRunner {
    /// 创建新的会话运行器
    pub fn new(
        session_id: Uuid,
        workspace_path: String,
        agent_type: String,
        config: Arc<Config>,
        event_tx: mpsc::Sender<DriverEvent>,
    ) -> (Self, SessionRunnerHandle) {
        let (command_tx, command_rx) = mpsc::channel(16);

        let handle = SessionRunnerHandle {
            command_tx,
            state: RunnerState::Starting,
            session_id,
        };

        let runner = Self {
            session_id,
            workspace_path: workspace_path.clone(),
            agent_type: agent_type.clone(),
            config: config.clone(),
            state: RunnerState::Starting,
            driver: create_driver(&agent_type, config, event_tx.clone()),
            command_rx,
            event_tx,
            pending_permissions: HashMap::new(),
            permission_timeouts: HashMap::new(),
            approval_cache: Vec::new(),
        };

        (runner, handle)
    }

    /// 运行会话主循环
    pub async fn run(mut self) {
        info!(session_id = %self.session_id, "SessionRunner started");

        // 启动 agent
        let config = DriverConfig {
            session_id: self.session_id,
            workspace_path: self.workspace_path.clone(),
            agent_type: self.agent_type.clone(),
            initial_message: None,
        };

        if let Err(e) = self.driver.start(config).await {
            error!(error = %e, "Failed to start agent");
            self.update_state(RunnerState::Error);
            self.report_status(SessionStatus::Error, Some(e.to_string() as String)).await;
            return;
        }

        self.update_state(RunnerState::Running);
        self.report_status(SessionStatus::Running, None).await;

        // 主循环
        loop {
            tokio::select! {
                // 处理命令
                cmd = self.command_rx.recv() => {
                    match cmd {
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd).await {
                                error!(error = %e, "Failed to handle command");
                            }
                        }
                        None => {
                            info!(session_id = %self.session_id, "Command channel closed");
                            break;
                        }
                    }
                }

                // 检查权限超时 (每秒检查一次)
                _ = tokio::time::sleep(std::time::Duration::from_secs(1)) => {
                    if let Err(e) = self.check_permission_timeouts().await {
                        error!(error = %e, "Failed to check permission timeouts");
                    }
                }
            }

            if self.state == RunnerState::Closed || self.state == RunnerState::Error {
                break;
            }
        }

        info!(session_id = %self.session_id, state = ?self.state, "SessionRunner ended");
    }

    /// 处理命令
    async fn handle_command(&mut self, cmd: RunnerCommand) -> Result<()> {
        match cmd {
            RunnerCommand::SendMessage { content } => {
                if self.state != RunnerState::Running {
                    return Err(DaemonError::SessionInvalidStatus {
                        current: format!("{:?}", self.state),
                        expected: "Running".to_string(),
                    });
                }
                self.driver.send_message(self.session_id, &content).await?;
                debug!(session_id = %self.session_id, "Message sent to agent");
            }

            RunnerCommand::Control { action } => {
                self.handle_control(action).await?;
            }

            RunnerCommand::RegisterPermission {
                permission_id,
                risk_type,
                target,
                summary,
            } => {
                // Check approval cache first
                if self.check_approval_cache(&risk_type, target.as_deref()) {
                    info!(
                        session_id = %self.session_id,
                        %permission_id,
                        risk_type,
                        target = ?target,
                        "Permission auto-approved by cache"
                    );
                    // Auto-approve via driver
                    self.driver
                        .respond_permission(
                            self.session_id,
                            permission_id,
                            PermissionDecision::ApproveSession,
                        )
                        .await?;
                } else {
                    // Cache miss - store pending permission and set timeout
                    self.pending_permissions.insert(
                        permission_id,
                        PendingPermission {
                            risk_type: risk_type.clone(),
                            target: target.clone(),
                            summary,
                        },
                    );
                    // Set timeout
                    let expires_at = Instant::now() + self.config.permission_timeout();
                    self.permission_timeouts.insert(permission_id, expires_at);
                    debug!(
                        session_id = %self.session_id,
                        %permission_id,
                        risk_type,
                        target = ?target,
                        timeout_secs = self.config.permission_timeout().as_secs(),
                        "Permission request registered with timeout"
                    );
                }
            }

            RunnerCommand::PermissionResponse { permission_id, decision } => {
                // Remove from pending and timeout tracking
                if let Some(pending) = self.pending_permissions.remove(&permission_id) {
                    // Remove timeout entry
                    self.permission_timeouts.remove(&permission_id);

                    // Forward response to driver
                    self.driver
                        .respond_permission(self.session_id, permission_id, decision)
                        .await?;

                    // 如果是 approve_session，添加到授权缓存
                    if decision == PermissionDecision::ApproveSession {
                        self.approval_cache.push(ApprovalRule {
                            risk_type: pending.risk_type.clone(),
                            target_pattern: pending.target.clone().unwrap_or("*".to_string()),
                        });
                        debug!(
                            session_id = %self.session_id,
                            risk_type = %pending.risk_type,
                            target = ?pending.target,
                            "Added approval rule to cache"
                        );
                    }
                }
            }

            RunnerCommand::Close => {
                self.handle_close().await?;
            }
        }

        Ok(())
    }

    /// 处理控制动作
    async fn handle_control(&mut self, action: SessionControlAction) -> Result<()> {
        match action {
            SessionControlAction::Pause => {
                self.update_state(RunnerState::Paused);
                self.driver.control(self.session_id, action).await?;
                self.report_status(SessionStatus::Paused, None).await;
            }
            SessionControlAction::Interrupt => {
                self.driver.control(self.session_id, action).await?;
            }
            SessionControlAction::Terminate => {
                self.driver.control(self.session_id, action).await?;
                self.update_state(RunnerState::Closed);
                self.report_status(SessionStatus::Archived, None).await;
            }
            SessionControlAction::Rerun => {
                // TODO: 实现 rerun 逻辑
                self.driver.control(self.session_id, action).await?;
            }
        }
        Ok(())
    }

    /// 处理关闭
    async fn handle_close(&mut self) -> Result<()> {
        self.update_state(RunnerState::Closing);
        self.driver.close(self.session_id).await?;
        self.update_state(RunnerState::Closed);
        self.report_status(SessionStatus::Archived, None).await;
        Ok(())
    }

    /// 更新状态
    fn update_state(&mut self, new_state: RunnerState) {
        debug!(
            session_id = %self.session_id,
            old_state = ?self.state,
            new_state = ?new_state,
            "State transition"
        );
        self.state = new_state;
    }

    /// 上报状态变更
    async fn report_status(&self, status: SessionStatus, summary: Option<String>) {
        let event = DriverEvent::StatusUpdate {
            session_id: self.session_id,
            status,
            summary,
        };
        if self.event_tx.send(event).await.is_err() {
            warn!(session_id = %self.session_id, "Failed to send status update");
        }
    }

    /// 检查授权缓存
    ///
    /// 检查给定的权限请求是否匹配任何已缓存的授权规则。
    /// risk_type 必须完全匹配，target 支持通配符匹配。
    pub fn check_approval_cache(&self, risk_type: &str, target: Option<&str>) -> bool {
        let target_str = target.unwrap_or("*");

        self.approval_cache.iter().any(|rule| {
            // Risk type must match exactly
            if rule.risk_type != risk_type {
                return false;
            }
            // Target must match pattern (supports wildcards)
            matches_pattern(&rule.target_pattern, target_str)
        })
    }

    /// 检查权限超时
    ///
    /// 检查所有等待中的权限请求，如果超时则发送超时响应。
    async fn check_permission_timeouts(&mut self) -> Result<()> {
        let now = Instant::now();

        // 收集所有超时的权限 ID
        let expired: Vec<Uuid> = self
            .permission_timeouts
            .iter()
            .filter(|(_, &expires_at)| now >= expires_at)
            .map(|(&id, _)| id)
            .collect();

        // 处理每个超时
        for permission_id in expired {
            warn!(
                session_id = %self.session_id,
                %permission_id,
                "Permission request timed out"
            );

            // 从 pending 和 timeout 中移除
            self.pending_permissions.remove(&permission_id);
            self.permission_timeouts.remove(&permission_id);

            // 发送超时响应给 driver
            self.driver
                .permission_timeout(self.session_id, permission_id)
                .await?;
        }

        Ok(())
    }
}

impl SessionRunnerHandle {
    /// 发送消息命令
    pub async fn send_message(&self, content: String) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::SendMessage { content })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    /// 发送控制命令
    pub async fn send_control(&self, action: SessionControlAction) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::Control { action })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    /// 发送关闭命令
    pub async fn send_close(&self) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::Close)
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    /// 注册权限请求（存储元数据用于 session 级授权缓存）
    pub async fn register_permission(
        &self,
        permission_id: Uuid,
        risk_type: String,
        target: Option<String>,
        summary: String,
    ) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::RegisterPermission {
                permission_id,
                risk_type,
                target,
                summary,
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }

    /// 发送权限响应
    pub async fn send_permission_response(
        &self,
        permission_id: Uuid,
        decision: PermissionDecision,
    ) -> Result<()> {
        self.command_tx
            .send(RunnerCommand::PermissionResponse {
                permission_id,
                decision,
            })
            .await
            .map_err(|_| DaemonError::ChannelSendFailed("command channel".to_string()))
    }
}

/// 待处理的权限请求
#[derive(Debug, Clone)]
pub struct PendingPermission {
    /// 风险类型
    pub risk_type: String,
    /// 目标资源
    pub target: Option<String>,
    /// 摘要描述
    pub summary: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_state_debug() {
        let state = RunnerState::Running;
        assert_eq!(format!("{:?}", state), "Running");
    }

    #[test]
    fn test_approval_rule_debug() {
        let rule = ApprovalRule {
            risk_type: "write_fs".to_string(),
            target_pattern: "*".to_string(),
        };
        assert!(format!("{:?}", rule).contains("write_fs"));
    }

    #[test]
    fn test_pending_permission_stores_metadata() {
        let pending = PendingPermission {
            risk_type: "write_fs".to_string(),
            target: Some("/workspace/test.txt".to_string()),
            summary: "Write to file".to_string(),
        };
        assert_eq!(pending.risk_type, "write_fs");
        assert_eq!(pending.target, Some("/workspace/test.txt".to_string()));
    }

    #[test]
    fn test_approval_rule_from_pending_permission() {
        let pending = PendingPermission {
            risk_type: "execute_bash".to_string(),
            target: Some("npm test".to_string()),
            summary: "Run tests".to_string(),
        };
        let rule = ApprovalRule {
            risk_type: pending.risk_type.clone(),
            target_pattern: pending.target.clone().unwrap_or("*".to_string()),
        };
        assert_eq!(rule.risk_type, "execute_bash");
        assert_eq!(rule.target_pattern, "npm test");
    }

    // ========== matches_pattern tests ==========

    #[test]
    fn test_matches_pattern_wildcard() {
        assert!(matches_pattern("*", "anything"));
        assert!(matches_pattern("*", ""));
        assert!(matches_pattern("*", "/some/long/path"));
    }

    #[test]
    fn test_matches_pattern_extension() {
        assert!(matches_pattern("*.txt", "file.txt"));
        assert!(matches_pattern("*.txt", "test.txt"));
        assert!(!matches_pattern("*.txt", "file.rs"));
        assert!(!matches_pattern("*.txt", "txt"));
    }

    #[test]
    fn test_matches_pattern_question() {
        assert!(matches_pattern("file?.txt", "file1.txt"));
        assert!(matches_pattern("file?.txt", "fileA.txt"));
        assert!(!matches_pattern("file?.txt", "file.txt"));
        assert!(!matches_pattern("file?.txt", "file12.txt"));
    }

    #[test]
    fn test_matches_pattern_path() {
        assert!(matches_pattern("/home/user/*", "/home/user/project"));
        assert!(matches_pattern("/home/user/*", "/home/user/"));
        assert!(!matches_pattern("/home/user/*", "/home/other/file"));
    }

    #[test]
    fn test_matches_pattern_empty() {
        assert!(matches_pattern("", ""));
        assert!(!matches_pattern("", "x"));
        assert!(!matches_pattern("x", ""));
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("exact", "exact"));
        assert!(!matches_pattern("exact", "Exact"));
        assert!(!matches_pattern("exact", "exact "));
    }

    #[test]
    fn test_matches_pattern_multiple_wildcards() {
        assert!(matches_pattern("*.rs", "lib.rs"));
        // Note: Our simple glob doesn't support ** special behavior
        // * matches any characters including /, so this works:
        assert!(matches_pattern("src/*", "src/main.rs"));
        assert!(matches_pattern("src/*", "src/lib/mod.rs"));
        // Pattern with multiple * works:
        assert!(matches_pattern("*.txt", "file.txt"));
        assert!(matches_pattern("*test*", "mytestfile"));
    }
}