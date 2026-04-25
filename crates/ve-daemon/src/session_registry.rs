//! Session Registry
//!
//! 管理所有活跃会话运行器。

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, oneshot, RwLock};
use tracing::{info, warn};
use uuid::Uuid;
use ve_shared::proto::SessionControlAction;

use crate::agent::DriverEvent;
use crate::config::Config;
use crate::error::DaemonError;
use crate::session_runner::{SessionRunner, SessionRunnerHandle};
use crate::Result;

/// Session Registry
pub struct SessionRegistry {
    /// 活跃会话映射
    runners: RwLock<HashMap<Uuid, SessionRunnerHandle>>,
    /// 配置引用
    config: Arc<Config>,
    /// 事件发送通道 (broadcast)
    event_tx: broadcast::Sender<DriverEvent>,
}

impl SessionRegistry {
    /// 创建新的注册中心
    pub fn new(config: Arc<Config>, event_tx: broadcast::Sender<DriverEvent>) -> Self {
        Self {
            runners: RwLock::new(HashMap::new()),
            config,
            event_tx,
        }
    }

    /// 创建新会话
    pub async fn create(
        &self,
        session_id: Uuid,
        workspace_path: String,
        agent_type: String,
        initial_message: Option<String>,
    ) -> Result<()> {
        // 使用单个 write lock 进行原子性的检查-插入操作
        let mut runners = self.runners.write().await;

        // 检查是否已存在
        if runners.contains_key(&session_id) {
            return Err(DaemonError::SessionCreateFailed {
                reason: "Session already exists".to_string(),
            });
        }

        // 检查最大并发数
        if runners.len() >= self.config.max_parallel_sessions {
            return Err(DaemonError::MaxSessionsReached {
                max: self.config.max_parallel_sessions,
            });
        }

        let (startup_tx, startup_rx) = oneshot::channel();
        let (runner, handle) = SessionRunner::new(
            session_id,
            workspace_path,
            agent_type,
            initial_message,
            self.config.clone(),
            self.event_tx.clone(),
            Some(startup_tx),
        );

        runners.insert(session_id, handle);
        drop(runners);

        tokio::spawn(async move {
            runner.run().await;
        });

        match tokio::time::timeout(self.config.ack_timeout(), startup_rx).await {
            Ok(Ok(Ok(()))) => {
                info!(%session_id, "Session created and started");
                Ok(())
            }
            Ok(Ok(Err(error))) => {
                self.remove(&session_id).await;
                Err(error)
            }
            Ok(Err(_)) => {
                self.remove(&session_id).await;
                Err(DaemonError::InternalTaskError(
                    "Session startup channel closed".to_string(),
                ))
            }
            Err(_) => {
                // 超时：发送 Close 命令让 runner 退出，然后移除
                let handle = self.get(&session_id).await;
                if let Some(handle) = handle {
                    let _ = handle.send_close().await;
                }
                self.remove(&session_id).await;
                Err(DaemonError::SessionInvalidStatus {
                    current: "timeout".to_string(),
                    expected: "session startup completion".to_string(),
                })
            }
        }
    }

    /// 创建恢复会话
    pub async fn create_rerun(
        &self,
        session_id: Uuid,
        workspace_path: String,
        agent_type: String,
        claude_session_id: String,
    ) -> Result<()> {
        let mut runners = self.runners.write().await;

        if runners.contains_key(&session_id) {
            return Err(DaemonError::SessionCreateFailed {
                reason: "Session already exists".to_string(),
            });
        }

        if runners.len() >= self.config.max_parallel_sessions {
            return Err(DaemonError::MaxSessionsReached {
                max: self.config.max_parallel_sessions,
            });
        }

        let (startup_tx, startup_rx) = oneshot::channel();
        let (runner, handle) = SessionRunner::new_rerun(
            session_id,
            workspace_path,
            agent_type,
            claude_session_id,
            self.config.clone(),
            self.event_tx.clone(),
            Some(startup_tx),
        );

        runners.insert(session_id, handle);
        drop(runners);

        tokio::spawn(async move {
            runner.run().await;
        });

        match tokio::time::timeout(self.config.ack_timeout(), startup_rx).await {
            Ok(Ok(Ok(()))) => {
                info!(%session_id, "Session rerun created and started");
                Ok(())
            }
            Ok(Ok(Err(error))) => {
                self.remove(&session_id).await;
                Err(error)
            }
            Ok(Err(_)) => {
                self.remove(&session_id).await;
                Err(DaemonError::InternalTaskError(
                    "Session rerun startup channel closed".to_string(),
                ))
            }
            Err(_) => {
                let handle = self.get(&session_id).await;
                if let Some(handle) = handle {
                    let _ = handle.send_close().await;
                }
                self.remove(&session_id).await;
                Err(DaemonError::SessionInvalidStatus {
                    current: "timeout".to_string(),
                    expected: "session rerun startup completion".to_string(),
                })
            }
        }
    }

