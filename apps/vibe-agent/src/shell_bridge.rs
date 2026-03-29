use anyhow::{Context, Result};
use std::{env, path::PathBuf, process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{RwLock, mpsc, watch},
    task::JoinHandle,
};
use vibe_core::{ShellBridgeEvent, ShellBridgeRequest, ShellStreamKind};

const DEFAULT_SHELL_BRIDGE_PORT: u16 = 19_090;
const SHELL_BRIDGE_POLL_MS: u64 = 100;

pub struct ShellBridgeRuntime {
    shutdown_tx: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl ShellBridgeRuntime {
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true);
        let _ = self.task.await;
    }
}

pub fn start_shell_bridge_server(
    enabled: bool,
    working_root: PathBuf,
    shared_token: Arc<RwLock<Option<String>>>,
) -> Option<ShellBridgeRuntime> {
    if !enabled {
        return None;
    }

    let port = shell_bridge_port();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let task = tokio::spawn(async move {
        if let Err(error) = run_server(port, working_root, shared_token, shutdown_rx).await {
            eprintln!("shell bridge server stopped: {error:#}");
        }
    });

    Some(ShellBridgeRuntime { shutdown_tx, task })
}

fn shell_bridge_port() -> u16 {
    env::var("VIBE_AGENT_SHELL_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|port| *port > 0)
        .unwrap_or(DEFAULT_SHELL_BRIDGE_PORT)
}

async fn run_server(
    port: u16,
    working_root: PathBuf,
    shared_token: Arc<RwLock<Option<String>>>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let bind_addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind shell bridge on {bind_addr}"))?;
    eprintln!("shell bridge listening on {bind_addr}");

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => break,
            accepted = listener.accept() => {
                let (stream, remote_addr) = accepted
                    .with_context(|| format!("failed to accept shell bridge connection on {bind_addr}"))?;
                let working_root = working_root.clone();
                let shared_token = shared_token.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(stream, working_root, shared_token).await {
                        eprintln!("shell bridge connection {remote_addr} failed: {error:#}");
                    }
                });
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    stream: TcpStream,
    working_root: PathBuf,
    shared_token: Arc<RwLock<Option<String>>>,
) -> Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    let Some(line) = lines
        .next_line()
        .await
        .context("failed to read shell bridge start frame")?
    else {
        return Ok(());
    };
    let request = serde_json::from_str::<ShellBridgeRequest>(&line)
        .context("invalid shell bridge start frame")?;
    let (cwd, _session_id) = match request {
        ShellBridgeRequest::Start {
            token,
            session_id,
            cwd,
        } => {
            let shared_token = shared_token.read().await.clone();
            if shared_token.as_deref() != token.as_deref() {
                send_event(
                    &mut write_half,
                    &ShellBridgeEvent::Error {
                        message: "shell bridge token rejected".to_string(),
                    },
                )
                .await?;
                return Ok(());
            }
            (cwd, session_id)
        }
        _ => {
            send_event(
                &mut write_half,
                &ShellBridgeEvent::Error {
                    message: "expected shell bridge start request".to_string(),
                },
            )
            .await?;
            return Ok(());
        }
    };

    let cwd = super::resolve_task_cwd(&working_root, cwd.as_deref());
    let cwd = match super::ensure_task_cwd(&cwd) {
        Ok(cwd) => cwd,
        Err(error) => {
            send_event(
                &mut write_half,
                &ShellBridgeEvent::Error {
                    message: error.to_string(),
                },
            )
            .await?;
            return Ok(());
        }
    };

    let (mut command, shell_label) = super::shell_runtime::build_relay_shell_command();
    command
        .current_dir(&cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            send_event(
                &mut write_half,
                &ShellBridgeEvent::Error {
                    message: format!("failed to spawn shell: {error}"),
                },
            )
            .await?;
            return Ok(());
        }
    };

    let mut stdin = match child.stdin.take() {
        Some(stdin) => stdin,
        None => {
            send_event(
                &mut write_half,
                &ShellBridgeEvent::Error {
                    message: "shell stdin unavailable".to_string(),
                },
            )
            .await?;
            return Ok(());
        }
    };
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            send_event(
                &mut write_half,
                &ShellBridgeEvent::Error {
                    message: "shell stdout unavailable".to_string(),
                },
            )
            .await?;
            return Ok(());
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            send_event(
                &mut write_half,
                &ShellBridgeEvent::Error {
                    message: "shell stderr unavailable".to_string(),
                },
            )
            .await?;
            return Ok(());
        }
    };

    let (output_tx, mut output_rx) = mpsc::unbounded_channel();
    spawn_shell_reader(stdout, ShellStreamKind::Stdout, output_tx.clone());
    spawn_shell_reader(stderr, ShellStreamKind::Stderr, output_tx);

    send_event(
        &mut write_half,
        &ShellBridgeEvent::Started {
            shell: shell_label,
            cwd: cwd.display().to_string(),
        },
    )
    .await?;

    let mut interval = tokio::time::interval(Duration::from_millis(SHELL_BRIDGE_POLL_MS));
    let mut close_requested = false;
    let mut close_sent = false;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                if close_requested && !close_sent {
                    let _ = child.kill().await;
                    close_sent = true;
                }

                if let Some(exit_status) = child.try_wait().context("failed to poll shell bridge child")? {
                    let error = if close_requested || exit_status.success() {
                        None
                    } else {
                        Some(format!("shell exited with {exit_status}"))
                    };
                    send_event(
                        &mut write_half,
                        &ShellBridgeEvent::Exited {
                            exit_code: exit_status.code(),
                            close_requested,
                            error,
                        },
                    )
                    .await?;
                    break;
                }
            }
            request = lines.next_line() => {
                match request.context("failed to read shell bridge request")? {
                    Some(line) => {
                        let request = serde_json::from_str::<ShellBridgeRequest>(&line)
                            .context("invalid shell bridge request frame")?;
                        match request {
                            ShellBridgeRequest::Input { data } => {
                                stdin
                                    .write_all(data.as_bytes())
                                    .await
                                    .context("failed to write shell bridge stdin")?;
                                let _ = stdin.flush().await;
                            }
                            ShellBridgeRequest::Close => {
                                close_requested = true;
                            }
                            ShellBridgeRequest::Start { .. } => {}
                        }
                    }
                    None => {
                        close_requested = true;
                    }
                }
            }
            output = output_rx.recv() => {
                let Some(event) = output else {
                    continue;
                };
                send_event(&mut write_half, &event).await?;
            }
        }
    }

    Ok(())
}

fn spawn_shell_reader<R>(
    reader: R,
    stream: ShellStreamKind,
    tx: mpsc::UnboundedSender<ShellBridgeEvent>,
) where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut reader = reader;
        let mut buffer = [0_u8; 4096];
        loop {
            match tokio::io::AsyncReadExt::read(&mut reader, &mut buffer).await {
                Ok(0) => break,
                Ok(size) => {
                    let data = String::from_utf8_lossy(&buffer[..size]).to_string();
                    if tx
                        .send(ShellBridgeEvent::Output {
                            stream: stream.clone(),
                            data,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(error) => {
                    let _ = tx.send(ShellBridgeEvent::Error {
                        message: format!("shell stream read failed: {error}"),
                    });
                    break;
                }
            }
        }
    });
}

async fn send_event<W>(writer: &mut W, event: &ShellBridgeEvent) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut payload =
        serde_json::to_string(event).context("failed to encode shell bridge event")?;
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .await
        .context("failed to write shell bridge event")?;
    writer
        .flush()
        .await
        .context("failed to flush shell bridge event")?;
    Ok(())
}
