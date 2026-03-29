use super::*;

use crate::store::{ConversationEntry, TaskEntry};
use crate::tasks::{
    dispatch_next_task_for_device, preferred_task_transport, push_task_event_entry, task_detail,
};
use vibe_core::ConversationInputRequestStatus;

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(super) struct ConversationListQuery {
    pub(super) device_id: Option<String>,
    pub(super) archived: Option<bool>,
}

pub(super) async fn list_conversations(
    State(state): State<AppState>,
    Query(query): Query<ConversationListQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<ConversationRecord>>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;

    let mut conversations = {
        let store = state.store.read().await;
        store
            .conversations
            .values()
            .map(|entry| entry.record.clone())
            .filter(|record| record.tenant_id == actor.tenant_id)
            .filter(|record| {
                query
                    .device_id
                    .as_deref()
                    .is_none_or(|device_id| record.device_id == device_id)
            })
            .filter(|record| {
                query
                    .archived
                    .is_none_or(|archived| record.archived == archived)
            })
            .collect::<Vec<_>>()
    };
    conversations.sort_by(|left, right| {
        right
            .updated_at_epoch_ms
            .cmp(&left.updated_at_epoch_ms)
            .then_with(|| left.id.cmp(&right.id))
    });

    Ok(Json(conversations))
}

pub(super) async fn get_conversation(
    Path(conversation_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ConversationDetailResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;

    let detail = {
        let store = state.store.read().await;
        let Some(entry) = store.conversations.get(&conversation_id) else {
            return Err(ApiError::not_found(
                "conversation_not_found",
                "Conversation not found",
            ));
        };
        ensure_tenant_access(&actor, &entry.record.tenant_id)?;
        conversation_detail_from_store(&store, entry)
    };

    Ok(Json(detail))
}

pub(super) async fn create_conversation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateConversationRequest>,
) -> Result<Json<CreateConversationResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    if payload.prompt.trim().is_empty() {
        return Err(ApiError::bad_request(
            "prompt_required",
            "Conversation prompt is required",
        ));
    }

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

    let mut conversation =
        ConversationRecord::new(&payload, provider.execution_protocol.clone(), &actor);
    let task = TaskRecord::new(
        CreateTaskRequest {
            device_id: payload.device_id.clone(),
            conversation_id: Some(conversation.id.clone()),
            provider: payload.provider.clone(),
            prompt: payload.prompt.clone(),
            cwd: payload.cwd.clone(),
            model: payload.model.clone(),
            title: payload.title.clone(),
            provider_session_id: None,
        },
        provider.execution_protocol,
        preferred_task_transport(&state, &device),
        &actor,
    );
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
    conversation.latest_task_id = Some(task.id.clone());
    conversation.updated_at_epoch_ms = task.created_at_epoch_ms;
    store.conversations.insert(
        conversation.id.clone(),
        ConversationEntry {
            record: conversation.clone(),
            task_ids: vec![task.id.clone()],
        },
    );

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
    tokio::spawn({
        let state = state.clone();
        let device_id = task.device_id.clone();
        async move {
            if let Err(error) = dispatch_next_task_for_device(&state, &device_id).await {
                eprintln!("task dispatch for device {device_id} failed: {error:#}");
            }
        }
    });

    Ok(Json(CreateConversationResponse { conversation, task }))
}

