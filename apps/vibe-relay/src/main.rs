use anyhow::Context;
use async_stream::stream;
use axum::{
    Json, Router,
    extract::{
        Path, Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode, Uri},
    response::sse::{Event, KeepAlive, Sse},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use futures_util::StreamExt;
use serde::Deserialize;
use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, RwLock as StdRwLock},
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{RwLock, broadcast, mpsc},
    task::JoinHandle,
};
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use vibe_core::{
    ActorIdentity, AppConfig, AppendShellOutputRequest, AppendTaskEventsRequest, AuditAction,
    AuditOutcome, AuditRecord, ClaimPortForwardResponse, ClaimShellSessionResponse,
    ClaimTaskResponse, CreatePortForwardRequest, CreatePortForwardResponse,
    CreateShellInputRequest, CreateShellSessionRequest, CreateShellSessionResponse,
    CreateTaskRequest, CreateTaskResponse, DEFAULT_TENANT_ID, DEFAULT_USER_ID,
    DEVICE_OFFLINE_AFTER_MS, DeviceCapability, DeviceRecord, HeartbeatRequest, HeartbeatResponse,
    MembershipRecord, OverlayState, PortForwardBridgeEvent, PortForwardBridgeRequest,
    PortForwardDetailResponse, PortForwardRecord, PortForwardStatus, PortForwardTransportKind,
    PortForwardTunnelControl, ProviderKind, RegisterDeviceRequest, RegisterDeviceResponse,
    RelayEventEnvelope, RelayEventType, ReportPortForwardStateRequest, ServiceHealth,
    ShellBridgeEvent, ShellBridgeRequest, ShellInputRecord, ShellOutputChunk,
    ShellPendingInputResponse, ShellSessionDetailResponse, ShellSessionRecord, ShellSessionStatus,
    ShellStreamKind, ShellTransportKind, TaskBridgeEvent, TaskBridgeRequest, TaskDetailResponse,
    TaskEvent, TaskRecord, TaskStatus, TaskTransportKind, TenantRecord, UserRecord,
    default_app_config, now_epoch_millis,
};

mod auth;
mod config;
mod easytier;
mod git;
mod port_forwards;
mod shell;
mod storage;
mod store;
mod tasks;
mod workspace;

use auth::{
    AuthenticatedSubject, authenticate_control_or_device, ensure_authenticated_device_matches,
    issue_device_credential, require_control_actor, require_device_auth,
    require_registration_actor,
};
#[cfg(test)]
use auth::{query_access_token, request_access_token};
use config::RelayConfig;
use easytier::{RelayEasyTierOptions, start_managed_relay_easytier};
use git::{GitRequestEntry, claim_next_git_request, complete_git_request, inspect_git_workspace};
#[cfg(test)]
use port_forwards::{
    PortForwardListQuery, preferred_port_forward_transport, read_bridge_frame_line,
};
use port_forwards::{
    claim_next_port_forward, close_port_forward, create_port_forward, get_port_forward,
    list_port_forwards, port_forward_tunnel_websocket, report_port_forward_state,
};
#[cfg(test)]
use shell::{ShellSessionListQuery, preferred_shell_transport};
use shell::{
    append_shell_input, append_shell_output, claim_next_shell_session, close_shell_session,
    create_shell_session, get_shell_pending_input, get_shell_session, list_shell_sessions,
    shell_session_websocket,
};
use storage::{RelayStorage, build_relay_storage};
use store::{DeviceCredentialRecord, PortForwardEntry, RelayStore, ShellSessionEntry, TaskEntry};
#[cfg(test)]
use store::{load_relay_store, persist_relay_store};
#[cfg(test)]
use tasks::{TaskListQuery, dispatch_next_task_for_device, preferred_task_transport, task_detail};
use tasks::{append_task_events, cancel_task, claim_next_task, create_task, get_task, list_tasks};
use workspace::{
    WorkspaceRequestEntry, browse_workspace, claim_next_workspace_request,
    complete_workspace_request, preview_workspace_file,
};

#[derive(Clone)]
struct AppState {
    store: Arc<RwLock<RelayStore>>,
    storage: Arc<dyn RelayStorage>,
    events_tx: broadcast::Sender<RelayEventEnvelope>,
    shell_session_updates: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
    workspace_requests: Arc<RwLock<HashMap<String, WorkspaceRequestEntry>>>,
    git_requests: Arc<RwLock<HashMap<String, GitRequestEntry>>>,
    overlay_bridge_health: Arc<StdRwLock<HashMap<OverlayBridgeKey, OverlayBridgeHealth>>>,
    config: Arc<RelayConfig>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum OverlayBridgeKind {
    Task,
    Shell,
    PortForward,
}

impl OverlayBridgeKind {
    fn label(self) -> &'static str {
        match self {
            Self::Task => "task",
            Self::Shell => "shell",
            Self::PortForward => "port-forward",
        }
    }

    fn port(self, config: &RelayConfig) -> u16 {
        match self {
            Self::Task => config.task_bridge_port,
            Self::Shell => config.shell_bridge_port,
            Self::PortForward => config.port_forward_bridge_port,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct OverlayBridgeKey {
    device_id: String,
    kind: OverlayBridgeKind,
}

#[derive(Clone, Debug)]
struct OverlayBridgeHealth {
    next_probe_after_epoch_ms: u64,
    last_error: String,
    probe_in_flight: bool,
}

fn overlay_bridge_key(device_id: &str, kind: OverlayBridgeKind) -> OverlayBridgeKey {
    OverlayBridgeKey {
        device_id: device_id.to_string(),
        kind,
    }
}

fn overlay_bridge_backoff_deadline(now: u64, delay_ms: u64) -> u64 {
    now.saturating_add(delay_ms.max(1))
}

fn overlay_bridge_read_health(
    state: &AppState,
) -> std::sync::RwLockReadGuard<'_, HashMap<OverlayBridgeKey, OverlayBridgeHealth>> {
    state
        .overlay_bridge_health
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn overlay_bridge_write_health(
    state: &AppState,
) -> std::sync::RwLockWriteGuard<'_, HashMap<OverlayBridgeKey, OverlayBridgeHealth>> {
    state
        .overlay_bridge_health
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn overlay_bridge_is_available(state: &AppState, device_id: &str, kind: OverlayBridgeKind) -> bool {
    let health = overlay_bridge_read_health(state);
    !health.contains_key(&overlay_bridge_key(device_id, kind))
}

fn clear_overlay_bridge_unavailable(state: &AppState, device_id: &str, kind: OverlayBridgeKind) {
    let mut health = overlay_bridge_write_health(state);
    health.remove(&overlay_bridge_key(device_id, kind));
}

fn update_overlay_bridge_probe_deadline(
    state: &AppState,
    key: &OverlayBridgeKey,
    last_error: String,
    delay_ms: u64,
) {
    let mut health = overlay_bridge_write_health(state);
    if let Some(entry) = health.get_mut(key) {
        entry.next_probe_after_epoch_ms =
            overlay_bridge_backoff_deadline(now_epoch_millis(), delay_ms);
        entry.last_error = last_error;
    }
}

fn mark_overlay_bridge_unavailable(
    state: &AppState,
    device_id: &str,
    kind: OverlayBridgeKind,
    last_error: impl Into<String>,
) {
    let key = overlay_bridge_key(device_id, kind);
    let last_error = last_error.into();
    let mut should_spawn_probe = false;
    {
        let mut health = overlay_bridge_write_health(state);
        let entry = health
            .entry(key.clone())
            .or_insert_with(|| OverlayBridgeHealth {
                next_probe_after_epoch_ms: 0,
                last_error: String::new(),
                probe_in_flight: false,
            });
        entry.next_probe_after_epoch_ms = overlay_bridge_backoff_deadline(
            now_epoch_millis(),
            state.config.overlay_bridge_recovery_cooldown_ms,
        );
        entry.last_error = last_error;
        if !entry.probe_in_flight {
            entry.probe_in_flight = true;
            should_spawn_probe = true;
        }
    }

    if should_spawn_probe {
        let probe_state = state.clone();
        tokio::spawn(async move {
            overlay_bridge_probe_loop(probe_state, key).await;
        });
    }
}

async fn overlay_bridge_probe_loop(state: AppState, key: OverlayBridgeKey) {
    loop {
        let next_probe_after_epoch_ms = {
            let health = overlay_bridge_read_health(&state);
            let Some(entry) = health.get(&key) else {
                return;
            };
            entry.next_probe_after_epoch_ms
        };

        let now = now_epoch_millis();
        if next_probe_after_epoch_ms > now {
            tokio::time::sleep(Duration::from_millis(next_probe_after_epoch_ms - now)).await;
        }

        let probe_target = {
            let store = state.store.read().await;
            store.devices.get(&key.device_id).map(|device| {
                (
                    device.overlay.state.clone(),
                    device.overlay.node_ip.clone(),
                    key.kind.port(&state.config),
                )
            })
        };

        let Some((overlay_state, node_ip, port)) = probe_target else {
            clear_overlay_bridge_unavailable(&state, &key.device_id, key.kind);
            return;
        };

        let Some(node_ip) = node_ip.filter(|value| !value.trim().is_empty()) else {
            update_overlay_bridge_probe_deadline(
                &state,
                &key,
                format!(
                    "{} bridge probe waiting for overlay node IP",
                    key.kind.label()
                ),
                state.config.overlay_bridge_probe_interval_ms,
            );
            continue;
        };

        if !matches!(overlay_state, OverlayState::Connected) {
            update_overlay_bridge_probe_deadline(
                &state,
                &key,
                format!(
                    "{} bridge probe waiting for overlay state {:?}",
                    key.kind.label(),
                    overlay_state
                ),
                state.config.overlay_bridge_probe_interval_ms,
            );
            continue;
        }

        let connect_timeout =
            Duration::from_millis(state.config.overlay_bridge_connect_timeout_ms.max(1));
        match tokio::time::timeout(
            connect_timeout,
            TcpStream::connect((node_ip.as_str(), port)),
        )
        .await
        {
            Ok(Ok(stream)) => {
                drop(stream);
                clear_overlay_bridge_unavailable(&state, &key.device_id, key.kind);
                return;
            }
            Ok(Err(error)) => {
                update_overlay_bridge_probe_deadline(
                    &state,
                    &key,
                    format!(
                        "failed to reconnect {} bridge {}:{}: {}",
                        key.kind.label(),
                        node_ip,
                        port,
                        error
                    ),
                    state.config.overlay_bridge_probe_interval_ms,
                );
            }
            Err(_) => {
                update_overlay_bridge_probe_deadline(
                    &state,
                    &key,
                    format!(
                        "timed out reconnecting {} bridge {}:{} after {} ms",
                        key.kind.label(),
                        node_ip,
                        port,
                        state.config.overlay_bridge_connect_timeout_ms
                    ),
                    state.config.overlay_bridge_probe_interval_ms,
                );
            }
        }
    }
}

async fn connect_overlay_bridge(
    state: &AppState,
    device_id: &str,
    kind: OverlayBridgeKind,
    node_ip: &str,
) -> Result<TcpStream, String> {
    let port = kind.port(&state.config);
    let connect_timeout =
        Duration::from_millis(state.config.overlay_bridge_connect_timeout_ms.max(1));
    match tokio::time::timeout(connect_timeout, TcpStream::connect((node_ip, port))).await {
        Ok(Ok(stream)) => {
            clear_overlay_bridge_unavailable(state, device_id, kind);
            Ok(stream)
        }
        Ok(Err(error)) => {
            let message = format!(
                "failed to connect {} bridge {}:{}: {}",
                kind.label(),
                node_ip,
                port,
                error
            );
            mark_overlay_bridge_unavailable(state, device_id, kind, message.clone());
            Err(message)
        }
        Err(_) => {
            let message = format!(
                "timed out connecting {} bridge {}:{} after {} ms",
                kind.label(),
                node_ip,
                port,
                state.config.overlay_bridge_connect_timeout_ms
            );
            mark_overlay_bridge_unavailable(state, device_id, kind, message.clone());
            Err(message)
        }
    }
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    fn unauthorized(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
            message: message.into(),
        }
    }

    fn forbidden(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code,
            message: message.into(),
        }
    }

    fn not_found(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
            message: message.into(),
        }
    }

