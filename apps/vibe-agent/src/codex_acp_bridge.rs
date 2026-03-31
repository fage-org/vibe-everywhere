use super::*;
use crate::providers::{build_codex_command, provider_stdout_session_id};

#[derive(Clone)]
struct CodexBridgeConfig {
    codex_command: String,
    cwd: PathBuf,
    execution_mode: TaskExecutionMode,
    initial_model: Option<String>,
}

#[derive(Clone)]
struct CodexBridgeSession {
    session_id: String,
    codex_session_id: Option<String>,
    cwd: PathBuf,
    execution_mode: TaskExecutionMode,
    model: Option<String>,
}

#[derive(Default)]
struct CodexBridgeState {
    sessions: HashMap<String, CodexBridgeSession>,
    aliases: HashMap<String, String>,
    active_runs: HashMap<String, Arc<Mutex<Child>>>,
}

pub(crate) async fn run_stdio_bridge(
    codex_command: String,
    cwd: PathBuf,
    execution_mode: TaskExecutionMode,
    model: Option<String>,
) -> Result<()> {
    let config = CodexBridgeConfig {
        codex_command,
        cwd,
        execution_mode,
        initial_model: model,
    };
    let state = Arc::new(Mutex::new(CodexBridgeState::default()));
    let stdout = Arc::new(Mutex::new(tokio::io::stdout()));
    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    while let Some(line) = lines
        .next_line()
        .await
        .context("failed to read Codex ACP bridge input")?
    {
        let message: Value = serde_json::from_str(&line)
            .with_context(|| format!("invalid bridge message: {line}"))?;
        let method = message
            .get("method")
            .and_then(Value::as_str)
            .map(str::to_string);
        let id = message.get("id").cloned();
        let params = message.get("params").cloned().unwrap_or_else(|| json!({}));

        match (id, method.as_deref()) {
            (Some(id), Some("initialize")) => {
                write_bridge_message(
                    &stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "protocolVersion": ACP_PROTOCOL_VERSION,
                            "agentInfo": {
                                "name": "vibe-agent-codex-bridge",
                                "version": env!("CARGO_PKG_VERSION")
                            },
                            "agentCapabilities": {
                                "sessionCapabilities": {
                                    "list": {},
                                    "resume": {},
                                }
                            }
                        }
                    }),
                )
                .await?;
            }
            (Some(id), Some("session/new")) => {
                let session = CodexBridgeSession {
                    session_id: Uuid::new_v4().to_string(),
                    codex_session_id: None,
                    cwd: config.cwd.clone(),
                    execution_mode: config.execution_mode.clone(),
                    model: config.initial_model.clone(),
                };
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": session_result(&session),
                });
                let session_id = session.session_id.clone();
                state.lock().await.sessions.insert(session_id, session);
                write_bridge_message(&stdout, &response).await?;
            }
            (Some(id), Some("session/resume")) => {
                let session_id = params
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .context("session/resume missing sessionId")?
                    .to_string();
                let resolved = resolve_session_id(&state, &session_id).await;
                let session = CodexBridgeSession {
                    session_id: resolved.clone(),
                    codex_session_id: Some(resolved.clone()),
                    cwd: config.cwd.clone(),
                    execution_mode: config.execution_mode.clone(),
                    model: config.initial_model.clone(),
                };
                state
                    .lock()
                    .await
                    .sessions
                    .insert(resolved.clone(), session.clone());
                write_bridge_message(
                    &stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": session_result(&session),
                    }),
                )
                .await?;
            }
            (Some(id), Some("session/list")) => {
                let sessions = list_sessions(&state).await;
                write_bridge_message(
                    &stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "sessions": sessions,
                        }
                    }),
                )
                .await?;
            }
            (Some(id), Some("session/set_config_option")) => {
                let session_id = params
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .context("session/set_config_option missing sessionId")?
                    .to_string();
                let config_id = params
                    .get("configId")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let value = params
                    .get("value")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let resolved = resolve_session_id(&state, &session_id).await;
                if config_id == "model"
                    && let Some(value) = value.clone()
                    && let Some(session) = state.lock().await.sessions.get_mut(&resolved)
                {
                    session.model = Some(value);
                }
                write_bridge_message(
                    &stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "sessionId": resolved,
                            "configId": config_id,
                            "value": value,
                        }
                    }),
                )
                .await?;
                if config_id == "model" {
                    write_bridge_message(
                        &stdout,
                        &json!({
                            "jsonrpc": "2.0",
                            "method": "session/update",
                            "params": {
                                "sessionId": resolved,
                                "update": {
                                    "sessionUpdate": "config_option_update",
                                    "configOptions": [
                                        {
                                            "id": "model",
                                            "category": "model",
                                            "currentValue": value,
                                        }
                                    ]
                                }
                            }
                        }),
                    )
                    .await?;
                }
            }
            (Some(id), Some("session/prompt")) => {
                let session_id = params
                    .get("sessionId")
                    .and_then(Value::as_str)
                    .context("session/prompt missing sessionId")?
                    .to_string();
                let prompt =
                    flatten_prompt_text(&params).context("session/prompt missing prompt text")?;
                let state = state.clone();
                let stdout = stdout.clone();
                let config = config.clone();
                tokio::spawn(async move {
                    let response = run_prompt(&state, &stdout, &config, &session_id, &prompt).await;
                    match response {
                        Ok(result) => {
                            let _ = write_bridge_message(
                                &stdout,
                                &json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "result": result,
                                }),
                            )
                            .await;
                        }
                        Err(error) => {
                            let _ = write_bridge_message(
                                &stdout,
                                &json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "error": {
                                        "code": -32000,
                                        "message": error.to_string(),
                                    }
                                }),
                            )
                            .await;
                        }
                    }
                });
            }
            (None, Some("session/cancel")) => {
                if let Some(session_id) = params.get("sessionId").and_then(Value::as_str) {
                    let resolved = resolve_session_id(&state, session_id).await;
                    if let Some(child) = state.lock().await.active_runs.get(&resolved).cloned() {
                        let _ = child.lock().await.kill().await;
                    }
                }
            }
            (Some(id), Some(method)) => {
                write_bridge_message(
                    &stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": format!("unsupported Codex ACP bridge method: {method}"),
                        }
                    }),
                )
                .await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn run_prompt(
    state: &Arc<Mutex<CodexBridgeState>>,
    stdout: &Arc<Mutex<tokio::io::Stdout>>,
    config: &CodexBridgeConfig,
    session_id: &str,
    prompt: &str,
) -> Result<Value> {
    let resolved = resolve_session_id(state, session_id).await;
    let session = state
        .lock()
        .await
        .sessions
        .get(&resolved)
        .cloned()
        .with_context(|| format!("unknown bridge session: {session_id}"))?;

    let mut command = build_codex_command(
        &config.codex_command,
        session.codex_session_id.as_deref(),
        prompt,
        &session.cwd,
        session.model.as_deref(),
        session.execution_mode.clone(),
    );
    command
        .current_dir(&session.cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", config.codex_command))?;
    let stdout_lines = child
        .stdout
        .take()
        .map(|handle| BufReader::new(handle).lines())
        .context("Codex bridge missing stdout")?;
    let stderr_lines = child
        .stderr
        .take()
        .map(|handle| BufReader::new(handle).lines())
        .context("Codex bridge missing stderr")?;
    let child = Arc::new(Mutex::new(child));
    {
        state
            .lock()
            .await
            .active_runs
            .insert(resolved.clone(), child.clone());
    }

    let mut stdout_lines = Some(stdout_lines);
    let mut stderr_lines = Some(stderr_lines);
    let mut final_usage: Option<Value> = None;
    let mut stderr_buffer = String::new();

    loop {
        tokio::select! {
            result = next_line(&mut stdout_lines), if stdout_lines.is_some() => {
                match result? {
                    Some(line) => {
                        if let Some(canonical_session_id) = provider_stdout_session_id(&ProviderKind::Codex, &line) {
                            promote_session_id(state, &resolved, canonical_session_id.clone()).await;
                            write_bridge_message(
                                stdout,
                                &json!({
                                    "jsonrpc": "2.0",
                                    "method": "session/update",
                                    "params": {
                                        "sessionId": canonical_session_id,
                                        "update": {
                                            "sessionUpdate": "session_info_update",
                                            "sessionId": canonical_session_id,
                                        }
                                    }
                                }),
                            ).await?;
                        }
                        if let Ok(message) = serde_json::from_str::<Value>(&line)
                            && message.get("type").and_then(Value::as_str) == Some("turn.completed")
                            && let Some(usage) = message.get("usage").cloned()
                        {
                            final_usage = Some(usage);
                        }
                        for update in codex_line_to_acp_updates(&line) {
                            let active_session_id = current_session_id(state, &resolved).await;
                            write_bridge_message(
                                stdout,
                                &json!({
                                    "jsonrpc": "2.0",
                                    "method": "session/update",
                                    "params": {
                                        "sessionId": active_session_id,
                                        "update": update,
                                    }
                                }),
                            ).await?;
                        }
                    }
                    None => stdout_lines = None,
                }
            }
            result = next_line(&mut stderr_lines), if stderr_lines.is_some() => {
                match result? {
                    Some(line) => {
                        if !stderr_buffer.is_empty() {
                            stderr_buffer.push('\n');
                        }
                        stderr_buffer.push_str(&line);
                    }
                    None => stderr_lines = None,
                }
            }
            status = async {
                let mut guard = child.lock().await;
                guard.wait().await
            }, if stdout_lines.is_none() && stderr_lines.is_none() => {
                let status = status.context("failed waiting for Codex bridge child")?;
                state.lock().await.active_runs.remove(&resolved);
                if status.success() {
                    return Ok(json!({
                        "stopReason": "end_turn",
                        "usage": final_usage,
                    }));
                }
                if status.code().is_none() || matches!(status.code(), Some(130 | 143)) {
                    return Ok(json!({
                        "stopReason": "cancelled",
                        "usage": final_usage,
                    }));
                }
                let stderr_message = stderr_buffer.trim();
                let stderr_message = if stderr_message.is_empty() {
                    format!("codex exited with status {:?}", status.code())
                } else {
                    stderr_message.to_string()
                };
                bail!(stderr_message);
            }
        }
    }
}

fn session_result(session: &CodexBridgeSession) -> Value {
    let mut result = json!({
        "sessionId": session.session_id,
    });
    if let Some(model) = session.model.as_deref() {
        result["configOptions"] = json!([
            {
                "id": "model",
                "category": "model",
                "currentValue": model,
                "options": [
                    {
                        "value": model,
                        "name": model
                    }
                ]
            }
        ]);
    }
    result
}

fn listed_session(session: &CodexBridgeSession) -> Value {
    let mut result = json!({
        "sessionId": session.session_id,
    });
    if let Some(model) = session.model.as_deref() {
        result["title"] = Value::String(format!("Codex ({model})"));
    }
    result
}

fn flatten_prompt_text(params: &Value) -> Result<String> {
    let prompt = params
        .get("prompt")
        .and_then(Value::as_array)
        .context("missing prompt blocks")?;
    let text = prompt
        .iter()
        .filter_map(|block| {
            if block.get("type").and_then(Value::as_str) == Some("text") {
                return block
                    .get("text")
                    .and_then(Value::as_str)
                    .map(str::to_string);
            }
            None
        })
        .collect::<Vec<_>>()
        .join("\n");
    if text.trim().is_empty() {
        bail!("prompt did not contain text content");
    }
    Ok(text)
}

fn codex_line_to_acp_updates(line: &str) -> Vec<Value> {
    let Ok(message) = serde_json::from_str::<Value>(line) else {
        return vec![text_update("user_message_chunk", line)];
    };

    let Some(event_type) = message.get("type").and_then(Value::as_str) else {
        return vec![text_update("user_message_chunk", &compact_json(&message))];
    };

    match event_type {
        "thread.started" => message
            .get("thread_id")
            .and_then(Value::as_str)
            .map(|thread_id| {
                vec![json!({
                    "sessionUpdate": "session_info_update",
                    "sessionId": thread_id,
                })]
            })
            .unwrap_or_default(),
        "turn.completed" => message
            .get("usage")
            .map(|usage| {
                vec![json!({
                    "sessionUpdate": "usage_update",
                    "used": usage.get("total_tokens").or_else(|| usage.get("totalTokens")).and_then(Value::as_u64).unwrap_or(0),
                    "size": usage.get("input_tokens").or_else(|| usage.get("inputTokens")).and_then(Value::as_u64).unwrap_or(0)
                        + usage.get("output_tokens").or_else(|| usage.get("outputTokens")).and_then(Value::as_u64).unwrap_or(0),
                })]
            })
            .unwrap_or_default(),
        "item.started" => message
            .get("item")
            .map(|item| codex_item_to_acp_updates(item, true))
            .unwrap_or_default(),
        "item.completed" => message
            .get("item")
            .map(|item| codex_item_to_acp_updates(item, false))
            .unwrap_or_default(),
        _ => vec![text_update(
            "user_message_chunk",
            &format!("Codex event {event_type}: {}", compact_json(&message)),
        )],
    }
}

fn codex_item_to_acp_updates(item: &Value, started: bool) -> Vec<Value> {
    let item_type = item
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("unknown_item");

    if item_type == "agent_message" {
        if let Some(text) = item.get("text").and_then(Value::as_str).map(str::trim)
            && !text.is_empty()
        {
            return vec![text_update("agent_message_chunk", text)];
        }
        return Vec::new();
    }

    if item_type.contains("reason") || item_type.contains("thought") {
        return vec![text_update(
            "agent_thought_chunk",
            &codex_item_summary(item_type, item, started),
        )];
    }

    if item_type.contains("tool")
        || item_type.contains("command")
        || item_type.contains("patch")
        || item_type.contains("shell")
    {
        let tool_call_id = item
            .get("call_id")
            .or_else(|| item.get("id"))
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("codex-tool-{}", Uuid::new_v4()));
        let title = item
            .get("title")
            .or_else(|| item.get("name"))
            .or_else(|| item.get("command"))
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or(item_type)
            .to_string();
        let status = if started { "in_progress" } else { "completed" };
        if started {
            return vec![json!({
                "sessionUpdate": "tool_call",
                "title": title,
                "kind": "codex",
                "status": status,
                "toolCallId": tool_call_id,
            })];
        }
        let content = item
            .get("output")
            .map(compact_json)
            .or_else(|| item.get("text").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_else(|| codex_item_summary(item_type, item, started));
        return vec![json!({
            "sessionUpdate": "tool_call_update",
            "title": title,
            "kind": "codex",
            "status": status,
            "toolCallId": tool_call_id,
            "content": [{"type": "text", "text": content}],
        })];
    }
    vec![text_update(
        "user_message_chunk",
        &codex_item_summary(item_type, item, started),
    )]
}

fn text_update(kind: &str, text: &str) -> Value {
    json!({
        "sessionUpdate": kind,
        "content": [{"type": "text", "text": text}],
    })
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

async fn resolve_session_id(state: &Arc<Mutex<CodexBridgeState>>, session_id: &str) -> String {
    let guard = state.lock().await;
    guard
        .aliases
        .get(session_id)
        .cloned()
        .unwrap_or_else(|| session_id.to_string())
}

async fn current_session_id(state: &Arc<Mutex<CodexBridgeState>>, session_id: &str) -> String {
    let guard = state.lock().await;
    if let Some(session) = guard.sessions.get(session_id) {
        return session.session_id.clone();
    }
    guard
        .aliases
        .get(session_id)
        .cloned()
        .unwrap_or_else(|| session_id.to_string())
}

async fn list_sessions(state: &Arc<Mutex<CodexBridgeState>>) -> Vec<Value> {
    let guard = state.lock().await;
    guard.sessions.values().map(listed_session).collect()
}

async fn promote_session_id(
    state: &Arc<Mutex<CodexBridgeState>>,
    old_session_id: &str,
    new_session_id: String,
) {
    let mut guard = state.lock().await;
    if old_session_id == new_session_id {
        if let Some(session) = guard.sessions.get_mut(old_session_id) {
            session.codex_session_id = Some(new_session_id);
        }
        return;
    }
    if let Some(mut session) = guard.sessions.remove(old_session_id) {
        session.session_id = new_session_id.clone();
        session.codex_session_id = Some(new_session_id.clone());
        guard
            .aliases
            .insert(old_session_id.to_string(), new_session_id.clone());
        guard.sessions.insert(new_session_id.clone(), session);
        if let Some(active_run) = guard.active_runs.remove(old_session_id) {
            guard.active_runs.insert(new_session_id, active_run);
        }
    }
}

async fn write_bridge_message(
    stdout: &Arc<Mutex<tokio::io::Stdout>>,
    message: &Value,
) -> Result<()> {
    let mut payload = serde_json::to_vec(message).context("failed to encode bridge message")?;
    payload.push(b'\n');
    let mut stdout = stdout.lock().await;
    stdout
        .write_all(&payload)
        .await
        .context("failed to write bridge message")?;
    stdout
        .flush()
        .await
        .context("failed to flush bridge message")
}

async fn next_line<R>(lines: &mut Option<tokio::io::Lines<BufReader<R>>>) -> Result<Option<String>>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let Some(lines) = lines.as_mut() else {
        return Ok(None);
    };
    lines
        .next_line()
        .await
        .context("failed reading Codex bridge process output")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_session(id: &str, model: Option<&str>) -> CodexBridgeSession {
        CodexBridgeSession {
            session_id: id.to_string(),
            codex_session_id: None,
            cwd: PathBuf::from("/tmp/project"),
            execution_mode: TaskExecutionMode::WorkspaceWrite,
            model: model.map(str::to_string),
        }
    }

    #[tokio::test]
    async fn promote_session_id_moves_alias_and_active_session_key() {
        let state = Arc::new(Mutex::new(CodexBridgeState::default()));
        state.lock().await.sessions.insert(
            "temp-session".to_string(),
            test_session("temp-session", Some("gpt-5.4")),
        );

        promote_session_id(&state, "temp-session", "thread_123".to_string()).await;

        let guard = state.lock().await;
        assert!(guard.sessions.get("temp-session").is_none());
        assert!(guard.sessions.get("thread_123").is_some());
        assert_eq!(
            guard.aliases.get("temp-session").map(String::as_str),
            Some("thread_123")
        );
    }

    #[tokio::test]
    async fn list_sessions_returns_current_bridge_sessions() {
        let state = Arc::new(Mutex::new(CodexBridgeState::default()));
        state.lock().await.sessions.insert(
            "thread_1".to_string(),
            test_session("thread_1", Some("gpt-5.4")),
        );
        state
            .lock()
            .await
            .sessions
            .insert("thread_2".to_string(), test_session("thread_2", None));

        let sessions = list_sessions(&state).await;

        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().any(|session| {
            session.get("sessionId").and_then(Value::as_str) == Some("thread_1")
        }));
        assert!(sessions.iter().any(|session| {
            session.get("sessionId").and_then(Value::as_str) == Some("thread_2")
        }));
    }

    #[test]
    fn session_result_exposes_model_config_options() {
        let session = test_session("thread_123", Some("gpt-5.4"));

        let result = session_result(&session);

        assert_eq!(
            result.get("sessionId").and_then(Value::as_str),
            Some("thread_123")
        );
        assert_eq!(
            result
                .get("configOptions")
                .and_then(Value::as_array)
                .and_then(|options| options.first())
                .and_then(|option| option.get("currentValue"))
                .and_then(Value::as_str),
            Some("gpt-5.4")
        );
    }

    #[test]
    fn codex_line_to_acp_updates_maps_agent_message_chunks() {
        let updates = codex_line_to_acp_updates(
            r#"{"type":"item.completed","item":{"type":"agent_message","text":"hello"}}"#,
        );

        assert_eq!(updates.len(), 1);
        assert_eq!(
            updates[0].get("sessionUpdate").and_then(Value::as_str),
            Some("agent_message_chunk")
        );
    }

    #[test]
    fn codex_line_to_acp_updates_maps_tool_call_lifecycle() {
        let started = codex_line_to_acp_updates(
            r#"{"type":"item.started","item":{"id":"tool_1","type":"shell_command","command":"ls"}}"#,
        );
        let completed = codex_line_to_acp_updates(
            r#"{"type":"item.completed","item":{"id":"tool_1","type":"shell_command","command":"ls","output":{"stdout":"ok"}}}"#,
        );

        assert_eq!(
            started[0].get("sessionUpdate").and_then(Value::as_str),
            Some("tool_call")
        );
        assert_eq!(
            completed[0].get("sessionUpdate").and_then(Value::as_str),
            Some("tool_call_update")
        );
    }
}
