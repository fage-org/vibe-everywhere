//! Claude Code CLI Driver
//!
//! Manages Claude Code CLI subprocess lifecycle, I/O, and stream-json parsing.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command as TokioCommand};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use ve_shared::models::PermissionDecision;
use ve_shared::proto::SessionControlAction;
use ve_shared::types::SessionStatus;

use super::{AgentDriver, DriverConfig, DriverEvent};
use crate::config::Config;
use crate::error::DaemonError;
use crate::Result;

/// Claude Code CLI Driver
#[allow(dead_code)]
pub struct ClaudeCodeDriver {
    /// Configuration reference
    config: Arc<Config>,

    /// Subprocess handle
    child: Option<Child>,

    /// stdin writer
    stdin: Option<ChildStdin>,

    /// Event sender channel
    event_tx: tokio::sync::broadcast::Sender<DriverEvent>,

    /// Current session ID
    session_id: Option<Uuid>,

    /// Background stdout reader task for the active CLI process
    stdout_reader: Option<tokio::task::JoinHandle<()>>,

    /// Claude session ID (returned by CLI, for --resume support)
    claude_session_id: Option<String>,

    /// Workspace path (for rerun support)
    workspace_path: Option<String>,

    /// Pending permission requests (for tracking permission responses)
    pending_permissions: HashMap<String, tokio::sync::oneshot::Sender<PermissionDecision>>,
}

/// Stream-JSON event types from Claude CLI (supports both verbose and non-verbose formats)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamJsonEvent {
    /// Partial output (streaming) — non-verbose format
    Partial { content: String },

    /// Complete message — non-verbose format: `{"type":"message",...}`
    Message { message: ClaudeMessage },

    /// Assistant message — verbose format: `{"type":"assistant","message":{...}}`
    #[serde(rename = "assistant")]
    Assistant { message: ClaudeMessage },

    /// Tool use event
    #[serde(rename = "tool_use")]
    ToolUse {
        tool_name: String,
        tool_input: serde_json::Value,
    },

    /// Tool result event
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_name: String,
        tool_result: serde_json::Value,
    },

    /// Result/completion
    Result { summary: Option<String> },

    /// Error
    Error { message: String },

    /// Session ID (sent at start)
    #[serde(rename = "session_id")]
    SessionId { session_id: String },
}

/// Claude message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: Vec<ContentBlock>,
}

/// Content block in a Claude message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: Option<String>,
    /// Tool name (for tool_use blocks)
    pub name: Option<String>,
    /// Tool input (for tool_use blocks)
    pub input: Option<serde_json::Value>,
}

impl ClaudeCodeDriver {
    /// Create a new Claude Code Driver
    pub fn new(config: Arc<Config>, event_tx: tokio::sync::broadcast::Sender<DriverEvent>) -> Self {
        Self {
            config,
            child: None,
            stdin: None,
            event_tx,
            session_id: None,
            stdout_reader: None,
            claude_session_id: None,
            workspace_path: None,
            pending_permissions: HashMap::new(),
        }
    }

    /// Check if CLI executable exists
    pub fn check_cli_exists(&self) -> Result<()> {
        which::which(&self.config.claude_command).map_err(|_| DaemonError::CliNotFound {
            command: self.config.claude_command.clone(),
        })?;
        Ok(())
    }