    fn conflict(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
            message: message.into(),
        }
    }

    fn internal(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct AuditListQuery {
    limit: Option<usize>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({
                "error": self.code,
                "message": self.message,
            })),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind_host = std::env::var("VIBE_RELAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let bind_port = std::env::var("VIBE_RELAY_PORT").unwrap_or_else(|_| "8787".to_string());
    let config = Arc::new(RelayConfig::from_env(&bind_host, &bind_port));
    let storage = build_relay_storage(config.storage_kind.clone(), config.state_file.clone());
    let mut store = storage.load()?;
    if seed_governance_records(&mut store, &config.default_actor()) {
        storage.save(&store)?;
    }
    let (events_tx, _) = broadcast::channel(256);
    let state = AppState {
        store: Arc::new(RwLock::new(store)),
        storage,
        events_tx,
        shell_session_updates: Arc::new(RwLock::new(HashMap::new())),
        workspace_requests: Arc::new(RwLock::new(HashMap::new())),
        git_requests: Arc::new(RwLock::new(HashMap::new())),
        overlay_bridge_health: Arc::new(StdRwLock::new(HashMap::new())),
        config: config.clone(),
    };
    let easytier_runtime = start_managed_relay_easytier(RelayEasyTierOptions::from_env());

    tokio::spawn(device_presence_loop(state.clone()));
    let app = build_app(state.clone());

    let bind_addr = format!("{bind_host}:{bind_port}");
    let listener = TcpListener::bind(&bind_addr).await?;

    println!("vibe-relay listening on http://{bind_addr}");
    println!(
        "relay storage ({}): {}",
        format!("{:?}", config.storage_kind).to_lowercase(),
        state.storage.descriptor()
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    if let Some(runtime) = easytier_runtime {
        runtime.shutdown().await;
    }

    Ok(())
}

fn build_app(state: AppState) -> Router {
    let protected_api = Router::new()
        .route("/api/devices", get(list_devices))
        .route("/api/devices/register", post(register_device))
        .route("/api/devices/:device_id/heartbeat", post(device_heartbeat))
        .route(
            "/api/devices/:device_id/tasks/claim-next",
            post(claim_next_task),
        )
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route("/api/tasks/:task_id", get(get_task))
        .route("/api/tasks/:task_id/cancel", post(cancel_task))
        .route("/api/tasks/:task_id/events", post(append_task_events))
        .route(
            "/api/shell/sessions",
            get(list_shell_sessions).post(create_shell_session),
        )
        .route("/api/shell/sessions/:session_id", get(get_shell_session))
        .route(
            "/api/shell/sessions/:session_id/ws",
            get(shell_session_websocket),
        )
        .route(
            "/api/shell/sessions/:session_id/input",
            get(get_shell_pending_input).post(append_shell_input),
        )
        .route(
            "/api/shell/sessions/:session_id/output",
            post(append_shell_output),
        )
        .route(
            "/api/shell/sessions/:session_id/close",
            post(close_shell_session),
        )
        .route(
            "/api/devices/:device_id/shell/claim-next",
            post(claim_next_shell_session),
        )
        .route(
            "/api/devices/:device_id/port-forwards/claim-next",
            post(claim_next_port_forward),
        )
        .route(
            "/api/devices/:device_id/workspace/claim-next",
            post(claim_next_workspace_request),
        )
        .route(
            "/api/devices/:device_id/git/claim-next",
            post(claim_next_git_request),
        )
        .route(
            "/api/port-forwards",
            get(list_port_forwards).post(create_port_forward),
        )
        .route("/api/port-forwards/:forward_id", get(get_port_forward))
        .route(
            "/api/port-forwards/:forward_id/report",
            post(report_port_forward_state),
        )
        .route(
            "/api/port-forwards/:forward_id/tunnel/ws",
            get(port_forward_tunnel_websocket),
        )
        .route(
            "/api/port-forwards/:forward_id/close",
            post(close_port_forward),
        )
        .route("/api/workspace/browse", post(browse_workspace))
        .route("/api/workspace/preview", post(preview_workspace_file))
        .route("/api/git/inspect", post(inspect_git_workspace))
        .route(
            "/api/workspace/requests/:request_id/complete",
            post(complete_workspace_request),
        )
        .route(
            "/api/git/requests/:request_id/complete",
            post(complete_git_request),
        )
        .route("/api/audit/events", get(list_audit_events))
        .route("/api/events/stream", get(events_stream));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/app-config", get(app_config))
        .merge(protected_api)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<ServiceHealth> {
    let store = state.store.read().await;
    let online_device_count = store
        .devices
        .values()
        .filter(|device| device.online)
        .count();

    Json(ServiceHealth {
        service: "vibe-relay".to_string(),
        status: "ok".to_string(),
        device_count: store.devices.len(),
        online_device_count,
        task_count: store.tasks.len(),
    })
}

async fn app_config(State(state): State<AppState>) -> Result<Json<AppConfig>, ApiError> {
    Ok(Json(default_app_config(
        state.config.public_base_url.clone(),
        state.config.access_token.is_some(),
        state.config.deployment_metadata(),
        state.config.storage_kind.clone(),
        state.config.default_actor(),
    )))
}

async fn list_devices(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DeviceRecord>>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;

    let store = state.store.read().await;
    let mut devices = store
        .devices
        .values()
        .filter(|device| device.tenant_id == actor.tenant_id)
        .cloned()
        .collect::<Vec<_>>();
    devices.sort_by(|left, right| {
        right
            .online
            .cmp(&left.online)
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(Json(devices))
}

async fn register_device(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<RegisterDeviceRequest>,
) -> Result<Json<RegisterDeviceResponse>, ApiError> {
    let actor = require_registration_actor(&state, &headers, None).await?;

    let mut store = state.store.write().await;
    seed_governance_records(&mut store, &actor);
    let device_id = if payload.id.trim().is_empty() {
        Uuid::new_v4().to_string()
    } else {
        payload.id
    };

    let current_task_id = store
        .devices
        .get(&device_id)
        .and_then(|device| device.current_task_id.clone());

    let device = DeviceRecord {
        tenant_id: actor.tenant_id.clone(),
        user_id: actor.user_id.clone(),
        id: device_id.clone(),
        name: payload.name,
        platform: payload.platform,
        capabilities: payload.capabilities,
        metadata: payload.metadata,
        providers: payload.providers,
        overlay: payload.overlay,
        online: true,
        last_seen_epoch_ms: now_epoch_millis(),
        current_task_id,
    };
    let device_credential = issue_device_credential();
    store.device_credentials.insert(
        device.id.clone(),
        DeviceCredentialRecord {
            device_id: device.id.clone(),
            token: device_credential.clone(),
            issued_at_epoch_ms: now_epoch_millis(),
        },
    );

    store.devices.insert(device_id, device.clone());
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    emit_device(&state, device.clone()).await;
    record_audit(
        &state,
        &actor,
        AuditAction::DeviceRegistered,
        "device",
        device.id.clone(),
        AuditOutcome::Succeeded,
        None,
    )
    .await?;

    Ok(Json(RegisterDeviceResponse {
        device,
        device_credential,
    }))
}

async fn device_heartbeat(
    Path(device_id): Path<String>,
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<HeartbeatRequest>,
) -> Result<Json<HeartbeatResponse>, ApiError> {
    let auth = require_device_auth(&state, &headers, None).await?;
    ensure_authenticated_device_matches(&auth, &device_id)?;

    let mut store = state.store.write().await;
    let now = now_epoch_millis();
    let Some(device) = store.devices.get_mut(&device_id) else {
        return Err(ApiError::not_found(
            "device_not_found",
            "Device not found; register device first",
        ));
    };
    ensure_tenant_access(&auth.actor, &device.tenant_id)?;

    device.online = true;
    device.last_seen_epoch_ms = now;
    device.metadata = payload.metadata;
    device.providers = payload.providers;
    device.overlay = payload.overlay;
    device.current_task_id = payload.current_task_id;
    let response = device.clone();
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(&state, &snapshot)?;
    emit_device(&state, response.clone()).await;

    Ok(Json(HeartbeatResponse { device: response }))
}

async fn list_audit_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuditListQuery>,
) -> Result<Json<Vec<AuditRecord>>, ApiError> {
    let actor = require_control_actor(&state, &headers, None).await?;
    ensure_actor_can_read(&actor)?;

    let store = state.store.read().await;
    let mut records = store
        .audit_records
        .iter()
        .filter(|record| record.tenant_id == actor.tenant_id)
        .cloned()
        .collect::<Vec<_>>();
    records.sort_by(|left, right| {
        right
            .timestamp_epoch_ms
            .cmp(&left.timestamp_epoch_ms)
            .then_with(|| right.id.cmp(&left.id))
    });
    records.truncate(
        query
            .limit
            .unwrap_or(AUDIT_RECORD_LIMIT_MAX)
            .min(AUDIT_RECORD_LIMIT_MAX),
    );

    Ok(Json(records))
}

fn api_error_to_anyhow(error: ApiError) -> anyhow::Error {
    anyhow::anyhow!("{}: {}", error.code, error.message)
}

fn ensure_actor_can_read(actor: &ActorIdentity) -> Result<(), ApiError> {
    if actor.role.can_read_control_plane() {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "read_forbidden",
            "The current actor cannot read relay resources",
        ))
    }
}

fn ensure_actor_can_write(actor: &ActorIdentity) -> Result<(), ApiError> {
    if actor.role.can_write_control_plane() {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "write_forbidden",
            "The current actor cannot modify relay resources",
        ))
    }
}

fn ensure_tenant_access(actor: &ActorIdentity, tenant_id: &str) -> Result<(), ApiError> {
    if actor.tenant_id == tenant_id {
        Ok(())
    } else {
        Err(ApiError::forbidden(
            "tenant_forbidden",
            "The current actor cannot access resources from another tenant",
        ))
    }
}

async fn load_device_credential(state: &AppState, device_id: &str) -> Option<String> {
    state
        .store
        .read()
        .await
        .device_credentials
        .get(device_id)
        .map(|credential| credential.token.clone())
}

fn seed_governance_records(store: &mut RelayStore, actor: &ActorIdentity) -> bool {
    let mut changed = false;
    let now = now_epoch_millis();

    if !store.tenants.contains_key(&actor.tenant_id) {
        store.tenants.insert(
            actor.tenant_id.clone(),
            TenantRecord {
                id: actor.tenant_id.clone(),
                name: if actor.tenant_id == DEFAULT_TENANT_ID {
                    "Personal Workspace".to_string()
                } else {
                    actor.tenant_id.clone()
                },
                created_at_epoch_ms: now,
            },
        );
        changed = true;
    }

    if !store.users.contains_key(&actor.user_id) {
        store.users.insert(
            actor.user_id.clone(),
            UserRecord {
                id: actor.user_id.clone(),
                display_name: if actor.user_id == DEFAULT_USER_ID {
                    "Local Admin".to_string()
                } else {
                    actor.user_id.clone()
                },
                created_at_epoch_ms: now,
            },
        );
        changed = true;
    }

    if let Some(membership) = store.memberships.iter_mut().find(|membership| {
        membership.tenant_id == actor.tenant_id && membership.user_id == actor.user_id
    }) {
        if membership.role != actor.role {
            membership.role = actor.role.clone();
            changed = true;
        }
    } else {
        store.memberships.push(MembershipRecord {
            tenant_id: actor.tenant_id.clone(),
            user_id: actor.user_id.clone(),
            role: actor.role.clone(),
            created_at_epoch_ms: now,
        });
        changed = true;
    }

    changed
}

async fn record_audit(
    state: &AppState,
    actor: &ActorIdentity,
    action: AuditAction,
    resource_kind: impl Into<String>,
    resource_id: impl Into<String>,
    outcome: AuditOutcome,
    message: Option<String>,
) -> Result<(), ApiError> {
    let mut store = state.store.write().await;
    seed_governance_records(&mut store, actor);
    store.audit_records.push(AuditRecord {
        id: Uuid::new_v4().to_string(),
        tenant_id: actor.tenant_id.clone(),
        user_id: actor.user_id.clone(),
        actor_role: actor.role.clone(),
        action,
        resource_kind: resource_kind.into(),
        resource_id: resource_id.into(),
        outcome,
        message,
        timestamp_epoch_ms: now_epoch_millis(),
    });
    if store.audit_records.len() > AUDIT_RECORD_LIMIT_MAX {
        let excess = store.audit_records.len() - AUDIT_RECORD_LIMIT_MAX;
        store.audit_records.drain(0..excess);
    }
    let snapshot = store.clone();
    drop(store);

    persist_snapshot(state, &snapshot)
}

