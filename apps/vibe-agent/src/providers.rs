use anyhow::Result;
use serde_json::Value;
use std::path::Path;
use tokio::process::Command;
use vibe_core::{
    ExecutionProtocol, ProviderKind, ProviderStatus, TaskEventInput, TaskEventKind,
    TaskExecutionMode, TaskRecord,
};

pub(crate) fn build_provider_command(
    provider: &ProviderStatus,
    task: &TaskRecord,
    cwd: &Path,
) -> Result<Command> {
    let mut command = Command::new(&provider.command);
    let prompt = render_task_prompt(task);

    match task.provider {
        ProviderKind::Codex => {
            command.arg("exec");
            if let Some(session_id) = task.provider_session_id.as_deref() {
                command.arg("resume").arg(session_id);
            }
            command
                .arg("--json")
                .arg("--skip-git-repo-check")
                .arg("--sandbox")
                .arg(codex_sandbox_mode(task.execution_mode.clone()))
                .arg("--ask-for-approval")
                .arg(codex_approval_policy(task.execution_mode.clone()))
                .arg("-C")
                .arg(cwd);
            if let Some(model) = &task.model {
                command.arg("-m").arg(model);
            }
            command.arg(&prompt);
        }
        ProviderKind::ClaudeCode => {
            command.arg("-p");
            if let Some(session_id) = task.provider_session_id.as_deref() {
                command.arg("--resume").arg(session_id);
            }
            command
                .arg("--output-format")
                .arg("stream-json")
                .arg("--verbose")
                .arg("--include-partial-messages")
                .arg("--permission-mode")
                .arg(claude_permission_mode(task.execution_mode.clone()))
                .arg("--add-dir")
                .arg(cwd);
            if let Some(allowed_tools) = std::env::var("VIBE_CLAUDE_ALLOWED_TOOLS")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
            {
                command.arg("--allowedTools").arg(allowed_tools);
            }
            if let Some(disallowed_tools) = claude_disallowed_tools(task.execution_mode.clone()) {
                command.arg("--disallowedTools").arg(disallowed_tools);
            }
            if let Some(model) = &task.model {
                command.arg("--model").arg(model);
            }
            command.arg(&prompt);
        }
        ProviderKind::OpenCode => {
            command.arg("run");
            if let Some(session_id) = task.provider_session_id.as_deref() {
                command.arg("--session").arg(session_id);
            } else {
                command.arg("--title").arg(&task.title);
            }
            if let Some(model) = &task.model {
                command.arg("--model").arg(model);
            }
            command.arg(&prompt);
        }
    }

    Ok(command)
}

fn codex_sandbox_mode(execution_mode: TaskExecutionMode) -> &'static str {
    match execution_mode {
        TaskExecutionMode::ReadOnly => "read-only",
        TaskExecutionMode::WorkspaceWrite | TaskExecutionMode::WorkspaceWriteAndTest => {
            "workspace-write"
        }
    }
}

fn codex_approval_policy(execution_mode: TaskExecutionMode) -> &'static str {
    match execution_mode {
        TaskExecutionMode::WorkspaceWrite => "untrusted",
        TaskExecutionMode::ReadOnly | TaskExecutionMode::WorkspaceWriteAndTest => "never",
    }
}

fn claude_permission_mode(execution_mode: TaskExecutionMode) -> String {
    let default_mode = std::env::var("VIBE_CLAUDE_PERMISSION_MODE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "acceptEdits".to_string());

    match execution_mode {
        TaskExecutionMode::ReadOnly => std::env::var("VIBE_CLAUDE_PERMISSION_MODE_READ_ONLY")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "plan".to_string()),
        TaskExecutionMode::WorkspaceWrite => std::env::var("VIBE_CLAUDE_PERMISSION_MODE_WRITE")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| default_mode.clone()),
        TaskExecutionMode::WorkspaceWriteAndTest => {
            std::env::var("VIBE_CLAUDE_PERMISSION_MODE_WRITE_AND_TEST")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(default_mode)
        }
    }
}