    /// Spawn stdout reader task
    fn spawn_stdout_reader(
        &self,
        stdout: ChildStdout,
        session_id: Uuid,
    ) -> tokio::task::JoinHandle<()> {
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let reader = AsyncBufReader::new(stdout).lines();
            let mut lines = reader;

            while let Some(line) = lines.next_line().await.transpose() {
                let line = match line {
                    Ok(l) => l,
                    Err(e) => {
                        error!(error = %e, "Failed to read stdout line");
                        break;
                    }
                };

                if line.is_empty() {
                    continue;
                }

                debug!(line = %line, "CLI stdout line");

                // Parse stream-json
                match serde_json::from_str::<StreamJsonEvent>(&line) {
                    Ok(event) => {
                        if let Err(e) =
                            Self::handle_stream_event(&event_tx, session_id, event).await
                        {
                            error!(error = %e, "Failed to handle stream event");
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, line = %line, "Failed to parse stream-json");
                    }
                }
            }

            info!(%session_id, "CLI stdout reader ended");
        })
    }

    /// Handle a stream-json event from CLI
    async fn handle_stream_event(
        event_tx: &tokio::sync::broadcast::Sender<DriverEvent>,
        session_id: Uuid,
        event: StreamJsonEvent,
    ) -> Result<()> {
        match event {
            StreamJsonEvent::Partial { content } => {
                event_tx
                    .send(DriverEvent::SessionEvent {
                        session_id,
                        event_type: "log".to_string(),
                        data: serde_json::json!({ "content": content }),
                    })
                    .ok();
            }

            StreamJsonEvent::Message { message } | StreamJsonEvent::Assistant { message } => {
                if message.role == "assistant" {
                    let text = message
                        .content
                        .iter()
                        .filter_map(|c| {
                            // Extract text blocks
                            if let Some(t) = c.text.as_ref() {
                                return Some(t.clone());
                            }
                            // For tool_use blocks, include a readable summary
                            if c.content_type == "tool_use" {
                                let name = c.name.as_deref().unwrap_or("unknown");
                                let input = c
                                    .input
                                    .as_ref()
                                    .map(|v| serde_json::to_string(v).unwrap_or_default())
                                    .unwrap_or_default();
                                return Some(format!("[Using tool: {name}] {input}"));
                            }
                            None
                        })
                        .collect::<Vec<_>>()
                        .join("");

                    event_tx
                        .send(DriverEvent::SessionEvent {
                            session_id,
                            event_type: "agent_reply".to_string(),
                            data: serde_json::json!({ "content": text }),
                        })
                        .ok();
                }
            }

            StreamJsonEvent::ToolUse {
                tool_name,
                tool_input,
            } => {
                // Check if this is a permission request
                if tool_name == "mcp__ve_daemon__permission_prompt" {
                    // Parse permission request
                    if let Some(risk_type) = tool_input.get("risk_type").and_then(|v| v.as_str()) {
                        let permission_id = Uuid::new_v4();

                        event_tx
                            .send(DriverEvent::PermissionRequest {
                                permission_id,
                                session_id,
                                risk_type: risk_type.to_string(),
                                summary: tool_input
                                    .get("summary")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                target: tool_input
                                    .get("target")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                            })
                            .ok();
                    }
                } else {
                    event_tx
                        .send(DriverEvent::SessionEvent {
                            session_id,
                            event_type: "tool_call".to_string(),
                            data: serde_json::json!({
                                "tool_name": tool_name,
                                "tool_input": tool_input,
                            }),
                        })
                        .ok();
                }
            }

            StreamJsonEvent::ToolResult {
                tool_name,
                tool_result,
            } => {
                event_tx
                    .send(DriverEvent::SessionEvent {
                        session_id,
                        event_type: "tool_result".to_string(),
                        data: serde_json::json!({
                            "tool_name": tool_name,
                            "tool_result": tool_result,
                        }),
                    })
                    .ok();
            }

            StreamJsonEvent::Result { summary } => {
                event_tx
                    .send(DriverEvent::StatusUpdate {
                        session_id,
                        status: SessionStatus::Running,
                        summary,
                        close_reason: None,
                    })
                    .ok();
            }

            StreamJsonEvent::Error { message } => {
                event_tx
                    .send(DriverEvent::FatalError {
                        session_id,
                        message: message.clone(),
                    })
                    .ok();

                event_tx
                    .send(DriverEvent::StatusUpdate {
                        session_id,
                        status: SessionStatus::Error,
                        summary: Some(message),
                        close_reason: None,
                    })
                    .ok();
            }

            StreamJsonEvent::SessionId {
                session_id: claude_sid,
            } => {
                info!(%session_id, claude_session_id = %claude_sid, "CLI session started");

                // Send ClaudeSessionId event for --resume support
                event_tx
                    .send(DriverEvent::ClaudeSessionId {
                        session_id,
                        claude_session_id: claude_sid.clone(),
                    })
                    .ok();
            }
        }

        Ok(())
    }

    /// Write to stdin
    async fn write_stdin(&mut self, content: &str) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or(DaemonError::CliStdinWriteFailed)?;

        let input = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [
                    { "type": "text", "text": content }
                ]
            }
        });
        let line =
            serde_json::to_string(&input).map_err(|e| DaemonError::CliStdoutParseFailed {
                reason: e.to_string(),
            })?;

        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;
        stdin
            .flush()
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;

        Ok(())
    }

    fn permission_bridge_dir(&self) -> PathBuf {
        self.config.config_dir.join("permission-bridge")
    }

    fn permission_mcp_server_script() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../scripts/permission_prompt_mcp.py")
    }

    fn permission_mcp_config_json(&self, session_id: Uuid) -> Result<String> {
        let script_path = Self::permission_mcp_server_script();
        if !script_path.exists() {
            return Err(DaemonError::CliStartFailed {
                reason: format!(
                    "Permission MCP server script not found: {}",
                    script_path.display()
                ),
            });
        }

        Ok(serde_json::json!({
            "mcpServers": {
                "ve_daemon": {
                    "command": "python3",
                    "args": [script_path.to_string_lossy().to_string()],
                    "env": {
                        "VE_PERMISSION_BRIDGE_DIR": self.permission_bridge_dir().to_string_lossy().to_string(),
                        "VE_PERMISSION_SESSION_ID": session_id.to_string(),
                        "VE_PERMISSION_BRIDGE_TIMEOUT_SECS": self.config.permission_timeout_secs.to_string(),
                    }
                }
            }
        })
        .to_string())
    }
}

