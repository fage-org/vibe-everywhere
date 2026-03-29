use super::*;

enum ClaimNextTaskOutcome {
    Task(Option<TaskRecord>),
    DeviceMissing,
}

#[derive(Debug)]
struct TaskCompletion {
    status: TaskStatus,
    pub(crate) exit_code: Option<i32>,
    pub(crate) error: Option<String>,
    message: String,
}

impl TaskCompletion {
    fn succeeded(message: impl Into<String>) -> Self {
        Self {
            status: TaskStatus::Succeeded,
            exit_code: Some(0),
            error: None,
            message: message.into(),
        }
    }

    fn failed(
        message: impl Into<String>,
        error: impl Into<String>,
        exit_code: Option<i32>,
    ) -> Self {
        Self {
            status: TaskStatus::Failed,
            exit_code,
            error: Some(error.into()),
            message: message.into(),
        }
    }

    fn canceled(message: impl Into<String>, exit_code: Option<i32>) -> Self {
        Self {
            status: TaskStatus::Canceled,
            exit_code,
            error: None,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TaskExecutionUpdate {
    pub(crate) status: Option<TaskStatus>,
    pub(crate) execution_protocol: Option<ExecutionProtocol>,
    pub(crate) events: Vec<TaskEventInput>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) error: Option<String>,
}

impl TaskExecutionUpdate {
    fn event(kind: TaskEventKind, message: impl Into<String>) -> Self {
        Self {
            status: None,
            execution_protocol: None,
            events: vec![TaskEventInput {
                kind,
                message: message.into(),
            }],
            exit_code: None,
            error: None,
        }
    }
}

#[allow(async_fn_in_trait)]
pub(crate) trait TaskExecutionSink {
    async fn push_update(&mut self, update: TaskExecutionUpdate) -> Result<()>;

    async fn is_cancel_requested(&mut self) -> Result<bool>;

    async fn push_event(&mut self, kind: TaskEventKind, message: impl Into<String>) -> Result<()> {
        self.push_update(TaskExecutionUpdate::event(kind, message))
            .await
    }
}

struct RelayTaskExecutionSink<'a> {
    client: &'a reqwest::Client,
    relay_url: &'a str,
    task_id: &'a str,
    device_id: &'a str,
    auth: &'a AgentAuthState,
}

impl TaskExecutionSink for RelayTaskExecutionSink<'_> {
    async fn push_update(&mut self, update: TaskExecutionUpdate) -> Result<()> {
        push_task_update(
            self.client,
            self.relay_url,
            self.task_id,
            self.auth,
            AppendTaskEventsRequest {
                device_id: self.device_id.to_string(),
                status: update.status,
                execution_protocol: update.execution_protocol,
                events: update.events,
                exit_code: update.exit_code,
                error: update.error,
            },
        )
        .await
    }

    async fn is_cancel_requested(&mut self) -> Result<bool> {
        is_task_cancel_requested(self.client, self.relay_url, self.task_id, self.auth).await
    }
}

struct AcpProcess {
    child: Child,
    stdin: ChildStdin,
    stdout_lines: Option<tokio::io::Lines<BufReader<ChildStdout>>>,
    stderr_lines: Option<tokio::io::Lines<BufReader<ChildStderr>>>,
    next_request_id: u64,
    session_root: PathBuf,
    terminals: HashMap<String, ManagedTerminal>,
}

#[derive(Clone)]
struct ManagedTerminal {
    child: Arc<Mutex<Child>>,
    output: Arc<RwLock<String>>,
    output_limit: Option<usize>,
    exit_status: Arc<RwLock<Option<TerminalExitStatus>>>,
}

#[derive(Debug, Clone)]
struct TerminalExitStatus {
    pub(crate) exit_code: Option<i32>,
    signal: Option<String>,
}

pub(crate) async fn try_mark_local_task_started(
    current_task_id: &Arc<RwLock<Option<String>>>,
    task_id: &str,
) -> bool {
    let mut guard = current_task_id.write().await;
    match guard.as_deref() {
        Some(existing) if existing != task_id => false,
        _ => {
            *guard = Some(task_id.to_string());
            true
        }
    }
}

pub(crate) async fn clear_local_task(current_task_id: &Arc<RwLock<Option<String>>>, task_id: &str) {
    let mut guard = current_task_id.write().await;
    if guard.as_deref() == Some(task_id) {
        *guard = None;
    }
}

pub(crate) async fn task_loop(
    client: reqwest::Client,
    relay_url: String,
    profile: AgentProfile,
    auth: AgentAuthState,
    shared: SharedState,
    working_root: PathBuf,
    poll_interval_ms: u64,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms));

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("agent received shutdown signal");
                break;
            }
            _ = interval.tick() => {
                if shared.current_task_id.read().await.is_some() {
                    continue;
                }

                match claim_next_task(&client, &relay_url, &profile.device_id, &auth).await {
                    Ok(ClaimNextTaskOutcome::Task(Some(task))) => {
                        if !try_mark_local_task_started(&shared.current_task_id, &task.id).await {
                            continue;
                        }

                        let task_id = task.id.clone();
                        let result = execute_task(
                            &client,
                            &relay_url,
                            &profile.device_id,
                            &auth,
                            &shared,
                            &working_root,
                            task,
                        ).await;

                        if let Err(error) = result {
                            eprintln!("task execution failed: {error:#}");
                        }

                        clear_local_task(&shared.current_task_id, &task_id).await;
                    }
                    Ok(ClaimNextTaskOutcome::Task(None)) => {}
                    Ok(ClaimNextTaskOutcome::DeviceMissing) => {
                        eprintln!(
                            "device {} missing on relay during task claim, re-registering",
                            profile.device_id
                        );
                        if let Err(error) =
                            register_current_device(&client, &relay_url, &profile, &shared, &auth).await
                        {
                            eprintln!("device re-registration failed: {error:#}");
                        }
                    }
                    Err(error) => {
                        eprintln!("failed to claim task: {error:#}");
                    }
                }
            }
        }
    }

    Ok(())
}