fn claude_disallowed_tools(execution_mode: TaskExecutionMode) -> Option<String> {
    match execution_mode {
        TaskExecutionMode::ReadOnly => std::env::var("VIBE_CLAUDE_DISALLOWED_TOOLS_READ_ONLY")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| Some(default_claude_read_only_disallowed_tools())),
        TaskExecutionMode::WorkspaceWrite => std::env::var("VIBE_CLAUDE_DISALLOWED_TOOLS_WRITE")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| Some(default_claude_test_disallowed_tools())),
        TaskExecutionMode::WorkspaceWriteAndTest => {
            std::env::var("VIBE_CLAUDE_DISALLOWED_TOOLS_WRITE_AND_TEST")
                .ok()
                .filter(|value| !value.trim().is_empty())
        }
    }
}

fn default_claude_read_only_disallowed_tools() -> String {
    ["Edit", "MultiEdit", "Write", "NotebookEdit", "Bash(*)"].join(",")
}

fn default_claude_test_disallowed_tools() -> String {
    [
        "Bash(cargo test:*)",
        "Bash(cargo nextest:*)",
        "Bash(npm test:*)",
        "Bash(pnpm test:*)",
        "Bash(yarn test:*)",
        "Bash(bun test:*)",
        "Bash(pytest:*)",
        "Bash(go test:*)",
        "Bash(gradle test:*)",
        "Bash(./gradlew test:*)",
        "Bash(mvn test:*)",
        "Bash(vitest:*)",
        "Bash(jest:*)",
        "Bash(playwright test:*)",
    ]
    .join(",")
}

fn render_task_prompt(task: &TaskRecord) -> String {
    let policy = match task.execution_mode {
        TaskExecutionMode::ReadOnly => {
            "Execution mode: read-only. Inspect, explain, and propose edits, but do not modify files, run destructive commands, or apply writes."
        }
        TaskExecutionMode::WorkspaceWrite => {
            "Execution mode: workspace write. You may edit files inside the current workspace, but avoid running test suites or broad verification commands unless the user explicitly asks."
        }
        TaskExecutionMode::WorkspaceWriteAndTest => {
            "Execution mode: workspace write and test. You may edit files inside the current workspace and run focused verification or test commands needed to validate the change."
        }
    };

    format!("{policy}\n\nUser request:\n{}", task.prompt)
}

pub(crate) fn detect_providers() -> Vec<ProviderStatus> {
    provider_definitions()
        .into_iter()
        .map(|definition| {
            detect_provider(
                definition.kind,
                definition.command_env,
                definition.default_command,
                definition.supports_acp,
            )
        })
        .collect()
}

pub(crate) fn acp_update_to_events(params: &Value) -> Vec<TaskEventInput> {
    let Some(update) = params.get("update") else {
        return vec![];
    };
    let Some(kind) = update.get("sessionUpdate").and_then(Value::as_str) else {
        return vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: compact_json(update),
        }];
    };

    match kind {
        "agent_message_chunk" => {
            text_events_from_content(update.get("content"), TaskEventKind::AssistantDelta)
        }
        "user_message_chunk" => {
            text_events_from_content(update.get("content"), TaskEventKind::System)
        }
        "agent_thought_chunk" | "thought_message_chunk" => {
            text_events_from_content(update.get("content"), TaskEventKind::Status)
        }
        "tool_call" => vec![TaskEventInput {
            kind: TaskEventKind::ToolCall,
            message: format_tool_call_message(update),
        }],
        "tool_call_update" => {
            let mut message = format_tool_call_message(update);
            let content = flatten_content_text(update.get("content"));
            if !content.is_empty() {
                message.push('\n');
                message.push_str(&content);
            }
            vec![TaskEventInput {
                kind: TaskEventKind::ToolOutput,
                message,
            }]
        }
        "plan" => vec![TaskEventInput {
            kind: TaskEventKind::Status,
            message: format_plan_update(update),
        }],
        "session_info_update" => vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format_session_info_update(update),
        }],
        "usage_update" => vec![TaskEventInput {
            kind: TaskEventKind::Status,
            message: format_usage_update(update),
        }],
        "available_commands_update" => vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format_available_commands_update(update),
        }],
        "config_option_update" => vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format_config_update(update),
        }],
        "current_mode_update" => vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format!(
                "ACP mode changed to {}",
                update
                    .get("currentModeId")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            ),
        }],
        _ => vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: compact_json(update),
        }],
    }
}

pub(crate) fn provider_stdout_to_task_events(
    provider_kind: &ProviderKind,
    line: &str,
) -> Vec<TaskEventInput> {
    match provider_kind {
        ProviderKind::Codex => codex_jsonl_to_task_events(line),
        ProviderKind::ClaudeCode => claude_jsonl_to_task_events(line),
        ProviderKind::OpenCode => vec![TaskEventInput {
            kind: TaskEventKind::ProviderStdout,
            message: line.to_string(),
        }],
    }
}

