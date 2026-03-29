use anyhow::{Context, Result, bail};
use std::{env, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{RwLock, watch},
    task::JoinHandle,
};
use vibe_core::{PortForwardBridgeEvent, PortForwardBridgeRequest};

const DEFAULT_PORT_FORWARD_BRIDGE_PORT: u16 = 19_091;
const MAX_BRIDGE_FRAME_BYTES: usize = 8 * 1024;

pub struct PortForwardBridgeRuntime {
    shutdown_tx: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl PortForwardBridgeRuntime {
    pub async fn shutdown(self) {
        let _ = self.shutdown_tx.send(true);
        let _ = self.task.await;
    }
}

pub fn start_port_forward_bridge_server(
    enabled: bool,
    shared_token: Arc<RwLock<Option<String>>>,
) -> Option<PortForwardBridgeRuntime> {
    if !enabled {
        return None;
    }

    let port = port_forward_bridge_port();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let task = tokio::spawn(async move {
        if let Err(error) = run_server(port, shared_token, shutdown_rx).await {
            eprintln!("port forward bridge server stopped: {error:#}");
        }
    });

    Some(PortForwardBridgeRuntime { shutdown_tx, task })
}

fn port_forward_bridge_port() -> u16 {
    env::var("VIBE_AGENT_PORT_FORWARD_BRIDGE_PORT")
        .ok()
        .and_then(|value| value.trim().parse::<u16>().ok())
        .filter(|port| *port > 0)
        .unwrap_or(DEFAULT_PORT_FORWARD_BRIDGE_PORT)
}

async fn run_server(
    port: u16,
    shared_token: Arc<RwLock<Option<String>>>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let bind_addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind port forward bridge on {bind_addr}"))?;
    eprintln!("port forward bridge listening on {bind_addr}");

    loop {
        tokio::select! {
            _ = shutdown_rx.changed() => break,
            accepted = listener.accept() => {
                let (stream, remote_addr) = accepted
                    .with_context(|| format!("failed to accept port forward bridge connection on {bind_addr}"))?;
                let shared_token = shared_token.clone();
                tokio::spawn(async move {
                    if let Err(error) = handle_connection(stream, shared_token).await {
                        eprintln!("port forward bridge connection {remote_addr} failed: {error:#}");
                    }
                });
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    mut stream: TcpStream,
    shared_token: Arc<RwLock<Option<String>>>,
) -> Result<()> {
    let Some(line) = read_frame_line(&mut stream).await? else {
        return Ok(());
    };
    let request = serde_json::from_str::<PortForwardBridgeRequest>(&line)
        .context("invalid port forward bridge request frame")?;

    let (target_host, target_port, _forward_id) = match request {
        PortForwardBridgeRequest::Start {
            token,
            forward_id,
            target_host,
            target_port,
        } => {
            let shared_token = shared_token.read().await.clone();
            if shared_token.as_deref() != token.as_deref() {
                send_event(
                    &mut stream,
                    &PortForwardBridgeEvent::Error {
                        message: "port forward bridge token rejected".to_string(),
                    },
                )
                .await?;
                return Ok(());
            }
            (target_host, target_port, forward_id)
        }
    };

    let mut target = match TcpStream::connect((target_host.as_str(), target_port)).await {
        Ok(target) => target,
        Err(error) => {
            send_event(
                &mut stream,
                &PortForwardBridgeEvent::Error {
                    message: format!(
                        "failed to connect target {}:{}: {}",
                        target_host, target_port, error
                    ),
                },
            )
            .await?;
            return Ok(());
        }
    };

    send_event(&mut stream, &PortForwardBridgeEvent::Ready).await?;
    tokio::io::copy_bidirectional(&mut stream, &mut target)
        .await
        .context("port forward bridge stream copy failed")?;

    Ok(())
}

async fn read_frame_line(stream: &mut TcpStream) -> Result<Option<String>> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1];

    loop {
        let size = stream
            .read(&mut buffer)
            .await
            .context("failed to read port forward bridge frame")?;
        if size == 0 {
            if bytes.is_empty() {
                return Ok(None);
            }
            bail!("port forward bridge frame ended before newline");
        }

        if buffer[0] == b'\n' {
            break;
        }

        bytes.push(buffer[0]);
        if bytes.len() > MAX_BRIDGE_FRAME_BYTES {
            bail!("port forward bridge frame exceeded {MAX_BRIDGE_FRAME_BYTES} bytes");
        }
    }

    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }

    let line = String::from_utf8(bytes).context("port forward bridge frame is not valid UTF-8")?;
    Ok(Some(line))
}

async fn send_event<W>(writer: &mut W, event: &PortForwardBridgeEvent) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut payload =
        serde_json::to_string(event).context("failed to encode port forward bridge event")?;
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .await
        .context("failed to write port forward bridge event")?;
    writer
        .flush()
        .await
        .context("failed to flush port forward bridge event")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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