async fn claim_next_task(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
    auth: &AgentAuthState,
) -> Result<ClaimNextTaskOutcome> {
    let endpoint = format!("{relay_url}/api/devices/{device_id}/tasks/claim-next");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.post(endpoint), device_credential.as_deref())
        .send()
        .await
        .context("failed to claim task")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ClaimNextTaskOutcome::DeviceMissing);
    }

    let response = response
        .error_for_status()
        .context("relay rejected task claim")?
        .json::<ClaimTaskResponse>()
        .await
        .context("failed to decode claim task response")?;

    Ok(ClaimNextTaskOutcome::Task(response.task))
}

async fn execute_task(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
    auth: &AgentAuthState,
    shared: &SharedState,
    working_root: &Path,
    task: TaskRecord,
) -> Result<()> {
    let task_id = task.id.clone();
    let mut sink = RelayTaskExecutionSink {
        client,
        relay_url,
        task_id: &task_id,
        device_id,
        auth,
    };
    execute_task_with_sink(&mut sink, &shared.providers, working_root, task).await
}

pub(crate) async fn execute_task_with_sink<S>(
    sink: &mut S,
    providers: &[ProviderStatus],
    working_root: &Path,
    task: TaskRecord,
) -> Result<()>
where
    S: TaskExecutionSink,
{
    let execution_protocol = task.execution_protocol.clone();
    let provider = match providers
        .iter()
        .find(|provider| provider.kind == task.provider)
        .cloned()
    {
        Some(provider) => provider,
        None => {
            sink.push_update(TaskExecutionUpdate {
                status: Some(TaskStatus::Failed),
                execution_protocol: Some(execution_protocol),
                events: vec![TaskEventInput {
                    kind: TaskEventKind::System,
                    message: "provider missing from device capabilities".to_string(),
                }],
                exit_code: None,
                error: Some("provider missing from device capabilities".to_string()),
            })
            .await?;
            return Ok(());
        }
    };

    let cwd = resolve_task_cwd(working_root, task.cwd.as_deref());
    let cwd = match ensure_task_cwd(&cwd) {
        Ok(cwd) => cwd,
        Err(error) => {
            let message = format!("task working directory is unavailable: {error}");
            sink.push_update(TaskExecutionUpdate {
                status: Some(TaskStatus::Failed),
                execution_protocol: Some(execution_protocol),
                events: vec![TaskEventInput {
                    kind: TaskEventKind::System,
                    message: message.clone(),
                }],
                exit_code: None,
                error: Some(message),
            })
            .await?;
            return Ok(());
        }
    };

    let startup_events = vec![
        TaskEventInput {
            kind: TaskEventKind::System,
            message: format!(
                "Starting {} task over {}",
                task.provider.label(),
                execution_protocol.label()
            ),
        },
        TaskEventInput {
            kind: TaskEventKind::System,
            message: format!("cwd={}", cwd.display()),
        },
    ];

    if !provider.available {
        let message = provider
            .error
            .unwrap_or_else(|| "provider is not available on this device".to_string());
        sink.push_update(TaskExecutionUpdate {
            status: Some(TaskStatus::Failed),
            execution_protocol: Some(execution_protocol),
            events: vec![TaskEventInput {
                kind: TaskEventKind::System,
                message: message.clone(),
            }],
            exit_code: None,
            error: Some(message),
        })
        .await?;
        return Ok(());
    }

    sink.push_update(TaskExecutionUpdate {
        status: Some(TaskStatus::Running),
        execution_protocol: Some(execution_protocol.clone()),
        events: startup_events,
        exit_code: None,
        error: None,
    })
    .await?;

    let completion = match execution_protocol {
        ExecutionProtocol::Cli => execute_cli_task(sink, &provider, &task, &cwd).await,
        ExecutionProtocol::Acp => match provider.kind {
            ProviderKind::OpenCode => execute_opencode_acp_task(sink, &provider, &task, &cwd).await,
            ProviderKind::Codex => {
                execute_codex_embedded_acp_task(sink, &provider, &task, &cwd).await
            }
            ProviderKind::ClaudeCode => {
                bail!("ACP transport is not implemented for Claude Code yet")
            }
        },
    };

    let completion = match completion {
        Ok(result) => result,
        Err(error) => TaskCompletion::failed(
            format!("{} task failed", task.provider.label()),
            error.to_string(),
            None,
        ),
    };

    sink.push_update(TaskExecutionUpdate {
        status: Some(completion.status),
        execution_protocol: Some(execution_protocol),
        events: vec![TaskEventInput {
            kind: TaskEventKind::System,
            message: completion.message,
        }],
        exit_code: completion.exit_code,
        error: completion.error,
    })
    .await?;

    Ok(())
}