pub(crate) fn provider_stdout_session_id(
    provider_kind: &ProviderKind,
    line: &str,
) -> Option<String> {
    match provider_kind {
        ProviderKind::Codex => codex_jsonl_session_id(line),
        ProviderKind::ClaudeCode => claude_jsonl_session_id(line),
        ProviderKind::OpenCode => None,
    }
}

pub(crate) fn claude_jsonl_to_task_events(line: &str) -> Vec<TaskEventInput> {
    let message = match serde_json::from_str::<Value>(line) {
        Ok(value) => value,
        Err(_) => {
            return vec![TaskEventInput {
                kind: TaskEventKind::ProviderStdout,
                message: line.to_string(),
            }];
        }
    };

    let Some(event_type) = message.get("type").and_then(Value::as_str) else {
        return vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: compact_json(&message),
        }];
    };

    match event_type {
        "system" => claude_system_to_task_events(&message),
        "assistant" => claude_assistant_to_task_events(&message),
        "result" => claude_result_to_task_events(&message),
        "user" => claude_user_to_task_events(&message),
        _ => vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format!("Claude event {event_type}: {}", compact_json(&message)),
        }],
    }
}

fn claude_jsonl_session_id(line: &str) -> Option<String> {
    let message = serde_json::from_str::<Value>(line).ok()?;
    if message.get("type").and_then(Value::as_str) != Some("system") {
        return None;
    }
    if message.get("subtype").and_then(Value::as_str) != Some("init") {
        return None;
    }
    message
        .get("session_id")
        .and_then(Value::as_str)
        .map(str::to_string)
}

pub(crate) fn codex_jsonl_to_task_events(line: &str) -> Vec<TaskEventInput> {
    let message = match serde_json::from_str::<Value>(line) {
        Ok(value) => value,
        Err(_) => {
            return vec![TaskEventInput {
                kind: TaskEventKind::ProviderStdout,
                message: line.to_string(),
            }];
        }
    };

    let Some(event_type) = message.get("type").and_then(Value::as_str) else {
        return vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: compact_json(&message),
        }];
    };

    match event_type {
        "thread.started" => message
            .get("thread_id")
            .and_then(Value::as_str)
            .map(|thread_id| TaskEventInput {
                kind: TaskEventKind::System,
                message: format!("Codex session started: {thread_id}"),
            })
            .into_iter()
            .collect(),
        "turn.started" => Vec::new(),
        "turn.completed" => {
            let usage = message
                .get("usage")
                .map(compact_json)
                .unwrap_or_else(|| "{}".to_string());
            vec![TaskEventInput {
                kind: TaskEventKind::Status,
                message: format!("Codex turn completed with usage {usage}"),
            }]
        }
        "item.started" => message
            .get("item")
            .map(|item| codex_item_to_task_events(item, true))
            .unwrap_or_default(),
        "item.completed" => message
            .get("item")
            .map(|item| codex_item_to_task_events(item, false))
            .unwrap_or_default(),
        _ => vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format!("Codex event {event_type}: {}", compact_json(&message)),
        }],
    }
}

fn codex_jsonl_session_id(line: &str) -> Option<String> {
    let message = serde_json::from_str::<Value>(line).ok()?;
    if message.get("type").and_then(Value::as_str) != Some("thread.started") {
        return None;
    }
    message
        .get("thread_id")
        .and_then(Value::as_str)
        .map(str::to_string)
}

struct ProviderDefinition {
    kind: ProviderKind,
    command_env: &'static str,
    default_command: &'static str,
    supports_acp: bool,
}

fn detect_provider(
    kind: ProviderKind,
    command_env: &str,
    default_command: &str,
    supports_acp: bool,
) -> ProviderStatus {
    let command = std::env::var(command_env).unwrap_or_else(|_| default_command.to_string());
    let execution_protocol = advertised_execution_protocol(kind.clone(), supports_acp);

    match which::which(&command) {
        Ok(path) => {
            let version = std::process::Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .map(|output| {
                    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if !stdout.is_empty() { stdout } else { stderr }
                })
                .filter(|value| !value.is_empty());

            ProviderStatus {
                kind,
                command: path.to_string_lossy().to_string(),
                available: true,
                version,
                execution_protocol,
                supports_acp,
                error: None,
            }
        }
        Err(error) => ProviderStatus {
            kind,
            command,
            available: false,
            version: None,
            execution_protocol,
            supports_acp,
            error: Some(error.to_string()),
        },
    }
}

