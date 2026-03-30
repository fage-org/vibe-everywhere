use super::*;

enum ClaimNextShellSessionOutcome {
    Session(Option<ShellSessionRecord>),
    Unsupported,
}

struct ShellOutputMessage {
    stream: ShellStreamKind,
    data: String,
}

pub(crate) async fn shell_session_loop(
    client: reqwest::Client,
    relay_url: String,
    profile: AgentProfile,
    auth: AgentAuthState,
    _shared: SharedState,
    working_root: PathBuf,
    poll_interval_ms: u64,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms));

    loop {
        interval.tick().await;

        match claim_next_shell_session(&client, &relay_url, &profile.device_id, &auth).await {
            Ok(ClaimNextShellSessionOutcome::Session(Some(session))) => {
                let session_client = client.clone();
                let session_relay_url = relay_url.clone();
                let session_device_id = profile.device_id.clone();
                let session_auth = auth.clone();
                let session_working_root = working_root.clone();
                tokio::spawn(async move {
                    let session_id = session.id.clone();
                    if let Err(error) = run_shell_session(
                        session_client,
                        session_relay_url,
                        session_device_id,
                        session_auth,
                        session_working_root,
                        session,
                    )
                    .await
                    {
                        eprintln!("shell session {session_id} failed: {error:#}");
                    }
                });
            }
            Ok(ClaimNextShellSessionOutcome::Session(None)) => {}
            Ok(ClaimNextShellSessionOutcome::Unsupported) => {
                println!("[agent] relay does not expose shell control APIs; disabling shell loop");
                return Ok(());
            }
            Err(error) => {
                eprintln!("failed to claim shell session: {error:#}");
            }
        }
    }
}

async fn claim_next_shell_session(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
    auth: &AgentAuthState,
) -> Result<ClaimNextShellSessionOutcome> {
    let endpoint = format!("{relay_url}/api/devices/{device_id}/shell/claim-next");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.post(endpoint), device_credential.as_deref())
        .send()
        .await
        .context("failed to claim shell session")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ClaimNextShellSessionOutcome::Unsupported);
    }

    let response = response
        .error_for_status()
        .context("relay rejected shell session claim")?
        .json::<ClaimShellSessionResponse>()
        .await
        .context("failed to decode claim shell session response")?;

    Ok(ClaimNextShellSessionOutcome::Session(response.session))
}

async fn run_shell_session(
    client: reqwest::Client,
    relay_url: String,
    device_id: String,
    auth: AgentAuthState,
    working_root: PathBuf,
    session: ShellSessionRecord,
) -> Result<()> {
    let cwd = resolve_task_cwd(&working_root, session.cwd.as_deref());
    let cwd = match ensure_task_cwd(&cwd) {
        Ok(cwd) => cwd,
        Err(error) => {
            report_shell_session_start_failure(
                &client,
                &relay_url,
                &session.id,
                &device_id,
                &auth,
                error.to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    let (mut command, shell_label) = build_relay_shell_command();
    command
        .current_dir(&cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            report_shell_session_start_failure(
                &client,
                &relay_url,
                &session.id,
                &device_id,
                &auth,
                format!("failed to spawn shell: {error}"),
            )
            .await?;
            return Ok(());
        }
    };

    let mut stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            report_shell_session_start_failure(
                &client,
                &relay_url,
                &session.id,
                &device_id,
                &auth,
                "shell stdin unavailable".to_string(),
            )
            .await?;
            return Ok(());
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            report_shell_session_start_failure(
                &client,
                &relay_url,
                &session.id,
                &device_id,
                &auth,
                "shell stdout unavailable".to_string(),
            )
            .await?;
            return Ok(());
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            report_shell_session_start_failure(
                &client,
                &relay_url,
                &session.id,
                &device_id,
                &auth,
                "shell stderr unavailable".to_string(),
            )
            .await?;
            return Ok(());
        }
    };

    let (output_tx, mut output_rx) = mpsc::unbounded_channel();
    spawn_shell_reader(stdout, ShellStreamKind::Stdout, output_tx.clone());
    spawn_shell_reader(stderr, ShellStreamKind::Stderr, output_tx);

    append_shell_output_update(
        &client,
        &relay_url,
        &session.id,
        &auth,
        AppendShellOutputRequest {
            device_id: device_id.clone(),
            status: Some(ShellSessionStatus::Active),
            outputs: vec![
                ShellOutputChunkInput {
                    stream: ShellStreamKind::System,
                    data: format!("Relay shell started via {}", shell_label),
                },
                ShellOutputChunkInput {
                    stream: ShellStreamKind::System,
                    data: format!("cwd={}", cwd.display()),
                },
            ],
            exit_code: None,
            error: None,
        },
    )
    .await?;

    let mut last_input_seq = 0;
    let mut interval = tokio::time::interval(Duration::from_millis(TERMINAL_POLL_MS));

    loop {
        interval.tick().await;

        let pending =
            fetch_shell_pending_input(&client, &relay_url, &session.id, last_input_seq, &auth)
                .await?;
        for input in pending.inputs {
            stdin
                .write_all(input.data.as_bytes())
                .await
                .with_context(|| format!("failed to write shell input for {}", session.id))?;
            let _ = stdin.flush().await;
            last_input_seq = input.seq;
        }

        let close_requested = pending.session.close_requested
            || matches!(
                pending.session.status,
                ShellSessionStatus::CloseRequested | ShellSessionStatus::Closed
            );
        if close_requested {
            let _ = child.kill().await;
        }

        let outputs = drain_shell_output(&mut output_rx);
        if !outputs.is_empty() {
            append_shell_output_update(
                &client,
                &relay_url,
                &session.id,
                &auth,
                AppendShellOutputRequest {
                    device_id: device_id.clone(),
                    status: None,
                    outputs,
                    exit_code: None,
                    error: None,
                },
            )
            .await?;
        }

        if let Some(exit_status) = child
            .try_wait()
            .with_context(|| format!("failed to poll shell session {}", session.id))?
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let mut final_outputs = drain_shell_output(&mut output_rx);
            final_outputs.push(ShellOutputChunkInput {
                stream: ShellStreamKind::System,
                data: format!("Shell session ended ({exit_status})"),
            });
            let final_status = if close_requested {
                ShellSessionStatus::Closed
            } else if exit_status.success() {
                ShellSessionStatus::Succeeded
            } else {
                ShellSessionStatus::Failed
            };
            let error = if close_requested || exit_status.success() {
                None
            } else {
                Some(format!("shell exited with {exit_status}"))
            };
            append_shell_output_update(
                &client,
                &relay_url,
                &session.id,
                &auth,
                AppendShellOutputRequest {
                    device_id: device_id.clone(),
                    status: Some(final_status),
                    outputs: final_outputs,
                    exit_code: exit_status.code(),
                    error,
                },
            )
            .await?;
            break;
        }
    }

    Ok(())
}