async fn execute_cli_task<S>(
    sink: &mut S,
    provider: &ProviderStatus,
    task: &TaskRecord,
    cwd: &Path,
) -> Result<TaskCompletion>
where
    S: TaskExecutionSink,
{
    let mut command = build_provider_command(provider, task, cwd)?;
    command
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", provider.command))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let mut stdout_lines = stdout.map(|handle| BufReader::new(handle).lines());
    let mut stderr_lines = stderr.map(|handle| BufReader::new(handle).lines());
    let mut cancel_interval = tokio::time::interval(Duration::from_millis(ACP_CANCEL_POLL_MS));
    let mut cancel_requested = false;

    loop {
        tokio::select! {
            result = next_line(&mut stdout_lines), if stdout_lines.is_some() => {
                match result? {
                    Some(line) => {
                        for event in provider_stdout_to_task_events(&provider.kind, &line) {
                            sink.push_event(event.kind, event.message).await?;
                        }
                    }
                    None => stdout_lines = None,
                }
            }
            result = next_line(&mut stderr_lines), if stderr_lines.is_some() => {
                match result? {
                    Some(line) => {
                        sink.push_event(TaskEventKind::ProviderStderr, line).await?;
                    }
                    None => stderr_lines = None,
                }
            }
            _ = cancel_interval.tick() => {
                if !cancel_requested && sink.is_cancel_requested().await? {
                    cancel_requested = true;
                    let _ = child.kill().await;
                }
            }
            status = child.wait() => {
                let status = status.context("failed waiting on child process")?;
                drain_remaining_output(sink, &provider.kind, &mut stdout_lines, &mut stderr_lines).await?;

                let completion = if cancel_requested {
                    TaskCompletion::canceled("Task canceled", status.code())
                } else if status.success() {
                    TaskCompletion::succeeded("Task finished successfully")
                } else {
                    TaskCompletion::failed(
                        "Task failed",
                        format!("process exited with status {:?}", status.code()),
                        status.code(),
                    )
                };
                break Ok(completion);
            }
        }
    }
}

async fn execute_codex_embedded_acp_task<S>(
    sink: &mut S,
    provider: &ProviderStatus,
    task: &TaskRecord,
    cwd: &Path,
) -> Result<TaskCompletion>
where
    S: TaskExecutionSink,
{
    if provider.kind != ProviderKind::Codex {
        bail!("embedded Codex ACP adapter only supports the Codex provider");
    }

    let mut command = build_provider_command(provider, task, cwd)?;
    command
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", provider.command))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let mut stdout_lines = stdout.map(|handle| BufReader::new(handle).lines());
    let mut stderr_lines = stderr.map(|handle| BufReader::new(handle).lines());
    let mut cancel_interval = tokio::time::interval(Duration::from_millis(ACP_CANCEL_POLL_MS));
    let mut cancel_requested = false;

    loop {
        tokio::select! {
            result = next_line(&mut stdout_lines), if stdout_lines.is_some() => {
                match result? {
                    Some(line) => {
                        for event in provider_stdout_to_task_events(&provider.kind, &line) {
                            sink.push_event(event.kind, event.message).await?;
                        }
                    }
                    None => stdout_lines = None,
                }
            }
            result = next_line(&mut stderr_lines), if stderr_lines.is_some() => {
                match result? {
                    Some(line) => {
                        sink.push_event(TaskEventKind::ProviderStderr, line).await?;
                    }
                    None => stderr_lines = None,
                }
            }
            _ = cancel_interval.tick() => {
                if !cancel_requested && sink.is_cancel_requested().await? {
                    cancel_requested = true;
                    let _ = child.kill().await;
                }
            }
            status = child.wait() => {
                let status = status.context("failed waiting on embedded Codex adapter process")?;
                drain_remaining_output(sink, &provider.kind, &mut stdout_lines, &mut stderr_lines).await?;

                let completion = if cancel_requested {
                    TaskCompletion::canceled("Embedded Codex ACP task canceled", status.code())
                } else if status.success() {
                    TaskCompletion::succeeded("Embedded Codex ACP task finished successfully")
                } else {
                    TaskCompletion::failed(
                        "Embedded Codex ACP task failed",
                        format!("codex exited with status {:?}", status.code()),
                        status.code(),
                    )
                };
                break Ok(completion);
            }
        }
    }
}