fn advertised_execution_protocol(kind: ProviderKind, supports_acp: bool) -> ExecutionProtocol {
    if supports_acp && matches!(kind, ProviderKind::OpenCode) {
        ExecutionProtocol::Acp
    } else {
        ExecutionProtocol::Cli
    }
}

fn provider_definitions() -> Vec<ProviderDefinition> {
    vec![
        ProviderDefinition {
            kind: ProviderKind::Codex,
            command_env: "VIBE_CODEX_COMMAND",
            default_command: "codex",
            supports_acp: false,
        },
        ProviderDefinition {
            kind: ProviderKind::ClaudeCode,
            command_env: "VIBE_CLAUDE_COMMAND",
            default_command: "claude",
            supports_acp: false,
        },
        ProviderDefinition {
            kind: ProviderKind::OpenCode,
            command_env: "VIBE_OPENCODE_COMMAND",
            default_command: "opencode",
            supports_acp: true,
        },
    ]
}

fn text_events_from_content(content: Option<&Value>, kind: TaskEventKind) -> Vec<TaskEventInput> {
    let text = flatten_content_text(content);
    if text.is_empty() {
        return vec![];
    }
    vec![TaskEventInput {
        kind,
        message: text,
    }]
}

fn flatten_content_text(content: Option<&Value>) -> String {
    let Some(content) = content else {
        return String::new();
    };

    match content {
        Value::String(text) => text.to_string(),
        Value::Array(items) => items
            .iter()
            .map(flatten_content_block)
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(_) => flatten_content_block(content),
        _ => String::new(),
    }
}