    /// 获取会话句柄
    pub async fn get(&self, session_id: &Uuid) -> Option<SessionRunnerHandle> {
        let runners = self.runners.read().await;
        runners.get(session_id).cloned()
    }

    /// 移除会话
    pub async fn remove(&self, session_id: &Uuid) {
        let mut runners = self.runners.write().await;
        if runners.remove(session_id).is_some() {
            info!(%session_id, "Session removed from registry");
        }
    }

    pub async fn send_message_and_wait(&self, session_id: Uuid, content: String) -> Result<()> {
        let handle = self
            .get(&session_id)
            .await
            .ok_or(DaemonError::SessionNotFound {
                session_id: session_id.to_string(),
            })?;
        handle
            .send_message_and_wait(content, self.config.ack_timeout())
            .await
    }

    pub async fn send_control_and_wait(
        &self,
        session_id: Uuid,
        action: SessionControlAction,
    ) -> Result<()> {
        let handle = self
            .get(&session_id)
            .await
            .ok_or(DaemonError::SessionNotFound {
                session_id: session_id.to_string(),
            })?;
        handle
            .send_control_and_wait(action, self.config.ack_timeout())
            .await?;

        if action == SessionControlAction::Terminate {
            self.remove(&session_id).await;
        }

        Ok(())
    }

    pub async fn list_active_session_ids(&self) -> Vec<Uuid> {
        let runners = self.runners.read().await;
        runners.keys().copied().collect()
    }

    /// 关闭所有会话
    pub async fn shutdown_all(&self) {
        info!("Shutting down all sessions");
        let runners = self.runners.read().await;
        for (session_id, handle) in runners.iter() {
            if let Err(e) = handle.send_close().await {
                warn!(%session_id, error = %e, "Failed to send close command");
            }
        }
    }

    /// 获取活跃会话数
    pub async fn active_count(&self) -> usize {
        self.runners.read().await.len()
    }