async fn events_stream(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let actor = require_control_actor(&state, &headers, Some(&uri)).await?;
    ensure_actor_can_read(&actor)?;
    let mut receiver = state.events_tx.subscribe();

    let event_stream = stream! {
        loop {
            match receiver.recv().await {
                Ok(message) => {
                    if message.tenant_id != actor.tenant_id {
                        continue;
                    }
                    let data = serde_json::to_string(&message).unwrap_or_else(|_| "{}".to_string());
                    yield Ok(Event::default().event(message.event_type.as_str()).data(data));
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    };

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

async fn device_presence_loop(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_millis(HEARTBEAT_SWEEP_MS));

    loop {
        interval.tick().await;
        let mut store = state.store.write().await;
        let now = now_epoch_millis();
        let mut changed_devices = Vec::new();
        let mut should_persist = false;

        for device in store.devices.values_mut() {
            if device.online
                && now.saturating_sub(device.last_seen_epoch_ms) > DEVICE_OFFLINE_AFTER_MS
            {
                device.online = false;
                changed_devices.push(device.clone());
                should_persist = true;
            }
        }

        let snapshot = if should_persist {
            Some(store.clone())
        } else {
            None
        };
        drop(store);

        if let Some(snapshot) = snapshot {
            if let Err(error) = persist_snapshot(&state, &snapshot) {
                eprintln!("failed to persist relay state: {}", error.message);
            }
        }
        for device in changed_devices {
            emit_device(&state, device).await;
        }
    }
}

async fn emit_device(state: &AppState, device: DeviceRecord) {
    let _ = state.events_tx.send(RelayEventEnvelope {
        tenant_id: device.tenant_id.clone(),
        event_type: RelayEventType::DeviceUpdated,
        device: Some(device),
        task: None,
        task_event: None,
    });
}

async fn emit_task(state: &AppState, task: TaskRecord) {
    let _ = state.events_tx.send(RelayEventEnvelope {
        tenant_id: task.tenant_id.clone(),
        event_type: RelayEventType::TaskUpdated,
        device: None,
        task: Some(task),
        task_event: None,
    });
}

async fn emit_task_event(state: &AppState, task_event: TaskEvent) {
    let tenant_id = state
        .store
        .read()
        .await
        .tasks
        .get(&task_event.task_id)
        .map(|entry| entry.record.tenant_id.clone())
        .unwrap_or_default();
    let _ = state.events_tx.send(RelayEventEnvelope {
        tenant_id,
        event_type: RelayEventType::TaskEvent,
        device: None,
        task: None,
        task_event: Some(task_event),
    });
}

fn persist_snapshot(state: &AppState, snapshot: &RelayStore) -> Result<(), ApiError> {
    state.storage.save(snapshot).map_err(|error| {
        ApiError::internal(
            "persist_failed",
            format!("failed to persist relay state: {error}"),
        )
    })
}

async fn shell_session_sender(state: &AppState, session_id: &str) -> broadcast::Sender<String> {
    let mut updates = state.shell_session_updates.write().await;
    if let Some(sender) = updates.get(session_id) {
        return sender.clone();
    }

    let (sender, _) = broadcast::channel(64);
    updates.insert(session_id.to_string(), sender.clone());
    sender
}

async fn publish_shell_session_detail(state: &AppState, detail: &ShellSessionDetailResponse) {
    let Ok(payload) = serde_json::to_string(detail) else {
        return;
    };
    let sender = shell_session_sender(state, &detail.session.id).await;
    let _ = sender.send(payload);
}

fn shell_session_detail(entry: &ShellSessionEntry) -> ShellSessionDetailResponse {
    ShellSessionDetailResponse {
        session: entry.record.clone(),
        inputs: entry.inputs.clone(),
        outputs: entry.outputs.clone(),
    }
}

fn push_shell_output(
    entry: &mut ShellSessionEntry,
    stream: vibe_core::ShellStreamKind,
    data: String,
    timestamp_epoch_ms: u64,
) {
    entry.record.last_output_seq += 1;
    entry.outputs.push(ShellOutputChunk {
        seq: entry.record.last_output_seq,
        session_id: entry.record.id.clone(),
        stream,
        data,
        timestamp_epoch_ms,
    });
    if entry.outputs.len() > SHELL_OUTPUT_LIMIT_MAX {
        let excess = entry.outputs.len() - SHELL_OUTPUT_LIMIT_MAX;
        entry.outputs.drain(0..excess);
    }
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

const HEARTBEAT_SWEEP_MS: u64 = 5_000;
const TASK_LIST_LIMIT_MAX: usize = 500;
const SHELL_SESSION_LIST_LIMIT_MAX: usize = 100;
const SHELL_OUTPUT_LIMIT_MAX: usize = 1_024;
const PORT_FORWARD_LIST_LIMIT_MAX: usize = 100;
const AUDIT_RECORD_LIMIT_MAX: usize = 500;
const DEFAULT_SHELL_BRIDGE_PORT: u16 = 19_090;
const DEFAULT_PORT_FORWARD_BRIDGE_PORT: u16 = 19_091;
const DEFAULT_TASK_BRIDGE_PORT: u16 = 19_092;
const SHELL_BRIDGE_POLL_MS: u64 = 100;
const TASK_BRIDGE_POLL_MS: u64 = 100;
const MAX_BRIDGE_FRAME_BYTES: usize = 8 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request as HttpRequest};
    use futures_util::SinkExt;
    use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
    use vibe_core::UserRole;

    #[test]
    fn query_access_token_extracts_and_decodes_token() {
        let token = query_access_token("foo=bar&access_token=alpha%2Bbeta%3D");
        assert_eq!(token.as_deref(), Some("alpha+beta="));
    }

    #[test]
    fn request_access_token_prefers_bearer_header() {
        let request = HttpRequest::builder()
            .uri("/api/tasks?access_token=query-token")
            .header(axum::http::header::AUTHORIZATION, "Bearer header-token")
            .body(Body::empty())
            .unwrap();

        assert_eq!(
            request_access_token(&request),
            Some("header-token".to_string())
        );
    }

    fn test_state() -> AppState {
        test_state_with_store(RelayStore::default())
    }

    fn test_state_with_store(store: RelayStore) -> AppState {
        test_state_with_store_and_config(store, |_| {})
    }

    fn test_state_with_store_and_config<F>(mut store: RelayStore, configure: F) -> AppState
    where
        F: FnOnce(&mut RelayConfig),
    {
        let (events_tx, _) = broadcast::channel(16);
        for device_id in store.devices.keys() {
            store
                .device_credentials
                .entry(device_id.clone())
                .or_insert_with(|| DeviceCredentialRecord {
                    device_id: device_id.clone(),
                    token: test_device_token(device_id),
                    issued_at_epoch_ms: 1,
                });
        }
        let mut config = RelayConfig {
            public_base_url: "http://127.0.0.1:8787".to_string(),
            access_token: None,
            enrollment_token: None,
            state_file: std::env::temp_dir()
                .join(format!("vibe-relay-test-{}", Uuid::new_v4()))
                .join("relay-state.json"),
            forward_host: "127.0.0.1".to_string(),
            forward_bind_host: "127.0.0.1".to_string(),
            forward_port_start: 39000,
            forward_port_end: 39999,
            shell_bridge_port: DEFAULT_SHELL_BRIDGE_PORT,
            port_forward_bridge_port: DEFAULT_PORT_FORWARD_BRIDGE_PORT,
            task_bridge_port: DEFAULT_TASK_BRIDGE_PORT,
            overlay_bridge_connect_timeout_ms: 100,
            overlay_bridge_start_timeout_ms: 200,
            overlay_bridge_recovery_cooldown_ms: 50,
            overlay_bridge_probe_interval_ms: 50,
            deployment_mode: vibe_core::DeploymentMode::SelfHosted,
            documentation_url: None,
            storage_kind: vibe_core::StorageKind::Memory,
            default_tenant_id: DEFAULT_TENANT_ID.to_string(),
            default_user_id: DEFAULT_USER_ID.to_string(),
            default_user_role: UserRole::Owner,
        };
        configure(&mut config);

        AppState {
            store: Arc::new(RwLock::new(store)),
            storage: build_relay_storage(config.storage_kind.clone(), config.state_file.clone()),
            events_tx,
            shell_session_updates: Arc::new(RwLock::new(HashMap::new())),
            workspace_requests: Arc::new(RwLock::new(HashMap::new())),
            git_requests: Arc::new(RwLock::new(HashMap::new())),
            overlay_bridge_health: Arc::new(StdRwLock::new(HashMap::new())),
            config: Arc::new(config),
        }
    }

    fn empty_heartbeat_request() -> HeartbeatRequest {
        HeartbeatRequest {
            metadata: Default::default(),
            providers: vec![],
            overlay: Default::default(),
            current_task_id: None,
        }
    }

    fn test_headers() -> HeaderMap {
        HeaderMap::new()
    }

    fn test_device_token(device_id: &str) -> String {
        format!("test-device-token-{device_id}")
    }

    fn test_device_headers(_state: &AppState, device_id: &str) -> HeaderMap {
        let token = test_device_token(device_id);
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        headers
    }

    fn test_device(id: &str, capabilities: Vec<DeviceCapability>) -> DeviceRecord {
        DeviceRecord {
            tenant_id: "personal".to_string(),
            user_id: "local-admin".to_string(),
            id: id.to_string(),
            name: format!("device-{id}"),
            platform: vibe_core::DevicePlatform::Linux,
            capabilities,
            metadata: HashMap::new().into_iter().collect(),
            providers: vec![],
            overlay: Default::default(),
            online: true,
            last_seen_epoch_ms: 1,
            current_task_id: None,
        }
    }

    fn test_task(
        id: &str,
        device_id: &str,
        provider: ProviderKind,
        status: TaskStatus,
        created_at_epoch_ms: u64,
    ) -> TaskEntry {
        TaskEntry {
            record: TaskRecord {
                tenant_id: "personal".to_string(),
                user_id: "local-admin".to_string(),
                id: id.to_string(),
                device_id: device_id.to_string(),
                title: format!("Task {id}"),
                provider,
                execution_protocol: vibe_core::ExecutionProtocol::Cli,
                prompt: "prompt".to_string(),
                cwd: None,
                model: None,
                transport: TaskTransportKind::RelayPolling,
                status,
                cancel_requested: false,
                created_at_epoch_ms,
                started_at_epoch_ms: None,
                finished_at_epoch_ms: None,
                exit_code: None,
                error: None,
                last_event_seq: 0,
            },
            events: vec![],
        }
    }

    fn test_provider(
        kind: ProviderKind,
        execution_protocol: vibe_core::ExecutionProtocol,
    ) -> vibe_core::ProviderStatus {
        let command = kind.label().to_lowercase();
        let supports_acp = matches!(execution_protocol, vibe_core::ExecutionProtocol::Acp);
        vibe_core::ProviderStatus {
            kind,
            command,
            available: true,
            version: Some("test".to_string()),
            execution_protocol,
            supports_acp,
            error: None,
        }
    }

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

    fn test_overlay_device(id: &str, providers: Vec<vibe_core::ProviderStatus>) -> DeviceRecord {
        let mut device = test_device(id, vec![DeviceCapability::AiSession]);
        device.providers = providers;
        device.overlay.state = OverlayState::Connected;
        device.overlay.node_ip = Some(test_local_tcp_host());
        device
    }

    async fn send_task_bridge_event_for_test<W>(writer: &mut W, event: &TaskBridgeEvent)
    where
        W: AsyncWrite + Unpin,
    {
        let mut payload = serde_json::to_string(event).unwrap();
        payload.push('\n');
        writer.write_all(payload.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    }

    async fn read_task_bridge_request_for_test<R>(
        lines: &mut tokio::io::Lines<BufReader<R>>,
    ) -> Option<TaskBridgeRequest>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        let line = lines.next_line().await.unwrap()?;
        Some(serde_json::from_str::<TaskBridgeRequest>(&line).unwrap())
    }

    async fn wait_for_task_detail<F>(
        state: &AppState,
        task_id: &str,
        predicate: F,
    ) -> TaskDetailResponse
    where
        F: Fn(&TaskDetailResponse) -> bool,
    {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let detail = {
                    let store = state.store.read().await;
                    store.tasks.get(task_id).map(task_detail)
                };
                if let Some(detail) = detail
                    && predicate(&detail)
                {
                    return detail;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }

    async fn wait_for_port_forward_detail<F>(
        state: &AppState,
        forward_id: &str,
        predicate: F,
    ) -> PortForwardDetailResponse
    where
        F: Fn(&PortForwardDetailResponse) -> bool,
    {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let detail = {
                    let store = state.store.read().await;
                    store
                        .port_forwards
                        .get(forward_id)
                        .map(|entry| PortForwardDetailResponse {
                            forward: entry.record.clone(),
                        })
                };
                if let Some(detail) = detail
                    && predicate(&detail)
                {
                    return detail;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }

    async fn wait_for_workspace_claim(
        state: &AppState,
        device_id: &str,
    ) -> vibe_core::WorkspaceOperationRequest {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let Json(response) = claim_next_workspace_request(
                    Path(device_id.to_string()),
                    State(state.clone()),
                    test_device_headers(state, device_id),
                )
                .await
                .unwrap();
                if let Some(request) = response.request {
                    return request;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }

    async fn wait_for_git_claim(
        state: &AppState,
        device_id: &str,
    ) -> vibe_core::GitOperationRequest {
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                let Json(response) = claim_next_git_request(
                    Path(device_id.to_string()),
                    State(state.clone()),
                    test_device_headers(state, device_id),
                )
                .await
                .unwrap();
                if let Some(request) = response.request {
                    return request;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap()
    }

    async fn read_port_forward_bridge_request_for_test(
        stream: &mut TcpStream,
    ) -> Option<PortForwardBridgeRequest> {
        let line = read_bridge_frame_line(stream).await.unwrap()?;
        Some(serde_json::from_str::<PortForwardBridgeRequest>(&line).unwrap())
    }

    async fn send_port_forward_bridge_event_for_test(
        stream: &mut TcpStream,
        event: &PortForwardBridgeEvent,
    ) {
        let mut payload = serde_json::to_string(event).unwrap();
        payload.push('\n');
        stream.write_all(payload.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    }

    type TestWsStream =
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>;

    fn test_websocket_url(base_url: &str, path_and_query: &str) -> String {
        if let Some(rest) = base_url.strip_prefix("http://") {
            return format!("ws://{rest}{path_and_query}");
        }
        if let Some(rest) = base_url.strip_prefix("https://") {
            return format!("wss://{rest}{path_and_query}");
        }
        panic!("unexpected base url: {base_url}");
    }

    async fn connect_test_tcp_client(host: &str, port: u16) -> TcpStream {
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

    async fn read_tunnel_control_for_test(
        ws_stream: &mut TestWsStream,
    ) -> PortForwardTunnelControl {
        let message = tokio::time::timeout(Duration::from_secs(2), ws_stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        match message {
            WsMessage::Text(payload) => {
                serde_json::from_str::<PortForwardTunnelControl>(&payload).unwrap()
            }
            other => panic!("unexpected websocket message: {other:?}"),
        }
    }

    async fn read_tunnel_binary_for_test(ws_stream: &mut TestWsStream) -> Vec<u8> {
        let message = tokio::time::timeout(Duration::from_secs(2), ws_stream.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        match message {
            WsMessage::Binary(payload) => payload.to_vec(),
            other => panic!("unexpected websocket message: {other:?}"),
        }
    }

    async fn spawn_test_server(
        state: AppState,
    ) -> (String, tokio::sync::oneshot::Sender<()>, JoinHandle<()>) {
        let listener = TcpListener::bind((test_local_tcp_host(), 0)).await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        let app = build_app(state);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .unwrap();
        });

        (
            format!("http://{}:{}", local_addr.ip(), local_addr.port()),
            shutdown_tx,
            task,
        )
    }

    async fn read_next_sse_event(
        response: &mut reqwest::Response,
        buffer: &mut String,
    ) -> Option<(String, RelayEventEnvelope)> {
        let mut event_type = None;
        let mut data_lines = Vec::new();

        loop {
            while let Some(position) = buffer.find('\n') {
                let mut line: String = buffer.drain(..=position).collect();
                while matches!(line.chars().last(), Some('\n' | '\r')) {
                    line.pop();
                }

                if line.is_empty() {
                    if !data_lines.is_empty() || event_type.is_some() {
                        let payload = data_lines.join("\n");
                        let envelope =
                            serde_json::from_str::<RelayEventEnvelope>(&payload).unwrap();
                        return Some((event_type.unwrap_or_default(), envelope));
                    }
                    continue;
                }

                if line.starts_with(':') {
                    continue;
                }
                if let Some(value) = line.strip_prefix("event:") {
                    event_type = Some(value.trim().to_string());
                    continue;
                }
                if let Some(value) = line.strip_prefix("data:") {
                    data_lines.push(value.trim_start().to_string());
                }
            }

            match response.chunk().await.unwrap() {
                Some(chunk) => buffer.push_str(std::str::from_utf8(&chunk).unwrap()),
                None => {
                    if !data_lines.is_empty() || event_type.is_some() {
                        let payload = data_lines.join("\n");
                        let envelope =
                            serde_json::from_str::<RelayEventEnvelope>(&payload).unwrap();
                        return Some((event_type.unwrap_or_default(), envelope));
                    }
                    return None;
                }
            }
        }
    }

    fn test_port_forward(
        id: &str,
        device_id: &str,
        status: PortForwardStatus,
        relay_port: u16,
        created_at_epoch_ms: u64,
    ) -> PortForwardEntry {
        PortForwardEntry {
            record: PortForwardRecord {
                tenant_id: "personal".to_string(),
                user_id: "local-admin".to_string(),
                id: id.to_string(),
                device_id: device_id.to_string(),
                protocol: vibe_core::PortForwardProtocol::Tcp,
                relay_host: "127.0.0.1".to_string(),
                relay_port,
                target_host: "127.0.0.1".to_string(),
                target_port: 22,
                transport: PortForwardTransportKind::RelayTunnel,
                status,
                created_at_epoch_ms,
                started_at_epoch_ms: None,
                finished_at_epoch_ms: None,
                error: None,
            },
        }
    }

    #[tokio::test]
    async fn heartbeat_requires_device_credential() {
        let error = device_heartbeat(
            Path("missing-device".to_string()),
            State(test_state()),
            test_headers(),
            Json(empty_heartbeat_request()),
        )
        .await
        .unwrap_err();

        assert_eq!(error.status, StatusCode::UNAUTHORIZED);
        assert_eq!(error.code, "device_auth_required");
    }

    #[tokio::test]
    async fn claim_next_task_requires_device_credential() {
        let error = claim_next_task(
            Path("missing-device".to_string()),
            State(test_state()),
            test_headers(),
        )
        .await
        .unwrap_err();

        assert_eq!(error.status, StatusCode::UNAUTHORIZED);
        assert_eq!(error.code, "device_auth_required");
    }

    #[test]
    fn preferred_task_transport_uses_overlay_when_device_is_connected() {
        let state = test_state();
        let mut device = test_device("device-1", vec![DeviceCapability::AiSession]);
        device.overlay.state = OverlayState::Connected;
        device.overlay.node_ip = Some("10.144.0.2".to_string());

        assert_eq!(
            preferred_task_transport(&state, &device),
            TaskTransportKind::OverlayProxy
        );
    }

    #[tokio::test]
    async fn create_task_uses_relay_polling_when_task_bridge_is_marked_unavailable() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_overlay_device(
                    "device-1",
                    vec![test_provider(
                        ProviderKind::Codex,
                        vibe_core::ExecutionProtocol::Cli,
                    )],
                ),
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        });
        mark_overlay_bridge_unavailable(
            &state,
            "device-1",
            OverlayBridgeKind::Task,
            "simulated task bridge failure",
        );

        let Json(created) = create_task(
            State(state),
            test_headers(),
            Json(CreateTaskRequest {
                device_id: "device-1".to_string(),
                provider: ProviderKind::Codex,
                prompt: "hello".to_string(),
                cwd: None,
                model: None,
                title: Some("fallback task".to_string()),
            }),
        )
        .await
        .unwrap();

        assert_eq!(created.task.transport, TaskTransportKind::RelayPolling);
    }

    #[tokio::test]
    async fn claim_next_task_skips_overlay_proxy_pending_tasks() {
        let mut overlay = test_task(
            "task-overlay",
            "device-1",
            ProviderKind::Codex,
            TaskStatus::Pending,
            10,
        );
        overlay.record.transport = TaskTransportKind::OverlayProxy;

        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::AiSession]),
            )]),
            tasks: HashMap::from([
                ("task-overlay".to_string(), overlay),
                (
                    "task-relay".to_string(),
                    test_task(
                        "task-relay",
                        "device-1",
                        ProviderKind::Codex,
                        TaskStatus::Pending,
                        20,
                    ),
                ),
            ]),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        });

        let Json(claimed) = claim_next_task(
            Path("device-1".to_string()),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
        )
        .await
        .unwrap();

        assert_eq!(
            claimed.task.as_ref().map(|task| task.id.as_str()),
            Some("task-relay")
        );

        let store = state.store.read().await;
        assert_eq!(
            store.tasks["task-overlay"].record.status,
            TaskStatus::Pending
        );
        assert_eq!(
            store.tasks["task-relay"].record.status,
            TaskStatus::Assigned
        );
    }

    #[tokio::test]
    async fn browse_workspace_rejects_device_without_capability() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([("device-1".to_string(), test_device("device-1", vec![]))]),
            ..RelayStore::default()
        });

        let error = browse_workspace(
            State(state),
            test_headers(),
            Json(vibe_core::WorkspaceBrowseRequest {
                device_id: "device-1".to_string(),
                session_cwd: Some("src".to_string()),
                path: Some("docs".to_string()),
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(error.status, StatusCode::CONFLICT);
        assert_eq!(error.code, "workspace_browse_unavailable");
    }

    #[tokio::test]
    async fn browse_workspace_round_trip_claims_and_completes_request() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::WorkspaceBrowse]),
            )]),
            ..RelayStore::default()
        });

        let browse_state = state.clone();
        let browse_task = tokio::spawn(async move {
            browse_workspace(
                State(browse_state),
                test_headers(),
                Json(vibe_core::WorkspaceBrowseRequest {
                    device_id: "device-1".to_string(),
                    session_cwd: Some("src".to_string()),
                    path: Some("docs".to_string()),
                }),
            )
            .await
        });

        let request = wait_for_workspace_claim(&state, "device-1").await;
        let request_id = request.id().to_string();
        match &request {
            vibe_core::WorkspaceOperationRequest::Browse {
                device_id,
                session_cwd,
                path,
                ..
            } => {
                assert_eq!(device_id, "device-1");
                assert_eq!(session_cwd.as_deref(), Some("src"));
                assert_eq!(path.as_deref(), Some("docs"));
            }
            other => panic!("unexpected workspace request: {other:?}"),
        }

        complete_workspace_request(
            Path(request_id),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
            Json(vibe_core::CompleteWorkspaceOperationRequest {
                device_id: "device-1".to_string(),
                result: vibe_core::WorkspaceOperationResult::Browse {
                    response: vibe_core::WorkspaceBrowseResponse {
                        device_id: "device-1".to_string(),
                        root_path: "/repo".to_string(),
                        path: "/repo/docs".to_string(),
                        parent_path: Some("/repo".to_string()),
                        entries: vec![vibe_core::WorkspaceEntry {
                            path: "/repo/docs/readme.md".to_string(),
                            name: "readme.md".to_string(),
                            kind: vibe_core::WorkspaceEntryKind::File,
                            size_bytes: Some(128),
                            modified_at_epoch_ms: Some(42),
                        }],
                    },
                },
            }),
        )
        .await
        .unwrap();

        let Json(response) = browse_task.await.unwrap().unwrap();
        assert_eq!(response.device_id, "device-1");
        assert_eq!(response.root_path, "/repo");
        assert_eq!(response.path, "/repo/docs");
        assert_eq!(response.entries.len(), 1);

        let requests = state.workspace_requests.read().await;
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn complete_workspace_request_rejects_device_mismatch() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([
                (
                    "device-1".to_string(),
                    test_device("device-1", vec![DeviceCapability::WorkspaceBrowse]),
                ),
                ("device-2".to_string(), test_device("device-2", vec![])),
            ]),
            ..RelayStore::default()
        });

        let preview_state = state.clone();
        let preview_task = tokio::spawn(async move {
            preview_workspace_file(
                State(preview_state),
                test_headers(),
                Json(vibe_core::WorkspaceFilePreviewRequest {
                    device_id: "device-1".to_string(),
                    session_cwd: Some("src".to_string()),
                    path: "README.md".to_string(),
                    line: Some(2),
                    limit: Some(3),
                }),
            )
            .await
        });

        let request = wait_for_workspace_claim(&state, "device-1").await;
        let request_id = request.id().to_string();
        let error = complete_workspace_request(
            Path(request_id.clone()),
            State(state.clone()),
            test_device_headers(&state, "device-2"),
            Json(vibe_core::CompleteWorkspaceOperationRequest {
                device_id: "device-2".to_string(),
                result: vibe_core::WorkspaceOperationResult::Error {
                    message: "wrong device".to_string(),
                },
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, "workspace_device_mismatch");

        complete_workspace_request(
            Path(request_id),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
            Json(vibe_core::CompleteWorkspaceOperationRequest {
                device_id: "device-1".to_string(),
                result: vibe_core::WorkspaceOperationResult::Preview {
                    response: vibe_core::WorkspaceFilePreviewResponse {
                        device_id: "device-1".to_string(),
                        root_path: "/repo".to_string(),
                        path: "/repo/README.md".to_string(),
                        kind: vibe_core::WorkspacePreviewKind::Text,
                        content: Some("line 2\nline 3".to_string()),
                        truncated: false,
                        line: Some(2),
                        total_lines: Some(4),
                        size_bytes: 24,
                    },
                },
            }),
        )
        .await
        .unwrap();

        let Json(response) = preview_task.await.unwrap().unwrap();
        assert_eq!(response.kind, vibe_core::WorkspacePreviewKind::Text);
        assert_eq!(response.line, Some(2));
    }

    #[tokio::test]
    async fn inspect_git_workspace_rejects_device_without_capability() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([("device-1".to_string(), test_device("device-1", vec![]))]),
            ..RelayStore::default()
        });

        let error = inspect_git_workspace(
            State(state),
            test_headers(),
            Json(vibe_core::GitInspectRequest {
                device_id: "device-1".to_string(),
                session_cwd: Some("src".to_string()),
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(error.status, StatusCode::CONFLICT);
        assert_eq!(error.code, "git_inspect_unavailable");
    }

    #[tokio::test]
    async fn inspect_git_workspace_round_trip_claims_and_completes_request() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::GitInspect]),
            )]),
            ..RelayStore::default()
        });

        let inspect_state = state.clone();
        let inspect_task = tokio::spawn(async move {
            inspect_git_workspace(
                State(inspect_state),
                test_headers(),
                Json(vibe_core::GitInspectRequest {
                    device_id: "device-1".to_string(),
                    session_cwd: Some("src".to_string()),
                }),
            )
            .await
        });

        let request = wait_for_git_claim(&state, "device-1").await;
        let request_id = request.id().to_string();
        match &request {
            vibe_core::GitOperationRequest::Inspect {
                device_id,
                session_cwd,
                ..
            } => {
                assert_eq!(device_id, "device-1");
                assert_eq!(session_cwd.as_deref(), Some("src"));
            }
        }

        complete_git_request(
            Path(request_id),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
            Json(vibe_core::CompleteGitOperationRequest {
                device_id: "device-1".to_string(),
                result: vibe_core::GitOperationResult::Inspect {
                    response: vibe_core::GitInspectResponse {
                        device_id: "device-1".to_string(),
                        workspace_root: "/repo".to_string(),
                        repo_root: Some("/repo".to_string()),
                        scope_path: Some("src".to_string()),
                        state: vibe_core::GitInspectState::Ready,
                        branch_name: Some("main".to_string()),
                        upstream_branch: Some("origin/main".to_string()),
                        ahead_count: 1,
                        behind_count: 0,
                        has_commits: true,
                        changed_files: vec![vibe_core::GitChangedFile {
                            path: "/repo/src/main.rs".to_string(),
                            repo_path: "src/main.rs".to_string(),
                            status: vibe_core::GitFileStatus::Modified,
                            staged: false,
                            unstaged: true,
                        }],
                        recent_commits: vec![vibe_core::GitCommitSummary {
                            id: "0123456789abcdef".to_string(),
                            short_id: "0123456".to_string(),
                            summary: "initial".to_string(),
                            author_name: "Vibe Test".to_string(),
                            committed_at_epoch_ms: 10,
                        }],
                        diff_stats: vibe_core::GitDiffStats {
                            changed_files: 1,
                            staged_files: 0,
                            unstaged_files: 1,
                            untracked_files: 0,
                            conflicted_files: 0,
                            staged_additions: 0,
                            staged_deletions: 0,
                            unstaged_additions: 12,
                            unstaged_deletions: 2,
                        },
                    },
                },
            }),
        )
        .await
        .unwrap();

        let Json(response) = inspect_task.await.unwrap().unwrap();
        assert_eq!(response.state, vibe_core::GitInspectState::Ready);
        assert_eq!(response.branch_name.as_deref(), Some("main"));
        assert_eq!(response.changed_files.len(), 1);

        let requests = state.git_requests.read().await;
        assert!(requests.is_empty());
    }

    #[tokio::test]
    async fn complete_git_request_rejects_device_mismatch() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([
                (
                    "device-1".to_string(),
                    test_device("device-1", vec![DeviceCapability::GitInspect]),
                ),
                ("device-2".to_string(), test_device("device-2", vec![])),
            ]),
            ..RelayStore::default()
        });

        let inspect_state = state.clone();
        let inspect_task = tokio::spawn(async move {
            inspect_git_workspace(
                State(inspect_state),
                test_headers(),
                Json(vibe_core::GitInspectRequest {
                    device_id: "device-1".to_string(),
                    session_cwd: None,
                }),
            )
            .await
        });

        let request = wait_for_git_claim(&state, "device-1").await;
        let request_id = request.id().to_string();
        let error = complete_git_request(
            Path(request_id.clone()),
            State(state.clone()),
            test_device_headers(&state, "device-2"),
            Json(vibe_core::CompleteGitOperationRequest {
                device_id: "device-2".to_string(),
                result: vibe_core::GitOperationResult::Error {
                    message: "wrong device".to_string(),
                },
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, "git_device_mismatch");

        complete_git_request(
            Path(request_id),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
            Json(vibe_core::CompleteGitOperationRequest {
                device_id: "device-1".to_string(),
                result: vibe_core::GitOperationResult::Error {
                    message: "git missing".to_string(),
                },
            }),
        )
        .await
        .unwrap();

        let error = inspect_task.await.unwrap().unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, "git_inspect_failed");
    }

    #[tokio::test]
    async fn task_sse_stream_reports_overlay_lifecycle_until_completion() {
        let listener = TcpListener::bind((test_local_tcp_host(), 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([(
                    "device-1".to_string(),
                    test_overlay_device(
                        "device-1",
                        vec![test_provider(
                            ProviderKind::Codex,
                            vibe_core::ExecutionProtocol::Cli,
                        )],
                    ),
                )]),
                tasks: HashMap::new(),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.task_bridge_port = port;
            },
        );
        let (base_url, shutdown_tx, server) = spawn_test_server(state.clone()).await;

        let bridge = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, mut write_half) = stream.into_split();
            let mut lines = BufReader::new(read_half).lines();
            let request = read_task_bridge_request_for_test(&mut lines).await.unwrap();
            match request {
                TaskBridgeRequest::Start { task, .. } => {
                    assert_eq!(task.device_id, "device-1");
                    assert_eq!(task.transport, TaskTransportKind::OverlayProxy);
                }
                other => panic!("unexpected bridge request: {other:?}"),
            }

            send_task_bridge_event_for_test(
                &mut write_half,
                &TaskBridgeEvent::Update {
                    status: Some(TaskStatus::Running),
                    execution_protocol: Some(vibe_core::ExecutionProtocol::Cli),
                    events: vec![vibe_core::TaskEventInput {
                        kind: vibe_core::TaskEventKind::System,
                        message: "overlay running via sse".to_string(),
                    }],
                    exit_code: None,
                    error: None,
                },
            )
            .await;
            send_task_bridge_event_for_test(
                &mut write_half,
                &TaskBridgeEvent::Update {
                    status: Some(TaskStatus::Succeeded),
                    execution_protocol: None,
                    events: vec![vibe_core::TaskEventInput {
                        kind: vibe_core::TaskEventKind::System,
                        message: "overlay finished via sse".to_string(),
                    }],
                    exit_code: Some(0),
                    error: None,
                },
            )
            .await;
        });

        let client = reqwest::Client::new();
        let mut events_response = client
            .get(format!("{base_url}/api/events/stream"))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
        let mut sse_buffer = String::new();

        let created = client
            .post(format!("{base_url}/api/tasks"))
            .json(&CreateTaskRequest {
                device_id: "device-1".to_string(),
                provider: ProviderKind::Codex,
                prompt: "say hi".to_string(),
                cwd: None,
                model: None,
                title: Some("overlay sse task".to_string()),
            })
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<CreateTaskResponse>()
            .await
            .unwrap();
        let task_id = created.task.id.clone();

        let mut saw_queued_event = false;
        let mut saw_running_update = false;
        let mut saw_succeeded_update = false;
        let mut saw_running_event = false;
        let mut saw_finished_event = false;
        let deadline = tokio::time::Instant::now() + Duration::from_secs(3);

        while tokio::time::Instant::now() < deadline {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            let event = tokio::time::timeout(
                remaining,
                read_next_sse_event(&mut events_response, &mut sse_buffer),
            )
            .await
            .unwrap();
            let Some((event_name, envelope)) = event else {
                break;
            };

            match event_name.as_str() {
                "task_updated" => {
                    if let Some(task) = envelope.task
                        && task.id == task_id
                    {
                        saw_running_update |= task.status == TaskStatus::Running;
                        saw_succeeded_update |= task.status == TaskStatus::Succeeded;
                    }
                }
                "task_event" => {
                    if let Some(task_event) = envelope.task_event
                        && task_event.task_id == task_id
                    {
                        saw_queued_event |= task_event.message == "Task queued";
                        saw_running_event |= task_event.message == "overlay running via sse";
                        saw_finished_event |= task_event.message == "overlay finished via sse";
                    }
                }
                _ => {}
            }

            if saw_queued_event
                && saw_running_update
                && saw_succeeded_update
                && saw_running_event
                && saw_finished_event
            {
                break;
            }
        }

        let detail = client
            .get(format!("{base_url}/api/tasks/{task_id}"))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<TaskDetailResponse>()
            .await
            .unwrap();

        assert!(saw_queued_event);
        assert!(saw_running_update);
        assert!(saw_succeeded_update);
        assert!(saw_running_event);
        assert!(saw_finished_event);
        assert_eq!(detail.task.status, TaskStatus::Succeeded);
        assert_eq!(detail.task.exit_code, Some(0));
        assert!(
            detail
                .events
                .iter()
                .any(|event| event.message == "overlay running via sse")
        );
        assert!(
            detail
                .events
                .iter()
                .any(|event| event.message == "overlay finished via sse")
        );

        drop(events_response);
        let _ = shutdown_tx.send(());
        server.await.unwrap();
        bridge.await.unwrap();
    }

    #[tokio::test]
    async fn create_task_dispatches_overlay_task_and_applies_bridge_updates() {
        let listener = TcpListener::bind((test_local_tcp_host(), 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([(
                    "device-1".to_string(),
                    test_overlay_device(
                        "device-1",
                        vec![test_provider(
                            ProviderKind::Codex,
                            vibe_core::ExecutionProtocol::Cli,
                        )],
                    ),
                )]),
                tasks: HashMap::new(),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.task_bridge_port = port;
            },
        );

        let bridge = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, mut write_half) = stream.into_split();
            let mut lines = BufReader::new(read_half).lines();
            let request = read_task_bridge_request_for_test(&mut lines).await.unwrap();
            match request {
                TaskBridgeRequest::Start { task, .. } => {
                    assert_eq!(task.id.len(), 36);
                    assert_eq!(task.device_id, "device-1");
                    assert_eq!(task.transport, TaskTransportKind::OverlayProxy);
                }
                other => panic!("unexpected bridge request: {other:?}"),
            }

            send_task_bridge_event_for_test(
                &mut write_half,
                &TaskBridgeEvent::Update {
                    status: Some(TaskStatus::Running),
                    execution_protocol: Some(vibe_core::ExecutionProtocol::Cli),
                    events: vec![vibe_core::TaskEventInput {
                        kind: vibe_core::TaskEventKind::System,
                        message: "overlay running".to_string(),
                    }],
                    exit_code: None,
                    error: None,
                },
            )
            .await;
            send_task_bridge_event_for_test(
                &mut write_half,
                &TaskBridgeEvent::Update {
                    status: Some(TaskStatus::Succeeded),
                    execution_protocol: None,
                    events: vec![vibe_core::TaskEventInput {
                        kind: vibe_core::TaskEventKind::System,
                        message: "overlay finished".to_string(),
                    }],
                    exit_code: Some(0),
                    error: None,
                },
            )
            .await;
        });

        let Json(created) = create_task(
            State(state.clone()),
            test_headers(),
            Json(CreateTaskRequest {
                device_id: "device-1".to_string(),
                provider: ProviderKind::Codex,
                prompt: "say hi".to_string(),
                cwd: None,
                model: None,
                title: Some("overlay task".to_string()),
            }),
        )
        .await
        .unwrap();

        assert_eq!(created.task.transport, TaskTransportKind::OverlayProxy);

        let detail = wait_for_task_detail(&state, &created.task.id, |detail| {
            detail.task.status == TaskStatus::Succeeded
        })
        .await;
        assert_eq!(detail.task.status, TaskStatus::Succeeded);
        assert_eq!(detail.task.exit_code, Some(0));
        assert!(detail.task.started_at_epoch_ms.is_some());
        assert!(detail.task.finished_at_epoch_ms.is_some());
        assert!(
            detail
                .events
                .iter()
                .any(|event| event.message == "overlay running")
        );
        assert!(
            detail
                .events
                .iter()
                .any(|event| event.message == "overlay finished")
        );
        assert!(
            state.store.read().await.devices["device-1"]
                .current_task_id
                .is_none()
        );

        bridge.await.unwrap();
    }

    #[tokio::test]
    async fn overlay_task_cancel_sends_bridge_cancel_and_records_canceled() {
        let listener = TcpListener::bind((test_local_tcp_host(), 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut task = test_task(
            "task-1",
            "device-1",
            ProviderKind::Codex,
            TaskStatus::Pending,
            10,
        );
        task.record.transport = TaskTransportKind::OverlayProxy;
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([(
                    "device-1".to_string(),
                    test_overlay_device("device-1", vec![]),
                )]),
                tasks: HashMap::from([("task-1".to_string(), task)]),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.task_bridge_port = port;
            },
        );

        let bridge = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, mut write_half) = stream.into_split();
            let mut lines = BufReader::new(read_half).lines();
            let request = read_task_bridge_request_for_test(&mut lines).await.unwrap();
            assert!(matches!(request, TaskBridgeRequest::Start { .. }));

            send_task_bridge_event_for_test(
                &mut write_half,
                &TaskBridgeEvent::Update {
                    status: Some(TaskStatus::Running),
                    execution_protocol: Some(vibe_core::ExecutionProtocol::Cli),
                    events: vec![vibe_core::TaskEventInput {
                        kind: vibe_core::TaskEventKind::System,
                        message: "running before cancel".to_string(),
                    }],
                    exit_code: None,
                    error: None,
                },
            )
            .await;

            let cancel = tokio::time::timeout(
                Duration::from_secs(2),
                read_task_bridge_request_for_test(&mut lines),
            )
            .await
            .unwrap()
            .unwrap();
            assert!(matches!(cancel, TaskBridgeRequest::Cancel));

            send_task_bridge_event_for_test(
                &mut write_half,
                &TaskBridgeEvent::Update {
                    status: Some(TaskStatus::Canceled),
                    execution_protocol: None,
                    events: vec![vibe_core::TaskEventInput {
                        kind: vibe_core::TaskEventKind::System,
                        message: "task canceled by bridge".to_string(),
                    }],
                    exit_code: None,
                    error: None,
                },
            )
            .await;
        });

        dispatch_next_task_for_device(&state, "device-1")
            .await
            .unwrap();
        let _ = wait_for_task_detail(&state, "task-1", |detail| {
            detail.task.status == TaskStatus::Running
        })
        .await;

        let Json(cancelled) = cancel_task(
            Path("task-1".to_string()),
            State(state.clone()),
            test_headers(),
        )
        .await
        .unwrap();
        assert_eq!(cancelled.task.status, TaskStatus::CancelRequested);

        let detail = wait_for_task_detail(&state, "task-1", |detail| {
            detail.task.status == TaskStatus::Canceled
        })
        .await;
        assert_eq!(detail.task.status, TaskStatus::Canceled);
        assert!(
            detail
                .events
                .iter()
                .any(|event| event.message == "task canceled by bridge")
        );
        assert!(
            state.store.read().await.devices["device-1"]
                .current_task_id
                .is_none()
        );

        bridge.await.unwrap();
    }

    #[tokio::test]
    async fn overlay_task_connect_failure_falls_back_to_relay_polling() {
        let listener = TcpListener::bind((test_local_tcp_host(), 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut task = test_task(
            "task-1",
            "device-1",
            ProviderKind::Codex,
            TaskStatus::Pending,
            10,
        );
        task.record.transport = TaskTransportKind::OverlayProxy;
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([(
                    "device-1".to_string(),
                    test_overlay_device("device-1", vec![]),
                )]),
                tasks: HashMap::from([("task-1".to_string(), task)]),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.task_bridge_port = port;
            },
        );

        dispatch_next_task_for_device(&state, "device-1")
            .await
            .unwrap();

        let detail = wait_for_task_detail(&state, "task-1", |detail| {
            detail.task.transport == TaskTransportKind::RelayPolling
                && detail.task.status == TaskStatus::Pending
        })
        .await;
        assert_eq!(detail.task.transport, TaskTransportKind::RelayPolling);
        assert_eq!(detail.task.status, TaskStatus::Pending);
        assert!(
            detail
                .events
                .iter()
                .any(|event| event.message.contains("falling back to relay polling"))
        );
        assert!(
            state.store.read().await.devices["device-1"]
                .current_task_id
                .is_none()
        );
    }

    #[tokio::test]
    async fn overlay_task_start_timeout_falls_back_to_relay_polling() {
        let listener = TcpListener::bind((test_local_tcp_host(), 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let mut task = test_task(
            "task-1",
            "device-1",
            ProviderKind::Codex,
            TaskStatus::Pending,
            10,
        );
        task.record.transport = TaskTransportKind::OverlayProxy;
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([(
                    "device-1".to_string(),
                    test_overlay_device("device-1", vec![]),
                )]),
                tasks: HashMap::from([("task-1".to_string(), task)]),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.task_bridge_port = port;
                config.overlay_bridge_start_timeout_ms = 100;
            },
        );

        let bridge = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, _write_half) = stream.into_split();
            let mut lines = BufReader::new(read_half).lines();
            let request = read_task_bridge_request_for_test(&mut lines).await.unwrap();
            assert!(matches!(request, TaskBridgeRequest::Start { .. }));
            tokio::time::sleep(Duration::from_millis(300)).await;
        });

        dispatch_next_task_for_device(&state, "device-1")
            .await
            .unwrap();

        let detail = wait_for_task_detail(&state, "task-1", |detail| {
            detail.task.transport == TaskTransportKind::RelayPolling
                && detail.task.status == TaskStatus::Pending
        })
        .await;
        assert_eq!(detail.task.transport, TaskTransportKind::RelayPolling);
        assert_eq!(detail.task.status, TaskStatus::Pending);
        assert!(
            detail
                .events
                .iter()
                .any(|event| event.message.contains("did not acknowledge start"))
        );

        bridge.await.unwrap();
    }

    #[tokio::test]
    async fn overlay_task_bridge_probe_restores_overlay_preference_after_recovery() {
        let host = test_local_tcp_host();
        let listener = TcpListener::bind((host.as_str(), 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let device = {
            let mut device = test_overlay_device("device-1", vec![]);
            device.overlay.node_ip = Some(host.clone());
            device
        };
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([("device-1".to_string(), device.clone())]),
                tasks: HashMap::new(),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.task_bridge_port = port;
                config.overlay_bridge_recovery_cooldown_ms = 50;
                config.overlay_bridge_probe_interval_ms = 50;
                config.overlay_bridge_connect_timeout_ms = 100;
            },
        );

        mark_overlay_bridge_unavailable(
            &state,
            "device-1",
            OverlayBridgeKind::Task,
            "simulated task bridge failure",
        );
        assert_eq!(
            preferred_task_transport(&state, &device),
            TaskTransportKind::RelayPolling
        );

        let probe = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
        });

        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if preferred_task_transport(&state, &device) == TaskTransportKind::OverlayProxy {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();

        probe.await.unwrap();
    }

    #[tokio::test]
    async fn overlay_task_disconnect_after_start_marks_task_failed() {
        let listener = TcpListener::bind((test_local_tcp_host(), 0)).await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let mut task = test_task(
            "task-1",
            "device-1",
            ProviderKind::Codex,
            TaskStatus::Pending,
            10,
        );
        task.record.transport = TaskTransportKind::OverlayProxy;
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([(
                    "device-1".to_string(),
                    test_overlay_device("device-1", vec![]),
                )]),
                tasks: HashMap::from([("task-1".to_string(), task)]),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.task_bridge_port = port;
            },
        );

        let bridge = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, mut write_half) = stream.into_split();
            let mut lines = BufReader::new(read_half).lines();
            let request = read_task_bridge_request_for_test(&mut lines).await.unwrap();
            assert!(matches!(request, TaskBridgeRequest::Start { .. }));

            send_task_bridge_event_for_test(
                &mut write_half,
                &TaskBridgeEvent::Update {
                    status: Some(TaskStatus::Running),
                    execution_protocol: Some(vibe_core::ExecutionProtocol::Cli),
                    events: vec![vibe_core::TaskEventInput {
                        kind: vibe_core::TaskEventKind::System,
                        message: "running before disconnect".to_string(),
                    }],
                    exit_code: None,
                    error: None,
                },
            )
            .await;
        });

        dispatch_next_task_for_device(&state, "device-1")
            .await
            .unwrap();

        let detail = wait_for_task_detail(&state, "task-1", |detail| {
            detail.task.status == TaskStatus::Failed
        })
        .await;
        assert_eq!(detail.task.status, TaskStatus::Failed);
        assert!(
            detail
                .task
                .error
                .as_deref()
                .is_some_and(|error| error.contains("closed unexpectedly"))
        );
        assert!(
            detail
                .events
                .iter()
                .any(|event| event.message.contains("closed unexpectedly"))
        );
        assert!(
            state.store.read().await.devices["device-1"]
                .current_task_id
                .is_none()
        );

        bridge.await.unwrap();
    }

    #[tokio::test]
    async fn list_tasks_applies_filters_and_limit() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::new(),
            tasks: HashMap::from([
                (
                    "task-1".to_string(),
                    test_task(
                        "task-1",
                        "device-1",
                        ProviderKind::Codex,
                        TaskStatus::Running,
                        10,
                    ),
                ),
                (
                    "task-2".to_string(),
                    test_task(
                        "task-2",
                        "device-1",
                        ProviderKind::Codex,
                        TaskStatus::Running,
                        20,
                    ),
                ),
                (
                    "task-3".to_string(),
                    test_task(
                        "task-3",
                        "device-2",
                        ProviderKind::Codex,
                        TaskStatus::Running,
                        30,
                    ),
                ),
                (
                    "task-4".to_string(),
                    test_task(
                        "task-4",
                        "device-1",
                        ProviderKind::ClaudeCode,
                        TaskStatus::Pending,
                        40,
                    ),
                ),
            ]),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        });

        let Json(tasks) = list_tasks(
            State(state),
            Query(TaskListQuery {
                device_id: Some("device-1".to_string()),
                status: Some(TaskStatus::Running),
                provider: Some(ProviderKind::Codex),
                limit: Some(1),
            }),
            test_headers(),
        )
        .await
        .unwrap();

        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].id, "task-2");
    }

    #[tokio::test]
    async fn create_claim_and_close_shell_session_round_trip() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::Shell]),
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        });

        let Json(created) = create_shell_session(
            State(state.clone()),
            test_headers(),
            Json(CreateShellSessionRequest {
                device_id: "device-1".to_string(),
                cwd: Some("/tmp".to_string()),
            }),
        )
        .await
        .unwrap();
        assert_eq!(created.session.status, ShellSessionStatus::Pending);
        assert_eq!(created.session.transport, ShellTransportKind::RelayPolling);

        let Json(claimed) = claim_next_shell_session(
            Path("device-1".to_string()),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
        )
        .await
        .unwrap();
        assert_eq!(
            claimed.session.as_ref().map(|session| &session.status),
            Some(&ShellSessionStatus::Active)
        );

        let Json(detail) = append_shell_input(
            Path(created.session.id.clone()),
            State(state.clone()),
            test_headers(),
            Json(CreateShellInputRequest {
                data: "pwd\n".to_string(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(detail.inputs.len(), 1);
        assert_eq!(detail.inputs[0].data, "pwd\n");

        let Json(closed) = close_shell_session(
            Path(created.session.id.clone()),
            State(state),
            test_headers(),
        )
        .await
        .unwrap();
        assert_eq!(closed.session.status, ShellSessionStatus::CloseRequested);
        assert!(closed.session.close_requested);
        assert_eq!(
            closed.outputs.last().map(|chunk| chunk.data.as_str()),
            Some("Shell session close requested")
        );
    }

    #[tokio::test]
    async fn list_shell_sessions_applies_filters_and_limit() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::Shell]),
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::from([
                (
                    "shell-1".to_string(),
                    ShellSessionEntry {
                        record: ShellSessionRecord {
                            tenant_id: "personal".to_string(),
                            user_id: "local-admin".to_string(),
                            id: "shell-1".to_string(),
                            device_id: "device-1".to_string(),
                            cwd: None,
                            transport: ShellTransportKind::RelayPolling,
                            status: ShellSessionStatus::Pending,
                            close_requested: false,
                            created_at_epoch_ms: 10,
                            started_at_epoch_ms: None,
                            finished_at_epoch_ms: None,
                            exit_code: None,
                            error: None,
                            last_input_seq: 0,
                            last_output_seq: 0,
                        },
                        inputs: vec![],
                        outputs: vec![],
                    },
                ),
                (
                    "shell-2".to_string(),
                    ShellSessionEntry {
                        record: ShellSessionRecord {
                            tenant_id: "personal".to_string(),
                            user_id: "local-admin".to_string(),
                            id: "shell-2".to_string(),
                            device_id: "device-1".to_string(),
                            cwd: None,
                            transport: ShellTransportKind::RelayPolling,
                            status: ShellSessionStatus::Active,
                            close_requested: false,
                            created_at_epoch_ms: 20,
                            started_at_epoch_ms: Some(20),
                            finished_at_epoch_ms: None,
                            exit_code: None,
                            error: None,
                            last_input_seq: 0,
                            last_output_seq: 0,
                        },
                        inputs: vec![],
                        outputs: vec![],
                    },
                ),
            ]),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        });

        let Json(sessions) = list_shell_sessions(
            State(state),
            Query(ShellSessionListQuery {
                device_id: Some("device-1".to_string()),
                status: Some(ShellSessionStatus::Active),
                limit: Some(1),
            }),
            test_headers(),
        )
        .await
        .unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "shell-2");
    }

    #[test]
    fn preferred_shell_transport_uses_overlay_when_device_is_connected() {
        let state = test_state();
        let mut device = test_device("device-1", vec![DeviceCapability::Shell]);
        device.overlay.state = OverlayState::Connected;
        device.overlay.node_ip = Some("10.144.0.2".to_string());

        assert_eq!(
            preferred_shell_transport(&state, &device),
            ShellTransportKind::OverlayProxy
        );
    }

    #[tokio::test]
    async fn create_shell_session_uses_relay_polling_when_shell_bridge_is_marked_unavailable() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([("device-1".to_string(), {
                let mut device = test_device("device-1", vec![DeviceCapability::Shell]);
                device.overlay.state = OverlayState::Connected;
                device.overlay.node_ip = Some("10.144.0.2".to_string());
                device
            })]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        });
        mark_overlay_bridge_unavailable(
            &state,
            "device-1",
            OverlayBridgeKind::Shell,
            "simulated shell bridge failure",
        );

        let Json(created) = create_shell_session(
            State(state),
            test_headers(),
            Json(CreateShellSessionRequest {
                device_id: "device-1".to_string(),
                cwd: None,
            }),
        )
        .await
        .unwrap();

        assert_eq!(created.session.transport, ShellTransportKind::RelayPolling);
    }

    #[tokio::test]
    async fn claim_next_shell_session_skips_overlay_proxy_pending_sessions() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::Shell]),
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::from([
                (
                    "shell-overlay".to_string(),
                    ShellSessionEntry {
                        record: ShellSessionRecord {
                            tenant_id: "personal".to_string(),
                            user_id: "local-admin".to_string(),
                            id: "shell-overlay".to_string(),
                            device_id: "device-1".to_string(),
                            cwd: None,
                            transport: ShellTransportKind::OverlayProxy,
                            status: ShellSessionStatus::Pending,
                            close_requested: false,
                            created_at_epoch_ms: 10,
                            started_at_epoch_ms: None,
                            finished_at_epoch_ms: None,
                            exit_code: None,
                            error: None,
                            last_input_seq: 0,
                            last_output_seq: 0,
                        },
                        inputs: vec![],
                        outputs: vec![],
                    },
                ),
                (
                    "shell-relay".to_string(),
                    ShellSessionEntry {
                        record: ShellSessionRecord {
                            tenant_id: "personal".to_string(),
                            user_id: "local-admin".to_string(),
                            id: "shell-relay".to_string(),
                            device_id: "device-1".to_string(),
                            cwd: None,
                            transport: ShellTransportKind::RelayPolling,
                            status: ShellSessionStatus::Pending,
                            close_requested: false,
                            created_at_epoch_ms: 20,
                            started_at_epoch_ms: None,
                            finished_at_epoch_ms: None,
                            exit_code: None,
                            error: None,
                            last_input_seq: 0,
                            last_output_seq: 0,
                        },
                        inputs: vec![],
                        outputs: vec![],
                    },
                ),
            ]),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        });

        let Json(claimed) = claim_next_shell_session(
            Path("device-1".to_string()),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
        )
        .await
        .unwrap();

        assert_eq!(
            claimed.session.as_ref().map(|session| session.id.as_str()),
            Some("shell-relay")
        );

        let store = state.store.read().await;
        assert_eq!(
            store.shell_sessions["shell-overlay"].record.status,
            ShellSessionStatus::Pending
        );
        assert_eq!(
            store.shell_sessions["shell-relay"].record.status,
            ShellSessionStatus::Active
        );
    }

    #[tokio::test]
    async fn relay_tunnel_port_forward_proxies_tcp_data_via_websocket() {
        let host = test_local_tcp_host();
        let forward_listener = TcpListener::bind((host.as_str(), 0)).await.unwrap();
        let forward_port = forward_listener.local_addr().unwrap().port();
        drop(forward_listener);

        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([(
                    "device-1".to_string(),
                    test_device("device-1", vec![DeviceCapability::Shell]),
                )]),
                tasks: HashMap::new(),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.forward_host = host.clone();
                config.forward_bind_host = host.clone();
                config.forward_port_start = forward_port;
                config.forward_port_end = forward_port;
            },
        );
        let (base_url, shutdown_tx, server) = spawn_test_server(state.clone()).await;
        let client = reqwest::Client::new();

        let created = client
            .post(format!("{base_url}/api/port-forwards"))
            .json(&CreatePortForwardRequest {
                device_id: "device-1".to_string(),
                protocol: vibe_core::PortForwardProtocol::Tcp,
                target_host: "127.0.0.1".to_string(),
                target_port: 8080,
            })
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<CreatePortForwardResponse>()
            .await
            .unwrap();
        assert_eq!(
            created.forward.transport,
            PortForwardTransportKind::RelayTunnel
        );
        assert_eq!(created.forward.status, PortForwardStatus::Pending);

        let claimed = client
            .post(format!(
                "{base_url}/api/devices/{}/port-forwards/claim-next",
                created.forward.device_id
            ))
            .bearer_auth(test_device_token(&created.forward.device_id))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<ClaimPortForwardResponse>()
            .await
            .unwrap();
        let claimed_forward = claimed.forward.unwrap();
        assert_eq!(claimed_forward.status, PortForwardStatus::Active);
        assert_eq!(
            claimed_forward.transport,
            PortForwardTransportKind::RelayTunnel
        );

        let ws_url = test_websocket_url(
            &base_url,
            &format!(
                "/api/port-forwards/{}/tunnel/ws?deviceId=device-1&access_token={}",
                claimed_forward.id,
                test_device_token("device-1")
            ),
        );
        let (mut ws_stream, _) = connect_async(ws_url.as_str()).await.unwrap();

        let mut relay_client =
            connect_test_tcp_client(&claimed_forward.relay_host, claimed_forward.relay_port).await;
        let control = read_tunnel_control_for_test(&mut ws_stream).await;
        assert_eq!(control, PortForwardTunnelControl::ClientConnected);

        let payload = b"relay-tunnel-smoke";
        let reply = b"relay-tunnel-reply";
        ws_stream
            .send(WsMessage::Text(
                serde_json::to_string(&PortForwardTunnelControl::TargetConnected)
                    .unwrap()
                    .into(),
            ))
            .await
            .unwrap();
        relay_client.write_all(payload).await.unwrap();
        let tunneled = read_tunnel_binary_for_test(&mut ws_stream).await;
        assert_eq!(tunneled, payload);

        ws_stream
            .send(WsMessage::Binary(reply.to_vec().into()))
            .await
            .unwrap();
        let mut actual_reply = vec![0_u8; reply.len()];
        relay_client.read_exact(&mut actual_reply).await.unwrap();
        assert_eq!(actual_reply, reply);
        drop(relay_client);

        let control = read_tunnel_control_for_test(&mut ws_stream).await;
        assert_eq!(control, PortForwardTunnelControl::ClientClosed);

        let closed = client
            .post(format!(
                "{base_url}/api/port-forwards/{}/close",
                claimed_forward.id
            ))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<PortForwardDetailResponse>()
            .await
            .unwrap();
        assert_eq!(closed.forward.status, PortForwardStatus::CloseRequested);
        assert_eq!(
            closed.forward.transport,
            PortForwardTransportKind::RelayTunnel
        );

        let close_frame = tokio::time::timeout(Duration::from_secs(2), ws_stream.next())
            .await
            .unwrap();
        match close_frame {
            Some(Ok(WsMessage::Close(_))) | None => {}
            Some(Ok(other)) => panic!("unexpected websocket message: {other:?}"),
            Some(Err(error)) => panic!("unexpected websocket error: {error}"),
        }

        let detail = wait_for_port_forward_detail(&state, &claimed_forward.id, |detail| {
            detail.forward.status == PortForwardStatus::CloseRequested
        })
        .await;
        assert_eq!(
            detail.forward.transport,
            PortForwardTransportKind::RelayTunnel
        );
        assert_eq!(detail.forward.status, PortForwardStatus::CloseRequested);

        let _ = shutdown_tx.send(());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn overlay_port_forward_proxies_tcp_data_and_closes_cleanly() {
        let overlay_host = test_local_tcp_host();
        let forward_listener = TcpListener::bind((overlay_host.as_str(), 0)).await.unwrap();
        let forward_port = forward_listener.local_addr().unwrap().port();
        drop(forward_listener);
        let bridge_listener = TcpListener::bind((overlay_host.as_str(), 0)).await.unwrap();
        let bridge_port = bridge_listener.local_addr().unwrap().port();
        let target_listener = TcpListener::bind((overlay_host.as_str(), 0)).await.unwrap();
        let target_addr = target_listener.local_addr().unwrap();

        let mut device = test_overlay_device("device-1", vec![]);
        device.capabilities = vec![DeviceCapability::Shell];
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([("device-1".to_string(), device)]),
                tasks: HashMap::new(),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.forward_host = overlay_host.clone();
                config.forward_bind_host = overlay_host.clone();
                config.forward_port_start = forward_port;
                config.forward_port_end = forward_port;
                config.port_forward_bridge_port = bridge_port;
            },
        );
        let (base_url, shutdown_tx, server) = spawn_test_server(state.clone()).await;

        let target_task = tokio::spawn(async move {
            let (mut stream, _) = target_listener.accept().await.unwrap();
            let mut payload = Vec::new();
            stream.read_to_end(&mut payload).await.unwrap();
            assert_eq!(payload, b"overlay-port-forward-smoke");
            stream
                .write_all(b"target-reply:overlay-port-forward-smoke")
                .await
                .unwrap();
        });

        let client = reqwest::Client::new();
        let created = client
            .post(format!("{base_url}/api/port-forwards"))
            .json(&CreatePortForwardRequest {
                device_id: "device-1".to_string(),
                protocol: vibe_core::PortForwardProtocol::Tcp,
                target_host: target_addr.ip().to_string(),
                target_port: target_addr.port(),
            })
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<CreatePortForwardResponse>()
            .await
            .unwrap();
        assert_eq!(
            created.forward.transport,
            PortForwardTransportKind::OverlayProxy
        );

        let expected_forward_id = created.forward.id.clone();
        let expected_target_host = target_addr.ip().to_string();
        let expected_target_port = target_addr.port();
        let bridge_task = tokio::spawn(async move {
            let (mut bridge, _) = bridge_listener.accept().await.unwrap();
            let request = read_port_forward_bridge_request_for_test(&mut bridge)
                .await
                .unwrap();
            match request {
                PortForwardBridgeRequest::Start {
                    token,
                    forward_id,
                    target_host,
                    target_port,
                } => {
                    assert_eq!(token, Some(test_device_token("device-1")));
                    assert_eq!(forward_id, expected_forward_id);
                    assert_eq!(target_host, expected_target_host);
                    assert_eq!(target_port, expected_target_port);
                }
            }
            send_port_forward_bridge_event_for_test(&mut bridge, &PortForwardBridgeEvent::Ready)
                .await;
            let mut target = TcpStream::connect(target_addr).await.unwrap();
            tokio::io::copy_bidirectional(&mut bridge, &mut target)
                .await
                .unwrap();
        });

        let active_detail = wait_for_port_forward_detail(&state, &created.forward.id, |detail| {
            detail.forward.status == PortForwardStatus::Active
        })
        .await;
        assert_eq!(
            active_detail.forward.transport,
            PortForwardTransportKind::OverlayProxy
        );
        assert!(active_detail.forward.started_at_epoch_ms.is_some());

        let mut relay_client = TcpStream::connect((
            created.forward.relay_host.as_str(),
            created.forward.relay_port,
        ))
        .await
        .unwrap();
        relay_client
            .write_all(b"overlay-port-forward-smoke")
            .await
            .unwrap();
        relay_client.shutdown().await.unwrap();
        let mut reply = Vec::new();
        relay_client.read_to_end(&mut reply).await.unwrap();
        assert_eq!(reply, b"target-reply:overlay-port-forward-smoke");

        let close = client
            .post(format!(
                "{base_url}/api/port-forwards/{}/close",
                created.forward.id
            ))
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json::<PortForwardDetailResponse>()
            .await
            .unwrap();
        assert_eq!(close.forward.status, PortForwardStatus::CloseRequested);

        let closed_detail = wait_for_port_forward_detail(&state, &created.forward.id, |detail| {
            detail.forward.status == PortForwardStatus::Closed
        })
        .await;
        assert_eq!(
            closed_detail.forward.transport,
            PortForwardTransportKind::OverlayProxy
        );
        assert!(closed_detail.forward.finished_at_epoch_ms.is_some());

        bridge_task.await.unwrap();
        target_task.await.unwrap();
        let _ = shutdown_tx.send(());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn overlay_port_forward_bridge_connect_failure_falls_back_to_relay_tunnel() {
        let overlay_host = test_local_tcp_host();
        let forward_listener = TcpListener::bind((overlay_host.as_str(), 0)).await.unwrap();
        let forward_port = forward_listener.local_addr().unwrap().port();
        drop(forward_listener);
        let listener = TcpListener::bind((overlay_host.as_str(), 0)).await.unwrap();
        let bridge_port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut device = test_overlay_device("device-1", vec![]);
        device.capabilities = vec![DeviceCapability::Shell];
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([("device-1".to_string(), device)]),
                tasks: HashMap::new(),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.forward_host = overlay_host.clone();
                config.forward_bind_host = overlay_host.clone();
                config.forward_port_start = forward_port;
                config.forward_port_end = forward_port;
                config.port_forward_bridge_port = bridge_port;
            },
        );

        let Json(created) = create_port_forward(
            State(state.clone()),
            test_headers(),
            Json(CreatePortForwardRequest {
                device_id: "device-1".to_string(),
                protocol: vibe_core::PortForwardProtocol::Tcp,
                target_host: overlay_host.clone(),
                target_port: 8080,
            }),
        )
        .await
        .unwrap();
        assert_eq!(
            created.forward.transport,
            PortForwardTransportKind::OverlayProxy
        );

        let active_detail = wait_for_port_forward_detail(&state, &created.forward.id, |detail| {
            detail.forward.status == PortForwardStatus::Active
        })
        .await;
        assert_eq!(
            active_detail.forward.transport,
            PortForwardTransportKind::OverlayProxy
        );

        let mut relay_client = TcpStream::connect((
            created.forward.relay_host.as_str(),
            created.forward.relay_port,
        ))
        .await
        .unwrap();
        relay_client.write_all(b"trigger-fallback").await.unwrap();
        relay_client.shutdown().await.unwrap();
        let mut ignored = Vec::new();
        let _ = relay_client.read_to_end(&mut ignored).await;

        let detail = wait_for_port_forward_detail(&state, &created.forward.id, |detail| {
            detail.forward.transport == PortForwardTransportKind::RelayTunnel
                && detail.forward.status == PortForwardStatus::Pending
        })
        .await;
        assert_eq!(
            detail.forward.transport,
            PortForwardTransportKind::RelayTunnel
        );
        assert_eq!(detail.forward.status, PortForwardStatus::Pending);
        assert!(
            detail
                .forward
                .error
                .as_deref()
                .is_some_and(|message| message.contains("falling back to relay tunnel"))
        );
    }

    #[tokio::test]
    async fn create_claim_and_close_port_forward_round_trip() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::Shell]),
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        });

        let Json(created) = create_port_forward(
            State(state.clone()),
            test_headers(),
            Json(CreatePortForwardRequest {
                device_id: "device-1".to_string(),
                protocol: vibe_core::PortForwardProtocol::Tcp,
                target_host: "127.0.0.1".to_string(),
                target_port: 8080,
            }),
        )
        .await
        .unwrap();
        assert_eq!(created.forward.status, PortForwardStatus::Pending);
        assert_eq!(created.forward.relay_host, "127.0.0.1");
        assert_eq!(
            created.forward.transport,
            PortForwardTransportKind::RelayTunnel
        );

        let Json(claimed) = claim_next_port_forward(
            Path("device-1".to_string()),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
        )
        .await
        .unwrap();
        assert_eq!(
            claimed.forward.as_ref().map(|forward| &forward.status),
            Some(&PortForwardStatus::Active)
        );

        let Json(closed) = close_port_forward(
            Path(created.forward.id.clone()),
            State(state),
            test_headers(),
        )
        .await
        .unwrap();
        assert_eq!(closed.forward.status, PortForwardStatus::CloseRequested);
    }

    #[tokio::test]
    async fn create_port_forward_requires_configured_forward_host() {
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([(
                    "device-1".to_string(),
                    test_device("device-1", vec![DeviceCapability::Shell]),
                )]),
                tasks: HashMap::new(),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.public_base_url = String::new();
                config.forward_host = String::new();
            },
        );

        let error = create_port_forward(
            State(state),
            test_headers(),
            Json(CreatePortForwardRequest {
                device_id: "device-1".to_string(),
                protocol: vibe_core::PortForwardProtocol::Tcp,
                target_host: "127.0.0.1".to_string(),
                target_port: 8080,
            }),
        )
        .await
        .unwrap_err();

        assert_eq!(error.code, "forward_host_unconfigured");
        assert!(
            error
                .message
                .contains("VIBE_RELAY_FORWARD_HOST or VIBE_PUBLIC_RELAY_BASE_URL")
        );
    }

    #[tokio::test]
    async fn report_port_forward_state_updates_error_and_terminal_status() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::Shell]),
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::from([(
                "forward-1".to_string(),
                test_port_forward(
                    "forward-1",
                    "device-1",
                    PortForwardStatus::Active,
                    39001,
                    10,
                ),
            )]),
            ..RelayStore::default()
        });

        let Json(detail) = report_port_forward_state(
            Path("forward-1".to_string()),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
            Json(ReportPortForwardStateRequest {
                device_id: "device-1".to_string(),
                status: Some(PortForwardStatus::Closed),
                error: None,
                clear_error: true,
            }),
        )
        .await
        .unwrap();

        assert_eq!(detail.forward.status, PortForwardStatus::Closed);
        assert!(detail.forward.finished_at_epoch_ms.is_some());
        assert_eq!(detail.forward.error, None);
    }

    #[tokio::test]
    async fn list_port_forwards_applies_filters_and_limit() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::Shell]),
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::from([
                (
                    "forward-1".to_string(),
                    test_port_forward(
                        "forward-1",
                        "device-1",
                        PortForwardStatus::Pending,
                        39001,
                        10,
                    ),
                ),
                (
                    "forward-2".to_string(),
                    test_port_forward(
                        "forward-2",
                        "device-1",
                        PortForwardStatus::Active,
                        39002,
                        20,
                    ),
                ),
            ]),
            ..RelayStore::default()
        });

        let Json(forwards) = list_port_forwards(
            State(state),
            Query(PortForwardListQuery {
                device_id: Some("device-1".to_string()),
                status: Some(PortForwardStatus::Active),
                limit: Some(1),
            }),
            test_headers(),
        )
        .await
        .unwrap();

        assert_eq!(forwards.len(), 1);
        assert_eq!(forwards[0].id, "forward-2");
    }

    #[test]
    fn preferred_port_forward_transport_uses_overlay_when_device_is_connected() {
        let state = test_state();
        let mut device = test_device("device-1", vec![DeviceCapability::Shell]);
        device.overlay.state = OverlayState::Connected;
        device.overlay.node_ip = Some("10.144.0.2".to_string());

        assert_eq!(
            preferred_port_forward_transport(&state, &device),
            PortForwardTransportKind::OverlayProxy
        );
    }

    #[tokio::test]
    async fn create_port_forward_uses_relay_tunnel_when_bridge_is_marked_unavailable() {
        let host = test_local_tcp_host();
        let state = test_state_with_store_and_config(
            RelayStore {
                devices: HashMap::from([("device-1".to_string(), {
                    let mut device = test_device("device-1", vec![DeviceCapability::Shell]);
                    device.overlay.state = OverlayState::Connected;
                    device.overlay.node_ip = Some(host.clone());
                    device
                })]),
                tasks: HashMap::new(),
                shell_sessions: HashMap::new(),
                port_forwards: HashMap::new(),
                ..RelayStore::default()
            },
            |config| {
                config.forward_host = host.clone();
                config.forward_bind_host = host.clone();
            },
        );
        mark_overlay_bridge_unavailable(
            &state,
            "device-1",
            OverlayBridgeKind::PortForward,
            "simulated port-forward bridge failure",
        );

        let Json(created) = create_port_forward(
            State(state),
            test_headers(),
            Json(CreatePortForwardRequest {
                device_id: "device-1".to_string(),
                protocol: vibe_core::PortForwardProtocol::Tcp,
                target_host: "127.0.0.1".to_string(),
                target_port: 8080,
            }),
        )
        .await
        .unwrap();

        assert_eq!(
            created.forward.transport,
            PortForwardTransportKind::RelayTunnel
        );
    }

    #[tokio::test]
    async fn claim_next_port_forward_skips_overlay_proxy_pending_forwards() {
        let state = test_state_with_store(RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                test_device("device-1", vec![DeviceCapability::Shell]),
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::from([
                (
                    "forward-overlay".to_string(),
                    PortForwardEntry {
                        record: PortForwardRecord {
                            tenant_id: "personal".to_string(),
                            user_id: "local-admin".to_string(),
                            id: "forward-overlay".to_string(),
                            device_id: "device-1".to_string(),
                            protocol: vibe_core::PortForwardProtocol::Tcp,
                            relay_host: "127.0.0.1".to_string(),
                            relay_port: 39001,
                            target_host: "127.0.0.1".to_string(),
                            target_port: 22,
                            transport: PortForwardTransportKind::OverlayProxy,
                            status: PortForwardStatus::Pending,
                            created_at_epoch_ms: 10,
                            started_at_epoch_ms: None,
                            finished_at_epoch_ms: None,
                            error: None,
                        },
                    },
                ),
                (
                    "forward-relay".to_string(),
                    PortForwardEntry {
                        record: PortForwardRecord {
                            tenant_id: "personal".to_string(),
                            user_id: "local-admin".to_string(),
                            id: "forward-relay".to_string(),
                            device_id: "device-1".to_string(),
                            protocol: vibe_core::PortForwardProtocol::Tcp,
                            relay_host: "127.0.0.1".to_string(),
                            relay_port: 39002,
                            target_host: "127.0.0.1".to_string(),
                            target_port: 22,
                            transport: PortForwardTransportKind::RelayTunnel,
                            status: PortForwardStatus::Pending,
                            created_at_epoch_ms: 20,
                            started_at_epoch_ms: None,
                            finished_at_epoch_ms: None,
                            error: None,
                        },
                    },
                ),
            ]),
            ..RelayStore::default()
        });

        let Json(claimed) = claim_next_port_forward(
            Path("device-1".to_string()),
            State(state.clone()),
            test_device_headers(&state, "device-1"),
        )
        .await
        .unwrap();

        assert_eq!(
            claimed.forward.as_ref().map(|forward| forward.id.as_str()),
            Some("forward-relay")
        );

        let store = state.store.read().await;
        assert_eq!(
            store.port_forwards["forward-overlay"].record.status,
            PortForwardStatus::Pending
        );
        assert_eq!(
            store.port_forwards["forward-relay"].record.status,
            PortForwardStatus::Active
        );
    }

    #[test]
    fn persist_and_load_store_round_trip() {
        let dir = std::env::temp_dir().join(format!("vibe-relay-test-{}", Uuid::new_v4()));
        let path = dir.join("relay-state.json");
        let store = RelayStore {
            devices: HashMap::from([(
                "device-1".to_string(),
                DeviceRecord {
                    tenant_id: "personal".to_string(),
                    user_id: "local-admin".to_string(),
                    id: "device-1".to_string(),
                    name: "Workstation".to_string(),
                    platform: vibe_core::DevicePlatform::Linux,
                    capabilities: vec![vibe_core::DeviceCapability::AiSession],
                    metadata: HashMap::new().into_iter().collect(),
                    providers: vec![],
                    overlay: Default::default(),
                    online: true,
                    last_seen_epoch_ms: 1,
                    current_task_id: None,
                },
            )]),
            tasks: HashMap::new(),
            shell_sessions: HashMap::new(),
            port_forwards: HashMap::new(),
            ..RelayStore::default()
        };

        persist_relay_store(&path, &store).unwrap();
        let loaded = load_relay_store(&path).unwrap();
        assert_eq!(loaded, store);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
