use super::*;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct PortForwardListQuery {
    pub(super) device_id: Option<String>,
    pub(super) status: Option<PortForwardStatus>,
    pub(super) limit: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct PortForwardTunnelQuery {
    pub(super) device_id: Option<String>,
}

#[derive(Debug)]
struct RelayPortForwardClient {
    connection_id: u64,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

struct ActiveOverlayPortForwardClient {
    connection_id: u64,
    task: JoinHandle<()>,
}

#[derive(Debug)]
enum RelayPortForwardClientEvent {
    Data { connection_id: u64, data: Vec<u8> },
    Closed { connection_id: u64 },
}

#[derive(Debug)]
enum OverlayPortForwardClientEvent {
    Ready {
        connection_id: u64,
    },
    Closed {
        connection_id: u64,
        error: Option<String>,
        fallback_to_relay_tunnel: bool,
    },
}

pub(super) async fn list_port_forwards(
    State(state): State<AppState>,
    Query(query): Query<PortForwardListQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<PortForwardRecord>>, ApiError> {
    let actor = actor_from_headers(&state, &headers);
    ensure_actor_can_read(&actor)?;

    let store = state.store.read().await;
    let mut forwards = store
        .port_forwards
        .values()
        .map(|entry| entry.record.clone())
        .filter(|forward| forward.tenant_id == actor.tenant_id)
        .filter(|forward| {
            query
                .device_id
                .as_deref()
                .is_none_or(|device_id| forward.device_id == device_id)
        })
        .filter(|forward| {
            query
                .status
                .as_ref()
                .is_none_or(|status| &forward.status == status)
        })
        .collect::<Vec<_>>();
    forwards.sort_by(|left, right| {
        right
            .created_at_epoch_ms
            .cmp(&left.created_at_epoch_ms)
            .then_with(|| left.id.cmp(&right.id))
    });

    if let Some(limit) = query.limit {
        forwards.truncate(limit.min(PORT_FORWARD_LIST_LIMIT_MAX));
    }

    Ok(Json(forwards))
}

pub(super) async fn create_port_forward(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreatePortForwardRequest>,
) -> Result<Json<CreatePortForwardResponse>, ApiError> {
    let actor = actor_from_headers(&state, &headers);
    ensure_actor_can_write(&actor)?;

    let (forward, snapshot, start_overlay_proxy) = {
        let mut store = state.store.write().await;
        let device = store
            .devices
            .get(&payload.device_id)
            .cloned()
            .ok_or_else(|| ApiError::not_found("device_not_found", "Device not found"))?;
        ensure_tenant_access(&actor, &device.tenant_id)?;

        let relay_port = reserve_forward_port(
            &store,
            state.config.forward_port_start,
            state.config.forward_port_end,
        )
        .ok_or_else(|| {
            ApiError::conflict(
                "forward_port_exhausted",
                "No relay forward port is currently available",
            )
        })?;

        let transport = preferred_port_forward_transport(&device);
        let forward = PortForwardRecord::new(
            payload,
            state.config.forward_host.clone(),
            relay_port,
            transport.clone(),
            &actor,
        );
        let start_overlay_proxy = transport == PortForwardTransportKind::OverlayProxy;
        store.port_forwards.insert(
            forward.id.clone(),
            PortForwardEntry {
                record: forward.clone(),
            },
        );
        (forward, store.clone(), start_overlay_proxy)
    };

    persist_snapshot(&state, &snapshot)?;
    record_audit(
        &state,
        &actor,
        AuditAction::PreviewCreated,
        "preview",
        forward.id.clone(),
        AuditOutcome::Succeeded,
        None,
    )
    .await?;

    if start_overlay_proxy {
        let overlay_state = state.clone();
        let forward_id = forward.id.clone();
        tokio::spawn(async move {
            run_overlay_port_forward(overlay_state, forward_id).await;
        });
    }

    Ok(Json(CreatePortForwardResponse { forward }))
}

pub(super) async fn get_port_forward(
    Path(forward_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PortForwardDetailResponse>, ApiError> {
    let actor = actor_from_headers(&state, &headers);
    ensure_actor_can_read(&actor)?;

    let store = state.store.read().await;
    let Some(entry) = store.port_forwards.get(&forward_id) else {
        return Err(ApiError::not_found(
            "port_forward_not_found",
            "Port forward not found",
        ));
    };
    ensure_tenant_access(&actor, &entry.record.tenant_id)?;

    Ok(Json(PortForwardDetailResponse {
        forward: entry.record.clone(),
    }))
}

pub(super) async fn report_port_forward_state(
    Path(forward_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReportPortForwardStateRequest>,
) -> Result<Json<PortForwardDetailResponse>, ApiError> {
    let actor = actor_from_headers(&state, &headers);
    ensure_actor_can_write(&actor)?;

    let device_id = {
        let store = state.store.read().await;
        let Some(entry) = store.port_forwards.get(&forward_id) else {
            return Err(ApiError::not_found(
                "port_forward_not_found",
                "Port forward not found",
            ));
        };
        ensure_tenant_access(&actor, &entry.record.tenant_id)?;
        entry.record.device_id.clone()
    };

    if device_id != payload.device_id {
        return Err(ApiError::conflict(
            "device_mismatch",
            "Port forward device does not match report source",
        ));
    }

    let forward = apply_port_forward_report_internal(
        &state,
        &forward_id,
        payload.status,
        payload.error,
        payload.clear_error,
    )
    .await?
    .ok_or_else(|| ApiError::not_found("port_forward_not_found", "Port forward not found"))?;

    Ok(Json(PortForwardDetailResponse { forward }))
}

pub(super) async fn port_forward_tunnel_websocket(
    Path(forward_id): Path<String>,
    Query(query): Query<PortForwardTunnelQuery>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let actor = actor_from_headers(&state, &headers);
    ensure_actor_can_read(&actor)?;

    let device_id = query
        .device_id
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            ApiError::bad_request(
                "device_id_required",
                "deviceId is required for port forward tunnels",
            )
        })?;

    let forward = {
        let store = state.store.read().await;
        let Some(entry) = store.port_forwards.get(&forward_id) else {
            return Err(ApiError::not_found(
                "port_forward_not_found",
                "Port forward not found",
            ));
        };
        ensure_tenant_access(&actor, &entry.record.tenant_id)?;
        if entry.record.device_id != device_id {
            return Err(ApiError::conflict(
                "device_mismatch",
                "Port forward device does not match tunnel source",
            ));
        }
        if entry.record.transport != PortForwardTransportKind::RelayTunnel {
            return Err(ApiError::conflict(
                "port_forward_transport_mismatch",
                "Port forward is managed by the relay overlay proxy",
            ));
        }
        if !matches!(
            entry.record.status,
            PortForwardStatus::Active | PortForwardStatus::CloseRequested
        ) {
            return Err(ApiError::conflict(
                "port_forward_inactive",
                "Port forward is not currently active",
            ));
        }
        entry.record.clone()
    };

    Ok(ws.on_upgrade(move |socket| async move {
        handle_port_forward_tunnel_socket(state, forward, socket).await;
    }))
}

async fn handle_port_forward_tunnel_socket(
    state: AppState,
    forward: PortForwardRecord,
    mut socket: WebSocket,
) {
    let bind_address = format!("{}:{}", state.config.forward_bind_host, forward.relay_port);
    let listener = match TcpListener::bind(&bind_address).await {
        Ok(listener) => {
            let _ = apply_port_forward_report_internal(&state, &forward.id, None, None, true).await;
            listener
        }
        Err(error) => {
            let message = format!("failed to bind relay listener on {bind_address}: {error}");
            eprintln!("port forward {} tunnel bind failed: {message}", forward.id);
            let _ =
                apply_port_forward_report_internal(&state, &forward.id, None, Some(message), false)
                    .await;
            return;
        }
    };

    let (accepted_tx, mut accepted_rx) = mpsc::unbounded_channel::<TcpStream>();
    let accept_forward_id = forward.id.clone();
    let accept_task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    if accepted_tx.send(stream).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    eprintln!(
                        "port forward {} accept loop stopped: {}",
                        accept_forward_id, error
                    );
                    break;
                }
            }
        }
    });

    let (client_events_tx, mut client_events_rx) =
        mpsc::unbounded_channel::<RelayPortForwardClientEvent>();
    let mut active_client: Option<RelayPortForwardClient> = None;
    let mut next_connection_id = 0_u64;
    let mut status_interval = tokio::time::interval(Duration::from_millis(1_000));

    loop {
        tokio::select! {
            accepted = accepted_rx.recv() => {
                let Some(stream) = accepted else {
                    break;
                };

                if active_client.is_some() {
                    let mut stream = stream;
                    let _ = stream.shutdown().await;
                    continue;
                }

                next_connection_id += 1;
                let connection_id = next_connection_id;
                let (reader, writer) = stream.into_split();
                active_client = Some(RelayPortForwardClient {
                    connection_id,
                    writer,
                });
                spawn_port_forward_tcp_reader(reader, connection_id, client_events_tx.clone());

                if send_port_forward_control(
                    &mut socket,
                    &PortForwardTunnelControl::ClientConnected,
                )
                .await
                .is_err()
                {
                    break;
                }
            }
            event = client_events_rx.recv() => {
                match event {
                    Some(RelayPortForwardClientEvent::Data { connection_id, data }) => {
                        if active_client
                            .as_ref()
                            .is_some_and(|client| client.connection_id == connection_id)
                            && socket.send(Message::Binary(data.into())).await.is_err()
                        {
                            break;
                        }
                    }
                    Some(RelayPortForwardClientEvent::Closed { connection_id }) => {
                        if active_client
                            .as_ref()
                            .is_some_and(|client| client.connection_id == connection_id)
                        {
                            close_active_port_forward_client(&mut active_client).await;
                            if send_port_forward_control(
                                &mut socket,
                                &PortForwardTunnelControl::ClientClosed,
                            )
                            .await
                            .is_err()
                            {
                                break;
                            }
                        }
                    }
                    None => break,
                }
            }
            message = socket.next() => {
                match message {
                    Some(Ok(Message::Binary(data))) => {
                        if let Some(client) = active_client.as_mut()
                            && client.writer.write_all(&data).await.is_err()
                        {
                            close_active_port_forward_client(&mut active_client).await;
                            if send_port_forward_control(
                                &mut socket,
                                &PortForwardTunnelControl::ClientClosed,
                            )
                            .await
                            .is_err()
                            {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Text(payload))) => {
                        match serde_json::from_str::<PortForwardTunnelControl>(&payload) {
                            Ok(PortForwardTunnelControl::TargetConnectFailed { .. })
                            | Ok(PortForwardTunnelControl::TargetClosed { .. }) => {
                                close_active_port_forward_client(&mut active_client).await;
                            }
                            Ok(PortForwardTunnelControl::TargetConnected)
                            | Ok(PortForwardTunnelControl::ClientConnected)
                            | Ok(PortForwardTunnelControl::ClientClosed) => {}
                            Err(error) => {
                                eprintln!(
                                    "port forward {} control parse failed: {}",
                                    forward.id, error
                                );
                            }
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        eprintln!("port forward {} websocket failed: {}", forward.id, error);
                        break;
                    }
                }
            }
            _ = status_interval.tick() => {
                match load_port_forward_record(&state, &forward.id).await {
                    Some(record) if matches!(
                        record.status,
                        PortForwardStatus::CloseRequested
                            | PortForwardStatus::Closed
                            | PortForwardStatus::Failed
                    ) => {
                        close_active_port_forward_client(&mut active_client).await;
                        let _ = socket.send(Message::Close(None)).await;
                        break;
                    }
                    Some(_) => {}
                    None => break,
                }
            }
        }
    }

    accept_task.abort();
    close_active_port_forward_client(&mut active_client).await;
}

pub(super) async fn claim_next_port_forward(
    Path(device_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ClaimPortForwardResponse>, ApiError> {
    let actor = actor_from_headers(&state, &headers);
    ensure_actor_can_write(&actor)?;

    let mut store = state.store.write().await;
    let Some(device) = store.devices.get(&device_id) else {
        return Err(ApiError::not_found(
            "device_not_found",
            "Device not found; register device first",
        ));
    };
    ensure_tenant_access(&actor, &device.tenant_id)?;

    let mut next_forwards = store
        .port_forwards
        .values()
        .filter(|entry| {
            entry.record.device_id == device_id
                && entry.record.tenant_id == actor.tenant_id
                && entry.record.status == PortForwardStatus::Pending
                && entry.record.transport == PortForwardTransportKind::RelayTunnel
        })
        .map(|entry| entry.record.clone())
        .collect::<Vec<_>>();
    next_forwards.sort_by(|left, right| {
        left.created_at_epoch_ms
            .cmp(&right.created_at_epoch_ms)
            .then_with(|| left.id.cmp(&right.id))
    });

    let Some(forward) = next_forwards.into_iter().next() else {
        return Ok(Json(ClaimPortForwardResponse { forward: None }));
    };

    if let Some(entry) = store.port_forwards.get_mut(&forward.id) {
        entry.record.status = PortForwardStatus::Active;
        entry.record.error = None;
        if entry.record.started_at_epoch_ms.is_none() {
            entry.record.started_at_epoch_ms = Some(now_epoch_millis());
        }
    }

    let forward = store
        .port_forwards
        .get(&forward.id)
        .map(|entry| entry.record.clone());
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;

    Ok(Json(ClaimPortForwardResponse { forward }))
}

pub(super) async fn close_port_forward(
    Path(forward_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<PortForwardDetailResponse>, ApiError> {
    let actor = actor_from_headers(&state, &headers);
    ensure_actor_can_write(&actor)?;

    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let Some(entry) = store.port_forwards.get_mut(&forward_id) else {
        return Err(ApiError::not_found(
            "port_forward_not_found",
            "Port forward not found",
        ));
    };
    ensure_tenant_access(&actor, &entry.record.tenant_id)?;

    match entry.record.status {
        PortForwardStatus::Pending => {
            entry.record.status = PortForwardStatus::Closed;
            entry.record.finished_at_epoch_ms = Some(now);
        }
        PortForwardStatus::Active => {
            entry.record.status = PortForwardStatus::CloseRequested;
        }
        PortForwardStatus::CloseRequested
        | PortForwardStatus::Closed
        | PortForwardStatus::Failed => {}
    }

    let detail = PortForwardDetailResponse {
        forward: entry.record.clone(),
    };
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    record_audit(
        &state,
        &actor,
        AuditAction::PreviewClosed,
        "preview",
        detail.forward.id.clone(),
        AuditOutcome::Succeeded,
        None,
    )
    .await?;

    Ok(Json(detail))
}

pub(super) fn preferred_port_forward_transport(device: &DeviceRecord) -> PortForwardTransportKind {
    if matches!(device.overlay.state, OverlayState::Connected)
        && device
            .overlay
            .node_ip
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
    {
        PortForwardTransportKind::OverlayProxy
    } else {
        PortForwardTransportKind::RelayTunnel
    }
}

async fn run_overlay_port_forward(state: AppState, forward_id: String) {
    if let Err(error) = run_overlay_port_forward_inner(&state, &forward_id).await {
        eprintln!("overlay port forward {forward_id} failed: {error:#}");
        if let Err(update_error) = fail_overlay_port_forward(
            &state,
            &forward_id,
            format!("overlay port forward failed: {error}"),
        )
        .await
        {
            eprintln!("overlay port forward {forward_id} failure update failed: {update_error:#}");
        }
    }
}

async fn run_overlay_port_forward_inner(state: &AppState, forward_id: &str) -> anyhow::Result<()> {
    let (forward, overlay_state, node_ip) = {
        let store = state.store.read().await;
        let Some(entry) = store.port_forwards.get(forward_id) else {
            return Ok(());
        };
        if entry.record.transport != PortForwardTransportKind::OverlayProxy
            || entry.record.status.is_terminal()
        {
            return Ok(());
        }
        let Some(device) = store.devices.get(&entry.record.device_id) else {
            return Ok(());
        };
        (
            entry.record.clone(),
            device.overlay.state.clone(),
            device.overlay.node_ip.clone(),
        )
    };

    if !matches!(overlay_state, OverlayState::Connected) {
        fallback_port_forward_to_relay_tunnel(
            state,
            forward_id,
            format!("overlay state is {overlay_state:?}"),
        )
        .await?;
        return Ok(());
    }

    let Some(node_ip) = node_ip.filter(|value| !value.trim().is_empty()) else {
        fallback_port_forward_to_relay_tunnel(
            state,
            forward_id,
            "device did not publish an overlay node IP",
        )
        .await?;
        return Ok(());
    };

    let bind_address = format!("{}:{}", state.config.forward_bind_host, forward.relay_port);
    let listener = TcpListener::bind(&bind_address)
        .await
        .with_context(|| format!("failed to bind relay listener on {bind_address}"))?;
    apply_port_forward_report_internal(
        state,
        forward_id,
        Some(PortForwardStatus::Active),
        None,
        true,
    )
    .await
    .map_err(api_error_to_anyhow)?;

    let (accepted_tx, mut accepted_rx) = mpsc::unbounded_channel::<TcpStream>();
    let accept_forward_id = forward.id.clone();
    let accept_task = tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    if accepted_tx.send(stream).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    eprintln!(
                        "overlay port forward {} accept loop stopped: {}",
                        accept_forward_id, error
                    );
                    break;
                }
            }
        }
    });

    let (client_events_tx, mut client_events_rx) =
        mpsc::unbounded_channel::<OverlayPortForwardClientEvent>();
    let mut active_client: Option<ActiveOverlayPortForwardClient> = None;
    let mut next_connection_id = 0_u64;
    let mut status_interval = tokio::time::interval(Duration::from_millis(1_000));

    loop {
        tokio::select! {
            accepted = accepted_rx.recv() => {
                let Some(stream) = accepted else {
                    break;
                };

                if active_client.is_some() {
                    let mut stream = stream;
                    let _ = stream.shutdown().await;
                    continue;
                }

                next_connection_id += 1;
                let connection_id = next_connection_id;
                let task = tokio::spawn(run_overlay_port_forward_client_session(
                    stream,
                    node_ip.clone(),
                    state.config.port_forward_bridge_port,
                    state.config.access_token.clone(),
                    forward.clone(),
                    connection_id,
                    client_events_tx.clone(),
                ));
                active_client = Some(ActiveOverlayPortForwardClient {
                    connection_id,
                    task,
                });
            }
            event = client_events_rx.recv() => {
                match event {
                    Some(OverlayPortForwardClientEvent::Ready { connection_id }) => {
                        if active_client
                            .as_ref()
                            .is_some_and(|client| client.connection_id == connection_id)
                        {
                            let _ = apply_port_forward_report_internal(
                                state,
                                forward_id,
                                None,
                                None,
                                true,
                            )
                            .await;
                        }
                    }
                    Some(OverlayPortForwardClientEvent::Closed {
                        connection_id,
                        error,
                        fallback_to_relay_tunnel,
                    }) => {
                        if active_client
                            .as_ref()
                            .is_some_and(|client| client.connection_id == connection_id)
                        {
                            active_client = None;
                            if fallback_to_relay_tunnel {
                                fallback_port_forward_to_relay_tunnel(
                                    state,
                                    forward_id,
                                    error.unwrap_or_else(|| {
                                        "overlay bridge connection failed".to_string()
                                    }),
                                )
                                .await?;
                                break;
                            }
                            if let Some(message) = error {
                                let _ = apply_port_forward_report_internal(
                                    state,
                                    forward_id,
                                    None,
                                    Some(message),
                                    false,
                                )
                                .await;
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = status_interval.tick() => {
                match load_port_forward_record(state, forward_id).await {
                    Some(record) if record.transport != PortForwardTransportKind::OverlayProxy => {
                        close_active_overlay_port_forward_client(&mut active_client).await;
                        break;
                    }
                    Some(record) if matches!(record.status, PortForwardStatus::CloseRequested) => {
                        close_active_overlay_port_forward_client(&mut active_client).await;
                        let _ = apply_port_forward_report_internal(
                            state,
                            forward_id,
                            Some(PortForwardStatus::Closed),
                            None,
                            true,
                        )
                        .await;
                        break;
                    }
                    Some(record)
                        if matches!(record.status, PortForwardStatus::Closed | PortForwardStatus::Failed) =>
                    {
                        close_active_overlay_port_forward_client(&mut active_client).await;
                        break;
                    }
                    Some(_) => {}
                    None => break,
                }
            }
        }
    }

    accept_task.abort();
    close_active_overlay_port_forward_client(&mut active_client).await;
    Ok(())
}

async fn run_overlay_port_forward_client_session(
    mut client: TcpStream,
    node_ip: String,
    bridge_port: u16,
    access_token: Option<String>,
    forward: PortForwardRecord,
    connection_id: u64,
    events_tx: mpsc::UnboundedSender<OverlayPortForwardClientEvent>,
) {
    let mut bridge = match TcpStream::connect((node_ip.as_str(), bridge_port)).await {
        Ok(bridge) => bridge,
        Err(error) => {
            let _ = events_tx.send(OverlayPortForwardClientEvent::Closed {
                connection_id,
                error: Some(format!(
                    "failed to connect overlay bridge {}:{}: {}",
                    node_ip, bridge_port, error
                )),
                fallback_to_relay_tunnel: true,
            });
            return;
        }
    };

    let request = PortForwardBridgeRequest::Start {
        token: access_token,
        forward_id: forward.id.clone(),
        target_host: forward.target_host.clone(),
        target_port: forward.target_port,
    };
    if let Err(error) = write_port_forward_bridge_request(&mut bridge, &request).await {
        let _ = events_tx.send(OverlayPortForwardClientEvent::Closed {
            connection_id,
            error: Some(format!("failed to send overlay bridge request: {error}")),
            fallback_to_relay_tunnel: true,
        });
        return;
    }

    match read_port_forward_bridge_event(&mut bridge).await {
        Ok(Some(PortForwardBridgeEvent::Ready)) => {
            let _ = events_tx.send(OverlayPortForwardClientEvent::Ready { connection_id });
        }
        Ok(Some(PortForwardBridgeEvent::Error { message })) => {
            let _ = events_tx.send(OverlayPortForwardClientEvent::Closed {
                connection_id,
                error: Some(message),
                fallback_to_relay_tunnel: false,
            });
            return;
        }
        Ok(None) => {
            let _ = events_tx.send(OverlayPortForwardClientEvent::Closed {
                connection_id,
                error: Some("overlay bridge closed before forwarding started".to_string()),
                fallback_to_relay_tunnel: true,
            });
            return;
        }
        Err(error) => {
            let _ = events_tx.send(OverlayPortForwardClientEvent::Closed {
                connection_id,
                error: Some(format!("failed to read overlay bridge response: {error}")),
                fallback_to_relay_tunnel: true,
            });
            return;
        }
    }

    let result = tokio::io::copy_bidirectional(&mut client, &mut bridge).await;
    let _ = events_tx.send(OverlayPortForwardClientEvent::Closed {
        connection_id,
        error: result
            .err()
            .map(|error| format!("overlay port forward stream failed: {error}")),
        fallback_to_relay_tunnel: false,
    });
}

async fn close_active_overlay_port_forward_client(
    active_client: &mut Option<ActiveOverlayPortForwardClient>,
) {
    if let Some(client) = active_client.take() {
        client.task.abort();
        let _ = client.task.await;
    }
}

async fn fallback_port_forward_to_relay_tunnel(
    state: &AppState,
    forward_id: &str,
    message: impl Into<String>,
) -> anyhow::Result<()> {
    let message = message.into();
    mutate_port_forward_record(state, forward_id, move |record, _now| {
        if record.transport != PortForwardTransportKind::OverlayProxy
            || record.status.is_terminal()
            || matches!(record.status, PortForwardStatus::CloseRequested)
        {
            return;
        }
        record.transport = PortForwardTransportKind::RelayTunnel;
        record.status = PortForwardStatus::Pending;
        record.started_at_epoch_ms = None;
        record.finished_at_epoch_ms = None;
        record.error = Some(format!(
            "Overlay port forward unavailable, falling back to relay tunnel: {message}"
        ));
    })
    .await
    .map_err(api_error_to_anyhow)?;
    Ok(())
}

async fn fail_overlay_port_forward(
    state: &AppState,
    forward_id: &str,
    message: impl Into<String>,
) -> anyhow::Result<()> {
    let message = message.into();
    mutate_port_forward_record(state, forward_id, move |record, now| {
        if record.transport != PortForwardTransportKind::OverlayProxy || record.status.is_terminal()
        {
            return;
        }
        if matches!(record.status, PortForwardStatus::CloseRequested) {
            record.status = PortForwardStatus::Closed;
            record.finished_at_epoch_ms = Some(now);
            record.error = None;
        } else {
            record.status = PortForwardStatus::Failed;
            record.finished_at_epoch_ms = Some(now);
            record.error = Some(message.clone());
        }
    })
    .await
    .map_err(api_error_to_anyhow)?;
    Ok(())
}

async fn mutate_port_forward_record<F>(
    state: &AppState,
    forward_id: &str,
    mutator: F,
) -> Result<Option<PortForwardRecord>, ApiError>
where
    F: FnOnce(&mut PortForwardRecord, u64),
{
    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let Some(entry) = store.port_forwards.get_mut(forward_id) else {
        return Ok(None);
    };

    mutator(&mut entry.record, now);

    let forward = entry.record.clone();
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(state, &snapshot)?;

    Ok(Some(forward))
}

async fn write_port_forward_bridge_request(
    stream: &mut TcpStream,
    request: &PortForwardBridgeRequest,
) -> anyhow::Result<()> {
    let mut payload =
        serde_json::to_string(request).context("failed to encode port forward bridge request")?;
    payload.push('\n');
    stream
        .write_all(payload.as_bytes())
        .await
        .context("failed to write port forward bridge request")?;
    stream
        .flush()
        .await
        .context("failed to flush port forward bridge request")?;
    Ok(())
}

async fn read_port_forward_bridge_event(
    stream: &mut TcpStream,
) -> anyhow::Result<Option<PortForwardBridgeEvent>> {
    let Some(line) = read_bridge_frame_line(stream).await? else {
        return Ok(None);
    };
    let event = serde_json::from_str::<PortForwardBridgeEvent>(&line)
        .context("failed to decode port forward bridge event")?;
    Ok(Some(event))
}

pub(super) async fn read_bridge_frame_line(
    stream: &mut TcpStream,
) -> anyhow::Result<Option<String>> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1];

    loop {
        let size = stream
            .read(&mut buffer)
            .await
            .context("failed to read bridge frame")?;
        if size == 0 {
            if bytes.is_empty() {
                return Ok(None);
            }
            anyhow::bail!("bridge frame ended before newline");
        }

        if buffer[0] == b'\n' {
            break;
        }

        bytes.push(buffer[0]);
        if bytes.len() > MAX_BRIDGE_FRAME_BYTES {
            anyhow::bail!("bridge frame exceeded {MAX_BRIDGE_FRAME_BYTES} bytes");
        }
    }

    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }

    let line = String::from_utf8(bytes).context("bridge frame is not valid UTF-8")?;
    Ok(Some(line))
}

async fn load_port_forward_record(state: &AppState, forward_id: &str) -> Option<PortForwardRecord> {
    let store = state.store.read().await;
    store
        .port_forwards
        .get(forward_id)
        .map(|entry| entry.record.clone())
}

async fn apply_port_forward_report_internal(
    state: &AppState,
    forward_id: &str,
    status: Option<PortForwardStatus>,
    error: Option<String>,
    clear_error: bool,
) -> Result<Option<PortForwardRecord>, ApiError> {
    let mut store = state.store.write().await;
    let Some(entry) = store.port_forwards.get_mut(forward_id) else {
        return Ok(None);
    };

    let now = now_epoch_millis();
    if let Some(status) = status {
        if matches!(status, PortForwardStatus::Active) && entry.record.started_at_epoch_ms.is_none()
        {
            entry.record.started_at_epoch_ms = Some(now);
        }
        if status.is_terminal() {
            entry.record.finished_at_epoch_ms = Some(now);
        }
        entry.record.status = status;
    }
    if clear_error {
        entry.record.error = None;
    }
    if let Some(error) = error {
        entry.record.error = Some(error);
    }

    let forward = entry.record.clone();
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(state, &snapshot)?;

    Ok(Some(forward))
}

fn spawn_port_forward_tcp_reader(
    mut reader: tokio::net::tcp::OwnedReadHalf,
    connection_id: u64,
    tx: mpsc::UnboundedSender<RelayPortForwardClientEvent>,
) {
    tokio::spawn(async move {
        let mut buffer = [0_u8; 16 * 1024];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => {
                    let _ = tx.send(RelayPortForwardClientEvent::Closed { connection_id });
                    break;
                }
                Ok(size) => {
                    if tx
                        .send(RelayPortForwardClientEvent::Data {
                            connection_id,
                            data: buffer[..size].to_vec(),
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(error) => {
                    eprintln!(
                        "port forward client {} read failed: {}",
                        connection_id, error
                    );
                    let _ = tx.send(RelayPortForwardClientEvent::Closed { connection_id });
                    break;
                }
            }
        }
    });
}

async fn close_active_port_forward_client(active_client: &mut Option<RelayPortForwardClient>) {
    if let Some(mut client) = active_client.take() {
        let _ = client.writer.shutdown().await;
    }
}

async fn send_port_forward_control(
    socket: &mut WebSocket,
    control: &PortForwardTunnelControl,
) -> Result<(), ()> {
    let payload = serde_json::to_string(control).map_err(|_| ())?;
    socket
        .send(Message::Text(payload.into()))
        .await
        .map_err(|_| ())
}

fn reserve_forward_port(store: &RelayStore, start: u16, end: u16) -> Option<u16> {
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    (start..=end).find(|candidate| {
        !store.port_forwards.values().any(|entry| {
            entry.record.relay_port == *candidate && !entry.record.status.is_terminal()
        })
    })
}
