use super::*;

enum ClaimNextPortForwardOutcome {
    Forward(Option<PortForwardRecord>),
    Unsupported,
}

enum PortForwardTunnelSessionOutcome {
    Reconnect,
    Closed,
}

struct ActiveTargetStream {
    connection_id: u64,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

enum TargetStreamEvent {
    Data {
        connection_id: u64,
        data: Vec<u8>,
    },
    Closed {
        connection_id: u64,
        message: Option<String>,
    },
}

pub(crate) async fn port_forward_loop(
    client: reqwest::Client,
    relay_url: String,
    profile: AgentProfile,
    auth: AgentAuthState,
    _shared: SharedState,
    poll_interval_ms: u64,
) -> Result<()> {
    let active_forwards = Arc::new(Mutex::new(HashSet::new()));
    let mut interval = tokio::time::interval(Duration::from_millis(poll_interval_ms.max(1_000)));

    loop {
        interval.tick().await;

        if let Err(error) = resume_port_forwards(
            client.clone(),
            relay_url.clone(),
            profile.device_id.clone(),
            auth.clone(),
            active_forwards.clone(),
            poll_interval_ms,
        )
        .await
        {
            eprintln!("failed to resume port forwards: {error:#}");
        }

        match claim_next_port_forward(&client, &relay_url, &profile.device_id, &auth).await {
            Ok(ClaimNextPortForwardOutcome::Forward(Some(forward))) => {
                spawn_port_forward_worker(
                    client.clone(),
                    relay_url.clone(),
                    profile.device_id.clone(),
                    auth.clone(),
                    active_forwards.clone(),
                    forward,
                    poll_interval_ms,
                )
                .await;
            }
            Ok(ClaimNextPortForwardOutcome::Forward(None)) => {}
            Ok(ClaimNextPortForwardOutcome::Unsupported) => {
                println!(
                    "[agent] relay does not expose port-forward control APIs; disabling port-forward loop"
                );
                return Ok(());
            }
            Err(error) => {
                eprintln!("failed to claim port forward: {error:#}");
            }
        }
    }
}

async fn resume_port_forwards(
    client: reqwest::Client,
    relay_url: String,
    device_id: String,
    auth: AgentAuthState,
    active_forwards: Arc<Mutex<HashSet<String>>>,
    poll_interval_ms: u64,
) -> Result<()> {
    let mut forwards = list_port_forwards_by_status(
        &client,
        &relay_url,
        &device_id,
        PortForwardStatus::Active,
        &auth,
    )
    .await?;
    forwards.extend(
        list_port_forwards_by_status(
            &client,
            &relay_url,
            &device_id,
            PortForwardStatus::CloseRequested,
            &auth,
        )
        .await?,
    );

    for forward in forwards {
        spawn_port_forward_worker(
            client.clone(),
            relay_url.clone(),
            device_id.clone(),
            auth.clone(),
            active_forwards.clone(),
            forward,
            poll_interval_ms,
        )
        .await;
    }

    Ok(())
}

async fn spawn_port_forward_worker(
    client: reqwest::Client,
    relay_url: String,
    device_id: String,
    auth: AgentAuthState,
    active_forwards: Arc<Mutex<HashSet<String>>>,
    forward: PortForwardRecord,
    poll_interval_ms: u64,
) {
    if forward.transport != PortForwardTransportKind::RelayTunnel {
        return;
    }

    let forward_id = forward.id.clone();
    {
        let mut active = active_forwards.lock().await;
        if !active.insert(forward_id.clone()) {
            return;
        }
    }

    tokio::spawn(async move {
        if let Err(error) = supervise_port_forward(
            client,
            relay_url,
            device_id,
            auth,
            forward,
            poll_interval_ms,
        )
        .await
        {
            eprintln!("port forward {forward_id} failed: {error:#}");
        }

        active_forwards.lock().await.remove(&forward_id);
    });
}

async fn claim_next_port_forward(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
    auth: &AgentAuthState,
) -> Result<ClaimNextPortForwardOutcome> {
    let endpoint = format!("{relay_url}/api/devices/{device_id}/port-forwards/claim-next");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.post(endpoint), device_credential.as_deref())
        .send()
        .await
        .context("failed to claim port forward")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(ClaimNextPortForwardOutcome::Unsupported);
    }

    let response = response
        .error_for_status()
        .context("relay rejected port-forward claim")?
        .json::<ClaimPortForwardResponse>()
        .await
        .context("failed to decode claim port-forward response")?;

    Ok(ClaimNextPortForwardOutcome::Forward(response.forward))
}

