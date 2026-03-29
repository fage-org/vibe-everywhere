use super::*;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct ShellSessionListQuery {
    pub(super) device_id: Option<String>,
    pub(super) status: Option<ShellSessionStatus>,
    pub(super) limit: Option<usize>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct ShellInputQuery {
    pub(super) after_seq: Option<u64>,
}

pub(super) async fn list_shell_sessions(
    State(state): State<AppState>,
    Query(query): Query<ShellSessionListQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<ShellSessionRecord>>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;

    let store = state.store.read().await;
    let mut sessions = store
        .shell_sessions
        .values()
        .map(|entry| entry.record.clone())
        .filter(|session| session.tenant_id == actor.tenant_id)
        .filter(|session| {
            query
                .device_id
                .as_deref()
                .is_none_or(|device_id| session.device_id == device_id)
        })
        .filter(|session| {
            query
                .status
                .as_ref()
                .is_none_or(|status| &session.status == status)
        })
        .collect::<Vec<_>>();
    sessions.sort_by(|left, right| {
        right
            .created_at_epoch_ms
            .cmp(&left.created_at_epoch_ms)
            .then_with(|| left.id.cmp(&right.id))
    });

    if let Some(limit) = query.limit {
        sessions.truncate(limit.min(SHELL_SESSION_LIST_LIMIT_MAX));
    }

    Ok(Json(sessions))
}

pub(super) async fn create_shell_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateShellSessionRequest>,
) -> Result<Json<CreateShellSessionResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    let (session, detail, snapshot, start_overlay_proxy) = {
        let mut store = state.store.write().await;
        let device = store
            .devices
            .get(&payload.device_id)
            .cloned()
            .ok_or_else(|| ApiError::not_found("device_not_found", "Device not found"))?;
        ensure_tenant_access(&actor, &device.tenant_id)?;
        if !device
            .capabilities
            .iter()
            .any(|capability| matches!(capability, DeviceCapability::Shell))
        {
            return Err(ApiError::bad_request(
                "shell_not_supported",
                format!("Device {} does not advertise shell support", device.name),
            ));
        }

        let transport = preferred_shell_transport(&state, &device);
        let session = ShellSessionRecord::new(payload, transport.clone(), &actor);
        let start_overlay_proxy = transport == ShellTransportKind::OverlayProxy;
        store.shell_sessions.insert(
            session.id.clone(),
            ShellSessionEntry {
                record: session.clone(),
                inputs: vec![],
                outputs: vec![],
            },
        );
        let detail = ShellSessionDetailResponse {
            session: session.clone(),
            inputs: vec![],
            outputs: vec![],
        };
        let snapshot = store.clone();
        (session, detail, snapshot, start_overlay_proxy)
    };

    persist_snapshot(&state, &snapshot)?;
    publish_shell_session_detail(&state, &detail).await;
    record_audit(
        &state,
        &actor,
        AuditAction::ShellSessionCreated,
        "shell_session",
        session.id.clone(),
        AuditOutcome::Succeeded,
        None,
    )
    .await?;

    if start_overlay_proxy {
        let overlay_state = state.clone();
        let overlay_session_id = session.id.clone();
        tokio::spawn(async move {
            run_overlay_shell_session(overlay_state, overlay_session_id).await;
        });
    }

    Ok(Json(CreateShellSessionResponse { session }))
}

pub(super) async fn get_shell_session(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ShellSessionDetailResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;

    let store = state.store.read().await;
    let Some(entry) = store.shell_sessions.get(&session_id) else {
        return Err(ApiError::not_found(
            "shell_session_not_found",
            "Shell session not found",
        ));
    };
    ensure_tenant_access(&actor, &entry.record.tenant_id)?;

    Ok(Json(shell_session_detail(entry)))
}