async fn report_shell_session_start_failure(
    client: &reqwest::Client,
    relay_url: &str,
    session_id: &str,
    device_id: &str,
    auth: &AgentAuthState,
    message: String,
) -> Result<()> {
    append_shell_output_update(
        client,
        relay_url,
        session_id,
        auth,
        AppendShellOutputRequest {
            device_id: device_id.to_string(),
            status: Some(ShellSessionStatus::Failed),
            outputs: vec![ShellOutputChunkInput {
                stream: ShellStreamKind::System,
                data: message.clone(),
            }],
            exit_code: None,
            error: Some(message),
        },
    )
    .await
}

async fn fetch_shell_pending_input(
    client: &reqwest::Client,
    relay_url: &str,
    session_id: &str,
    after_seq: u64,
    auth: &AgentAuthState,
) -> Result<ShellPendingInputResponse> {
    let endpoint =
        format!("{relay_url}/api/shell/sessions/{session_id}/input?afterSeq={after_seq}");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.get(endpoint), device_credential.as_deref())
        .send()
        .await
        .context("failed to fetch shell session input")?
        .error_for_status()
        .context("relay rejected shell session input request")?
        .json::<ShellPendingInputResponse>()
        .await
        .context("invalid shell session input response")?;

    Ok(response)
}

async fn append_shell_output_update(
    client: &reqwest::Client,
    relay_url: &str,
    session_id: &str,
    auth: &AgentAuthState,
    payload: AppendShellOutputRequest,
) -> Result<()> {
    if payload.outputs.is_empty()
        && payload.status.is_none()
        && payload.exit_code.is_none()
        && payload.error.is_none()
    {
        return Ok(());
    }

    let endpoint = format!("{relay_url}/api/shell/sessions/{session_id}/output");
    let device_credential = auth.device_credential().await;
    with_bearer(client.post(endpoint), device_credential.as_deref())
        .json(&payload)
        .send()
        .await
        .context("failed to append shell session output")?
        .error_for_status()
        .context("relay rejected shell session output")?;

    Ok(())
}

pub(crate) fn build_relay_shell_command() -> (Command, String) {
    if cfg!(windows) {
        let shell = std::env::var("COMSPEC")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "cmd.exe".to_string());
        let mut command = Command::new(&shell);
        command.arg("/Q");
        (command, shell)
    } else {
        let shell = std::env::var("SHELL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "/bin/sh".to_string());
        let mut command = Command::new(&shell);
        command.arg("-i");
        (command, shell)
    }
}

fn spawn_shell_reader<R>(
    reader: R,
    stream: ShellStreamKind,
    tx: mpsc::UnboundedSender<ShellOutputMessage>,
) where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut reader = reader;
        let mut buffer = [0_u8; 4096];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(size) => {
                    let data = String::from_utf8_lossy(&buffer[..size]).to_string();
                    if tx
                        .send(ShellOutputMessage {
                            stream: stream.clone(),
                            data,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(error) => {
                    let _ = tx.send(ShellOutputMessage {
                        stream: ShellStreamKind::System,
                        data: format!("shell stream read failed: {error}"),
                    });
                    break;
                }
            }
        }
    });
}

fn drain_shell_output(
    rx: &mut mpsc::UnboundedReceiver<ShellOutputMessage>,
) -> Vec<ShellOutputChunkInput> {
    let mut outputs = Vec::new();
    while let Ok(message) = rx.try_recv() {
        outputs.push(ShellOutputChunkInput {
            stream: message.stream,
            data: message.data,
        });
    }
    outputs
}