async fn list_port_forwards_by_status(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
    status: PortForwardStatus,
    auth: &AgentAuthState,
) -> Result<Vec<PortForwardRecord>> {
    let endpoint = format!("{relay_url}/api/port-forwards");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.get(endpoint), device_credential.as_deref())
        .query(&[
            ("deviceId", device_id.to_string()),
            (
                "status",
                port_forward_status_query_value(&status).to_string(),
            ),
            ("limit", "100".to_string()),
        ])
        .send()
        .await
        .context("failed to list port forwards")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(Vec::new());
    }

    let response = response
        .error_for_status()
        .context("relay rejected port-forward list")?
        .json::<Vec<PortForwardRecord>>()
        .await
        .context("failed to decode port-forward list response")?;

    Ok(response)
}

fn port_forward_status_query_value(status: &PortForwardStatus) -> &'static str {
    match status {
        PortForwardStatus::Pending => "pending",
        PortForwardStatus::Active => "active",
        PortForwardStatus::CloseRequested => "close_requested",
        PortForwardStatus::Closed => "closed",
        PortForwardStatus::Failed => "failed",
    }
}

async fn get_port_forward_detail(
    client: &reqwest::Client,
    relay_url: &str,
    forward_id: &str,
    auth: &AgentAuthState,
) -> Result<Option<PortForwardRecord>> {
    let endpoint = format!("{relay_url}/api/port-forwards/{forward_id}");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.get(endpoint), device_credential.as_deref())
        .send()
        .await
        .context("failed to fetch port-forward detail")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let response = response
        .error_for_status()
        .context("relay rejected port-forward detail request")?
        .json::<PortForwardDetailResponse>()
        .await
        .context("failed to decode port-forward detail response")?;

    Ok(Some(response.forward))
}

async fn report_port_forward_state(
    client: &reqwest::Client,
    relay_url: &str,
    forward_id: &str,
    payload: ReportPortForwardStateRequest,
    auth: &AgentAuthState,
) -> Result<Option<PortForwardRecord>> {
    let endpoint = format!("{relay_url}/api/port-forwards/{forward_id}/report");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.post(endpoint), device_credential.as_deref())
        .json(&payload)
        .send()
        .await
        .context("failed to report port-forward state")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let response = response
        .error_for_status()
        .context("relay rejected port-forward state report")?
        .json::<PortForwardDetailResponse>()
        .await
        .context("failed to decode port-forward state report")?;

    Ok(Some(response.forward))
}

async fn supervise_port_forward(
    client: reqwest::Client,
    relay_url: String,
    device_id: String,
    auth: AgentAuthState,
    forward: PortForwardRecord,
    poll_interval_ms: u64,
) -> Result<()> {
    let reconnect_delay = Duration::from_millis(poll_interval_ms.max(1_000));
    let mut current_forward = forward;

    loop {
        let Some(latest_forward) =
            get_port_forward_detail(&client, &relay_url, &current_forward.id, &auth).await?
        else {
            return Ok(());
        };
        current_forward = latest_forward;

        match current_forward.status {
            PortForwardStatus::Closed | PortForwardStatus::Failed => return Ok(()),
            PortForwardStatus::CloseRequested => {
                let _ = report_port_forward_state(
                    &client,
                    &relay_url,
                    &current_forward.id,
                    ReportPortForwardStateRequest {
                        device_id: device_id.clone(),
                        status: Some(PortForwardStatus::Closed),
                        error: None,
                        clear_error: true,
                    },
                    &auth,
                )
                .await;
                return Ok(());
            }
            PortForwardStatus::Pending => {
                tokio::time::sleep(reconnect_delay).await;
                continue;
            }
            PortForwardStatus::Active => {}
        }

        match run_port_forward_tunnel_session(
            &client,
            &relay_url,
            &device_id,
            &auth,
            &current_forward,
            poll_interval_ms,
        )
        .await
        {
            Ok(PortForwardTunnelSessionOutcome::Closed) => return Ok(()),
            Ok(PortForwardTunnelSessionOutcome::Reconnect) => {
                tokio::time::sleep(reconnect_delay).await;
            }
            Err(error) => {
                eprintln!(
                    "port forward {} tunnel session failed: {error:#}",
                    current_forward.id
                );
                tokio::time::sleep(reconnect_delay).await;
            }
        }
    }
}