async fn execute_opencode_acp_task<S>(
    sink: &mut S,
    provider: &ProviderStatus,
    task: &TaskRecord,
    cwd: &Path,
) -> Result<TaskCompletion>
where
    S: TaskExecutionSink,
{
    if provider.kind != ProviderKind::OpenCode {
        bail!("ACP transport is only implemented for OpenCode in this MVP");
    }

    let mut acp = spawn_acp_process(provider, cwd)?;

    let initialize_id = acp.next_request_id();
    send_rpc_request(
        &mut acp,
        initialize_id,
        "initialize",
        json!({
            "protocolVersion": ACP_PROTOCOL_VERSION,
            "clientCapabilities": {
                "fs": {
                    "readTextFile": true,
                    "writeTextFile": true
                },
                "terminal": true
            },
            "clientInfo": {
                "name": "vibe-agent",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
    .await?;
    let initialize_result =
        wait_for_rpc_response(sink, &mut acp, initialize_id, "initialize").await?;
    if let Some(auth_methods) = initialize_result
        .get("authMethods")
        .and_then(Value::as_array)
    {
        if !auth_methods.is_empty() {
            sink.push_event(
                TaskEventKind::System,
                format!(
                    "OpenCode ACP advertised auth methods: {}",
                    serde_json::to_string(auth_methods).unwrap_or_else(|_| "[]".to_string())
                ),
            )
            .await?;
        }
    }

    let session_new_id = acp.next_request_id();
    send_rpc_request(
        &mut acp,
        session_new_id,
        "session/new",
        json!({
            "cwd": cwd.to_string_lossy(),
            "mcpServers": []
        }),
    )
    .await?;
    let session_result =
        wait_for_rpc_response(sink, &mut acp, session_new_id, "session/new").await?;
    let session_id = session_result
        .get("sessionId")
        .and_then(Value::as_str)
        .map(str::to_string)
        .context("ACP session/new did not return a sessionId")?;

    sink.push_event(
        TaskEventKind::System,
        format!("OpenCode ACP session established: {session_id}"),
    )
    .await?;

    if let Some(model) = task.model.as_deref() {
        apply_acp_model_override(
            sink,
            &mut acp,
            &session_id,
            session_result.get("configOptions"),
            model,
        )
        .await?;
    }

    let prompt_id = acp.next_request_id();
    send_rpc_request(
        &mut acp,
        prompt_id,
        "session/prompt",
        json!({
            "sessionId": session_id,
            "prompt": [
                {
                    "type": "text",
                    "text": task.prompt
                }
            ]
        }),
    )
    .await?;

    let mut cancel_sent = false;
    let prompt_result =
        wait_for_prompt_response(sink, &mut acp, prompt_id, &session_id, &mut cancel_sent).await;

    cleanup_terminals(&mut acp).await;
    shutdown_acp_process(&mut acp).await;

    let prompt_result = prompt_result?;
    let stop_reason = prompt_result
        .get("stopReason")
        .and_then(Value::as_str)
        .unwrap_or("end_turn");

    let completion = match stop_reason {
        "cancelled" => TaskCompletion::canceled("ACP task canceled", None),
        "refusal" => TaskCompletion::failed(
            "ACP task refused to continue",
            "agent returned stopReason=refusal",
            None,
        ),
        "max_tokens" => TaskCompletion::succeeded("ACP task reached max tokens"),
        "max_turn_requests" => TaskCompletion::succeeded("ACP task reached max turn requests"),
        _ => TaskCompletion::succeeded("ACP task finished successfully"),
    };

    Ok(completion)
}

fn spawn_acp_process(provider: &ProviderStatus, cwd: &Path) -> Result<AcpProcess> {
    let mut command = Command::new(&provider.command);
    command
        .arg("acp")
        .arg("--cwd")
        .arg(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {} acp", provider.command))?;
    let stdin = child.stdin.take().context("ACP process missing stdin")?;
    let stdout = child.stdout.take().context("ACP process missing stdout")?;
    let stderr = child.stderr.take().context("ACP process missing stderr")?;

    Ok(AcpProcess {
        child,
        stdin,
        stdout_lines: Some(BufReader::new(stdout).lines()),
        stderr_lines: Some(BufReader::new(stderr).lines()),
        next_request_id: 1,
        session_root: cwd.to_path_buf(),
        terminals: HashMap::new(),
    })
}

impl AcpProcess {
    fn next_request_id(&mut self) -> u64 {
        let current = self.next_request_id;
        self.next_request_id += 1;
        current
    }
}

async fn send_rpc_request(
    acp: &mut AcpProcess,
    id: u64,
    method: &str,
    params: Value,
) -> Result<()> {
    let message = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });
    write_json_line(&mut acp.stdin, &message).await
}

async fn send_rpc_notification(acp: &mut AcpProcess, method: &str, params: Value) -> Result<()> {
    let message = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    });
    write_json_line(&mut acp.stdin, &message).await
}

async fn send_rpc_response(acp: &mut AcpProcess, id: Value, result: Value) -> Result<()> {
    let message = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    });
    write_json_line(&mut acp.stdin, &message).await
}

async fn send_rpc_error(
    acp: &mut AcpProcess,
    id: Value,
    code: i64,
    message: impl Into<String>,
) -> Result<()> {
    let message = json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into(),
        },
    });
    write_json_line(&mut acp.stdin, &message).await
}

async fn write_json_line(stdin: &mut ChildStdin, value: &Value) -> Result<()> {
    let serialized = serde_json::to_vec(value).context("failed to serialize JSON-RPC message")?;
    stdin
        .write_all(&serialized)
        .await
        .context("failed to write JSON-RPC request")?;
    stdin
        .write_all(b"\n")
        .await
        .context("failed to terminate JSON-RPC request")?;
    stdin
        .flush()
        .await
        .context("failed to flush JSON-RPC request")?;
    Ok(())
}

async fn wait_for_rpc_response<S>(
    sink: &mut S,
    acp: &mut AcpProcess,
    expected_id: u64,
    method_name: &str,
) -> Result<Value>
where
    S: TaskExecutionSink,
{
    loop {
        tokio::select! {
            result = next_line(&mut acp.stdout_lines), if acp.stdout_lines.is_some() => {
                match result? {
                    Some(line) => {
                        let message: Value = serde_json::from_str(&line)
                            .with_context(|| format!("failed to parse ACP stdout line: {line}"))?;
                        if is_expected_response(&message, expected_id) {
                            return extract_rpc_result(message, method_name);
                        }
                        handle_acp_message(sink, acp, &message, None, false).await?;
                    }
                    None => {
                        bail!("ACP transport closed before {method_name} completed");
                    }
                }
            }
            result = next_line(&mut acp.stderr_lines), if acp.stderr_lines.is_some() => {
                match result? {
                    Some(line) => {
                        sink.push_event(TaskEventKind::ProviderStderr, line).await?;
                    }
                    None => {
                        acp.stderr_lines = None;
                    }
                }
            }
        }
    }
}