    /// 关闭并移除会话 (原子操作)
    ///
    /// 如果发送关闭命令成功，会话将从 registry 移除。
    /// 如果发送失败，会话仍保留在 registry 中，可重试。
    pub async fn close_and_remove(&self, session_id: &Uuid) -> Result<()> {
        // 先获取 handle
        let handle = {
            let runners = self.runners.read().await;
            runners.get(session_id).cloned()
        };

        match handle {
            Some(handle) => {
                // 发送关闭命令
                handle
                    .send_close_and_wait(self.config.ack_timeout())
                    .await?;

                // 成功后才移除
                let mut runners = self.runners.write().await;
                runners.remove(session_id);
                info!(%session_id, "Session closed and removed");
                Ok(())
            }
            None => {
                warn!(%session_id, "Session not found for close");
                Err(DaemonError::SessionNotFound {
                    session_id: session_id.to_string(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Arc<Config> {
        Arc::new(Config {
            server_url: "https://test.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: std::path::PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 4,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            permission_mode: "default".to_string(),
            mock_mode: false,
        })
    }

    #[tokio::test]
    async fn test_registry_create_and_get() {
        let config = create_test_config();
        let (event_tx, _event_rx) = broadcast::channel(16);
        let registry = SessionRegistry::new(config, event_tx);

        let session_id = Uuid::new_v4();
        let result = registry
            .create(
                session_id,
                "/workspace".to_string(),
                "claude_code".to_string(),
                None,
            )
            .await;
        assert!(result.is_ok());

        let handle = registry.get(&session_id).await;
        assert!(handle.is_some());
    }

    #[tokio::test]
    async fn test_registry_duplicate_create() {
        let config = create_test_config();
        let (event_tx, _event_rx) = broadcast::channel(16);
        let registry = SessionRegistry::new(config, event_tx);

        let session_id = Uuid::new_v4();
        registry
            .create(
                session_id,
                "/workspace".to_string(),
                "claude_code".to_string(),
                None,
            )
            .await
            .unwrap();

        // 第二次创建相同 session_id 应该失败
        let result = registry
            .create(
                session_id,
                "/workspace".to_string(),
                "claude_code".to_string(),
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_max_sessions() {
        let config = Arc::new(Config {
            server_url: "https://test.com".to_string(),
            host_name: "test".to_string(),
            platform: "linux".to_string(),
            config_dir: std::path::PathBuf::from("/tmp"),
            log_format: "pretty".to_string(),
            log_level: "info".to_string(),
            heartbeat_interval_secs: 30,
            heartbeat_timeout_secs: 90,
            ack_timeout_secs: 30,
            permission_timeout_secs: 60,
            reconnect_backoff_min_ms: 1000,
            reconnect_backoff_max_ms: 30000,
            max_parallel_sessions: 2,
            file_read_text_limit_bytes: 262_144,
            file_tree_max_nodes: 20_000,
            claude_command: "claude".to_string(),
            default_model: "claude-sonnet-4-20250514".to_string(),
            permission_mode: "default".to_string(),
            mock_mode: false,
        });
        let (event_tx, _event_rx) = broadcast::channel(16);
        let registry = SessionRegistry::new(config, event_tx);

        // 创建 2 个会话
        for _ in 0..2 {
            let session_id = Uuid::new_v4();
            registry
                .create(
                    session_id,
                    "/workspace".to_string(),
                    "claude_code".to_string(),
                    None,
                )
                .await
                .unwrap();
        }

        // 第 3 个会话应该失败
        let session_id = Uuid::new_v4();
        let result = registry
            .create(
                session_id,
                "/workspace".to_string(),
                "claude_code".to_string(),
                None,
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_remove() {
        let config = create_test_config();
        let (event_tx, _event_rx) = broadcast::channel(16);
        let registry = SessionRegistry::new(config, event_tx);

        let session_id = Uuid::new_v4();
        registry
            .create(
                session_id,
                "/workspace".to_string(),
                "claude_code".to_string(),
                None,
            )
            .await
            .unwrap();

        registry.remove(&session_id).await;
        let handle = registry.get(&session_id).await;
        assert!(handle.is_none());
    }

    #[tokio::test]
    async fn test_registry_active_count() {
        let config = create_test_config();
        let (event_tx, _event_rx) = broadcast::channel(16);
        let registry = SessionRegistry::new(config, event_tx);

        assert_eq!(registry.active_count().await, 0);

        let session_id = Uuid::new_v4();
        registry
            .create(
                session_id,
                "/workspace".to_string(),
                "claude_code".to_string(),
                None,
            )
            .await
            .unwrap();

        assert_eq!(registry.active_count().await, 1);
    }

    /// M1 Fix Test: 验证并发创建相同 session_id 时只有一个成功
    /// 这个测试确保 TOCTOU 竞态条件被正确处理
    #[tokio::test]
    async fn test_registry_concurrent_create_same_session_id() {
        let config = create_test_config();
        let (event_tx, _event_rx) = broadcast::channel(16);
        let registry = Arc::new(SessionRegistry::new(config, event_tx));

        let session_id = Uuid::new_v4();
        let registry_clone = registry.clone();

        // 并发创建相同 session_id
        let handle1 = tokio::spawn(async move {
            registry_clone
                .create(
                    session_id,
                    "/workspace".to_string(),
                    "claude_code".to_string(),
                    None,
                )
                .await
        });

        let registry_clone = registry.clone();
        let handle2 = tokio::spawn(async move {
            registry_clone
                .create(
                    session_id,
                    "/workspace".to_string(),
                    "claude_code".to_string(),
                    None,
                )
                .await
        });

        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        // 必须只有一个成功
        let success_count = [result1.is_ok(), result2.is_ok()]
            .iter()
            .filter(|&&x| x)
            .count();
        assert_eq!(
            success_count, 1,
            "Only one create should succeed, but got {}",
            success_count
        );

        // 确认只有一个 session 被创建
        assert_eq!(registry.active_count().await, 1);
    }

    /// M2 Test: 验证 close_and_remove 的原子性行为
    #[tokio::test]
    async fn test_registry_close_and_remove_existing_session() {
        let config = create_test_config();
        let (event_tx, _event_rx) = broadcast::channel(16);
        let registry = SessionRegistry::new(config, event_tx);

        // 创建会话
        let session_id = Uuid::new_v4();
        registry
            .create(
                session_id,
                "/workspace".to_string(),
                "claude_code".to_string(),
                None,
            )
            .await
            .unwrap();

        assert_eq!(registry.active_count().await, 1);

        // 关闭并移除
        let result = registry.close_and_remove(&session_id).await;
        assert!(result.is_ok());
        assert_eq!(registry.active_count().await, 0);

        // 再次尝试关闭应该返回错误
        let result = registry.close_and_remove(&session_id).await;
        assert!(result.is_err());
    }

    /// M2 Test: 验证关闭不存在的 session
    #[tokio::test]
    async fn test_registry_close_nonexistent_session() {
        let config = create_test_config();
        let (event_tx, _event_rx) = broadcast::channel(16);
        let registry = SessionRegistry::new(config, event_tx);

        let session_id = Uuid::new_v4();
        let result = registry.close_and_remove(&session_id).await;
        assert!(result.is_err());

        // 验证错误类型
        match result {
            Err(DaemonError::SessionNotFound { .. }) => {}
            _ => panic!("Expected SessionNotFound error"),
        }
    }
}