async fn run_port_forward_tunnel_session(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
    auth: &AgentAuthState,
    forward: &PortForwardRecord,
    poll_interval_ms: u64,
) -> Result<PortForwardTunnelSessionOutcome> {
    let device_credential = auth.device_credential().await;
    let ws_url = build_relay_websocket_url(
        relay_url,
        &format!("/api/port-forwards/{}/tunnel/ws", forward.id),
        device_id,
        device_credential.as_deref(),
    )?;
    let (mut ws_stream, _) = connect_async(ws_url.as_str())
        .await
        .with_context(|| format!("failed to connect tunnel websocket for {}", forward.id))?;

    let (target_events_tx, mut target_events_rx) = mpsc::unbounded_channel::<TargetStreamEvent>();
    let mut active_target: Option<ActiveTargetStream> = None;
    let mut next_connection_id = 0_u64;
    let mut detail_interval =
        tokio::time::interval(Duration::from_millis(poll_interval_ms.max(1_000)));

    loop {
        tokio::select! {
            message = ws_stream.next() => {
                match message {
                    Some(Ok(WsMessage::Text(payload))) => {
                        match serde_json::from_str::<PortForwardTunnelControl>(&payload) {
                            Ok(PortForwardTunnelControl::ClientConnected) => {
                                close_active_target_stream(&mut active_target).await;
                                next_connection_id += 1;
                                let connection_id = next_connection_id;
                                match TcpStream::connect((forward.target_host.as_str(), forward.target_port)).await {
                                    Ok(stream) => {
                                        let (reader, writer) = stream.into_split();
                                        active_target = Some(ActiveTargetStream {
                                            connection_id,
                                            writer,
                                        });
                                        spawn_target_stream_reader(reader, connection_id, target_events_tx.clone());
                                        let _ = report_port_forward_state(
                                            client,
                                            relay_url,
                                            &forward.id,
                                            ReportPortForwardStateRequest {
                                                device_id: device_id.to_string(),
                                                status: None,
                                                error: None,
                                                clear_error: true,
                                            },
                                            auth,
                                        )
                                        .await;
                                        send_port_forward_tunnel_control(
                                            &mut ws_stream,
                                            PortForwardTunnelControl::TargetConnected,
                                        )
                                        .await?;
                                    }
                                    Err(error) => {
                                        let message = format!(
                                            "failed to connect target {}:{}: {}",
                                            forward.target_host, forward.target_port, error
                                        );
                                        let _ = report_port_forward_state(
                                            client,
                                            relay_url,
                                            &forward.id,
                                            ReportPortForwardStateRequest {
                                                device_id: device_id.to_string(),
                                                status: None,
                                                error: Some(message.clone()),
                                                clear_error: false,
                                            },
                                            auth,
                                        )
                                        .await;
                                        send_port_forward_tunnel_control(
                                            &mut ws_stream,
                                            PortForwardTunnelControl::TargetConnectFailed { message },
                                        )
                                        .await?;
                                    }
                                }
                            }
                            Ok(PortForwardTunnelControl::ClientClosed) => {
                                close_active_target_stream(&mut active_target).await;
                            }
                            Ok(PortForwardTunnelControl::TargetConnected)
                            | Ok(PortForwardTunnelControl::TargetConnectFailed { .. })
                            | Ok(PortForwardTunnelControl::TargetClosed { .. }) => {}
                            Err(error) => {
                                eprintln!(
                                    "port forward {} control parse failed: {}",
                                    forward.id, error
                                );
                            }
                        }
                    }
                    Some(Ok(WsMessage::Binary(data))) => {
                        if let Some(target) = active_target.as_mut()
                            && let Err(error) = target.writer.write_all(&data).await
                        {
                            let message = format!(
                                "failed to write target {}:{}: {}",
                                forward.target_host, forward.target_port, error
                            );
                            close_active_target_stream(&mut active_target).await;
                            let _ = report_port_forward_state(
                                client,
                                relay_url,
                                &forward.id,
                                ReportPortForwardStateRequest {
                                    device_id: device_id.to_string(),
                                    status: None,
                                    error: Some(message.clone()),
                                    clear_error: false,
                                },
                                auth,
                            )
                            .await;
                            send_port_forward_tunnel_control(
                                &mut ws_stream,
                                PortForwardTunnelControl::TargetClosed {
                                    message: Some(message),
                                },
                            )
                            .await?;
                        }
                    }
                    Some(Ok(WsMessage::Ping(payload))) => {
                        ws_stream
                            .send(WsMessage::Pong(payload))
                            .await
                            .context("failed to reply to tunnel ping")?;
                    }
                    Some(Ok(WsMessage::Close(_))) | None => {
                        close_active_target_stream(&mut active_target).await;
                        return Ok(PortForwardTunnelSessionOutcome::Reconnect);
                    }
                    Some(Ok(WsMessage::Pong(_))) => {}
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        close_active_target_stream(&mut active_target).await;
                        return Err(error).context("tunnel websocket stream failed");
                    }
                }
            }
            target_event = target_events_rx.recv() => {
                match target_event {
                    Some(TargetStreamEvent::Data { connection_id, data }) => {
                        if active_target
                            .as_ref()
                            .is_some_and(|target| target.connection_id == connection_id)
                        {
                            ws_stream
                                .send(WsMessage::Binary(data.into()))
                                .await
                                .context("failed to send target bytes to relay tunnel")?;
                        }
                    }
                    Some(TargetStreamEvent::Closed { connection_id, message }) => {
                        if active_target
                            .as_ref()
                            .is_some_and(|target| target.connection_id == connection_id)
                        {
                            close_active_target_stream(&mut active_target).await;
                            if let Some(error_message) = message.clone() {
                                let _ = report_port_forward_state(
                                    client,
                                    relay_url,
                                    &forward.id,
                                    ReportPortForwardStateRequest {
                                        device_id: device_id.to_string(),
                                        status: None,
                                        error: Some(error_message.clone()),
                                        clear_error: false,
                                    },
                                    auth,
                                )
                                .await;
                            }
                            send_port_forward_tunnel_control(
                                &mut ws_stream,
                                PortForwardTunnelControl::TargetClosed { message },
                            )
                            .await?;
                        }
                    }
                    None => {
                        close_active_target_stream(&mut active_target).await;
                        return Ok(PortForwardTunnelSessionOutcome::Reconnect);
                    }
                }
            }
            _ = detail_interval.tick() => {
                let Some(detail) = get_port_forward_detail(client, relay_url, &forward.id, auth).await? else {
                    close_active_target_stream(&mut active_target).await;
                    let _ = ws_stream.close(None).await;
                    return Ok(PortForwardTunnelSessionOutcome::Closed);
                };

                match detail.status {
                    PortForwardStatus::CloseRequested => {
                        close_active_target_stream(&mut active_target).await;
                        let _ = report_port_forward_state(
                            client,
                            relay_url,
                            &forward.id,
                            ReportPortForwardStateRequest {
                                device_id: device_id.to_string(),
                                status: Some(PortForwardStatus::Closed),
                                error: None,
                                clear_error: true,
                            },
                            auth,
                        )
                        .await;
                        let _ = ws_stream.close(None).await;
                        return Ok(PortForwardTunnelSessionOutcome::Closed);
                    }
                    PortForwardStatus::Closed | PortForwardStatus::Failed => {
                        close_active_target_stream(&mut active_target).await;
                        let _ = ws_stream.close(None).await;
                        return Ok(PortForwardTunnelSessionOutcome::Closed);
                    }
                    PortForwardStatus::Pending | PortForwardStatus::Active => {}
                }
            }
        }
    }
}