async fn wait_for_prompt_response<S>(
    sink: &mut S,
    acp: &mut AcpProcess,
    expected_id: u64,
    session_id: &str,
    cancel_sent: &mut bool,
) -> Result<Value>
where
    S: TaskExecutionSink,
{
    let mut cancel_interval = tokio::time::interval(Duration::from_millis(ACP_CANCEL_POLL_MS));

    loop {
        tokio::select! {
            result = next_line(&mut acp.stdout_lines), if acp.stdout_lines.is_some() => {
                match result? {
                    Some(line) => {
                        let message: Value = serde_json::from_str(&line)
                            .with_context(|| format!("failed to parse ACP stdout line: {line}"))?;
                        if is_expected_response(&message, expected_id) {
                            return extract_rpc_result(message, "session/prompt");
                        }
                        handle_acp_message(sink, acp, &message, Some(session_id), *cancel_sent).await?;
                    }
                    None => {
                        bail!("ACP transport closed before session/prompt completed");
                    }
                }
            }
            result = next_line(&mut acp.stderr_lines), if acp.stderr_lines.is_some() => {
                match result? {
                    Some(line) => {
                        sink.push_event(TaskEventKind::ProviderStderr, line).await?;
                    }
                    None => {
                        acp.stderr_lines = None;
                    }
                }
            }
            _ = cancel_interval.tick() => {
                if !*cancel_sent && sink.is_cancel_requested().await? {
                    *cancel_sent = true;
                    send_rpc_notification(acp, "session/cancel", json!({ "sessionId": session_id })).await?;
                    sink.push_event(
                        TaskEventKind::System,
                        "Cancellation sent to ACP agent".to_string(),
                    ).await?;
                }
            }
        }
    }
}

async fn handle_acp_message<S>(
    sink: &mut S,
    acp: &mut AcpProcess,
    message: &Value,
    active_session_id: Option<&str>,
    cancel_sent: bool,
) -> Result<()>
where
    S: TaskExecutionSink,
{
    if let Some(method) = message.get("method").and_then(Value::as_str) {
        match method {
            "session/update" => {
                if let Some(params) = message.get("params") {
                    for event in acp_update_to_events(params) {
                        sink.push_event(event.kind, event.message).await?;
                    }
                }
            }
            _ => {
                if message.get("id").is_some() {
                    handle_acp_request(
                        sink,
                        acp,
                        method,
                        message.get("id").cloned().unwrap_or(Value::Null),
                        message.get("params").cloned().unwrap_or_else(|| json!({})),
                        active_session_id,
                        cancel_sent,
                    )
                    .await?;
                }
            }
        }
    }

    Ok(())
}

async fn handle_acp_request<S>(
    sink: &mut S,
    acp: &mut AcpProcess,
    method: &str,
    id: Value,
    params: Value,
    active_session_id: Option<&str>,
    cancel_sent: bool,
) -> Result<()>
where
    S: TaskExecutionSink,
{
    match method {
        "session/request_permission" => {
            let response = build_permission_response(&params, cancel_sent)?;
            let message = permission_log_message(&params, &response);
            sink.push_event(TaskEventKind::System, message).await?;
            send_rpc_response(acp, id, response).await?
        }
        "fs/read_text_file" => match handle_read_text_file(&acp.session_root, &params) {
            Ok(result) => send_rpc_response(acp, id, result).await?,
            Err(error) => send_rpc_error(acp, id, -32000, error.to_string()).await?,
        },
        "fs/write_text_file" => match handle_write_text_file(&acp.session_root, &params) {
            Ok(result) => send_rpc_response(acp, id, result).await?,
            Err(error) => send_rpc_error(acp, id, -32000, error.to_string()).await?,
        },
        "terminal/create" => match handle_terminal_create(acp, &params) {
            Ok(result) => send_rpc_response(acp, id, result).await?,
            Err(error) => send_rpc_error(acp, id, -32000, error.to_string()).await?,
        },
        "terminal/output" => match handle_terminal_output(acp, &params).await {
            Ok(result) => send_rpc_response(acp, id, result).await?,
            Err(error) => send_rpc_error(acp, id, -32000, error.to_string()).await?,
        },
        "terminal/wait_for_exit" => {
            match handle_terminal_wait_for_exit(sink, acp, &params, active_session_id).await {
                Ok(result) => send_rpc_response(acp, id, result).await?,
                Err(error) => send_rpc_error(acp, id, -32000, error.to_string()).await?,
            }
        }
        "terminal/kill" => match handle_terminal_kill(acp, &params).await {
            Ok(result) => send_rpc_response(acp, id, result).await?,
            Err(error) => send_rpc_error(acp, id, -32000, error.to_string()).await?,
        },
        "terminal/release" => match handle_terminal_release(acp, &params).await {
            Ok(result) => send_rpc_response(acp, id, result).await?,
            Err(error) => send_rpc_error(acp, id, -32000, error.to_string()).await?,
        },
        _ => {
            send_rpc_error(
                acp,
                id,
                -32601,
                format!("unsupported ACP client method: {method}"),
            )
            .await?
        }
    }

    Ok(())
}

fn build_permission_response(params: &Value, cancel_sent: bool) -> Result<Value> {
    if cancel_sent {
        return Ok(json!({
            "outcome": {
                "outcome": "cancelled"
            }
        }));
    }

    let options = params
        .get("options")
        .and_then(Value::as_array)
        .context("permission request missing options")?;
    let selected = options
        .iter()
        .find(|option| option.get("kind").and_then(Value::as_str) == Some("allow_once"))
        .or_else(|| {
            options
                .iter()
                .find(|option| option.get("kind").and_then(Value::as_str) == Some("allow_always"))
        })
        .or_else(|| options.first())
        .context("permission request had no selectable options")?;
    let option_id = selected
        .get("optionId")
        .and_then(Value::as_str)
        .context("permission option missing optionId")?;

    Ok(json!({
        "outcome": {
            "outcome": "selected",
            "optionId": option_id
        }
    }))
}

fn permission_log_message(params: &Value, response: &Value) -> String {
    let title = params
        .get("toolCall")
        .and_then(|tool_call| tool_call.get("title"))
        .and_then(Value::as_str)
        .unwrap_or("tool call");

    match response
        .get("outcome")
        .and_then(|outcome| outcome.get("outcome"))
        .and_then(Value::as_str)
    {
        Some("cancelled") => format!("ACP permission cancelled for {title}"),
        _ => {
            let option_id = response
                .get("outcome")
                .and_then(|outcome| outcome.get("optionId"))
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            format!("ACP permission auto-selected {option_id} for {title}")
        }
    }
}