pub(super) async fn shell_session_websocket(
    Path(session_id): Path<String>,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, ApiError> {
    let actor = require_control_actor(&state, &headers, Some(&uri)).await?;
    ensure_actor_can_read(&actor)?;

    {
        let store = state.store.read().await;
        let Some(entry) = store.shell_sessions.get(&session_id) else {
            return Err(ApiError::not_found(
                "shell_session_not_found",
                "Shell session not found",
            ));
        };
        ensure_tenant_access(&actor, &entry.record.tenant_id)?;
    }

    Ok(ws.on_upgrade(move |socket| async move {
        handle_shell_session_socket(state, session_id, socket).await;
    }))
}

async fn handle_shell_session_socket(state: AppState, session_id: String, mut socket: WebSocket) {
    let initial_detail = {
        let store = state.store.read().await;
        store
            .shell_sessions
            .get(&session_id)
            .map(shell_session_detail)
    };

    let Some(detail) = initial_detail else {
        return;
    };

    if send_shell_session_snapshot(&mut socket, &detail)
        .await
        .is_err()
    {
        return;
    }

    let mut updates = shell_session_sender(&state, &session_id).await.subscribe();
    loop {
        tokio::select! {
            message = updates.recv() => {
                match message {
                    Ok(payload) => {
                        if socket
                            .send(Message::Text(payload.as_str().into()))
                            .await
                            .is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            message = socket.next() => {
                match message {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
}

async fn send_shell_session_snapshot(
    socket: &mut WebSocket,
    detail: &ShellSessionDetailResponse,
) -> Result<(), ()> {
    let payload = serde_json::to_string(detail).map_err(|_| ())?;
    socket
        .send(Message::Text(payload.as_str().into()))
        .await
        .map_err(|_| ())
}

pub(super) async fn append_shell_input(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateShellInputRequest>,
) -> Result<Json<ShellSessionDetailResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    if payload.data.is_empty() {
        return Err(ApiError::bad_request(
            "shell_input_empty",
            "Shell input must not be empty",
        ));
    }

    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let Some(entry) = store.shell_sessions.get_mut(&session_id) else {
        return Err(ApiError::not_found(
            "shell_session_not_found",
            "Shell session not found",
        ));
    };
    ensure_tenant_access(&actor, &entry.record.tenant_id)?;
    if entry.record.status.is_terminal() || entry.record.close_requested {
        return Err(ApiError::conflict(
            "shell_session_closed",
            "Shell session is not accepting new input",
        ));
    }

    entry.record.last_input_seq += 1;
    entry.inputs.push(ShellInputRecord {
        seq: entry.record.last_input_seq,
        session_id: session_id.clone(),
        data: payload.data,
        timestamp_epoch_ms: now,
    });

    let detail = shell_session_detail(entry);
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    publish_shell_session_detail(&state, &detail).await;

    Ok(Json(detail))
}

pub(super) async fn get_shell_pending_input(
    Path(session_id): Path<String>,
    Query(query): Query<ShellInputQuery>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ShellPendingInputResponse>, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;

    let store = state.store.read().await;
    let Some(entry) = store.shell_sessions.get(&session_id) else {
        return Err(ApiError::not_found(
            "shell_session_not_found",
            "Shell session not found",
        ));
    };
    ensure_tenant_access(&auth.actor, &entry.record.tenant_id)?;
    if entry.record.device_id != auth.device_id {
        return Err(ApiError::forbidden(
            "device_forbidden",
            "The current device credential cannot access another device shell session",
        ));
    }

    let after_seq = query.after_seq.unwrap_or_default();
    let inputs = entry
        .inputs
        .iter()
        .filter(|input| input.seq > after_seq)
        .cloned()
        .collect::<Vec<_>>();

    Ok(Json(ShellPendingInputResponse {
        session: entry.record.clone(),
        inputs,
    }))
}

pub(super) async fn claim_next_shell_session(
    Path(device_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ClaimShellSessionResponse>, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    ensure_authenticated_device_matches(&auth, &device_id)?;

    let mut store = state.store.write().await;
    let Some(device) = store.devices.get(&device_id) else {
        return Err(ApiError::not_found(
            "device_not_found",
            "Device not found; register device first",
        ));
    };
    ensure_tenant_access(&auth.actor, &device.tenant_id)?;

    let mut next_sessions = store
        .shell_sessions
        .values()
        .filter(|entry| {
            entry.record.device_id == device_id
                && entry.record.tenant_id == auth.actor.tenant_id
                && entry.record.status == ShellSessionStatus::Pending
                && entry.record.transport == ShellTransportKind::RelayPolling
        })
        .map(|entry| entry.record.clone())
        .collect::<Vec<_>>();
    next_sessions.sort_by(|left, right| {
        left.created_at_epoch_ms
            .cmp(&right.created_at_epoch_ms)
            .then_with(|| left.id.cmp(&right.id))
    });

    let Some(session) = next_sessions.into_iter().next() else {
        return Ok(Json(ClaimShellSessionResponse { session: None }));
    };

    if let Some(entry) = store.shell_sessions.get_mut(&session.id) {
        entry.record.status = ShellSessionStatus::Active;
        entry.record.error = None;
        if entry.record.started_at_epoch_ms.is_none() {
            entry.record.started_at_epoch_ms = Some(now_epoch_millis());
        }
    }

    let detail = store
        .shell_sessions
        .get(&session.id)
        .map(shell_session_detail);
    let session = detail.as_ref().map(|detail| detail.session.clone());
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    if let Some(detail) = detail.as_ref() {
        publish_shell_session_detail(&state, detail).await;
    }

    Ok(Json(ClaimShellSessionResponse { session }))
}

pub(super) async fn append_shell_output(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AppendShellOutputRequest>,
) -> Result<Json<ShellSessionDetailResponse>, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    if payload.device_id != auth.device_id {
        return Err(ApiError::forbidden(
            "device_forbidden",
            "The current device credential cannot publish shell output for another device",
        ));
    }
    {
        let store = state.store.read().await;
        let Some(entry) = store.shell_sessions.get(&session_id) else {
            return Err(ApiError::not_found(
                "shell_session_not_found",
                "Shell session not found",
            ));
        };
        ensure_tenant_access(&auth.actor, &entry.record.tenant_id)?;
        if entry.record.device_id != auth.device_id {
            return Err(ApiError::forbidden(
                "device_forbidden",
                "The current device credential cannot publish shell output for another device",
            ));
        }
    }

    let Some(detail) = append_shell_output_internal(&state, &session_id, payload).await? else {
        return Err(ApiError::not_found(
            "shell_session_not_found",
            "Shell session not found",
        ));
    };

    Ok(Json(detail))
}

pub(super) async fn close_shell_session(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ShellSessionDetailResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let Some(entry) = store.shell_sessions.get_mut(&session_id) else {
        return Err(ApiError::not_found(
            "shell_session_not_found",
            "Shell session not found",
        ));
    };
    ensure_tenant_access(&actor, &entry.record.tenant_id)?;

    match entry.record.status {
        ShellSessionStatus::Pending => {
            entry.record.status = ShellSessionStatus::Closed;
            entry.record.close_requested = false;
            entry.record.finished_at_epoch_ms = Some(now);
            push_shell_output(
                entry,
                vibe_core::ShellStreamKind::System,
                "Shell session closed before start".to_string(),
                now,
            );
        }
        ShellSessionStatus::Active => {
            entry.record.status = ShellSessionStatus::CloseRequested;
            entry.record.close_requested = true;
            push_shell_output(
                entry,
                vibe_core::ShellStreamKind::System,
                "Shell session close requested".to_string(),
                now,
            );
        }
        ShellSessionStatus::CloseRequested
        | ShellSessionStatus::Succeeded
        | ShellSessionStatus::Failed
        | ShellSessionStatus::Closed => {}
    }

    let detail = shell_session_detail(entry);
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    publish_shell_session_detail(&state, &detail).await;
    record_audit(
        &state,
        &actor,
        AuditAction::ShellSessionClosed,
        "shell_session",
        detail.session.id.clone(),
        AuditOutcome::Succeeded,
        None,
    )
    .await?;

    Ok(Json(detail))
}

pub(super) fn preferred_shell_transport(
    state: &AppState,
    device: &DeviceRecord,
) -> ShellTransportKind {
    if matches!(device.overlay.state, OverlayState::Connected)
        && device
            .overlay
            .node_ip
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        && overlay_bridge_is_available(state, &device.id, OverlayBridgeKind::Shell)
    {
        ShellTransportKind::OverlayProxy
    } else {
        ShellTransportKind::RelayPolling
    }
}

async fn run_overlay_shell_session(state: AppState, session_id: String) {
    if let Err(error) = run_overlay_shell_session_inner(&state, &session_id).await {
        eprintln!("overlay shell session {session_id} failed: {error:#}");
        if let Err(update_error) = fail_overlay_shell_session(
            &state,
            &session_id,
            format!("overlay shell bridge failed: {error}"),
        )
        .await
        {
            eprintln!("overlay shell session {session_id} failure update failed: {update_error:#}");
        }
    }
}

async fn run_overlay_shell_session_inner(state: &AppState, session_id: &str) -> anyhow::Result<()> {
    let (device_id, overlay_state, node_ip, cwd) = {
        let store = state.store.read().await;
        let Some(entry) = store.shell_sessions.get(session_id) else {
            return Ok(());
        };
        if entry.record.transport != ShellTransportKind::OverlayProxy
            || entry.record.status.is_terminal()
        {
            return Ok(());
        }
        let Some(device) = store.devices.get(&entry.record.device_id) else {
            return Ok(());
        };
        (
            entry.record.device_id.clone(),
            device.overlay.state.clone(),
            device.overlay.node_ip.clone(),
            entry.record.cwd.clone(),
        )
    };

    if !matches!(overlay_state, OverlayState::Connected) {
        fallback_shell_session_to_relay_polling(
            state,
            session_id,
            format!("overlay state is {overlay_state:?}"),
        )
        .await?;
        return Ok(());
    }

    let Some(node_ip) = node_ip.filter(|value| !value.trim().is_empty()) else {
        fallback_shell_session_to_relay_polling(
            state,
            session_id,
            "device did not publish an overlay node IP",
        )
        .await?;
        return Ok(());
    };

    let stream = match connect_overlay_bridge(
        state,
        &device_id,
        OverlayBridgeKind::Shell,
        node_ip.as_str(),
    )
    .await
    {
        Ok(stream) => stream,
        Err(message) => {
            fallback_shell_session_to_relay_polling(state, session_id, message).await?;
            return Ok(());
        }
    };
    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();
    let bridge_token = load_device_credential(state, &device_id).await;

    if let Err(error) = write_shell_bridge_request(
        &mut write_half,
        &ShellBridgeRequest::Start {
            token: bridge_token,
            session_id: session_id.to_string(),
            cwd,
        },
    )
    .await
    {
        mark_overlay_bridge_unavailable(
            state,
            &device_id,
            OverlayBridgeKind::Shell,
            format!("failed to send shell start request: {error}"),
        );
        fallback_shell_session_to_relay_polling(
            state,
            session_id,
            format!("failed to send shell start request: {error}"),
        )
        .await?;
        return Ok(());
    }

    let started = match tokio::time::timeout(
        Duration::from_millis(state.config.overlay_bridge_start_timeout_ms.max(1)),
        read_shell_bridge_event(&mut lines),
    )
    .await
    {
        Ok(Ok(Some(ShellBridgeEvent::Started { shell, cwd }))) => {
            append_shell_output_internal(
                state,
                session_id,
                AppendShellOutputRequest {
                    device_id: device_id.clone(),
                    status: Some(ShellSessionStatus::Active),
                    outputs: vec![
                        vibe_core::ShellOutputChunkInput {
                            stream: ShellStreamKind::System,
                            data: format!("Overlay shell started via {shell}"),
                        },
                        vibe_core::ShellOutputChunkInput {
                            stream: ShellStreamKind::System,
                            data: format!("cwd={cwd}"),
                        },
                    ],
                    exit_code: None,
                    error: None,
                },
            )
            .await
            .map_err(api_error_to_anyhow)?;
            true
        }
        Ok(Ok(Some(ShellBridgeEvent::Error { message }))) => {
            fallback_shell_session_to_relay_polling(
                state,
                session_id,
                format!("shell bridge rejected session: {message}"),
            )
            .await?;
            false
        }
        Ok(Ok(Some(other))) => {
            mark_overlay_bridge_unavailable(
                state,
                &device_id,
                OverlayBridgeKind::Shell,
                format!("unexpected shell bridge event before start: {other:?}"),
            );
            fallback_shell_session_to_relay_polling(
                state,
                session_id,
                format!("unexpected shell bridge event before start: {other:?}"),
            )
            .await?;
            false
        }
        Ok(Ok(None)) => {
            mark_overlay_bridge_unavailable(
                state,
                &device_id,
                OverlayBridgeKind::Shell,
                "shell bridge closed before session start",
            );
            fallback_shell_session_to_relay_polling(
                state,
                session_id,
                "shell bridge closed before session start",
            )
            .await?;
            false
        }
        Ok(Err(error)) => {
            mark_overlay_bridge_unavailable(
                state,
                &device_id,
                OverlayBridgeKind::Shell,
                format!("failed to read shell bridge start event: {error}"),
            );
            fallback_shell_session_to_relay_polling(
                state,
                session_id,
                format!("failed to read shell bridge start event: {error}"),
            )
            .await?;
            false
        }
        Err(_) => {
            let message = format!(
                "shell bridge did not acknowledge start within {} ms",
                state.config.overlay_bridge_start_timeout_ms
            );
            mark_overlay_bridge_unavailable(
                state,
                &device_id,
                OverlayBridgeKind::Shell,
                message.clone(),
            );
            fallback_shell_session_to_relay_polling(state, session_id, message).await?;
            false
        }
    };
    if !started {
        return Ok(());
    }

    let mut last_input_seq = 0;
    let mut close_sent = false;
    let mut interval = tokio::time::interval(Duration::from_millis(SHELL_BRIDGE_POLL_MS));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let Some(pending) = load_shell_pending_input_from_store(state, session_id, last_input_seq).await else {
                    return Ok(());
                };
                if pending.session.transport != ShellTransportKind::OverlayProxy {
                    return Ok(());
                }

                for input in pending.inputs {
                    write_shell_bridge_request(
                        &mut write_half,
                        &ShellBridgeRequest::Input { data: input.data.clone() },
                    )
                    .await
                    .with_context(|| format!("failed to forward shell input for {session_id}"))?;
                    last_input_seq = input.seq;
                }

                let should_close = pending.session.close_requested
                    || matches!(
                        pending.session.status,
                        ShellSessionStatus::CloseRequested | ShellSessionStatus::Closed
                    );
                if should_close && !close_sent {
                    write_shell_bridge_request(&mut write_half, &ShellBridgeRequest::Close)
                        .await
                        .with_context(|| format!("failed to send shell close request for {session_id}"))?;
                    close_sent = true;
                }

                if pending.session.status.is_terminal() {
                    return Ok(());
                }
            }
            event = read_shell_bridge_event(&mut lines) => {
                match event {
                    Ok(Some(ShellBridgeEvent::Output { stream, data })) => {
                        append_shell_output_internal(
                            state,
                            session_id,
                            AppendShellOutputRequest {
                                device_id: device_id.clone(),
                                status: None,
                                outputs: vec![vibe_core::ShellOutputChunkInput { stream, data }],
                                exit_code: None,
                                error: None,
                            },
                        )
                        .await
                        .map_err(api_error_to_anyhow)?;
                    }
                    Ok(Some(ShellBridgeEvent::Exited {
                        exit_code,
                        close_requested,
                        error,
                    })) => {
                        let status = if close_requested {
                            ShellSessionStatus::Closed
                        } else if error.is_none() && exit_code.unwrap_or_default() == 0 {
                            ShellSessionStatus::Succeeded
                        } else {
                            ShellSessionStatus::Failed
                        };
                        append_shell_output_internal(
                            state,
                            session_id,
                            AppendShellOutputRequest {
                                device_id: device_id.clone(),
                                status: Some(status),
                                outputs: vec![],
                                exit_code,
                                error,
                            },
                        )
                        .await
                        .map_err(api_error_to_anyhow)?;
                        return Ok(());
                    }
                    Ok(Some(ShellBridgeEvent::Error { message })) => {
                        fail_overlay_shell_session(state, session_id, message).await?;
                        return Ok(());
                    }
                    Ok(Some(ShellBridgeEvent::Started { .. })) => {}
                    Ok(None) => {
                        if close_sent {
                            fail_overlay_shell_session(
                                state,
                                session_id,
                                "overlay shell bridge closed after close request",
                            )
                            .await?;
                        } else {
                            mark_overlay_bridge_unavailable(
                                state,
                                &device_id,
                                OverlayBridgeKind::Shell,
                                "overlay shell bridge connection closed unexpectedly",
                            );
                            fail_overlay_shell_session(
                                state,
                                session_id,
                                "overlay shell bridge connection closed unexpectedly",
                            )
                            .await?;
                        }
                        return Ok(());
                    }
                    Err(error) => {
                        let message = format!("failed to read shell bridge event: {error}");
                        mark_overlay_bridge_unavailable(
                            state,
                            &device_id,
                            OverlayBridgeKind::Shell,
                            message.clone(),
                        );
                        fail_overlay_shell_session(state, session_id, message).await?;
                        return Ok(());
                    }
                }
            }
        }
    }
}

async fn load_shell_pending_input_from_store(
    state: &AppState,
    session_id: &str,
    after_seq: u64,
) -> Option<ShellPendingInputResponse> {
    let store = state.store.read().await;
    let entry = store.shell_sessions.get(session_id)?;
    let inputs = entry
        .inputs
        .iter()
        .filter(|input| input.seq > after_seq)
        .cloned()
        .collect::<Vec<_>>();

    Some(ShellPendingInputResponse {
        session: entry.record.clone(),
        inputs,
    })
}

async fn fallback_shell_session_to_relay_polling(
    state: &AppState,
    session_id: &str,
    message: impl Into<String>,
) -> anyhow::Result<()> {
    let message = message.into();
    mutate_shell_session(state, session_id, |entry, now| {
        if entry.record.transport != ShellTransportKind::OverlayProxy
            || entry.record.status != ShellSessionStatus::Pending
        {
            return;
        }
        entry.record.transport = ShellTransportKind::RelayPolling;
        entry.record.status = ShellSessionStatus::Pending;
        entry.record.close_requested = false;
        entry.record.started_at_epoch_ms = None;
        entry.record.finished_at_epoch_ms = None;
        entry.record.exit_code = None;
        entry.record.error = None;
        push_shell_output(
            entry,
            ShellStreamKind::System,
            format!("Overlay shell unavailable, falling back to relay polling: {message}"),
            now,
        );
    })
    .await
    .map_err(api_error_to_anyhow)?;
    Ok(())
}

async fn fail_overlay_shell_session(
    state: &AppState,
    session_id: &str,
    message: impl Into<String>,
) -> anyhow::Result<()> {
    let message = message.into();
    mutate_shell_session(state, session_id, |entry, now| {
        if entry.record.transport != ShellTransportKind::OverlayProxy
            || entry.record.status.is_terminal()
        {
            return;
        }
        let final_status = if entry.record.close_requested
            || matches!(entry.record.status, ShellSessionStatus::CloseRequested)
        {
            ShellSessionStatus::Closed
        } else {
            ShellSessionStatus::Failed
        };
        push_shell_output(entry, ShellStreamKind::System, message.clone(), now);
        entry.record.status = final_status.clone();
        entry.record.finished_at_epoch_ms = Some(now);
        entry.record.close_requested = false;
        entry.record.error = if matches!(final_status, ShellSessionStatus::Failed) {
            Some(message.clone())
        } else {
            None
        };
    })
    .await
    .map_err(api_error_to_anyhow)?;
    Ok(())
}

async fn mutate_shell_session<F>(
    state: &AppState,
    session_id: &str,
    mutator: F,
) -> Result<Option<ShellSessionDetailResponse>, ApiError>
where
    F: FnOnce(&mut ShellSessionEntry, u64),
{
    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let Some(entry) = store.shell_sessions.get_mut(session_id) else {
        return Ok(None);
    };
    mutator(entry, now);
    let detail = shell_session_detail(entry);
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(state, &snapshot)?;
    publish_shell_session_detail(state, &detail).await;

    Ok(Some(detail))
}

async fn append_shell_output_internal(
    state: &AppState,
    session_id: &str,
    payload: AppendShellOutputRequest,
) -> Result<Option<ShellSessionDetailResponse>, ApiError> {
    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let Some(entry) = store.shell_sessions.get_mut(session_id) else {
        return Ok(None);
    };
    if entry.record.device_id != payload.device_id {
        return Err(ApiError::conflict(
            "device_mismatch",
            "Shell session device does not match output source",
        ));
    }

    apply_shell_output_update(entry, payload, now);

    let detail = shell_session_detail(entry);
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(state, &snapshot)?;
    publish_shell_session_detail(state, &detail).await;

    Ok(Some(detail))
}

fn apply_shell_output_update(
    entry: &mut ShellSessionEntry,
    payload: AppendShellOutputRequest,
    now: u64,
) {
    for output in payload.outputs {
        push_shell_output(entry, output.stream, output.data, now);
    }

    if let Some(status) = payload.status {
        if matches!(status, ShellSessionStatus::Active)
            && entry.record.started_at_epoch_ms.is_none()
        {
            entry.record.started_at_epoch_ms = Some(now);
        }
        if matches!(status, ShellSessionStatus::CloseRequested) {
            entry.record.close_requested = true;
        }
        if matches!(status, ShellSessionStatus::Active) {
            entry.record.close_requested = false;
            entry.record.finished_at_epoch_ms = None;
        }
        if status.is_terminal() {
            entry.record.finished_at_epoch_ms = Some(now);
            entry.record.close_requested = false;
        }
        entry.record.status = status;
    }

    if let Some(exit_code) = payload.exit_code {
        entry.record.exit_code = Some(exit_code);
    }
    if let Some(error) = payload.error {
        entry.record.error = Some(error);
    }
}

async fn write_shell_bridge_request<W>(
    writer: &mut W,
    request: &ShellBridgeRequest,
) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut payload =
        serde_json::to_string(request).context("failed to encode shell bridge request")?;
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .await
        .context("failed to write shell bridge request")?;
    writer
        .flush()
        .await
        .context("failed to flush shell bridge request")?;
    Ok(())
}

async fn read_shell_bridge_event<R>(
    lines: &mut tokio::io::Lines<BufReader<R>>,
) -> anyhow::Result<Option<ShellBridgeEvent>>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let Some(line) = lines
        .next_line()
        .await
        .context("failed to read shell bridge event")?
    else {
        return Ok(None);
    };
    let event = serde_json::from_str::<ShellBridgeEvent>(&line)
        .context("failed to decode shell bridge event")?;
    Ok(Some(event))
}