fn flatten_content_block(block: &Value) -> String {
    if let Value::String(text) = block {
        return text.to_string();
    }

    let block_type = block
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match block_type {
        "text" => block
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        "resource" => block
            .get("resource")
            .and_then(|resource| resource.get("text"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        "resource_link" => block
            .get("uri")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        "diff" => {
            let path = block
                .get("path")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            format!("diff {path}")
        }
        "terminal" => {
            let terminal_id = block
                .get("terminalId")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            format!("terminal {terminal_id}")
        }
        _ => compact_json(block),
    }
}

fn format_tool_call_message(update: &Value) -> String {
    let title = update
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("tool call");
    let status = update
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let kind = update.get("kind").and_then(Value::as_str).unwrap_or("tool");
    let tool_call_id = update
        .get("toolCallId")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    format!("{title} [{kind}] status={status} id={tool_call_id}")
}

fn format_plan_update(update: &Value) -> String {
    if let Some(steps) = update.get("steps").and_then(Value::as_array) {
        let rendered = steps
            .iter()
            .filter_map(|step| {
                let title = step.get("title").and_then(Value::as_str)?;
                let status = step
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("pending");
                Some(format!("[{status}] {title}"))
            })
            .collect::<Vec<_>>();
        if !rendered.is_empty() {
            return format!("ACP plan\n{}", rendered.join("\n"));
        }
    }
    compact_json(update)
}

fn format_available_commands_update(update: &Value) -> String {
    let commands = update
        .get("availableCommands")
        .or_else(|| update.get("available_commands"))
        .and_then(Value::as_array);
    let Some(commands) = commands else {
        return compact_json(update);
    };

    let rendered = commands
        .iter()
        .filter_map(|command| {
            let name = command.get("name").and_then(Value::as_str)?.trim();
            if name.is_empty() {
                return None;
            }
            let description = command
                .get("description")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default();
            Some(if description.is_empty() {
                name.to_string()
            } else {
                format!("{name}: {description}")
            })
        })
        .collect::<Vec<_>>();

    if rendered.is_empty() {
        compact_json(update)
    } else {
        format!("ACP available commands: {}", rendered.join(", "))
    }
}

fn format_session_info_update(update: &Value) -> String {
    let mut parts = Vec::new();

    if let Some(title) = update.get("title") {
        match title {
            Value::Null => parts.push("title cleared".to_string()),
            Value::String(value) if !value.trim().is_empty() => {
                parts.push(format!("title={}", value.trim()))
            }
            _ => {}
        }
    }

    if let Some(updated_at) = update.get("updatedAt") {
        match updated_at {
            Value::Null => parts.push("updated_at cleared".to_string()),
            Value::String(value) if !value.trim().is_empty() => {
                parts.push(format!("updated_at={}", value.trim()))
            }
            _ => {}
        }
    }

    if parts.is_empty() {
        compact_json(update)
    } else {
        format!("ACP session info updated: {}", parts.join(", "))
    }
}

fn format_usage_update(update: &Value) -> String {
    let (Some(used), Some(size)) = (
        update.get("used").and_then(Value::as_u64),
        update.get("size").and_then(Value::as_u64),
    ) else {
        return compact_json(update);
    };

    let mut message = format!("ACP session usage: {used}/{size} tokens in context");
    if let Some(cost) = update.get("cost").and_then(Value::as_object) {
        if let (Some(amount), Some(currency)) = (
            cost.get("amount").and_then(Value::as_f64),
            cost.get("currency").and_then(Value::as_str),
        ) {
            message.push_str(&format!(" cost={amount:.4} {currency}"));
        }
    }
    message
}

fn format_config_update(update: &Value) -> String {
    if let Some(config_options) = update.get("configOptions").and_then(Value::as_array) {
        let rendered = config_options
            .iter()
            .filter_map(|config| {
                let config_id = config.get("id").and_then(Value::as_str)?;
                let current = config
                    .get("currentValue")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                Some(format!("{config_id}={current}"))
            })
            .collect::<Vec<_>>();
        if !rendered.is_empty() {
            return format!("ACP config updated: {}", rendered.join(", "));
        }
    }
    compact_json(update)
}

fn claude_system_to_task_events(message: &Value) -> Vec<TaskEventInput> {
    let subtype = message
        .get("subtype")
        .and_then(Value::as_str)
        .unwrap_or("event");

    if subtype == "init" {
        let session_id = message
            .get("session_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let model = message
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let permission_mode = message
            .get("permissionMode")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        return vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format!(
                "Claude session started: {session_id} model={model} permission_mode={permission_mode}"
            ),
        }];
    }

    vec![TaskEventInput {
        kind: TaskEventKind::System,
        message: format!("Claude {subtype}: {}", compact_json(message)),
    }]
}

fn claude_assistant_to_task_events(message: &Value) -> Vec<TaskEventInput> {
    let mut events = Vec::new();

    if let Some(content) = message
        .get("message")
        .and_then(|value| value.get("content"))
        .and_then(Value::as_array)
    {
        for block in content {
            events.extend(claude_content_block_to_events(block));
        }
    }

    if let Some(error) = message.get("error").and_then(Value::as_str) {
        events.push(TaskEventInput {
            kind: TaskEventKind::System,
            message: format!("Claude error: {error}"),
        });
    }

    if events.is_empty() {
        events.push(TaskEventInput {
            kind: TaskEventKind::System,
            message: format!("Claude assistant event: {}", compact_json(message)),
        });
    }

    events
}

fn claude_content_block_to_events(block: &Value) -> Vec<TaskEventInput> {
    let block_type = block
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown");

    match block_type {
        "text" => block
            .get("text")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|message| {
                vec![TaskEventInput {
                    kind: TaskEventKind::AssistantDelta,
                    message: message.to_string(),
                }]
            })
            .unwrap_or_default(),
        "thinking" | "redacted_thinking" => {
            let content = block
                .get("thinking")
                .and_then(Value::as_str)
                .or_else(|| block.get("text").and_then(Value::as_str))
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| format!("Claude {block_type}"));
            vec![TaskEventInput {
                kind: TaskEventKind::Status,
                message: content,
            }]
        }
        "tool_use" => {
            let name = block.get("name").and_then(Value::as_str).unwrap_or("tool");
            let tool_use_id = block.get("id").and_then(Value::as_str).unwrap_or("unknown");
            let mut message = format!("Claude tool use {name} id={tool_use_id}");
            if let Some(input) = block.get("input") {
                message.push_str(" input=");
                message.push_str(&compact_json(input));
            }
            vec![TaskEventInput {
                kind: TaskEventKind::ToolCall,
                message,
            }]
        }
        "tool_result" => {
            let tool_use_id = block
                .get("tool_use_id")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let mut message = format!("Claude tool result id={tool_use_id}");
            if block.get("is_error").and_then(Value::as_bool) == Some(true) {
                message.push_str(" error=true");
            }
            let content = flatten_content_text(block.get("content"));
            if !content.is_empty() {
                message.push('\n');
                message.push_str(&content);
            }
            vec![TaskEventInput {
                kind: TaskEventKind::ToolOutput,
                message,
            }]
        }
        _ if block_type.contains("tool") => vec![TaskEventInput {
            kind: TaskEventKind::ToolOutput,
            message: format!("Claude {block_type}: {}", compact_json(block)),
        }],
        _ => vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format!("Claude content {block_type}: {}", compact_json(block)),
        }],
    }
}