fn handle_read_text_file(session_root: &Path, params: &Value) -> Result<Value> {
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .context("fs/read_text_file missing path")?;
    let path = resolve_existing_path_within_root(session_root, path)?;
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let line = params
        .get("line")
        .and_then(Value::as_u64)
        .unwrap_or(1)
        .max(1) as usize;
    let limit = params
        .get("limit")
        .and_then(Value::as_u64)
        .map(|value| value as usize);
    let content = slice_text_by_lines(&content, line, limit);
    Ok(json!({ "content": content }))
}

fn handle_write_text_file(session_root: &Path, params: &Value) -> Result<Value> {
    let path = params
        .get("path")
        .and_then(Value::as_str)
        .context("fs/write_text_file missing path")?;
    let content = params
        .get("content")
        .and_then(Value::as_str)
        .context("fs/write_text_file missing content")?;
    let path = resolve_write_path_within_root(session_root, path)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create parent directory {}", parent.display()))?;
    }
    std::fs::write(&path, content)
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(json!({}))
}

fn handle_terminal_create(acp: &mut AcpProcess, params: &Value) -> Result<Value> {
    let command_name = params
        .get("command")
        .and_then(Value::as_str)
        .context("terminal/create missing command")?;
    let args = params
        .get("args")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let cwd = params
        .get("cwd")
        .and_then(Value::as_str)
        .map(|value| resolve_existing_dir_within_root(&acp.session_root, value))
        .transpose()?
        .unwrap_or_else(|| acp.session_root.clone());
    let env_vars = parse_terminal_env(params.get("env"));
    let output_limit = params
        .get("outputByteLimit")
        .and_then(Value::as_u64)
        .map(|value| value as usize);

    let terminal_id = Uuid::new_v4().to_string();
    let mut command = Command::new(command_name);
    command
        .args(&args)
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in env_vars {
        command.env(key, value);
    }

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn terminal command {command_name}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let output = Arc::new(RwLock::new(String::new()));
    if let Some(stdout) = stdout {
        spawn_terminal_output_collector(stdout, output.clone(), output_limit);
    }
    if let Some(stderr) = stderr {
        spawn_terminal_output_collector(stderr, output.clone(), output_limit);
    }

    acp.terminals.insert(
        terminal_id.clone(),
        ManagedTerminal {
            child: Arc::new(Mutex::new(child)),
            output,
            output_limit,
            exit_status: Arc::new(RwLock::new(None)),
        },
    );

    Ok(json!({ "terminalId": terminal_id }))
}

async fn handle_terminal_output(acp: &mut AcpProcess, params: &Value) -> Result<Value> {
    let terminal = lookup_terminal(acp, params)?;
    let exit_status = current_terminal_exit_status(&terminal).await?;
    let output = terminal.output.read().await.clone();
    let truncated = terminal
        .output_limit
        .map(|limit| output.len() >= limit)
        .unwrap_or(false);

    let mut result = serde_json::Map::new();
    result.insert("output".to_string(), Value::String(output));
    result.insert("truncated".to_string(), Value::Bool(truncated));
    if let Some(exit_status) = exit_status {
        result.insert(
            "exitStatus".to_string(),
            json!({
                "exitCode": exit_status.exit_code,
                "signal": exit_status.signal,
            }),
        );
    }

    Ok(Value::Object(result))
}

async fn handle_terminal_wait_for_exit<S>(
    sink: &mut S,
    acp: &mut AcpProcess,
    params: &Value,
    active_session_id: Option<&str>,
) -> Result<Value>
where
    S: TaskExecutionSink,
{
    let terminal = lookup_terminal(acp, params)?;
    let request_session_id = params
        .get("sessionId")
        .and_then(Value::as_str)
        .or(active_session_id)
        .unwrap_or_default()
        .to_string();
    let mut interval = tokio::time::interval(Duration::from_millis(TERMINAL_POLL_MS));

    loop {
        if let Some(exit_status) = current_terminal_exit_status(&terminal).await? {
            return Ok(json!({
                "exitCode": exit_status.exit_code,
                "signal": exit_status.signal,
            }));
        }

        interval.tick().await;
        if sink.is_cancel_requested().await? && !request_session_id.is_empty() {
            let _ = handle_terminal_kill(
                acp,
                &json!({
                    "sessionId": request_session_id,
                    "terminalId": params.get("terminalId").cloned().unwrap_or(Value::Null),
                }),
            )
            .await;
        }
    }
}

async fn handle_terminal_kill(acp: &mut AcpProcess, params: &Value) -> Result<Value> {
    let terminal = lookup_terminal(acp, params)?;
    let mut child = terminal.child.lock().await;
    let _ = child.kill().await;
    drop(child);
    let _ = current_terminal_exit_status(&terminal).await?;
    Ok(json!({}))
}

async fn handle_terminal_release(acp: &mut AcpProcess, params: &Value) -> Result<Value> {
    let terminal_id = params
        .get("terminalId")
        .and_then(Value::as_str)
        .context("terminal/release missing terminalId")?;
    let Some(terminal) = acp.terminals.remove(terminal_id) else {
        bail!("terminal not found: {terminal_id}");
    };
    let _ = ensure_terminal_stopped(&terminal).await?;
    Ok(json!({}))
}

fn lookup_terminal(acp: &AcpProcess, params: &Value) -> Result<ManagedTerminal> {
    let terminal_id = params
        .get("terminalId")
        .and_then(Value::as_str)
        .context("terminal request missing terminalId")?;
    acp.terminals
        .get(terminal_id)
        .cloned()
        .with_context(|| format!("terminal not found: {terminal_id}"))
}