fn spawn_target_stream_reader(
    mut reader: tokio::net::tcp::OwnedReadHalf,
    connection_id: u64,
    tx: mpsc::UnboundedSender<TargetStreamEvent>,
) {
    tokio::spawn(async move {
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => {
                    let _ = tx.send(TargetStreamEvent::Closed {
                        connection_id,
                        message: None,
                    });
                    break;
                }
                Ok(size) => {
                    if tx
                        .send(TargetStreamEvent::Data {
                            connection_id,
                            data: buffer[..size].to_vec(),
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(error) => {
                    let _ = tx.send(TargetStreamEvent::Closed {
                        connection_id,
                        message: Some(format!("target stream read failed: {error}")),
                    });
                    break;
                }
            }
        }
    });
}

async fn close_active_target_stream(active_target: &mut Option<ActiveTargetStream>) {
    if let Some(mut target) = active_target.take() {
        let _ = target.writer.shutdown().await;
    }
}

async fn send_port_forward_tunnel_control(
    ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    control: PortForwardTunnelControl,
) -> Result<()> {
    let payload = serde_json::to_string(&control).context("failed to encode tunnel control")?;
    ws_stream
        .send(WsMessage::Text(payload.into()))
        .await
        .context("failed to send tunnel control")?;
    Ok(())
}
