use anyhow::{Context, Result, bail};
use std::{
    env,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{RwLock, watch},
    task::JoinHandle,
};
use vibe_core::{TaskBridgeEvent, TaskBridgeRequest};

const DEFAULT_TASK_BRIDGE_PORT: u16 = 19_092;

pub struct TaskBridgeRuntime {
    shutdown_tx: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl TaskBridgeRuntime {
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true);
        let _ = self.task.await;
    }
}

pub fn start_task_bridge_server(
    enabled: bool,
    working_root: PathBuf,
    shared_token: Arc<RwLock<Option<String>>>,
    shared: super::SharedState,
) -> Option<TaskBridgeRuntime> {
    if !enabled {
        return None;
    }

    let port = task_bridge_port();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let task = tokio::spawn(async move {
        if let Err(error) = run_server(port, working_root, shared_token, shared, shutdown_rx).await
        {
            eprintln!("task bridge server stopped: {error:#}");
        }
    });

    Some(TaskBridgeRuntime { shutdown_tx, task })
}

fn task_bridge_port() -> u16 {
    env::var("VIBE_AGENT_TASK_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|port| *port > 0)
        .unwrap_or(DEFAULT_TASK_BRIDGE_PORT)
}

async fn run_server(
    port: u16,
    working_root: PathBuf,
    shared_token: Arc<RwLock<Option<String>>>,
    shared: super::SharedState,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let bind_addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind task bridge on {bind_addr}"))?;
    eprintln!("task bridge listening on {bind_addr}");

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => break,
            accepted = listener.accept() => {
                let (stream, remote_addr) = accepted
                    .with_context(|| format!("failed to accept task bridge connection on {bind_addr}"))?;
                let working_root = working_root.clone();
                let shared_token = shared_token.clone();
                let shared = shared.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(stream, working_root, shared_token, shared).await {
                        eprintln!("task bridge connection {remote_addr} failed: {error:#}");
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
    shared: super::SharedState,
) -> Result<()> {
    let (read_half, write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();

    let Some(line) = lines
        .next_line()
        .await
        .context("failed to read task bridge start frame")?
    else {
        return Ok(());
    };
    let request = serde_json::from_str::<TaskBridgeRequest>(&line)
        .context("invalid task bridge start frame")?;
    let task = match request {
        TaskBridgeRequest::Start { token, task } => {
            let shared_token = shared_token.read().await.clone();
            if shared_token.as_deref() != token.as_deref() {
                let mut sink =
                    BridgeTaskExecutionSink::new(write_half, Arc::new(AtomicBool::new(false)));
                sink.send_error("task bridge token rejected").await?;
                return Ok(());
            }
            task
        }
        TaskBridgeRequest::Cancel => {
            let mut sink =
                BridgeTaskExecutionSink::new(write_half, Arc::new(AtomicBool::new(false)));
            sink.send_error("expected task bridge start request")
                .await?;
            return Ok(());
        }
    };

    if !super::task_runtime::try_mark_local_task_started(&shared.current_task_id, &task.id).await {
        let mut sink = BridgeTaskExecutionSink::new(write_half, Arc::new(AtomicBool::new(false)));
        sink.send_error("another task is already running on this agent")
            .await?;
        return Ok(());
    }

    let cancel_requested = Arc::new(AtomicBool::new(false));
    let mut sink = BridgeTaskExecutionSink::new(write_half, cancel_requested.clone());
    let task_id = task.id.clone();
    let reader_cancel = cancel_requested.clone();
    let reader = tokio::spawn(async move {
        loop {
            match lines.next_line().await {
                Ok(Some(line)) => {
                    let request = serde_json::from_str::<TaskBridgeRequest>(&line)
                        .context("invalid task bridge request frame")?;
                    match request {
                        TaskBridgeRequest::Cancel => {
                            reader_cancel.store(true, Ordering::Relaxed);
                        }
                        TaskBridgeRequest::Start { .. } => {
                            bail!("unexpected task bridge start request after session start");
                        }
                    }
                }
                Ok(None) => {
                    reader_cancel.store(true, Ordering::Relaxed);
                    return Ok::<(), anyhow::Error>(());
                }
                Err(error) => {
                    reader_cancel.store(true, Ordering::Relaxed);
                    return Err(error).context("failed to read task bridge request");
                }
            }
        }
    });

    let result = super::task_runtime::execute_task_with_sink(
        &mut sink,
        &shared.providers,
        &working_root,
        task,
    )
    .await;
    cancel_requested.store(true, Ordering::Relaxed);
    reader.abort();
    super::task_runtime::clear_local_task(&shared.current_task_id, &task_id).await;

    if let Err(error) = result {
        let _ = sink
            .send_error(format!("task execution failed: {error}"))
            .await;
        return Err(error);
    }

    Ok(())
}

struct BridgeTaskExecutionSink<W> {
    writer: W,
    cancel_requested: Arc<AtomicBool>,
}

impl<W> BridgeTaskExecutionSink<W> {
    fn new(writer: W, cancel_requested: Arc<AtomicBool>) -> Self {
        Self {
            writer,
            cancel_requested,
        }
    }
}

impl<W> BridgeTaskExecutionSink<W>
where
    W: AsyncWrite + Unpin,
{
    async fn send_error(&mut self, message: impl Into<String>) -> Result<()> {
        send_event(
            &mut self.writer,
            &TaskBridgeEvent::Error {
                message: message.into(),
            },
        )
        .await
    }
}

impl<W> super::task_runtime::TaskExecutionSink for BridgeTaskExecutionSink<W>
where
    W: AsyncWrite + Unpin,
{
    async fn push_update(
        &mut self,
        update: super::task_runtime::TaskExecutionUpdate,
    ) -> Result<()> {
        send_event(
            &mut self.writer,
            &TaskBridgeEvent::Update {
                status: update.status,
                execution_protocol: update.execution_protocol,
                provider_session_id: update.provider_session_id,
                events: update.events,
                exit_code: update.exit_code,
                error: update.error,
            },
        )
        .await
    }

    async fn is_cancel_requested(&mut self) -> Result<bool> {
        Ok(self.cancel_requested.load(Ordering::Relaxed))
    }
}

async fn send_event<W>(writer: &mut W, event: &TaskBridgeEvent) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut payload = serde_json::to_string(event).context("failed to encode task bridge event")?;
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .await
        .context("failed to write task bridge event")?;
    writer
        .flush()
        .await
        .context("failed to flush task bridge event")?;
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::{
        collections::BTreeMap,
        fs,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
        time::Duration,
    };
    use tokio::{
        io::{AsyncRead, AsyncWriteExt},
        sync::RwLock,
    };
    use uuid::Uuid;
    use vibe_core::{
        CreateTaskRequest, ExecutionProtocol, OverlayNetworkStatus, ProviderKind, ProviderStatus,
        TaskEventKind, TaskRecord, TaskStatus, TaskTransportKind,
    };

    fn test_local_tcp_host() -> String {
        if let Some(host) = std::env::var("VIBE_TEST_TCP_HOST")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        {
            return host;
        }

        let socket = std::net::UdpSocket::bind(("0.0.0.0", 0)).unwrap();
        socket.connect(("8.8.8.8", 53)).unwrap();
        socket.local_addr().unwrap().ip().to_string()
    }

    fn reserve_test_port(host: &str) -> u16 {
        std::net::TcpListener::bind((host, 0))
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    fn write_fake_codex_binary(root: &Path) -> PathBuf {
        let path = root.join("fake-codex.sh");
        let script = r#"#!/bin/sh
printf '%s\n' '{"type":"thread.started","thread_id":"thread_test"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"task bridge ok"}}'
"#;
        fs::write(&path, script).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    fn test_shared_state(command: String) -> crate::SharedState {
        crate::SharedState {
            current_task_id: std::sync::Arc::new(RwLock::new(None)),
            metadata: BTreeMap::new(),
            providers: vec![ProviderStatus {
                kind: ProviderKind::Codex,
                command,
                available: true,
                version: Some("test".to_string()),
                execution_protocol: ExecutionProtocol::Cli,
                supports_acp: false,
                error: None,
            }],
            overlay: std::sync::Arc::new(RwLock::new(OverlayNetworkStatus::default())),
        }
    }

    fn test_task() -> TaskRecord {
        TaskRecord::new(
            CreateTaskRequest {
                device_id: "device-1".to_string(),
                conversation_id: None,
                provider: ProviderKind::Codex,
                prompt: "say hi".to_string(),
                cwd: None,
                model: None,
                title: Some("task bridge test".to_string()),
                provider_session_id: None,
            },
            ExecutionProtocol::Cli,
            TaskTransportKind::OverlayProxy,
            &vibe_core::ActorIdentity::personal_owner(),
        )
    }

    async fn connect_with_retry(host: &str, port: u16) -> TcpStream {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                match TcpStream::connect((host, port)).await {
                    Ok(stream) => return stream,
                    Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
                }
            }
        })
        .await
        .unwrap()
    }

    async fn write_request_for_test<W>(writer: &mut W, request: &TaskBridgeRequest)
    where
        W: AsyncWrite + Unpin,
    {
        let mut payload = serde_json::to_string(request).unwrap();
        payload.push('\n');
        writer.write_all(payload.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    }

    async fn read_event_for_test<R>(
        lines: &mut tokio::io::Lines<BufReader<R>>,
    ) -> Option<TaskBridgeEvent>
    where
        R: AsyncRead + Unpin,
    {
        let line = lines.next_line().await.unwrap()?;
        Some(serde_json::from_str::<TaskBridgeEvent>(&line).unwrap())
    }

    #[tokio::test]
    async fn task_bridge_executes_cli_task_and_streams_updates() {
        let host = test_local_tcp_host();
        let port = reserve_test_port(&host);
        let root = std::env::temp_dir().join(format!("vibe-agent-task-bridge-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let command = write_fake_codex_binary(&root);
        let shared = test_shared_state(command.to_string_lossy().to_string());
        let current_task_id = shared.current_task_id.clone();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let server_root = root.clone();
        let server_shared = shared.clone();
        let server = tokio::spawn(async move {
            run_server(
                port,
                server_root,
                Arc::new(RwLock::new(Some("shared-secret".to_string()))),
                server_shared,
                shutdown_rx,
            )
            .await
            .unwrap();
        });

        let stream = connect_with_retry(&host, port).await;
        let (read_half, mut write_half) = stream.into_split();
        let mut lines = BufReader::new(read_half).lines();
        write_request_for_test(
            &mut write_half,
            &TaskBridgeRequest::Start {
                token: Some("shared-secret".to_string()),
                task: test_task(),
            },
        )
        .await;

        let mut saw_running = false;
        let mut saw_assistant_delta = false;
        let mut saw_succeeded = false;

        while let Some(event) =
            tokio::time::timeout(Duration::from_secs(2), read_event_for_test(&mut lines))
                .await
                .unwrap()
        {
            match event {
                TaskBridgeEvent::Update {
                    status,
                    events,
                    exit_code,
                    ..
                } => {
                    saw_running |= status == Some(TaskStatus::Running);
                    saw_succeeded |= status == Some(TaskStatus::Succeeded);
                    saw_assistant_delta |= events.iter().any(|event| {
                        event.kind == TaskEventKind::AssistantDelta
                            && event.message == "task bridge ok"
                    });
                    if status == Some(TaskStatus::Succeeded) {
                        assert_eq!(exit_code, Some(0));
                        break;
                    }
                }
                TaskBridgeEvent::Error { message } => panic!("unexpected bridge error: {message}"),
            }
        }

        assert!(saw_running);
        assert!(saw_assistant_delta);
        assert!(saw_succeeded);
        assert!(current_task_id.read().await.is_none());

        drop(lines);
        let _ = shutdown_tx.send(true);
        server.await.unwrap();
        let _ = fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn task_bridge_rejects_invalid_token() {
        let host = test_local_tcp_host();
        let port = reserve_test_port(&host);
        let root = std::env::temp_dir().join(format!("vibe-agent-task-bridge-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let command = write_fake_codex_binary(&root);
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        let server = tokio::spawn(async move {
            run_server(
                port,
                root,
                Arc::new(RwLock::new(Some("shared-secret".to_string()))),
                test_shared_state(command.to_string_lossy().to_string()),
                shutdown_rx,
            )
            .await
            .unwrap();
        });

        let stream = connect_with_retry(&host, port).await;
        let (read_half, mut write_half) = stream.into_split();
        let mut lines = BufReader::new(read_half).lines();
        write_request_for_test(
            &mut write_half,
            &TaskBridgeRequest::Start {
                token: Some("wrong-secret".to_string()),
                task: test_task(),
            },
        )
        .await;

        let event = tokio::time::timeout(Duration::from_secs(2), read_event_for_test(&mut lines))
            .await
            .unwrap()
            .unwrap();
        match event {
            TaskBridgeEvent::Error { message } => {
                assert!(message.contains("token rejected"));
            }
            other => panic!("unexpected bridge event: {other:?}"),
        }

        drop(lines);
        let _ = shutdown_tx.send(true);
        server.await.unwrap();
    }
}