async fn current_terminal_exit_status(
    terminal: &ManagedTerminal,
) -> Result<Option<TerminalExitStatus>> {
    if let Some(exit_status) = terminal.exit_status.read().await.clone() {
        return Ok(Some(exit_status));
    }

    let mut child = terminal.child.lock().await;
    if let Some(status) = child.try_wait().context("failed to poll terminal status")? {
        let exit_status = exit_status_from_status(status);
        *terminal.exit_status.write().await = Some(exit_status.clone());
        Ok(Some(exit_status))
    } else {
        Ok(None)
    }
}

async fn ensure_terminal_stopped(terminal: &ManagedTerminal) -> Result<TerminalExitStatus> {
    if let Some(exit_status) = current_terminal_exit_status(terminal).await? {
        return Ok(exit_status);
    }

    {
        let mut child = terminal.child.lock().await;
        let _ = child.kill().await;
        let status = child
            .wait()
            .await
            .context("failed to wait for terminal process")?;
        let exit_status = exit_status_from_status(status);
        *terminal.exit_status.write().await = Some(exit_status.clone());
    }

    terminal
        .exit_status
        .read()
        .await
        .clone()
        .context("terminal exit status missing after shutdown")
}

fn spawn_terminal_output_collector<R>(
    reader: R,
    output: Arc<RwLock<String>>,
    output_limit: Option<usize>,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    let mut chunk = line;
                    chunk.push('\n');
                    append_terminal_output(&output, output_limit, &chunk).await;
                }
                Ok(None) => break,
                Err(error) => {
                    let chunk = format!("[terminal output read error] {error}\n");
                    append_terminal_output(&output, output_limit, &chunk).await;
                    break;
                }
            }
        }
    });
}

async fn append_terminal_output(
    output: &Arc<RwLock<String>>,
    output_limit: Option<usize>,
    chunk: &str,
) {
    let mut guard = output.write().await;
    guard.push_str(chunk);
    if let Some(limit) = output_limit {
        trim_to_byte_limit(&mut guard, limit);
    }
}

fn trim_to_byte_limit(buffer: &mut String, limit: usize) {
    if buffer.len() <= limit {
        return;
    }
    let overflow = buffer.len().saturating_sub(limit);
    let mut cut_index = overflow;
    while cut_index < buffer.len() && !buffer.is_char_boundary(cut_index) {
        cut_index += 1;
    }
    buffer.drain(..cut_index.min(buffer.len()));
}

async fn cleanup_terminals(acp: &mut AcpProcess) {
    let terminal_ids = acp.terminals.keys().cloned().collect::<Vec<_>>();
    for terminal_id in terminal_ids {
        if let Some(terminal) = acp.terminals.remove(&terminal_id) {
            let _ = ensure_terminal_stopped(&terminal).await;
        }
    }
}

async fn shutdown_acp_process(acp: &mut AcpProcess) {
    let _ = acp.child.kill().await;
    let _ = acp.child.wait().await;
}

async fn apply_acp_model_override<S>(
    sink: &mut S,
    acp: &mut AcpProcess,
    session_id: &str,
    config_options: Option<&Value>,
    requested_model: &str,
) -> Result<()>
where
    S: TaskExecutionSink,
{
    let Some(config_options) = config_options.and_then(Value::as_array) else {
        sink.push_event(
            TaskEventKind::System,
            format!(
                "ACP model override requested ({requested_model}) but the agent did not advertise config options"
            ),
        )
        .await?;
        return Ok(());
    };

    let Some((config_id, value_id, label)) = find_model_option(config_options, requested_model)
    else {
        sink.push_event(
            TaskEventKind::System,
            format!(
                "ACP model override requested ({requested_model}) but no matching model option was advertised"
            ),
        )
        .await?;
        return Ok(());
    };

    let request_id = acp.next_request_id();
    send_rpc_request(
        acp,
        request_id,
        "session/set_config_option",
        json!({
            "sessionId": session_id,
            "configId": config_id,
            "value": value_id,
        }),
    )
    .await?;
    let _ = wait_for_rpc_response(sink, acp, request_id, "session/set_config_option").await?;

    sink.push_event(
        TaskEventKind::System,
        format!("ACP model override applied: {label}"),
    )
    .await?;

    Ok(())
}

fn find_model_option(
    config_options: &[Value],
    requested_model: &str,
) -> Option<(String, String, String)> {
    for config_option in config_options {
        let config_id = config_option.get("id").and_then(Value::as_str)?;
        let category = config_option.get("category").and_then(Value::as_str);
        if category != Some("model") && config_id != "model" {
            continue;
        }

        if let Some(options) = config_option.get("options").and_then(Value::as_array) {
            for option in flatten_config_options(options) {
                if option.0.eq_ignore_ascii_case(requested_model)
                    || option.1.eq_ignore_ascii_case(requested_model)
                {
                    return Some((
                        config_id.to_string(),
                        option.0.to_string(),
                        option.1.to_string(),
                    ));
                }
            }
        }
    }

    None
}

fn flatten_config_options<'a>(options: &'a [Value]) -> Vec<(&'a str, &'a str)> {
    let mut flattened = Vec::new();
    for option in options {
        if let Some(value) = option.get("value").and_then(Value::as_str) {
            let name = option.get("name").and_then(Value::as_str).unwrap_or(value);
            flattened.push((value, name));
            continue;
        }

        if let Some(grouped) = option.get("options").and_then(Value::as_array) {
            for nested in grouped {
                if let Some(value) = nested.get("value").and_then(Value::as_str) {
                    let name = nested.get("name").and_then(Value::as_str).unwrap_or(value);
                    flattened.push((value, name));
                }
            }
        }
    }
    flattened
}