fn claude_result_to_task_events(message: &Value) -> Vec<TaskEventInput> {
    let subtype = message
        .get("subtype")
        .and_then(Value::as_str)
        .unwrap_or("result");
    let is_error = message
        .get("is_error")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let stop_reason = message
        .get("stop_reason")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let num_turns = message
        .get("num_turns")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let duration_ms = message
        .get("duration_ms")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let total_cost_usd = message
        .get("total_cost_usd")
        .map(compact_json)
        .unwrap_or_else(|| "null".to_string());

    let mut summary = format!(
        "Claude {subtype} stop_reason={stop_reason} turns={num_turns} duration_ms={duration_ms} cost_usd={total_cost_usd}"
    );
    if is_error {
        if let Some(result) = message
            .get("result")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            summary.push_str(" message=");
            summary.push_str(result);
        }
    }

    let mut events = vec![TaskEventInput {
        kind: if is_error {
            TaskEventKind::System
        } else {
            TaskEventKind::Status
        },
        message: summary,
    }];

    if let Some(permission_denials) = message
        .get("permission_denials")
        .and_then(Value::as_array)
        .filter(|items| !items.is_empty())
    {
        events.push(TaskEventInput {
            kind: TaskEventKind::System,
            message: format!(
                "Claude permission denials: {}",
                compact_json(&Value::Array(permission_denials.clone()))
            ),
        });
    }

    events
}

fn claude_user_to_task_events(message: &Value) -> Vec<TaskEventInput> {
    let Some(content) = message
        .get("message")
        .and_then(|value| value.get("content"))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };

    let text = content
        .iter()
        .map(flatten_content_block)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if text.is_empty() {
        Vec::new()
    } else {
        vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: format!("Claude user message: {text}"),
        }]
    }
}

fn codex_item_to_task_events(item: &Value, started: bool) -> Vec<TaskEventInput> {
    let item_type = item
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown_item");

    if item_type == "agent_message" {
        let text = item
            .get("text")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        return text
            .map(|message| {
                vec![TaskEventInput {
                    kind: TaskEventKind::AssistantDelta,
                    message,
                }]
            })
            .unwrap_or_default();
    }

    if item_type.contains("reason") || item_type.contains("thought") {
        let message = codex_item_summary(item_type, item, started);
        return vec![TaskEventInput {
            kind: TaskEventKind::Status,
            message,
        }];
    }

    if item_type.contains("tool")
        || item_type.contains("command")
        || item_type.contains("patch")
        || item_type.contains("shell")
    {
        let kind = if started {
            TaskEventKind::ToolCall
        } else {
            TaskEventKind::ToolOutput
        };
        let message = codex_item_summary(item_type, item, started);
        return vec![TaskEventInput { kind, message }];
    }

    let message = codex_item_summary(item_type, item, started);
    vec![TaskEventInput {
        kind: TaskEventKind::System,
        message,
    }]
}

