use super::*;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct TaskListQuery {
    pub(super) device_id: Option<String>,
    pub(super) status: Option<TaskStatus>,
    pub(super) provider: Option<ProviderKind>,
    pub(super) limit: Option<usize>,
}

pub(super) async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<TaskListQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<TaskRecord>>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;

    let store = state.store.read().await;
    let mut tasks = store
        .tasks
        .values()
        .map(|entry| entry.record.clone())
        .filter(|task| task.tenant_id == actor.tenant_id)
        .filter(|task| {
            query
                .device_id
                .as_deref()
                .is_none_or(|device_id| task.device_id == device_id)
        })
        .filter(|task| {
            query
                .status
                .as_ref()
                .is_none_or(|status| &task.status == status)
        })
        .filter(|task| {
            query
                .provider
                .as_ref()
                .is_none_or(|provider| &task.provider == provider)
        })
        .collect::<Vec<_>>();
    tasks.sort_by(|left, right| {
        right
            .created_at_epoch_ms
            .cmp(&left.created_at_epoch_ms)
            .then_with(|| left.id.cmp(&right.id))
    });

    if let Some(limit) = query.limit {
        tasks.truncate(limit.min(TASK_LIST_LIMIT_MAX));
    }

    Ok(Json(tasks))
}

pub(super) async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<CreateTaskResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    let mut store = state.store.write().await;
    let device = store
        .devices
        .get(&payload.device_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("device_not_found", "Device not found"))?;
    ensure_tenant_access(&actor, &device.tenant_id)?;
    let provider = device
        .providers
        .iter()
        .find(|provider| provider.kind == payload.provider)
        .cloned()
        .ok_or_else(|| {
            ApiError::bad_request(
                "provider_not_supported",
                format!(
                    "{} is not available on device {}",
                    payload.provider.label(),
                    device.name
                ),
            )
        })?;
    if !provider.available {
        return Err(ApiError::conflict(
            "provider_unavailable",
            provider
                .error
                .unwrap_or_else(|| "provider is currently unavailable on this device".to_string()),
        ));
    }

    let transport = preferred_task_transport(&state, &device);
    let task = TaskRecord::new(payload, provider.execution_protocol, transport, &actor);
    let queued_event = TaskEvent {
        seq: 1,
        task_id: task.id.clone(),
        device_id: task.device_id.clone(),
        kind: vibe_core::TaskEventKind::System,
        message: "Task queued".to_string(),
        timestamp_epoch_ms: now_epoch_millis(),
    };
    store.tasks.insert(
        task.id.clone(),
        TaskEntry {
            record: TaskRecord {
                last_event_seq: 1,
                ..task.clone()
            },
            events: vec![queued_event.clone()],
        },
    );
    let task = store
        .tasks
        .get(&task.id)
        .map(|entry| entry.record.clone())
        .ok_or_else(|| ApiError::internal("task_missing", "Task was not stored"))?;
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    emit_task(&state, task.clone()).await;
    emit_task_event(&state, queued_event).await;
    record_audit(
        &state,
        &actor,
        AuditAction::TaskCreated,
        "task",
        task.id.clone(),
        AuditOutcome::Succeeded,
        None,
    )
    .await?;
    schedule_next_task_dispatch(state.clone(), task.device_id.clone());

    Ok(Json(CreateTaskResponse { task }))
}