fn is_expected_response(message: &Value, expected_id: u64) -> bool {
    message.get("id") == Some(&json!(expected_id))
}

fn extract_rpc_result(message: Value, method_name: &str) -> Result<Value> {
    if let Some(error) = message.get("error") {
        let code = error
            .get("code")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown ACP error");
        bail!("ACP {method_name} failed ({code}): {message}");
    }

    message
        .get("result")
        .cloned()
        .with_context(|| format!("ACP {method_name} response missing result"))
}

fn resolve_existing_path_within_root(root: &Path, raw_path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(raw_path);
    if !path.is_absolute() {
        bail!("ACP paths must be absolute: {raw_path}");
    }
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", path.display()))?;
    ensure_path_within_root(root, &path)?;
    Ok(path)
}

fn resolve_existing_dir_within_root(root: &Path, raw_path: &str) -> Result<PathBuf> {
    let path = resolve_existing_path_within_root(root, raw_path)?;
    if !path.is_dir() {
        bail!("expected directory: {}", path.display());
    }
    Ok(path)
}

fn resolve_write_path_within_root(root: &Path, raw_path: &str) -> Result<PathBuf> {
    let path = PathBuf::from(raw_path);
    if !path.is_absolute() {
        bail!("ACP paths must be absolute: {raw_path}");
    }

    let anchor = path
        .ancestors()
        .find(|candidate| candidate.exists())
        .context("path has no existing ancestor")?
        .canonicalize()
        .context("failed to canonicalize write path ancestor")?;
    ensure_path_within_root(root, &anchor)?;
    Ok(path)
}

fn ensure_path_within_root(root: &Path, path: &Path) -> Result<()> {
    if path.starts_with(root) {
        Ok(())
    } else {
        bail!(
            "path {} is outside session root {}",
            path.display(),
            root.display()
        )
    }
}

fn slice_text_by_lines(content: &str, line: usize, limit: Option<usize>) -> String {
    let start = line.saturating_sub(1);
    let lines = content.lines().collect::<Vec<_>>();
    if start >= lines.len() {
        return String::new();
    }
    let end = limit
        .map(|limit| start.saturating_add(limit).min(lines.len()))
        .unwrap_or(lines.len());
    lines[start..end].join("\n")
}

fn parse_terminal_env(value: Option<&Value>) -> Vec<(String, String)> {
    match value {
        Some(Value::Object(map)) => map
            .iter()
            .filter_map(|(key, value)| value.as_str().map(|value| (key.clone(), value.to_string())))
            .collect(),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| {
                let name = item.get("name")?.as_str()?;
                let value = item.get("value")?.as_str()?;
                Some((name.to_string(), value.to_string()))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn exit_status_from_status(status: std::process::ExitStatus) -> TerminalExitStatus {
    TerminalExitStatus {
        exit_code: status.code(),
        signal: exit_status_signal(&status),
    }
}

#[cfg(unix)]
fn exit_status_signal(status: &std::process::ExitStatus) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;
    status.signal().map(|signal| signal.to_string())
}

#[cfg(not(unix))]
fn exit_status_signal(_status: &std::process::ExitStatus) -> Option<String> {
    None
}

async fn push_task_update(
    client: &reqwest::Client,
    relay_url: &str,
    task_id: &str,
    auth: &AgentAuthState,
    payload: AppendTaskEventsRequest,
) -> Result<()> {
    let endpoint = format!("{relay_url}/api/tasks/{task_id}/events");
    let device_credential = auth.device_credential().await;
    with_bearer(client.post(endpoint), device_credential.as_deref())
        .json(&payload)
        .send()
        .await
        .context("failed to push task update")?
        .error_for_status()
        .context("relay rejected task update")?;

    Ok(())
}

async fn fetch_task_detail(
    client: &reqwest::Client,
    relay_url: &str,
    task_id: &str,
    auth: &AgentAuthState,
) -> Result<TaskDetailResponse> {
    let endpoint = format!("{relay_url}/api/tasks/{task_id}");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.get(endpoint), device_credential.as_deref())
        .send()
        .await
        .context("failed to fetch task detail")?
        .error_for_status()
        .context("relay rejected task detail request")?
        .json::<TaskDetailResponse>()
        .await
        .context("invalid task detail response")?;

    Ok(response)
}

async fn is_task_cancel_requested(
    client: &reqwest::Client,
    relay_url: &str,
    task_id: &str,
    auth: &AgentAuthState,
) -> Result<bool> {
    let detail = fetch_task_detail(client, relay_url, task_id, auth).await?;
    Ok(matches!(detail.task.status, TaskStatus::CancelRequested))
}

async fn drain_remaining_output<S>(
    sink: &mut S,
    provider_kind: &ProviderKind,
    stdout_lines: &mut Option<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>,
    stderr_lines: &mut Option<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>,
) -> Result<()>
where
    S: TaskExecutionSink,
{
    while let Some(line) = next_line(stdout_lines).await? {
        for event in provider_stdout_to_task_events(provider_kind, &line) {
            sink.push_event(event.kind, event.message).await?;
        }
    }

    while let Some(line) = next_line(stderr_lines).await? {
        sink.push_event(TaskEventKind::ProviderStderr, line).await?;
    }

    Ok(())
}

async fn next_line<R>(lines: &mut Option<tokio::io::Lines<BufReader<R>>>) -> Result<Option<String>>
where
    R: tokio::io::AsyncRead + Unpin,
{
    match lines {
        Some(lines) => lines
            .next_line()
            .await
            .context("failed to read provider output"),
        None => Ok(None),
    }
}
