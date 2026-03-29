use anyhow::{Context, Result, bail};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::{Mutex, RwLock, mpsc},
};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message as WsMessage,
};
use uuid::Uuid;
use vibe_core::{
    AppendShellOutputRequest, AppendTaskEventsRequest, ClaimGitOperationResponse,
    ClaimPortForwardResponse, ClaimShellSessionResponse, ClaimTaskResponse,
    ClaimWorkspaceOperationResponse, CompleteGitOperationRequest,
    CompleteWorkspaceOperationRequest, ConversationInputOption, ConversationInputRequest,
    ConversationInputRequestStatus, CreateConversationInputRequest, DEFAULT_TENANT_ID,
    DEFAULT_USER_ID, DeviceCapability, DevicePlatform, ExecutionProtocol, GitChangedFile,
    GitCommitSummary, GitDiffStats, GitFileStatus, GitInspectResponse, GitInspectState,
    GitOperationRequest, GitOperationResult, HEARTBEAT_INTERVAL_MS, HeartbeatRequest,
    HeartbeatResponse, OverlayNetworkStatus, PortForwardDetailResponse, PortForwardRecord,
    PortForwardStatus, PortForwardTransportKind, PortForwardTunnelControl, ProviderKind,
    ProviderStatus, RegisterDeviceRequest, RegisterDeviceResponse, ReportPortForwardStateRequest,
    ShellOutputChunkInput, ShellPendingInputResponse, ShellSessionRecord, ShellSessionStatus,
    ShellStreamKind, TaskDetailResponse, TaskEventInput, TaskEventKind, TaskRecord, TaskStatus,
    WorkspaceBrowseResponse, WorkspaceEntry, WorkspaceEntryKind, WorkspaceFilePreviewResponse,
    WorkspaceOperationRequest, WorkspaceOperationResult, WorkspacePreviewKind,
};

mod config;
mod easytier;
mod git_runtime;
mod port_forward_bridge;
mod port_forward_runtime;
mod providers;
mod shell_bridge;
mod shell_runtime;
mod task_bridge;
mod task_runtime;
mod workspace_runtime;

use config::{
    base_metadata, build_relay_websocket_url, default_agent_identity_path, default_device_name,
    ensure_task_cwd, normalize_base_url, resolve_task_cwd, resolve_working_root,
};
use easytier::{AgentEasyTierOptions, initial_overlay_status, start_managed_agent_easytier};
use git_runtime::git_loop;
use port_forward_runtime::port_forward_loop;
use providers::{
    acp_update_to_events, build_provider_command, detect_providers, provider_stdout_session_id,
    provider_stdout_to_task_events,
};
#[cfg(test)]
use providers::{claude_jsonl_to_task_events, codex_jsonl_to_task_events};
use shell_runtime::shell_session_loop;
use task_runtime::task_loop;
use workspace_runtime::workspace_loop;

const ACP_PROTOCOL_VERSION: u64 = 1;
const ACP_CANCEL_POLL_MS: u64 = 1_000;
const TERMINAL_POLL_MS: u64 = 250;

#[derive(Debug, Parser, Clone)]
struct Cli {
    #[arg(long, env = "VIBE_RELAY_URL")]
    relay_url: Option<String>,

    #[arg(long, env = "VIBE_DEVICE_NAME")]
    device_name: Option<String>,

    #[arg(long, env = "VIBE_DEVICE_ID")]
    device_id: Option<String>,

    #[arg(long, env = "VIBE_TENANT_ID", default_value = DEFAULT_TENANT_ID)]
    tenant_id: String,

    #[arg(long, env = "VIBE_USER_ID", default_value = DEFAULT_USER_ID)]
    user_id: String,

    #[arg(long, env = "VIBE_WORKING_ROOT", default_value = ".")]
    working_root: PathBuf,

    #[arg(long, env = "VIBE_POLL_INTERVAL_MS", default_value_t = 2_000)]
    poll_interval_ms: u64,