pub(super) async fn get_task(
    Path(task_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TaskDetailResponse>, ApiError> {
    let subject = authenticate_control_or_device(&state, &headers, None).await?;
    let actor = subject.actor().clone();
    if let AuthenticatedSubject::Control(actor) = &subject {
        ensure_actor_can_read(actor)?;
    }

    let store = state.store.read().await;
    let Some(entry) = store.tasks.get(&task_id) else {
        return Err(ApiError::not_found("task_not_found", "Task not found"));
    };
    ensure_tenant_access(&actor, &entry.record.tenant_id)?;
    if let AuthenticatedSubject::Device(device) = &subject
        && entry.record.device_id != device.device_id
    {
        return Err(ApiError::forbidden(
            "device_forbidden",
            "The current device credential cannot access another device task",
        ));
    }

    Ok(Json(TaskDetailResponse {
        task: entry.record.clone(),
        events: entry.events.clone(),
    }))
}

pub(super) async fn claim_next_task(
    Path(device_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ClaimTaskResponse>, ApiError> {
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

    let current_task_id = device.current_task_id.clone();

    if let Some(task_id) = current_task_id {
        if let Some(entry) = store.tasks.get(&task_id) {
            if !entry.record.status.is_terminal() {
                return Ok(Json(ClaimTaskResponse { task: None }));
            }
        }
    }

    let mut next_task = store
        .tasks
        .values()
        .filter(|entry| {
            entry.record.device_id == device_id
                && entry.record.tenant_id == auth.actor.tenant_id
                && entry.record.status == TaskStatus::Pending
                && entry.record.transport == TaskTransportKind::RelayPolling
        })
        .map(|entry| entry.record.clone())
        .collect::<Vec<_>>();
    next_task.sort_by(|left, right| {
        left.created_at_epoch_ms
            .cmp(&right.created_at_epoch_ms)
            .then_with(|| left.id.cmp(&right.id))
    });

    let Some(task) = next_task.into_iter().next() else {
        return Ok(Json(ClaimTaskResponse { task: None }));
    };

    if let Some(entry) = store.tasks.get_mut(&task.id) {
        entry.record.status = TaskStatus::Assigned;
        entry.record.error = None;
    }
    if let Some(device) = store.devices.get_mut(&device_id) {
        device.current_task_id = Some(task.id.clone());
        device.last_seen_epoch_ms = now_epoch_millis();
    }

    let task = store.tasks.get(&task.id).map(|entry| entry.record.clone());
    let device = store.devices.get(&device_id).cloned();
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    if let Some(task) = task.clone() {
        emit_task(&state, task).await;
    }
    if let Some(device) = device {
        emit_device(&state, device).await;
    }

    Ok(Json(ClaimTaskResponse { task }))
}

pub(super) async fn append_task_events(
    Path(task_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AppendTaskEventsRequest>,
) -> Result<Json<TaskDetailResponse>, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    if payload.device_id != auth.device_id {
        return Err(ApiError::forbidden(
            "device_forbidden",
            "The current device credential cannot publish events for another device",
        ));
    }
    {
        let store = state.store.read().await;
        let Some(entry) = store.tasks.get(&task_id) else {
            return Err(ApiError::not_found("task_not_found", "Task not found"));
        };
        ensure_tenant_access(&auth.actor, &entry.record.tenant_id)?;
        if entry.record.device_id != auth.device_id {
            return Err(ApiError::forbidden(
                "device_forbidden",
                "The current device credential cannot publish events for another device",
            ));
        }
    }

    match append_task_events_internal(&state, &task_id, payload).await? {
        Some(detail) => Ok(Json(detail)),
        None => Err(ApiError::not_found("task_not_found", "Task not found")),
    }
}

pub(super) async fn cancel_task(
    Path(task_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TaskDetailResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let (task, event, detail, snapshot, should_schedule_next) = {
        let Some(entry) = store.tasks.get_mut(&task_id) else {
            return Err(ApiError::not_found("task_not_found", "Task not found"));
        };
        ensure_tenant_access(&actor, &entry.record.tenant_id)?;

        match entry.record.status {
            TaskStatus::Pending => {
                entry.record.status = TaskStatus::Canceled;
                entry.record.cancel_requested = false;
                entry.record.finished_at_epoch_ms = Some(now);
            }
            TaskStatus::Assigned | TaskStatus::Running | TaskStatus::CancelRequested => {
                entry.record.status = TaskStatus::CancelRequested;
                entry.record.cancel_requested = true;
            }
            TaskStatus::Succeeded | TaskStatus::Failed | TaskStatus::Canceled => {}
        }

        let event = push_task_event_entry(
            entry,
            entry.record.device_id.clone(),
            vibe_core::TaskEventKind::System,
            "Cancellation requested".to_string(),
            now,
        );
        let task = entry.record.clone();
        let detail = task_detail(entry);
        let snapshot = store.clone();
        let should_schedule_next = matches!(task.status, TaskStatus::Canceled);
        (task, event, detail, snapshot, should_schedule_next)
    };
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    emit_task(&state, task.clone()).await;
    emit_task_event(&state, event).await;
    record_audit(
        &state,
        &actor,
        AuditAction::TaskCanceled,
        "task",
        task.id.clone(),
        AuditOutcome::Succeeded,
        None,
    )
    .await?;
    if should_schedule_next {
        schedule_next_task_dispatch(state.clone(), task.device_id.clone());
    }

    Ok(Json(detail))
}

fn schedule_next_task_dispatch(state: AppState, device_id: String) {
    tokio::spawn(async move {
        if let Err(error) = dispatch_next_task_for_device(&state, &device_id).await {
            eprintln!("task dispatch for device {device_id} failed: {error:#}");
        }
    });
}

pub(super) async fn dispatch_next_task_for_device(
    state: &AppState,
    device_id: &str,
) -> anyhow::Result<()> {
    let (task_id, task_snapshot, device_snapshot, snapshot) = {
        let mut store = state.store.write().await;
        let Some(device) = store.devices.get(device_id) else {
            return Ok(());
        };

        if let Some(current_task_id) = device.current_task_id.clone()
            && store
                .tasks
                .get(&current_task_id)
                .is_some_and(|entry| !entry.record.status.is_terminal())
        {
            return Ok(());
        }

        let mut next_task = store
            .tasks
            .values()
            .filter(|entry| {
                entry.record.device_id == device_id && entry.record.status == TaskStatus::Pending
            })
            .map(|entry| entry.record.clone())
            .collect::<Vec<_>>();
        next_task.sort_by(|left, right| {
            left.created_at_epoch_ms
                .cmp(&right.created_at_epoch_ms)
                .then_with(|| left.id.cmp(&right.id))
        });

        let Some(task) = next_task.into_iter().next() else {
            return Ok(());
        };
        if task.transport != TaskTransportKind::OverlayProxy {
            return Ok(());
        }

        if let Some(entry) = store.tasks.get_mut(&task.id) {
            entry.record.status = TaskStatus::Assigned;
            entry.record.error = None;
        }
        if let Some(device) = store.devices.get_mut(device_id) {
            device.current_task_id = Some(task.id.clone());
            device.last_seen_epoch_ms = now_epoch_millis();
        }

        let task_snapshot = store.tasks.get(&task.id).map(|entry| entry.record.clone());
        let device_snapshot = store.devices.get(device_id).cloned();
        let snapshot = store.clone();
        (task.id, task_snapshot, device_snapshot, snapshot)
    };

    persist_snapshot(state, &snapshot).map_err(api_error_to_anyhow)?;
    if let Some(task) = task_snapshot {
        emit_task(state, task).await;
    }
    if let Some(device) = device_snapshot {
        emit_device(state, device).await;
    }

    let overlay_state = state.clone();
    tokio::spawn(async move {
        run_overlay_task(overlay_state, task_id).await;
    });

    Ok(())
}

async fn append_task_events_internal(
    state: &AppState,
    task_id: &str,
    payload: AppendTaskEventsRequest,
) -> Result<Option<TaskDetailResponse>, ApiError> {
    let device_id = payload.device_id.clone();
    let mut store = state.store.write().await;
    let now = now_epoch_millis();

    let (task, task_events, device_snapshot, detail, should_schedule_next) = {
        let Some(entry) = store.tasks.get_mut(task_id) else {
            return Ok(None);
        };
        if entry.record.device_id != payload.device_id {
            return Err(ApiError::conflict(
                "device_mismatch",
                "Task device does not match update source",
            ));
        }

        let mut emitted_events = Vec::with_capacity(payload.events.len());
        for input in payload.events {
            let event = push_task_event_entry(
                entry,
                entry.record.device_id.clone(),
                input.kind,
                input.message,
                now_epoch_millis(),
            );
            emitted_events.push(event);
        }

        if let Some(execution_protocol) = payload.execution_protocol.clone() {
            entry.record.execution_protocol = execution_protocol;
        }

        if let Some(status) = payload.status.clone() {
            if matches!(status, TaskStatus::Running) && entry.record.started_at_epoch_ms.is_none() {
                entry.record.started_at_epoch_ms = Some(now);
            }
            if status.is_terminal() {
                entry.record.finished_at_epoch_ms = Some(now);
                entry.record.cancel_requested = false;
            }
            if matches!(status, TaskStatus::Canceled) {
                entry.record.cancel_requested = false;
            }
            entry.record.status = status;
        }

        if let Some(exit_code) = payload.exit_code {
            entry.record.exit_code = Some(exit_code);
        }
        if let Some(error) = payload.error {
            entry.record.error = Some(error);
        }

        let task = entry.record.clone();
        let detail = task_detail(entry);
        let should_schedule_next = task.status.is_terminal();
        let device_snapshot = store.devices.get_mut(&device_id).map(|device| {
            if task.status.is_terminal() {
                if device.current_task_id.as_deref() == Some(task.id.as_str()) {
                    device.current_task_id = None;
                }
            } else {
                device.current_task_id = Some(task.id.clone());
            }
            device.last_seen_epoch_ms = now;
            device.online = true;
            device.clone()
        });

        (
            task,
            emitted_events,
            device_snapshot,
            detail,
            should_schedule_next,
        )
    };

    let snapshot = store.clone();
    drop(store);

    persist_snapshot(state, &snapshot)?;
    emit_task(state, task.clone()).await;
    for event in task_events {
        emit_task_event(state, event).await;
    }
    if let Some(device) = device_snapshot {
        emit_device(state, device).await;
    }
    if should_schedule_next {
        schedule_next_task_dispatch(state.clone(), device_id);
    }

    Ok(Some(detail))
}

pub(super) fn task_detail(entry: &TaskEntry) -> TaskDetailResponse {
    TaskDetailResponse {
        task: entry.record.clone(),
        events: entry.events.clone(),
    }
}

fn push_task_event_entry(
    entry: &mut TaskEntry,
    device_id: String,
    kind: vibe_core::TaskEventKind,
    message: String,
    timestamp_epoch_ms: u64,
) -> TaskEvent {
    entry.record.last_event_seq += 1;
    let event = TaskEvent {
        seq: entry.record.last_event_seq,
        task_id: entry.record.id.clone(),
        device_id,
        kind,
        message,
        timestamp_epoch_ms,
    };
    entry.events.push(event.clone());
    event
}

pub(super) fn preferred_task_transport(
    state: &AppState,
    device: &DeviceRecord,
) -> TaskTransportKind {
    if matches!(device.overlay.state, OverlayState::Connected)
        && device
            .overlay
            .node_ip
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
        && overlay_bridge_is_available(state, &device.id, OverlayBridgeKind::Task)
    {
        TaskTransportKind::OverlayProxy
    } else {
        TaskTransportKind::RelayPolling
    }
}

async fn run_overlay_task(state: AppState, task_id: String) {
    if let Err(error) = run_overlay_task_inner(&state, &task_id).await {
        eprintln!("overlay task {task_id} failed: {error:#}");
        if let Err(update_error) =
            fail_overlay_task(&state, &task_id, format!("overlay task failed: {error}")).await
        {
            eprintln!("overlay task {task_id} failure update failed: {update_error:#}");
        }
    }
}

async fn run_overlay_task_inner(state: &AppState, task_id: &str) -> anyhow::Result<()> {
    let (task, overlay_state, node_ip) = {
        let store = state.store.read().await;
        let Some(entry) = store.tasks.get(task_id) else {
            return Ok(());
        };
        if entry.record.transport != TaskTransportKind::OverlayProxy
            || entry.record.status.is_terminal()
            || entry.record.status != TaskStatus::Assigned
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
        fallback_task_to_relay_polling(
            state,
            task_id,
            format!("overlay state is {overlay_state:?}"),
        )
        .await?;
        return Ok(());
    }

    let Some(node_ip) = node_ip.filter(|value| !value.trim().is_empty()) else {
        fallback_task_to_relay_polling(state, task_id, "device did not publish an overlay node IP")
            .await?;
        return Ok(());
    };

    let stream = match connect_overlay_bridge(
        state,
        &task.device_id,
        OverlayBridgeKind::Task,
        node_ip.as_str(),
    )
    .await
    {
        Ok(stream) => stream,
        Err(message) => {
            fallback_task_to_relay_polling(state, task_id, message).await?;
            return Ok(());
        }
    };

    let (read_half, mut write_half) = stream.into_split();
    let mut lines = BufReader::new(read_half).lines();
    let bridge_token = load_device_credential(state, &task.device_id).await;
    if let Err(error) = write_task_bridge_request(
        &mut write_half,
        &TaskBridgeRequest::Start {
            token: bridge_token,
            task: task.clone(),
        },
    )
    .await
    {
        mark_overlay_bridge_unavailable(
            state,
            &task.device_id,
            OverlayBridgeKind::Task,
            format!("failed to send task bridge start request: {error}"),
        );
        fallback_task_to_relay_polling(
            state,
            task_id,
            format!("failed to send task bridge start request: {error}"),
        )
        .await?;
        return Ok(());
    }

    let mut started = false;
    let mut cancel_sent = false;
    let mut interval = tokio::time::interval(Duration::from_millis(TASK_BRIDGE_POLL_MS));
    let start_timeout = tokio::time::sleep(Duration::from_millis(
        state.config.overlay_bridge_start_timeout_ms.max(1),
    ));
    tokio::pin!(start_timeout);

    loop {
        tokio::select! {
            _ = &mut start_timeout, if !started => {
                let message = format!(
                    "task bridge did not acknowledge start within {} ms",
                    state.config.overlay_bridge_start_timeout_ms
                );
                mark_overlay_bridge_unavailable(
                    state,
                    &task.device_id,
                    OverlayBridgeKind::Task,
                    message.clone(),
                );
                fallback_task_to_relay_polling(state, task_id, message).await?;
                return Ok(());
            }
            _ = interval.tick() => {
                match load_task_record(state, task_id).await {
                    Some(record) if record.transport != TaskTransportKind::OverlayProxy => {
                        return Ok(());
                    }
                    Some(record) if record.status.is_terminal() => {
                        return Ok(());
                    }
                    Some(record) if matches!(record.status, TaskStatus::CancelRequested) && !cancel_sent => {
                        write_task_bridge_request(&mut write_half, &TaskBridgeRequest::Cancel)
                            .await
                            .with_context(|| format!("failed to send task cancel request for {task_id}"))?;
                        cancel_sent = true;
                    }
                    Some(_) => {}
                    None => return Ok(()),
                }
            }
            event = read_task_bridge_event(&mut lines) => {
                match event {
                    Ok(Some(TaskBridgeEvent::Update {
                        status,
                        execution_protocol,
                        events,
                        exit_code,
                        error,
                    })) => {
                        started = true;
                        let detail = append_task_events_internal(
                            state,
                            task_id,
                            AppendTaskEventsRequest {
                                device_id: task.device_id.clone(),
                                status,
                                execution_protocol,
                                events,
                                exit_code,
                                error,
                            },
                        )
                        .await
                        .map_err(api_error_to_anyhow)?;
                        if detail.as_ref().is_some_and(|detail| detail.task.status.is_terminal()) {
                            return Ok(());
                        }
                    }
                    Ok(Some(TaskBridgeEvent::Error { message })) => {
                        if !started {
                            fallback_task_to_relay_polling(
                                state,
                                task_id,
                                format!("task bridge rejected task: {message}"),
                            )
                            .await?;
                        } else {
                            fail_overlay_task(state, task_id, message).await?;
                        }
                        return Ok(());
                    }
                    Ok(None) => {
                        if !started {
                            mark_overlay_bridge_unavailable(
                                state,
                                &task.device_id,
                                OverlayBridgeKind::Task,
                                "overlay task bridge closed before task start",
                            );
                            fallback_task_to_relay_polling(
                                state,
                                task_id,
                                "overlay task bridge closed before task start",
                            )
                            .await?;
                        } else if cancel_sent {
                            fail_overlay_task(
                                state,
                                task_id,
                                "overlay task bridge closed after cancel request",
                            )
                            .await?;
                        } else {
                            mark_overlay_bridge_unavailable(
                                state,
                                &task.device_id,
                                OverlayBridgeKind::Task,
                                "overlay task bridge connection closed unexpectedly",
                            );
                            fail_overlay_task(
                                state,
                                task_id,
                                "overlay task bridge connection closed unexpectedly",
                            )
                            .await?;
                        }
                        return Ok(());
                    }
                    Err(error) => {
                        let message = format!("failed to read task bridge event: {error}");
                        mark_overlay_bridge_unavailable(
                            state,
                            &task.device_id,
                            OverlayBridgeKind::Task,
                            message.clone(),
                        );
                        if !started {
                            fallback_task_to_relay_polling(state, task_id, message).await?;
                        } else {
                            fail_overlay_task(state, task_id, message).await?;
                        }
                        return Ok(());
                    }
                }
            }
        }
    }
}

async fn load_task_record(state: &AppState, task_id: &str) -> Option<TaskRecord> {
    let store = state.store.read().await;
    store.tasks.get(task_id).map(|entry| entry.record.clone())
}

async fn fallback_task_to_relay_polling(
    state: &AppState,
    task_id: &str,
    message: impl Into<String>,
) -> anyhow::Result<()> {
    let message = message.into();
    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let (task, task_event, device_snapshot, snapshot, device_id) = {
        let Some(entry) = store.tasks.get_mut(task_id) else {
            return Ok(());
        };
        if entry.record.transport != TaskTransportKind::OverlayProxy
            || entry.record.status.is_terminal()
        {
            return Ok(());
        }

        entry.record.transport = TaskTransportKind::RelayPolling;
        entry.record.status = TaskStatus::Pending;
        entry.record.cancel_requested = false;
        entry.record.started_at_epoch_ms = None;
        entry.record.finished_at_epoch_ms = None;
        entry.record.exit_code = None;
        entry.record.error = None;
        let task_event = push_task_event_entry(
            entry,
            entry.record.device_id.clone(),
            vibe_core::TaskEventKind::System,
            format!("Overlay task unavailable, falling back to relay polling: {message}"),
            now,
        );
        let task = entry.record.clone();
        let device_id = task.device_id.clone();
        let device_snapshot = store.devices.get_mut(&device_id).map(|device| {
            if device.current_task_id.as_deref() == Some(task_id) {
                device.current_task_id = None;
            }
            device.last_seen_epoch_ms = now;
            device.online = true;
            device.clone()
        });
        let snapshot = store.clone();
        (task, task_event, device_snapshot, snapshot, device_id)
    };
    drop(store);

    persist_snapshot(state, &snapshot).map_err(api_error_to_anyhow)?;
    emit_task(state, task).await;
    emit_task_event(state, task_event).await;
    if let Some(device) = device_snapshot {
        emit_device(state, device).await;
    }
    schedule_next_task_dispatch(state.clone(), device_id);
    Ok(())
}

async fn fail_overlay_task(
    state: &AppState,
    task_id: &str,
    message: impl Into<String>,
) -> anyhow::Result<()> {
    let message = message.into();
    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let (task, task_event, device_snapshot, snapshot, device_id) = {
        let Some(entry) = store.tasks.get_mut(task_id) else {
            return Ok(());
        };
        if entry.record.transport != TaskTransportKind::OverlayProxy
            || entry.record.status.is_terminal()
        {
            return Ok(());
        }

        let final_status = if entry.record.cancel_requested
            || matches!(entry.record.status, TaskStatus::CancelRequested)
        {
            TaskStatus::Canceled
        } else {
            TaskStatus::Failed
        };
        let task_event = push_task_event_entry(
            entry,
            entry.record.device_id.clone(),
            vibe_core::TaskEventKind::System,
            message.clone(),
            now,
        );
        entry.record.status = final_status.clone();
        entry.record.cancel_requested = false;
        entry.record.finished_at_epoch_ms = Some(now);
        entry.record.error = if matches!(final_status, TaskStatus::Failed) {
            Some(message.clone())
        } else {
            None
        };
        let task = entry.record.clone();
        let device_id = task.device_id.clone();
        let device_snapshot = store.devices.get_mut(&device_id).map(|device| {
            if device.current_task_id.as_deref() == Some(task_id) {
                device.current_task_id = None;
            }
            device.last_seen_epoch_ms = now;
            device.online = true;
            device.clone()
        });
        let snapshot = store.clone();
        (task, task_event, device_snapshot, snapshot, device_id)
    };
    drop(store);

    persist_snapshot(state, &snapshot).map_err(api_error_to_anyhow)?;
    emit_task(state, task).await;
    emit_task_event(state, task_event).await;
    if let Some(device) = device_snapshot {
        emit_device(state, device).await;
    }
    schedule_next_task_dispatch(state.clone(), device_id);
    Ok(())
}

async fn write_task_bridge_request<W>(
    writer: &mut W,
    request: &TaskBridgeRequest,
) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut payload =
        serde_json::to_string(request).context("failed to encode task bridge request")?;
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .await
        .context("failed to write task bridge request")?;
    writer
        .flush()
        .await
        .context("failed to flush task bridge request")?;
    Ok(())
}

async fn read_task_bridge_event<R>(
    lines: &mut tokio::io::Lines<BufReader<R>>,
) -> anyhow::Result<Option<TaskBridgeEvent>>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let Some(line) = lines
        .next_line()
        .await
        .context("failed to read task bridge event")?
    else {
        return Ok(None);
    };
    let event = serde_json::from_str::<TaskBridgeEvent>(&line)
        .context("failed to decode task bridge event")?;
    Ok(Some(event))
}