    async fn read_event(stream: &mut TcpStream) -> Option<PortForwardBridgeEvent> {
        let line = read_frame_line(stream).await.unwrap()?;
        Some(serde_json::from_str::<PortForwardBridgeEvent>(&line).unwrap())
    }

    async fn write_request(stream: &mut TcpStream, request: &PortForwardBridgeRequest) {
        let mut payload = serde_json::to_string(request).unwrap();
        payload.push('\n');
        stream.write_all(payload.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    }

    #[tokio::test]
    async fn handle_connection_proxies_target_traffic() {
        let host = test_local_tcp_host();
        let target_listener = TcpListener::bind((host.as_str(), 0)).await.unwrap();
        let target_addr = target_listener.local_addr().unwrap();
        let bridge_listener = TcpListener::bind((host.as_str(), 0)).await.unwrap();
        let bridge_addr = bridge_listener.local_addr().unwrap();

        let target_task = tokio::spawn(async move {
            let (mut stream, _) = target_listener.accept().await.unwrap();
            let mut payload = Vec::new();
            stream.read_to_end(&mut payload).await.unwrap();
            assert_eq!(payload, b"bridge-smoke");
            stream.write_all(b"target:bridge-smoke").await.unwrap();
        });

        let bridge_task = tokio::spawn(async move {
            let (stream, _) = bridge_listener.accept().await.unwrap();
            handle_connection(
                stream,
                Arc::new(RwLock::new(Some("shared-token".to_string()))),
            )
            .await
            .unwrap();
        });

        let mut client = TcpStream::connect(bridge_addr).await.unwrap();
        write_request(
            &mut client,
            &PortForwardBridgeRequest::Start {
                token: Some("shared-token".to_string()),
                forward_id: "forward-1".to_string(),
                target_host: target_addr.ip().to_string(),
                target_port: target_addr.port(),
            },
        )
        .await;

        let event = read_event(&mut client).await.unwrap();
        assert_eq!(event, PortForwardBridgeEvent::Ready);

        client.write_all(b"bridge-smoke").await.unwrap();
        client.shutdown().await.unwrap();
        let mut reply = Vec::new();
        client.read_to_end(&mut reply).await.unwrap();
        assert_eq!(reply, b"target:bridge-smoke");

        target_task.await.unwrap();
        bridge_task.await.unwrap();
    }

    #[tokio::test]
    async fn handle_connection_rejects_invalid_token() {
        let host = test_local_tcp_host();
        let listener = TcpListener::bind((host.as_str(), 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();

        let bridge_task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            handle_connection(
                stream,
                Arc::new(RwLock::new(Some("shared-token".to_string()))),
            )
            .await
            .unwrap();
        });

        let mut client = TcpStream::connect(addr).await.unwrap();
        write_request(
            &mut client,
            &PortForwardBridgeRequest::Start {
                token: Some("wrong-token".to_string()),
                forward_id: "forward-1".to_string(),
                target_host: host,
                target_port: 22,
            },
        )
        .await;

        let event = tokio::time::timeout(Duration::from_secs(1), read_event(&mut client))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            event,
            PortForwardBridgeEvent::Error {
                message: "port forward bridge token rejected".to_string(),
            }
        );

        bridge_task.await.unwrap();
    }
}