    #[arg(long, env = "VIBE_HEARTBEAT_INTERVAL_MS", default_value_t = HEARTBEAT_INTERVAL_MS)]
    heartbeat_interval_ms: u64,

    #[arg(long, env = "VIBE_RELAY_ENROLLMENT_TOKEN")]
    relay_enrollment_token: Option<String>,

    #[arg(long, env = "VIBE_RELAY_ACCESS_TOKEN")]
    relay_access_token_compat: Option<String>,

    #[arg(long, env = "VIBE_EASYTIER_NETWORK_NAME")]
    easytier_network_name: Option<String>,

    #[arg(long, env = "VIBE_EASYTIER_BOOTSTRAP_URL")]
    easytier_bootstrap_url: Option<String>,

    #[arg(long, env = "VIBE_EASYTIER_NODE_IP")]
    easytier_node_ip: Option<String>,
}

#[derive(Clone)]
struct SharedState {
    current_task_id: Arc<RwLock<Option<String>>>,
    metadata: BTreeMap<String, String>,
    providers: Vec<ProviderStatus>,
    overlay: Arc<RwLock<OverlayNetworkStatus>>,
}

#[derive(Clone)]
struct AgentProfile {
    tenant_id: String,
    user_id: String,
    device_id: String,
    device_name: String,
    platform: DevicePlatform,
    capabilities: Vec<DeviceCapability>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AgentIdentityState {
    device_id: Option<String>,
    device_credential: Option<String>,
}

#[derive(Clone)]
struct AgentAuthState {
    enrollment_token: Option<String>,
    identity_path: PathBuf,
    device_credential: Arc<RwLock<Option<String>>>,
}

impl AgentAuthState {
    fn new(
        enrollment_token: Option<String>,
        identity_path: PathBuf,
        device_credential: Option<String>,
    ) -> Self {
        Self {
            enrollment_token,
            identity_path,
            device_credential: Arc::new(RwLock::new(device_credential)),
        }
    }

    fn enrollment_token(&self) -> Option<&str> {
        self.enrollment_token.as_deref()
    }

    async fn device_credential(&self) -> Option<String> {
        self.device_credential.read().await.clone()
    }

    fn device_credential_slot(&self) -> Arc<RwLock<Option<String>>> {
        self.device_credential.clone()
    }

    async fn update_device_identity(&self, device_id: &str, device_credential: &str) -> Result<()> {
        {
            let mut guard = self.device_credential.write().await;
            *guard = Some(device_credential.to_string());
        }

        persist_agent_identity_state(
            &self.identity_path,
            &AgentIdentityState {
                device_id: Some(device_id.to_string()),
                device_credential: Some(device_credential.to_string()),
            },
        )
    }
}

enum HeartbeatOutcome {
    Accepted,
    DeviceMissing,
}

fn advertised_device_capabilities() -> Vec<DeviceCapability> {
    vec![
        DeviceCapability::AiSession,
        DeviceCapability::Shell,
        DeviceCapability::WorkspaceBrowse,
        DeviceCapability::GitInspect,
    ]
}

fn resolve_relay_url(value: Option<&str>, allow_local_dev_fallback: bool) -> Result<String> {
    if let Some(relay_url) = value.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(normalize_base_url(relay_url));
    }

    if allow_local_dev_fallback {
        return Ok("http://127.0.0.1:8787".to_string());
    }

    bail!("missing relay URL; set --relay-url or VIBE_RELAY_URL")
}

fn trim_optional_token(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn load_agent_identity_state(path: &Path) -> Result<AgentIdentityState> {
    if !path.exists() {
        return Ok(AgentIdentityState::default());
    }

    let bytes = fs::read(path)
        .with_context(|| format!("failed to read agent identity state {}", path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("failed to decode agent identity state {}", path.display()))
}

fn persist_agent_identity_state(path: &Path, state: &AgentIdentityState) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create parent directory for agent identity state {}",
                path.display()
            )
        })?;
    }

    let encoded =
        serde_json::to_vec_pretty(state).context("failed to encode agent identity state")?;
    fs::write(path, encoded)
        .with_context(|| format!("failed to persist agent identity state {}", path.display()))
}