#[async_trait]
impl AgentDriver for ClaudeCodeDriver {
    async fn start(&mut self, config: DriverConfig) -> Result<()> {
        self.check_cli_exists()?;
        self.session_id = Some(config.session_id);
        self.workspace_path = Some(config.workspace_path.clone());
        let permission_mcp_config = self.permission_mcp_config_json(config.session_id)?;

        // Build command
        let mut cmd = TokioCommand::new(&self.config.claude_command);
        cmd.arg("-p")
            .arg("--verbose")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--permission-mode")
            .arg(&self.config.permission_mode)
            .arg("--mcp-config")
            .arg(permission_mcp_config)
            .arg("--permission-prompt-tool")
            .arg("mcp__ve_daemon__permission_prompt")
            .arg("--session-id")
            .arg(config.session_id.to_string())
            .arg("--model")
            .arg(&self.config.default_model)
            .arg("--add-dir")
            .arg(&config.workspace_path);
        // Note: initial_message is sent via stdin below, not as a CLI arg,
        // because --input-format stream-json makes the CLI ignore positional messages.

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn process
        let mut child = cmd.spawn().map_err(|e| DaemonError::CliStartFailed {
            reason: format!("Failed to spawn: {}", e),
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| DaemonError::CliStartFailed {
                reason: "Failed to get stdin".to_string(),
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| DaemonError::CliStartFailed {
                reason: "Failed to get stdout".to_string(),
            })?;

        self.stdin = Some(stdin);
        self.child = Some(child);

        // Spawn stdout reader task
        self.stdout_reader = Some(self.spawn_stdout_reader(stdout, config.session_id));

        // Send initial message via stdin (required by --input-format stream-json)
        if let Some(msg) = config.initial_message {
            self.write_stdin(&msg).await?;
        }

        info!(
            session_id = %config.session_id,
            workspace = %config.workspace_path,
            "Claude Code CLI started"
        );

        Ok(())
    }

    async fn send_message(&mut self, session_id: Uuid, content: &str) -> Result<()> {
        debug!(%session_id, content_len = content.len(), "Sending message to CLI");
        self.write_stdin(content).await
    }

    async fn control(&mut self, session_id: Uuid, action: SessionControlAction) -> Result<()> {
        match action {
            SessionControlAction::Interrupt => {
                return Err(DaemonError::Unknown(
                    "Interrupt is not supported safely for Claude Code sessions".to_string(),
                ));
            }
            SessionControlAction::Terminate => {
                // Send SIGTERM/KILL
                if let Some(ref mut child) = self.child {
                    child
                        .start_kill()
                        .map_err(|e| DaemonError::SessionCloseFailed {
                            reason: format!("Failed to kill: {}", e),
                        })?;
                }
            }
            SessionControlAction::Pause => {
                // Claude Code may not support pause, log warning
                warn!(%session_id, "Pause action not fully supported");
            }
            SessionControlAction::Rerun => {
                warn!(%session_id, "Rerun should be handled via archived resume flow");
            }
            SessionControlAction::Restart => {
                warn!(%session_id, "Restart should be called via driver.rerun(), not control()");
            }
        }
        Ok(())
    }

    async fn respond_permission(
        &mut self,
        _session_id: Uuid,
        _permission_id: Uuid,
        decision: PermissionDecision,
    ) -> Result<()> {
        // Write permission response to stdin
        let response = match decision {
            PermissionDecision::ApproveOnce => "approve_once",
            PermissionDecision::DenyOnce => "deny_once",
            PermissionDecision::ApproveSession => "approve_session",
        };

        let input = serde_json::json!({
            "type": "permission_response",
            "decision": response,
        });

        let stdin = self
            .stdin
            .as_mut()
            .ok_or(DaemonError::CliStdinWriteFailed)?;

        let line =
            serde_json::to_string(&input).map_err(|e| DaemonError::CliStdoutParseFailed {
                reason: e.to_string(),
            })?;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;
        stdin
            .flush()
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;

        debug!(decision = ?decision, "Permission response sent to CLI");
        Ok(())
    }

    async fn permission_timeout(&mut self, session_id: Uuid, permission_id: Uuid) -> Result<()> {
        warn!(%session_id, %permission_id, "Permission request timed out");

        // Send timeout response to CLI (deny with reason)
        let input = serde_json::json!({
            "type": "permission_response",
            "decision": "deny_once",  // Deny on timeout, but don't persist
            "reason": "timeout",
        });

        let stdin = self
            .stdin
            .as_mut()
            .ok_or(DaemonError::CliStdinWriteFailed)?;

        let line =
            serde_json::to_string(&input).map_err(|e| DaemonError::CliStdoutParseFailed {
                reason: e.to_string(),
            })?;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;
        stdin
            .flush()
            .await
            .map_err(|_| DaemonError::CliStdinWriteFailed)?;

        debug!(%permission_id, "Permission timeout response sent to CLI");
        Ok(())
    }

    async fn close(&mut self, session_id: Uuid) -> Result<()> {
        if let Some(ref mut child) = self.child {
            // Try graceful shutdown
            child.start_kill().ok();

            // Wait for process exit
            match tokio::time::timeout(std::time::Duration::from_secs(5), child.wait()).await {
                Ok(Ok(status)) => {
                    info!(%session_id, ?status, "CLI process exited");
                }
                Ok(Err(e)) => {
                    warn!(%session_id, error = %e, "Failed to wait for CLI exit");
                }
                Err(_) => {
                    warn!(%session_id, "CLI process did not exit in time, killing");
                    child.start_kill().ok();
                }
            }
        }

        if let Some(stdout_reader) = self.stdout_reader.take() {
            match tokio::time::timeout(std::time::Duration::from_secs(1), stdout_reader).await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => {
                    warn!(%session_id, error = %error, "stdout reader task failed");
                }
                Err(_) => {
                    warn!(%session_id, "stdout reader did not exit in time");
                }
            }
        }

        self.child = None;
        self.stdin = None;

        info!(%session_id, "Claude Code CLI closed");
        Ok(())
    }

    async fn rerun(
        &mut self,
        session_id: Uuid,
        workspace_path: &str,
        claude_session_id: &str,
    ) -> Result<()> {
        self.session_id = Some(session_id);
        self.workspace_path = Some(workspace_path.to_string());
        let permission_mcp_config = self.permission_mcp_config_json(session_id)?;

        // Close current CLI process first
        self.close(session_id).await?;

        // Build command with --resume flag
        let mut cmd = TokioCommand::new(&self.config.claude_command);
        cmd.arg("-p")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--input-format")
            .arg("stream-json")
            .arg("--permission-mode")
            .arg(&self.config.permission_mode)
            .arg("--mcp-config")
            .arg(permission_mcp_config)
            .arg("--permission-prompt-tool")
            .arg("mcp__ve_daemon__permission_prompt")
            .arg("--session-id")
            .arg(session_id.to_string())
            .arg("--model")
            .arg(&self.config.default_model)
            .arg("--add-dir")
            .arg(workspace_path)
            .arg("--resume")
            .arg(claude_session_id);

        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn process
        let mut child = cmd.spawn().map_err(|e| DaemonError::SessionRerunFailed {
            reason: format!("Failed to spawn: {}", e),
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| DaemonError::SessionRerunFailed {
                reason: "Failed to get stdin".to_string(),
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| DaemonError::SessionRerunFailed {
                reason: "Failed to get stdout".to_string(),
            })?;

        self.stdin = Some(stdin);
        self.child = Some(child);

        // Spawn stdout reader task
        self.stdout_reader = Some(self.spawn_stdout_reader(stdout, session_id));

        info!(
            %session_id,
            claude_session_id = %claude_session_id,
            workspace = %workspace_path,
            "Claude Code CLI resumed"
        );

        Ok(())
    }
}