pub(super) async fn send_conversation_message(
    Path(conversation_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SendConversationMessageRequest>,
) -> Result<Json<SendConversationMessageResponse>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    if payload.prompt.trim().is_empty() {
        return Err(ApiError::bad_request(
            "prompt_required",
            "Conversation prompt is required",
        ));
    }

    let mut store = state.store.write().await;
    let existing_conversation = store
        .conversations
        .get(&conversation_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("conversation_not_found", "Conversation not found"))?;
    ensure_tenant_access(&actor, &existing_conversation.record.tenant_id)?;
    if existing_conversation.record.archived {
        return Err(ApiError::conflict(
            "conversation_archived",
            "Conversation is archived",
        ));
    }

    let device = store
        .devices
        .get(&existing_conversation.record.device_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("device_not_found", "Device not found"))?;
    let provider = device
        .providers
        .iter()
        .find(|provider| provider.kind == existing_conversation.record.provider)
        .cloned()
        .ok_or_else(|| {
            ApiError::bad_request(
                "provider_not_supported",
                format!(
                    "{} is not available on device {}",
                    existing_conversation.record.provider.label(),
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

    let task = TaskRecord::new(
        CreateTaskRequest {
            device_id: existing_conversation.record.device_id.clone(),
            conversation_id: Some(existing_conversation.record.id.clone()),
            provider: existing_conversation.record.provider.clone(),
            prompt: payload.prompt.clone(),
            cwd: existing_conversation.record.cwd.clone(),
            model: payload
                .model
                .clone()
                .or_else(|| existing_conversation.record.model.clone()),
            title: payload
                .title
                .clone()
                .or_else(|| Some(existing_conversation.record.title.clone())),
            provider_session_id: existing_conversation.record.provider_session_id.clone(),
        },
        provider.execution_protocol,
        preferred_task_transport(&state, &device),
        &actor,
    );
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

    let conversation = {
        let entry = store
            .conversations
            .get_mut(&conversation_id)
            .ok_or_else(|| {
                ApiError::not_found("conversation_not_found", "Conversation not found")
            })?;
        if let Some(title) = payload
            .title
            .as_deref()
            .map(str::trim)
            .filter(|title| !title.is_empty())
        {
            entry.record.title = title.to_string();
        }
        entry.record.latest_task_id = Some(task.id.clone());
        entry.record.model = payload.model.clone().or_else(|| entry.record.model.clone());
        entry.record.updated_at_epoch_ms = now_epoch_millis();
        entry.task_ids.push(task.id.clone());
        entry.record.clone()
    };

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
    tokio::spawn({
        let state = state.clone();
        let device_id = task.device_id.clone();
        async move {
            if let Err(error) = dispatch_next_task_for_device(&state, &device_id).await {
                eprintln!("task dispatch for device {device_id} failed: {error:#}");
            }
        }
    });

    Ok(Json(SendConversationMessageResponse { conversation, task }))
}

pub(super) async fn archive_conversation(
    Path(conversation_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ConversationRecord>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    let conversation = {
        let mut store = state.store.write().await;
        let entry = store
            .conversations
            .get_mut(&conversation_id)
            .ok_or_else(|| {
                ApiError::not_found("conversation_not_found", "Conversation not found")
            })?;
        ensure_tenant_access(&actor, &entry.record.tenant_id)?;
        entry.record.archived = true;
        entry.record.updated_at_epoch_ms = now_epoch_millis();
        let conversation = entry.record.clone();
        let snapshot = store.clone();
        drop(store);
        persist_snapshot(&state, &snapshot)?;
        conversation
    };

    Ok(Json(conversation))
}

pub(super) async fn create_task_input_request(
    Path(task_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateConversationInputRequest>,
) -> Result<Json<ConversationInputRequest>, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    if payload.prompt.trim().is_empty() {
        return Err(ApiError::bad_request(
            "prompt_required",
            "Input request prompt is required",
        ));
    }
    if payload.options.is_empty() && !payload.allow_custom_input {
        return Err(ApiError::bad_request(
            "options_required",
            "Input request must provide options or allow custom input",
        ));
    }

    let now = now_epoch_millis();
    let (request, task_snapshot, task_event, device_snapshot, snapshot) = {
        let mut store = state.store.write().await;
        let (conversation_id, pending_request_id) = {
            let Some(entry) = store.tasks.get(&task_id) else {
                return Err(ApiError::not_found("task_not_found", "Task not found"));
            };
            ensure_tenant_access(&auth.actor, &entry.record.tenant_id)?;
            if entry.record.device_id != auth.device_id {
                return Err(ApiError::forbidden(
                    "device_forbidden",
                    "The current device credential cannot update another device task",
                ));
            }
            let conversation_id = entry.record.conversation_id.clone().ok_or_else(|| {
                ApiError::conflict(
                    "conversation_missing",
                    "Task is not attached to a conversation",
                )
            })?;
            (
                conversation_id,
                entry.record.pending_input_request_id.clone(),
            )
        };
        if pending_request_id.as_deref().is_some_and(|request_id| {
            store
                .input_requests
                .get(request_id)
                .is_some_and(|request| request.status == ConversationInputRequestStatus::Pending)
        }) {
            return Err(ApiError::conflict(
                "input_request_pending",
                "Task already has a pending input request",
            ));
        }
        let Some(entry) = store.tasks.get_mut(&task_id) else {
            return Err(ApiError::not_found("task_not_found", "Task not found"));
        };
        ensure_tenant_access(&auth.actor, &entry.record.tenant_id)?;
        if entry.record.device_id != auth.device_id {
            return Err(ApiError::forbidden(
                "device_forbidden",
                "The current device credential cannot update another device task",
            ));
        }
        let request = ConversationInputRequest {
            id: Uuid::new_v4().to_string(),
            conversation_id: conversation_id.clone(),
            task_id: task_id.clone(),
            prompt: payload.prompt.clone(),
            options: payload.options.clone(),
            allow_custom_input: payload.allow_custom_input,
            custom_input_placeholder: payload.custom_input_placeholder.clone(),
            status: ConversationInputRequestStatus::Pending,
            selected_option_id: None,
            response_text: None,
            created_at_epoch_ms: now,
            answered_at_epoch_ms: None,
        };
        entry.record.status = TaskStatus::WaitingInput;
        entry.record.pending_input_request_id = Some(request.id.clone());
        let task_event = push_task_event_entry(
            entry,
            entry.record.device_id.clone(),
            vibe_core::TaskEventKind::System,
            format!("Waiting for user input: {}", request.prompt),
            now,
        );
        let task_snapshot = entry.record.clone();
        if let Some(conversation) = store.conversations.get_mut(&conversation_id) {
            conversation.record.pending_input_request_id = Some(request.id.clone());
            conversation.record.latest_task_id = Some(task_snapshot.id.clone());
            conversation.record.updated_at_epoch_ms = now;
        }
        let device_snapshot = store.devices.get_mut(&auth.device_id).map(|device| {
            device.current_task_id = Some(task_snapshot.id.clone());
            device.last_seen_epoch_ms = now;
            device.online = true;
            device.clone()
        });
        store
            .input_requests
            .insert(request.id.clone(), request.clone());
        let snapshot = store.clone();
        (
            request,
            task_snapshot,
            task_event,
            device_snapshot,
            snapshot,
        )
    };

    persist_snapshot(&state, &snapshot)?;
    emit_task(&state, task_snapshot).await;
    emit_task_event(&state, task_event).await;
    if let Some(device) = device_snapshot {
        emit_device(&state, device).await;
    }

    Ok(Json(request))
}

pub(super) async fn get_task_input_request(
    Path((task_id, request_id)): Path<(String, String)>,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<ConversationInputRequest>, ApiError> {
    let subject = authenticate_control_or_device(&state, &headers, None).await?;
    let actor = subject.actor().clone();
    if let AuthenticatedSubject::Control(control_actor) = &subject {
        ensure_actor_can_read(control_actor)?;
    }

    let request = {
        let store = state.store.read().await;
        let Some(task_entry) = store.tasks.get(&task_id) else {
            return Err(ApiError::not_found("task_not_found", "Task not found"));
        };
        ensure_tenant_access(&actor, &task_entry.record.tenant_id)?;
        if let AuthenticatedSubject::Device(device) = &subject
            && task_entry.record.device_id != device.device_id
        {
            return Err(ApiError::forbidden(
                "device_forbidden",
                "The current device credential cannot access another device task",
            ));
        }
        let Some(request) = store.input_requests.get(&request_id) else {
            return Err(ApiError::not_found(
                "input_request_not_found",
                "Input request not found",
            ));
        };
        if request.task_id != task_id {
            return Err(ApiError::conflict(
                "task_mismatch",
                "Input request does not belong to the requested task",
            ));
        }
        request.clone()
    };

    Ok(Json(request))
}

pub(super) async fn respond_task_input_request(
    Path((task_id, request_id)): Path<(String, String)>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RespondConversationInputRequest>,
) -> Result<Json<ConversationInputRequest>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_write(&actor)?;

    let now = now_epoch_millis();
    let (request, task_snapshot, task_event, device_snapshot, snapshot) = {
        let mut store = state.store.write().await;
        let text = payload
            .text
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let (conversation_id, allow_custom_input, selected_option) = if let Some(task_entry) =
            store.tasks.get(&task_id)
        {
            ensure_tenant_access(&actor, &task_entry.record.tenant_id)?;
            if task_entry.record.pending_input_request_id.as_deref() != Some(request_id.as_str()) {
                return Err(ApiError::conflict(
                    "input_request_not_pending",
                    "Task is not waiting on the requested input",
                ));
            }
            let request = store.input_requests.get(&request_id).ok_or_else(|| {
                ApiError::not_found("input_request_not_found", "Input request not found")
            })?;
            if request.task_id != task_id {
                return Err(ApiError::conflict(
                    "task_mismatch",
                    "Input request does not belong to the requested task",
                ));
            }
            if request.status != ConversationInputRequestStatus::Pending {
                return Err(ApiError::conflict(
                    "input_request_closed",
                    "Input request is no longer pending",
                ));
            }
            let selected_option = if let Some(option_id) = payload.option_id.as_deref() {
                Some(
                    request
                        .options
                        .iter()
                        .find(|option| option.id == option_id)
                        .cloned()
                        .ok_or_else(|| {
                            ApiError::bad_request("option_invalid", "Selected option is invalid")
                        })?,
                )
            } else {
                None
            };
            (
                request.conversation_id.clone(),
                request.allow_custom_input,
                selected_option,
            )
        } else {
            return Err(ApiError::not_found("task_not_found", "Task not found"));
        };
        if selected_option.is_none() && text.is_none() {
            return Err(ApiError::bad_request(
                "response_required",
                "A selection or custom response is required",
            ));
        }
        if selected_option.is_none() && text.is_some() && !allow_custom_input {
            return Err(ApiError::bad_request(
                "custom_input_disabled",
                "Custom input is not allowed for this request",
            ));
        }
        if selected_option
            .as_ref()
            .is_some_and(|option| option.requires_text_input && text.is_none())
        {
            return Err(ApiError::bad_request(
                "response_text_required",
                "The selected option requires text input",
            ));
        }

        let request = {
            let request = store.input_requests.get_mut(&request_id).ok_or_else(|| {
                ApiError::not_found("input_request_not_found", "Input request not found")
            })?;
            request.status = ConversationInputRequestStatus::Answered;
            request.selected_option_id = payload.option_id.clone();
            request.response_text = text.clone();
            request.answered_at_epoch_ms = Some(now);
            request.clone()
        };

        let event_message = match (selected_option.as_ref(), text.as_deref()) {
            (Some(option), Some(text)) => {
                format!(
                    "User answered input request with {}: {}",
                    option.label, text
                )
            }
            (Some(option), None) => format!("User selected input option: {}", option.label),
            (None, Some(text)) => format!("User answered input request: {text}"),
            (None, None) => "User answered input request".to_string(),
        };
        let (task_snapshot, task_event) = {
            let task_entry = store
                .tasks
                .get_mut(&task_id)
                .ok_or_else(|| ApiError::not_found("task_not_found", "Task not found"))?;
            task_entry.record.status = TaskStatus::Running;
            task_entry.record.pending_input_request_id = None;
            let task_event = push_task_event_entry(
                task_entry,
                task_entry.record.device_id.clone(),
                vibe_core::TaskEventKind::System,
                event_message,
                now,
            );
            (task_entry.record.clone(), task_event)
        };

        if let Some(conversation) = store.conversations.get_mut(&conversation_id) {
            conversation.record.pending_input_request_id = None;
            conversation.record.latest_task_id = Some(task_snapshot.id.clone());
            conversation.record.updated_at_epoch_ms = now;
        }
        let device_snapshot = store
            .devices
            .get_mut(&task_snapshot.device_id)
            .map(|device| {
                device.current_task_id = Some(task_snapshot.id.clone());
                device.last_seen_epoch_ms = now;
                device.online = true;
                device.clone()
            });
        let snapshot = store.clone();
        (
            request.clone(),
            task_snapshot,
            task_event,
            device_snapshot,
            snapshot,
        )
    };

    persist_snapshot(&state, &snapshot)?;
    emit_task(&state, task_snapshot).await;
    emit_task_event(&state, task_event).await;
    if let Some(device) = device_snapshot {
        emit_device(&state, device).await;
    }

    Ok(Json(request))
}

fn conversation_detail_from_store(
    store: &RelayStore,
    entry: &ConversationEntry,
) -> ConversationDetailResponse {
    let mut tasks = entry
        .task_ids
        .iter()
        .filter_map(|task_id| store.tasks.get(task_id))
        .map(task_detail)
        .collect::<Vec<_>>();
    tasks.sort_by(|left, right| {
        left.task
            .created_at_epoch_ms
            .cmp(&right.task.created_at_epoch_ms)
            .then_with(|| left.task.id.cmp(&right.task.id))
    });
    let pending_input_request = entry
        .record
        .pending_input_request_id
        .as_deref()
        .and_then(|request_id| store.input_requests.get(request_id))
        .cloned();

    ConversationDetailResponse {
        conversation: entry.record.clone(),
        tasks,
        pending_input_request,
    }
}
