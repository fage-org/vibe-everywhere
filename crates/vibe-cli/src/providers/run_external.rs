use std::process::Stdio;

use anyhow::{Context, Result, bail};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
    sync::mpsc,
};

use crate::{
    agent::adapters::{normalize_output_line, render_event_for_output},
    agent::core::{ProviderKind, ProviderProcessInfo},
    sandbox::{SandboxManager, SandboxMode},
};

pub async fn run_external_provider(
    binary: &str,
    kind: ProviderKind,
    provider_session_id: String,
    prompt: String,
    working_dir: String,
    resume: bool,
    sandbox_mode: SandboxMode,
    output_sender: Option<mpsc::Sender<String>>,
    started_sender: Option<mpsc::Sender<ProviderProcessInfo>>,
) -> Result<String> {
    std::fs::create_dir_all(&working_dir)
        .with_context(|| format!("failed to create working directory {working_dir}"))?;
    let mut command = if binary.ends_with(".sh") {
        let mut command = Command::new("bash");
        command.arg(binary);
        command
    } else {
        Command::new(binary)
    };
    match kind {
        ProviderKind::Codex => {
            command.arg("exec");
            if resume {
                command.arg("resume").arg(&provider_session_id);
            } else {
                command.arg("--json").arg(&prompt);
            }
        }
        _ => {
            command.arg(&prompt);
            if resume {
                command.arg("--resume").arg(&provider_session_id);
            } else {
                command.arg("--session-id").arg(&provider_session_id);
            }
        }
    }
    SandboxManager::new(sandbox_mode).apply(kind, &mut command, &working_dir)?;

    let mut child = command
        .current_dir(&working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to run {binary}"))?;
    if let Some(sender) = started_sender
        && let Some(provider_pid) = child.id()
    {
        let _ = sender.send(ProviderProcessInfo { provider_pid }).await;
    }

    let stdout = child
        .stdout
        .take()
        .context("failed to capture provider stdout")?;
    let stderr = child
        .stderr
        .take()
        .context("failed to capture provider stderr")?;

    let stderr_task = tokio::spawn(async move {
        let mut stderr = BufReader::new(stderr);
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).await?;
        Ok::<String, std::io::Error>(String::from_utf8_lossy(&bytes).trim().to_string())
    });

    let mut stdout_lines = BufReader::new(stdout).lines();
    let mut collected = Vec::new();
    let mut sender = output_sender;
    while let Some(line) = stdout_lines.next_line().await? {
        if let Some(display_line) = render_event_for_output(&normalize_output_line(&line)) {
            collected.push(display_line);
        }
        if let Some(channel) = sender.as_mut() {
            let _ = channel.send(line).await;
        }
    }

    let status = child.wait().await?;
    let stderr = stderr_task.await.context("stderr reader task failed")??;
    let output = collected.join("\n");
    if !status.success() {
        bail!(
            "{} command failed (exit {}): {}",
            kind.as_str(),
            status.code().unwrap_or(1),
            if stderr.is_empty() {
                if output.is_empty() {
                    "no error output".to_string()
                } else {
                    output.clone()
                }
            } else {
                stderr
            }
        );
    }
    Ok(output.trim().to_string())
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt};

    use tempfile::TempDir;

    use crate::config::Config;

    use super::run_external_provider;

    fn make_script(temp_dir: &TempDir, name: &str, body: &str) -> String {
        let path = temp_dir.path().join(name);
        fs::write(&path, body).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path.display().to_string()
    }

    #[tokio::test]
    async fn run_external_provider_passes_session_id_for_resume() {
        let temp_dir = TempDir::new().unwrap();
        let script = make_script(
            &temp_dir,
            "mock-provider.sh",
            r#"#!/usr/bin/env bash
set -euo pipefail
echo "$*"
"#,
        );
        let mut config = Config::from_sources(
            Some("http://127.0.0.1:3005".into()),
            Some("https://app.vibe.engineering".into()),
            Some(temp_dir.path().as_os_str().to_owned()),
            Some(script.clone()),
        )
        .unwrap();
        config.codex_bin = script.clone();

        let output = run_external_provider(
            &config.codex_bin,
            crate::agent::core::ProviderKind::Codex,
            "provider-session".into(),
            "hello".into(),
            temp_dir.path().join("work").display().to_string(),
            true,
            crate::sandbox::SandboxMode::Disabled,
            None,
            None,
        )
        .await
        .unwrap();

        assert!(output.contains("resume"));
        assert!(output.contains("provider-session"));
    }
}