fn codex_item_summary(item_type: &str, item: &Value, started: bool) -> String {
    let phase = if started { "started" } else { "completed" };
    let mut details = Vec::new();

    for key in ["title", "name", "command", "text", "call_id", "status"] {
        if let Some(value) = item.get(key).and_then(Value::as_str) {
            let value = value.trim();
            if !value.is_empty() {
                details.push(format!("{key}={value}"));
            }
        }
    }

    if details.is_empty() {
        for key in ["arguments", "output", "metadata"] {
            if let Some(value) = item.get(key) {
                details.push(format!("{key}={}", compact_json(value)));
            }
        }
    }

    if details.is_empty() {
        format!("Codex {item_type} {phase}")
    } else {
        format!("Codex {item_type} {phase}: {}", details.join(" "))
    }
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;
    use vibe_core::TaskTransportKind;

    fn test_task(provider: ProviderKind, execution_mode: TaskExecutionMode) -> TaskRecord {
        TaskRecord {
            tenant_id: "personal".to_string(),
            user_id: "local-admin".to_string(),
            id: "task-1".to_string(),
            device_id: "device-1".to_string(),
            conversation_id: Some("conversation-1".to_string()),
            title: "Test task".to_string(),
            provider,
            execution_protocol: ExecutionProtocol::Cli,
            execution_mode,
            prompt: "hello".to_string(),
            cwd: Some("/tmp/project".to_string()),
            model: None,
            provider_session_id: None,
            pending_input_request_id: None,
            status: vibe_core::TaskStatus::Pending,
            cancel_requested: false,
            created_at_epoch_ms: 0,
            started_at_epoch_ms: None,
            finished_at_epoch_ms: None,
            exit_code: None,
            error: None,
            last_event_seq: 0,
            transport: TaskTransportKind::RelayPolling,
        }
    }

    fn test_provider(kind: ProviderKind) -> ProviderStatus {
        ProviderStatus {
            kind: kind.clone(),
            command: match kind {
                ProviderKind::Codex => "codex".to_string(),
                ProviderKind::ClaudeCode => "claude".to_string(),
                ProviderKind::OpenCode => "opencode".to_string(),
            },
            available: true,
            version: None,
            execution_protocol: if kind == ProviderKind::OpenCode {
                ExecutionProtocol::Acp
            } else {
                ExecutionProtocol::Cli
            },
            supports_acp: kind == ProviderKind::OpenCode,
            error: None,
        }
    }

    #[test]
    fn codex_is_no_longer_advertised_as_acp() {
        let codex = provider_definitions()
            .into_iter()
            .find(|definition| definition.kind == ProviderKind::Codex)
            .expect("codex provider definition");

        assert!(!codex.supports_acp);
        assert_eq!(
            advertised_execution_protocol(codex.kind, codex.supports_acp),
            ExecutionProtocol::Cli
        );
    }

    #[test]
    fn opencode_remains_advertised_as_acp() {
        let opencode = provider_definitions()
            .into_iter()
            .find(|definition| definition.kind == ProviderKind::OpenCode)
            .expect("opencode provider definition");

        assert!(opencode.supports_acp);
        assert_eq!(
            advertised_execution_protocol(opencode.kind, opencode.supports_acp),
            ExecutionProtocol::Acp
        );
    }

    #[test]
    fn stable_agent_thought_chunk_maps_to_status_event() {
        let events = acp_update_to_events(&json!({
            "update": {
                "sessionUpdate": "agent_thought_chunk",
                "content": [{"type": "text", "text": "thinking"}]
            }
        }));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::Status);
        assert_eq!(events[0].message, "thinking");
    }

    #[test]
    fn available_commands_update_maps_to_system_event() {
        let events = acp_update_to_events(&json!({
            "update": {
                "sessionUpdate": "available_commands_update",
                "availableCommands": [
                    {
                        "name": "create_plan",
                        "description": "Draft a plan for the task"
                    },
                    {
                        "name": "fix_tests",
                        "description": "Repair failing tests"
                    }
                ]
            }
        }));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::System);
        assert!(events[0].message.contains("create_plan"));
        assert!(events[0].message.contains("fix_tests"));
    }

    #[test]
    fn session_info_update_maps_to_system_event() {
        let events = acp_update_to_events(&json!({
            "update": {
                "sessionUpdate": "session_info_update",
                "title": "Refactor auth",
                "updatedAt": "2026-03-29T10:11:12Z"
            }
        }));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::System);
        assert!(events[0].message.contains("Refactor auth"));
        assert!(events[0].message.contains("2026-03-29T10:11:12Z"));
    }

    #[test]
    fn usage_update_maps_to_status_event() {
        let events = acp_update_to_events(&json!({
            "update": {
                "sessionUpdate": "usage_update",
                "used": 4096,
                "size": 128000,
                "cost": {
                    "amount": 1.23,
                    "currency": "USD"
                }
            }
        }));

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::Status);
        assert!(events[0].message.contains("4096/128000"));
        assert!(events[0].message.contains("USD"));
    }

    #[test]
    fn codex_cli_uses_read_only_sandbox_for_read_only_mode() {
        let provider = test_provider(ProviderKind::Codex);
        let task = test_task(ProviderKind::Codex, TaskExecutionMode::ReadOnly);
        let command = build_provider_command(&provider, &task, &PathBuf::from("/tmp/project"))
            .expect("codex command");
        let args = command
            .as_std()
            .get_args()
            .map(|value| value.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(
            args.windows(2)
                .any(|pair| pair == ["--sandbox", "read-only"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--ask-for-approval", "never"])
        );
    }

    #[test]
    fn codex_cli_uses_untrusted_approval_for_workspace_write_mode() {
        let provider = test_provider(ProviderKind::Codex);
        let task = test_task(ProviderKind::Codex, TaskExecutionMode::WorkspaceWrite);
        let command = build_provider_command(&provider, &task, &PathBuf::from("/tmp/project"))
            .expect("codex command");
        let args = command
            .as_std()
            .get_args()
            .map(|value| value.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(
            args.windows(2)
                .any(|pair| pair == ["--ask-for-approval", "untrusted"])
        );
    }

    #[test]
    fn codex_cli_uses_workspace_write_sandbox_and_never_approval_for_write_and_test() {
        let provider = test_provider(ProviderKind::Codex);
        let task = test_task(
            ProviderKind::Codex,
            TaskExecutionMode::WorkspaceWriteAndTest,
        );
        let command = build_provider_command(&provider, &task, &PathBuf::from("/tmp/project"))
            .expect("codex command");
        let args = command
            .as_std()
            .get_args()
            .map(|value| value.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(
            args.windows(2)
                .any(|pair| pair == ["--sandbox", "workspace-write"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--ask-for-approval", "never"])
        );
    }

    #[test]
    fn claude_cli_uses_plan_mode_for_read_only() {
        let provider = test_provider(ProviderKind::ClaudeCode);
        let task = test_task(ProviderKind::ClaudeCode, TaskExecutionMode::ReadOnly);
        let command = build_provider_command(&provider, &task, &PathBuf::from("/tmp/project"))
            .expect("claude command");
        let args = command
            .as_std()
            .get_args()
            .map(|value| value.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(
            args.windows(2)
                .any(|pair| pair == ["--permission-mode", "plan"])
        );
    }

    #[test]
    fn claude_read_only_disallows_edit_and_bash_tools() {
        let provider = test_provider(ProviderKind::ClaudeCode);
        let task = test_task(ProviderKind::ClaudeCode, TaskExecutionMode::ReadOnly);
        let command = build_provider_command(&provider, &task, &PathBuf::from("/tmp/project"))
            .expect("claude command");
        let args = command
            .as_std()
            .get_args()
            .map(|value| value.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        let disallowed = args
            .windows(2)
            .find(|pair| pair[0] == "--disallowedTools")
            .map(|pair| pair[1].clone())
            .expect("disallowed tools flag");

        assert!(disallowed.contains("Edit"));
        assert!(disallowed.contains("MultiEdit"));
        assert!(disallowed.contains("Write"));
        assert!(disallowed.contains("Bash(*)"));
    }

    #[test]
    fn claude_workspace_write_disallows_test_bash_commands() {
        let provider = test_provider(ProviderKind::ClaudeCode);
        let task = test_task(ProviderKind::ClaudeCode, TaskExecutionMode::WorkspaceWrite);
        let command = build_provider_command(&provider, &task, &PathBuf::from("/tmp/project"))
            .expect("claude command");
        let args = command
            .as_std()
            .get_args()
            .map(|value| value.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        let disallowed = args
            .windows(2)
            .find(|pair| pair[0] == "--disallowedTools")
            .map(|pair| pair[1].clone())
            .expect("disallowed tools flag");

        assert!(disallowed.contains("Bash(cargo test:*)"));
        assert!(disallowed.contains("Bash(pytest:*)"));
        assert!(disallowed.contains("Bash(playwright test:*)"));
    }

    #[test]
    fn claude_write_and_test_has_no_default_disallowed_tools() {
        let provider = test_provider(ProviderKind::ClaudeCode);
        let task = test_task(
            ProviderKind::ClaudeCode,
            TaskExecutionMode::WorkspaceWriteAndTest,
        );
        let command = build_provider_command(&provider, &task, &PathBuf::from("/tmp/project"))
            .expect("claude command");
        let args = command
            .as_std()
            .get_args()
            .map(|value| value.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        assert!(!args.windows(2).any(|pair| pair[0] == "--disallowedTools"));
    }
}