fn with_bearer(request: reqwest::RequestBuilder, token: Option<&str>) -> reqwest::RequestBuilder {
    match token.map(str::trim).filter(|value| !value.is_empty()) {
        Some(token) => request.header(AUTHORIZATION, format!("Bearer {token}")),
        None => request,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let relay_url = resolve_relay_url(cli.relay_url.as_deref(), cfg!(debug_assertions))?;
    let working_root = resolve_working_root(&cli.working_root)?;
    let identity_path = default_agent_identity_path(&working_root);
    let identity_state = load_agent_identity_state(&identity_path)?;
    let initial_device_id = cli
        .device_id
        .clone()
        .or(identity_state.device_id.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let initial_device_credential = identity_state
        .device_id
        .as_deref()
        .filter(|stored_id| *stored_id == initial_device_id)
        .and(identity_state.device_credential.clone());
    let enrollment_token = trim_optional_token(cli.relay_enrollment_token.as_deref())
        .or_else(|| trim_optional_token(cli.relay_access_token_compat.as_deref()));
    let auth = AgentAuthState::new(enrollment_token, identity_path, initial_device_credential);
    let mut profile = AgentProfile {
        tenant_id: cli.tenant_id.clone(),
        user_id: cli.user_id.clone(),
        device_id: initial_device_id,
        device_name: cli.device_name.clone().unwrap_or_else(default_device_name),
        platform: DevicePlatform::current(),
        capabilities: advertised_device_capabilities(),
    };
    let providers = detect_providers();
    let easytier_options = AgentEasyTierOptions::from_inputs(
        &profile.device_id,
        &profile.device_name,
        cli.easytier_network_name.clone(),
        cli.easytier_bootstrap_url.clone(),
        cli.easytier_node_ip.clone(),
    );
    let metadata = base_metadata(&working_root);
    let shared = SharedState {
        current_task_id: Arc::new(RwLock::new(None)),
        metadata,
        providers,
        overlay: Arc::new(RwLock::new(initial_overlay_status(&easytier_options))),
    };
    let easytier_enabled = easytier_options.enabled();
    let easytier_runtime = start_managed_agent_easytier(easytier_options, shared.overlay.clone());

    let shell_bridge_runtime = shell_bridge::start_shell_bridge_server(
        easytier_enabled,
        working_root.clone(),
        auth.device_credential_slot(),
    );
    let port_forward_bridge_runtime = port_forward_bridge::start_port_forward_bridge_server(
        easytier_enabled,
        auth.device_credential_slot(),
    );
    let task_bridge_runtime = task_bridge::start_task_bridge_server(
        easytier_enabled,
        working_root.clone(),
        auth.device_credential_slot(),
        shared.clone(),
    );

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .context("failed to build reqwest client")?;

    if auth.device_credential().await.is_none() {
        let registered_device =
            register_current_device(&client, &relay_url, &profile, &shared, &auth).await?;

        println!(
            "registered agent {} ({}) on {}",
            registered_device.device.name, registered_device.device.id, relay_url
        );

        profile.device_id = registered_device.device.id.clone();
    } else {
        println!(
            "loaded persisted agent credential for {} ({}) on {}",
            profile.device_name, profile.device_id, relay_url
        );
    }
    let heartbeat_state = shared.clone();
    let heartbeat_client = client.clone();
    let heartbeat_relay_url = relay_url.clone();
    let heartbeat_profile = profile.clone();
    let heartbeat_auth = auth.clone();
    let heartbeat_interval_ms = cli.heartbeat_interval_ms;

    tokio::spawn(async move {
        if let Err(error) = heartbeat_loop(
            heartbeat_client,
            heartbeat_relay_url,
            heartbeat_profile,
            heartbeat_auth,
            heartbeat_state,
            heartbeat_interval_ms,
        )
        .await
        {
            eprintln!("heartbeat loop stopped: {error:#}");
        }
    });

    let shell_client = client.clone();
    let shell_relay_url = relay_url.clone();
    let shell_profile = profile.clone();
    let shell_auth = auth.clone();
    let shell_shared = shared.clone();
    let shell_working_root = working_root.clone();
    let shell_poll_interval_ms = cli.poll_interval_ms;
    tokio::spawn(async move {
        if let Err(error) = shell_session_loop(
            shell_client,
            shell_relay_url,
            shell_profile,
            shell_auth,
            shell_shared,
            shell_working_root,
            shell_poll_interval_ms,
        )
        .await
        {
            eprintln!("shell session loop stopped: {error:#}");
        }
    });

    let port_forward_client = client.clone();
    let port_forward_relay_url = relay_url.clone();
    let port_forward_profile = profile.clone();
    let port_forward_auth = auth.clone();
    let port_forward_shared = shared.clone();
    let port_forward_poll_interval_ms = cli.poll_interval_ms;
    tokio::spawn(async move {
        if let Err(error) = port_forward_loop(
            port_forward_client,
            port_forward_relay_url,
            port_forward_profile,
            port_forward_auth,
            port_forward_shared,
            port_forward_poll_interval_ms,
        )
        .await
        {
            eprintln!("port forward loop stopped: {error:#}");
        }
    });

    let workspace_client = client.clone();
    let workspace_relay_url = relay_url.clone();
    let workspace_profile = profile.clone();
    let workspace_auth = auth.clone();
    let workspace_shared = shared.clone();
    let workspace_working_root = working_root.clone();
    let workspace_poll_interval_ms = cli.poll_interval_ms;
    tokio::spawn(async move {
        if let Err(error) = workspace_loop(
            workspace_client,
            workspace_relay_url,
            workspace_profile,
            workspace_auth,
            workspace_shared,
            workspace_working_root,
            workspace_poll_interval_ms,
        )
        .await
        {
            eprintln!("workspace loop stopped: {error:#}");
        }
    });

    let git_client = client.clone();
    let git_relay_url = relay_url.clone();
    let git_profile = profile.clone();
    let git_auth = auth.clone();
    let git_shared = shared.clone();
    let git_working_root = working_root.clone();
    let git_poll_interval_ms = cli.poll_interval_ms;
    tokio::spawn(async move {
        if let Err(error) = git_loop(
            git_client,
            git_relay_url,
            git_profile,
            git_auth,
            git_shared,
            git_working_root,
            git_poll_interval_ms,
        )
        .await
        {
            eprintln!("git loop stopped: {error:#}");
        }
    });

    let task_result = task_loop(
        client,
        relay_url,
        profile,
        auth,
        shared,
        working_root,
        cli.poll_interval_ms,
    )
    .await;

    if let Some(runtime) = task_bridge_runtime {
        runtime.shutdown().await;
    }
    if let Some(runtime) = port_forward_bridge_runtime {
        runtime.shutdown().await;
    }
    if let Some(runtime) = shell_bridge_runtime {
        runtime.shutdown().await;
    }
    if let Some(runtime) = easytier_runtime {
        runtime.shutdown().await;
    }

    task_result
}

async fn register_device(
    client: &reqwest::Client,
    relay_url: &str,
    payload: RegisterDeviceRequest,
    enrollment_token: Option<&str>,
) -> Result<RegisterDeviceResponse> {
    let endpoint = format!("{relay_url}/api/devices/register");
    let response = with_bearer(client.post(endpoint), enrollment_token)
        .json(&payload)
        .send()
        .await
        .context("failed to register device")?
        .error_for_status()
        .context("relay rejected device registration")?
        .json::<RegisterDeviceResponse>()
        .await
        .context("failed to decode register response")?;

    Ok(response)
}

async fn build_register_device_request(
    profile: &AgentProfile,
    shared: &SharedState,
) -> RegisterDeviceRequest {
    RegisterDeviceRequest {
        tenant_id: profile.tenant_id.clone(),
        user_id: profile.user_id.clone(),
        id: profile.device_id.clone(),
        name: profile.device_name.clone(),
        platform: profile.platform.clone(),
        capabilities: profile.capabilities.clone(),
        metadata: shared.metadata.clone(),
        providers: shared.providers.clone(),
        overlay: shared.overlay.read().await.clone(),
    }
}

async fn register_current_device(
    client: &reqwest::Client,
    relay_url: &str,
    profile: &AgentProfile,
    shared: &SharedState,
    auth: &AgentAuthState,
) -> Result<RegisterDeviceResponse> {
    let payload = build_register_device_request(profile, shared).await;
    let response = register_device(client, relay_url, payload, auth.enrollment_token()).await?;
    auth.update_device_identity(&response.device.id, &response.device_credential)
        .await?;
    Ok(response)
}

async fn heartbeat_loop(
    client: reqwest::Client,
    relay_url: String,
    profile: AgentProfile,
    auth: AgentAuthState,
    shared: SharedState,
    heartbeat_interval_ms: u64,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(heartbeat_interval_ms));

    loop {
        interval.tick().await;
        let current_task_id = shared.current_task_id.read().await.clone();
        let overlay = shared.overlay.read().await.clone();
        let payload = HeartbeatRequest {
            metadata: shared.metadata.clone(),
            providers: shared.providers.clone(),
            overlay,
            current_task_id,
        };

        match send_heartbeat(&client, &relay_url, &profile.device_id, &auth, payload).await {
            Ok(HeartbeatOutcome::Accepted) => {}
            Ok(HeartbeatOutcome::DeviceMissing) => {
                eprintln!(
                    "device {} missing on relay during heartbeat, re-registering",
                    profile.device_id
                );
                if let Err(error) =
                    register_current_device(&client, &relay_url, &profile, &shared, &auth).await
                {
                    eprintln!("device re-registration failed: {error:#}");
                }
            }
            Err(error) => {
                eprintln!("heartbeat failed: {error:#}");
            }
        }
    }
}

async fn send_heartbeat(
    client: &reqwest::Client,
    relay_url: &str,
    device_id: &str,
    auth: &AgentAuthState,
    payload: HeartbeatRequest,
) -> Result<HeartbeatOutcome> {
    let endpoint = format!("{relay_url}/api/devices/{device_id}/heartbeat");
    let device_credential = auth.device_credential().await;
    let response = with_bearer(client.post(endpoint), device_credential.as_deref())
        .json(&payload)
        .send()
        .await
        .context("failed to send heartbeat")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(HeartbeatOutcome::DeviceMissing);
    }

    let _ = response
        .error_for_status()
        .context("heartbeat rejected by relay")?
        .json::<HeartbeatResponse>()
        .await
        .context("invalid heartbeat response")?;

    Ok(HeartbeatOutcome::Accepted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_thread_started_maps_to_system_event() {
        let events =
            codex_jsonl_to_task_events(r#"{"type":"thread.started","thread_id":"thread_123"}"#);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::System);
        assert!(events[0].message.contains("thread_123"));
    }

    #[test]
    fn codex_agent_message_maps_to_assistant_delta() {
        let events = codex_jsonl_to_task_events(
            r#"{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"pong"}}"#,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::AssistantDelta);
        assert_eq!(events[0].message, "pong");
    }

    #[test]
    fn codex_tool_like_item_maps_to_tool_event() {
        let events = codex_jsonl_to_task_events(
            r#"{"type":"item.started","item":{"id":"item_1","type":"exec_command","command":"pwd"}}"#,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::ToolCall);
        assert!(events[0].message.contains("exec_command"));
        assert!(events[0].message.contains("pwd"));
    }

    #[test]
    fn claude_init_maps_to_system_event() {
        let events = claude_jsonl_to_task_events(
            r#"{"type":"system","subtype":"init","session_id":"session_123","model":"claude-sonnet-4-6","permissionMode":"acceptEdits"}"#,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::System);
        assert!(events[0].message.contains("session_123"));
        assert!(events[0].message.contains("acceptEdits"));
    }

    #[test]
    fn claude_text_message_maps_to_assistant_delta() {
        let events = claude_jsonl_to_task_events(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"pong"}]}}"#,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::AssistantDelta);
        assert_eq!(events[0].message, "pong");
    }

    #[test]
    fn claude_tool_use_maps_to_tool_call() {
        let events = claude_jsonl_to_task_events(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"toolu_1","name":"Bash","input":{"command":"pwd"}}]}}"#,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::ToolCall);
        assert!(events[0].message.contains("Bash"));
        assert!(events[0].message.contains("pwd"));
    }

    #[test]
    fn claude_result_error_maps_to_system_event() {
        let events = claude_jsonl_to_task_events(
            r#"{"type":"result","subtype":"success","is_error":true,"duration_ms":194,"num_turns":1,"stop_reason":"stop_sequence","result":"Not logged in"}"#,
        );
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, TaskEventKind::System);
        assert!(events[0].message.contains("Not logged in"));
        assert!(events[0].message.contains("stop_sequence"));
    }

    #[test]
    fn build_relay_websocket_url_rewrites_scheme_and_query() {
        let url = build_relay_websocket_url(
            "https://relay.example.com/base",
            "/api/port-forwards/forward-1/tunnel/ws",
            "device-1",
            Some("secret"),
        )
        .unwrap();

        assert_eq!(
            url,
            "wss://relay.example.com/base/api/port-forwards/forward-1/tunnel/ws?deviceId=device-1&access_token=secret"
        );
    }

    #[test]
    fn advertised_device_capabilities_match_current_mvp_surface() {
        assert_eq!(
            advertised_device_capabilities(),
            vec![
                DeviceCapability::AiSession,
                DeviceCapability::Shell,
                DeviceCapability::WorkspaceBrowse,
                DeviceCapability::GitInspect,
            ]
        );
    }

    #[test]
    fn resolve_relay_url_prefers_explicit_value() {
        let relay_url =
            resolve_relay_url(Some(" https://relay.example.com/base/ "), false).unwrap();

        assert_eq!(relay_url, "https://relay.example.com/base");
    }

    #[test]
    fn resolve_relay_url_keeps_local_dev_fallback_when_enabled() {
        let relay_url = resolve_relay_url(None, true).unwrap();

        assert_eq!(relay_url, "http://127.0.0.1:8787");
    }

    #[test]
    fn resolve_relay_url_requires_explicit_value_outside_dev_fallback() {
        let error = resolve_relay_url(None, false).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("missing relay URL; set --relay-url or VIBE_RELAY_URL")
        );
    }

    #[tokio::test]
    async fn build_register_device_request_uses_runtime_state() {
        let shared = SharedState {
            current_task_id: Arc::new(RwLock::new(None)),
            metadata: BTreeMap::from([("arch".to_string(), "x86_64".to_string())]),
            providers: vec![ProviderStatus {
                kind: ProviderKind::OpenCode,
                command: "opencode".to_string(),
                available: true,
                version: Some("1.0.0".to_string()),
                execution_protocol: ExecutionProtocol::Acp,
                supports_acp: true,
                error: None,
            }],
            overlay: Arc::new(RwLock::new(OverlayNetworkStatus::default())),
        };
        let profile = AgentProfile {
            tenant_id: "team-a".to_string(),
            user_id: "agent-1".to_string(),
            device_id: "device-1".to_string(),
            device_name: "Workstation".to_string(),
            platform: DevicePlatform::Linux,
            capabilities: advertised_device_capabilities(),
        };

        let request = build_register_device_request(&profile, &shared).await;

        assert_eq!(request.tenant_id, "team-a");
        assert_eq!(request.user_id, "agent-1");
        assert_eq!(request.id, "device-1");
        assert_eq!(request.name, "Workstation");
        assert_eq!(request.platform, DevicePlatform::Linux);
        assert_eq!(request.capabilities, profile.capabilities);
        assert_eq!(
            request.metadata.get("arch").map(String::as_str),
            Some("x86_64")
        );
        assert_eq!(request.providers.len(), 1);
        assert_eq!(request.overlay.mode, vibe_core::OverlayMode::Disabled);
    }
}
