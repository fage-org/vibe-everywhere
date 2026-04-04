use std::process::Stdio;

use anyhow::{Context, Result, bail};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
    sync::mpsc,
};

use crate::{
    agent::adapters::{normalize_output_line, render_event_for_output},
    agent::core::{
        ProviderBackend, ProviderKind, ProviderProcessInfo, ProviderRunRequest, ProviderRunResult,
    },
    config::Config,
    sandbox::{SandboxManager, SandboxMode},
};

#[derive(Debug, Clone)]
pub struct ClaudeRunRequest {
    pub provider_session_id: String,
    pub prompt: String,
    pub working_dir: String,
    pub resume: bool,
    pub sandbox_mode: SandboxMode,
    pub output_sender: Option<mpsc::Sender<String>>,
    pub started_sender: Option<mpsc::Sender<ProviderProcessInfo>>,
}

pub struct ClaudeBackend<'a> {
    config: &'a Config,
}

impl<'a> ClaudeBackend<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
}

impl ProviderBackend for ClaudeBackend<'_> {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Claude
    }

    async fn run(&self, request: ProviderRunRequest) -> Result<ProviderRunResult> {
        let output = run_claude(
            self.config,
            ClaudeRunRequest {
                provider_session_id: request.provider_session_id,
                prompt: request.prompt,
                working_dir: request.working_dir,
                resume: request.resume,
                sandbox_mode: request.sandbox_mode,
                output_sender: request.output_sender,
                started_sender: request.started_sender,
            },
        )
        .await?;
        Ok(ProviderRunResult { output })
    }
}

pub async fn run_claude(config: &Config, request: ClaudeRunRequest) -> Result<String> {
    std::fs::create_dir_all(&request.working_dir)
        .with_context(|| format!("failed to create working directory {}", request.working_dir))?;
    let mut command = Command::new(&config.claude_bin);
    command.arg("-p");
    if request.resume {
        command.arg("--resume").arg(&request.provider_session_id);
    } else {
        command
            .arg("--session-id")
            .arg(&request.provider_session_id);
    }
    SandboxManager::new(request.sandbox_mode).apply(
        ProviderKind::Claude,
        &mut command,
        &request.working_dir,
    )?;
    command
        .arg(&request.prompt)
        .current_dir(&request.working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("failed to run {}", config.claude_bin))?;
    if let Some(sender) = request.started_sender
        && let Some(provider_pid) = child.id()
    {
        let _ = sender.send(ProviderProcessInfo { provider_pid }).await;
    }

    let stdout = child
        .stdout
        .take()
        .context("failed to capture Claude stdout")?;
    let stderr = child
        .stderr
        .take()
        .context("failed to capture Claude stderr")?;

    let stderr_task = tokio::spawn(async move {
        let mut stderr = BufReader::new(stderr);
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).await?;
        Ok::<String, std::io::Error>(String::from_utf8_lossy(&bytes).trim().to_string())
    });

    let mut lines = BufReader::new(stdout).lines();
    let mut collected = Vec::new();
    let mut output_sender = request.output_sender;
    while let Some(line) = lines.next_line().await? {
        if let Some(display_line) = render_event_for_output(&normalize_output_line(&line)) {
            collected.push(display_line);
        }
        if let Some(sender) = output_sender.as_mut() {
            let _ = sender.send(line).await;
        }
    }

    let status = child.wait().await?;
    let stderr = stderr_task.await.context("stderr reader task failed")??;
    let stdout = collected.join("\n");
    if !status.success() {
        let detail = if !stderr.is_empty() { stderr } else { stdout };
        bail!(
            "Claude command failed (exit {}): {}",
            status.code().unwrap_or(1),
            if detail.is_empty() {
                "no error output".to_string()
            } else {
                detail
            }
        );
    }

    Ok(stdout.trim().to_string())
}
