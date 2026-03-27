use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub const DEFAULT_TENANT_ID: &str = "personal";
pub const DEFAULT_USER_ID: &str = "local-admin";
pub const DEVICE_OFFLINE_AFTER_MS: u64 = 15_000;
pub const HEARTBEAT_INTERVAL_MS: u64 = 5_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DevicePlatform {
    Windows,
    Macos,
    Linux,
    Ios,
    Android,
    Unknown,
}

impl DevicePlatform {
    pub fn current() -> Self {
        match std::env::consts::OS {
            "windows" => Self::Windows,
            "macos" => Self::Macos,
            "linux" => Self::Linux,
            "ios" => Self::Ios,
            "android" => Self::Android,
            _ => Self::Unknown,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::Macos => "macOS",
            Self::Linux => "Linux",
            Self::Ios => "iOS",
            Self::Android => "Android",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceCapability {
    AiSession,
    Shell,
    FileSync,
    WorkspaceBrowse,
    Notifications,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Codex,
    ClaudeCode,
    OpenCode,
}

impl ProviderKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Codex => "Codex",
            Self::ClaudeCode => "Claude Code",
            Self::OpenCode => "OpenCode",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionProtocol {
    Cli,
    Acp,
}

impl ExecutionProtocol {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Cli => "CLI",
            Self::Acp => "ACP",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayMode {
    EasyTierEmbedded,
    EasyTierSidecar,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayState {
    Connected,
    Degraded,
    Disabled,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub kind: ProviderKind,
    pub command: String,
    pub available: bool,
    pub version: Option<String>,
    pub execution_protocol: ExecutionProtocol,
    pub supports_acp: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OverlayNetworkStatus {
    pub mode: OverlayMode,
    pub state: OverlayState,
    pub network_name: Option<String>,
    pub node_ip: Option<String>,
    pub relay_url: Option<String>,
    pub binary_path: Option<String>,
    pub last_error: Option<String>,
}

impl Default for OverlayNetworkStatus {
    fn default() -> Self {
        Self {
            mode: OverlayMode::Disabled,
            state: OverlayState::Disabled,
            network_name: None,
            node_ip: None,
            relay_url: None,
            binary_path: None,
            last_error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    pub tenant_id: String,
    pub user_id: String,
    pub id: String,
    pub name: String,
    pub platform: DevicePlatform,
    pub capabilities: Vec<DeviceCapability>,
    pub metadata: BTreeMap<String, String>,
    pub providers: Vec<ProviderStatus>,
    pub overlay: OverlayNetworkStatus,
    pub online: bool,
    pub last_seen_epoch_ms: u64,
    pub current_task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub id: String,
    pub name: String,
    pub platform: DevicePlatform,
    pub capabilities: Vec<DeviceCapability>,
    pub metadata: BTreeMap<String, String>,
    pub providers: Vec<ProviderStatus>,
    pub overlay: OverlayNetworkStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceResponse {
    pub device: DeviceRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRequest {
    pub metadata: BTreeMap<String, String>,
    pub providers: Vec<ProviderStatus>,
    pub overlay: OverlayNetworkStatus,
    pub current_task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatResponse {
    pub device: DeviceRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Assigned,
    Running,
    CancelRequested,
    Succeeded,
    Failed,
    Canceled,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Canceled)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskTransportKind {
    RelayPolling,
    OverlayProxy,
}

impl Default for TaskTransportKind {
    fn default() -> Self {
        Self::RelayPolling
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskEventKind {
    System,
    Status,
    ProviderStdout,
    ProviderStderr,
    AssistantDelta,
    ToolCall,
    ToolOutput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskEvent {
    pub seq: u64,
    pub task_id: String,
    pub device_id: String,
    pub kind: TaskEventKind,
    pub message: String,
    pub timestamp_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskEventInput {
    pub kind: TaskEventKind,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub tenant_id: String,
    pub user_id: String,
    pub id: String,
    pub device_id: String,
    pub title: String,
    pub provider: ProviderKind,
    pub execution_protocol: ExecutionProtocol,
    pub prompt: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    #[serde(default)]
    pub transport: TaskTransportKind,
    pub status: TaskStatus,
    pub cancel_requested: bool,
    pub created_at_epoch_ms: u64,
    pub started_at_epoch_ms: Option<u64>,
    pub finished_at_epoch_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub last_event_seq: u64,
}

impl TaskRecord {
    pub fn new(
        request: CreateTaskRequest,
        execution_protocol: ExecutionProtocol,
        transport: TaskTransportKind,
    ) -> Self {
        Self {
            tenant_id: DEFAULT_TENANT_ID.to_string(),
            user_id: DEFAULT_USER_ID.to_string(),
            id: Uuid::new_v4().to_string(),
            device_id: request.device_id,
            title: request
                .title
                .filter(|title| !title.trim().is_empty())
                .unwrap_or_else(|| "Ad hoc AI task".to_string()),
            provider: request.provider,
            execution_protocol,
            prompt: request.prompt,
            cwd: request.cwd,
            model: request.model,
            transport,
            status: TaskStatus::Pending,
            cancel_requested: false,
            created_at_epoch_ms: now_epoch_millis(),
            started_at_epoch_ms: None,
            finished_at_epoch_ms: None,
            exit_code: None,
            error: None,
            last_event_seq: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub device_id: String,
    pub provider: ProviderKind,
    pub prompt: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskResponse {
    pub task: TaskRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaimTaskResponse {
    pub task: Option<TaskRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppendTaskEventsRequest {
    pub device_id: String,
    pub status: Option<TaskStatus>,
    pub execution_protocol: Option<ExecutionProtocol>,
    pub events: Vec<TaskEventInput>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetailResponse {
    pub task: TaskRecord,
    pub events: Vec<TaskEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskBridgeRequest {
    Start {
        token: Option<String>,
        task: TaskRecord,
    },
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TaskBridgeEvent {
    Update {
        status: Option<TaskStatus>,
        execution_protocol: Option<ExecutionProtocol>,
        events: Vec<TaskEventInput>,
        exit_code: Option<i32>,
        error: Option<String>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShellSessionStatus {
    Pending,
    Active,
    CloseRequested,
    Succeeded,
    Failed,
    Closed,
}

impl ShellSessionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Closed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShellTransportKind {
    RelayPolling,
    OverlayProxy,
}

impl Default for ShellTransportKind {
    fn default() -> Self {
        Self::RelayPolling
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShellStreamKind {
    Stdout,
    Stderr,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellSessionRecord {
    pub tenant_id: String,
    pub user_id: String,
    pub id: String,
    pub device_id: String,
    pub cwd: Option<String>,
    #[serde(default)]
    pub transport: ShellTransportKind,
    pub status: ShellSessionStatus,
    pub close_requested: bool,
    pub created_at_epoch_ms: u64,
    pub started_at_epoch_ms: Option<u64>,
    pub finished_at_epoch_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub last_input_seq: u64,
    pub last_output_seq: u64,
}

impl ShellSessionRecord {
    pub fn new(request: CreateShellSessionRequest, transport: ShellTransportKind) -> Self {
        Self {
            tenant_id: DEFAULT_TENANT_ID.to_string(),
            user_id: DEFAULT_USER_ID.to_string(),
            id: Uuid::new_v4().to_string(),
            device_id: request.device_id,
            cwd: request.cwd,
            transport,
            status: ShellSessionStatus::Pending,
            close_requested: false,
            created_at_epoch_ms: now_epoch_millis(),
            started_at_epoch_ms: None,
            finished_at_epoch_ms: None,
            exit_code: None,
            error: None,
            last_input_seq: 0,
            last_output_seq: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateShellSessionRequest {
    pub device_id: String,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateShellSessionResponse {
    pub session: ShellSessionRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaimShellSessionResponse {
    pub session: Option<ShellSessionRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateShellInputRequest {
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellInputRecord {
    pub seq: u64,
    pub session_id: String,
    pub data: String,
    pub timestamp_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellOutputChunk {
    pub seq: u64,
    pub session_id: String,
    pub stream: ShellStreamKind,
    pub data: String,
    pub timestamp_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellOutputChunkInput {
    pub stream: ShellStreamKind,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppendShellOutputRequest {
    pub device_id: String,
    pub status: Option<ShellSessionStatus>,
    pub outputs: Vec<ShellOutputChunkInput>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellPendingInputResponse {
    pub session: ShellSessionRecord,
    pub inputs: Vec<ShellInputRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellSessionDetailResponse {
    pub session: ShellSessionRecord,
    pub inputs: Vec<ShellInputRecord>,
    pub outputs: Vec<ShellOutputChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShellBridgeRequest {
    Start {
        token: Option<String>,
        session_id: String,
        cwd: Option<String>,
    },
    Input {
        data: String,
    },
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ShellBridgeEvent {
    Started {
        shell: String,
        cwd: String,
    },
    Output {
        stream: ShellStreamKind,
        data: String,
    },
    Exited {
        exit_code: Option<i32>,
        close_requested: bool,
        error: Option<String>,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortForwardProtocol {
    Tcp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortForwardStatus {
    Pending,
    Active,
    CloseRequested,
    Closed,
    Failed,
}

impl PortForwardStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Closed | Self::Failed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PortForwardTransportKind {
    RelayTunnel,
    OverlayProxy,
}

impl Default for PortForwardTransportKind {
    fn default() -> Self {
        Self::RelayTunnel
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardRecord {
    pub tenant_id: String,
    pub user_id: String,
    pub id: String,
    pub device_id: String,
    pub protocol: PortForwardProtocol,
    pub relay_host: String,
    pub relay_port: u16,
    pub target_host: String,
    pub target_port: u16,
    #[serde(default)]
    pub transport: PortForwardTransportKind,
    pub status: PortForwardStatus,
    pub created_at_epoch_ms: u64,
    pub started_at_epoch_ms: Option<u64>,
    pub finished_at_epoch_ms: Option<u64>,
    pub error: Option<String>,
}

impl PortForwardRecord {
    pub fn new(
        request: CreatePortForwardRequest,
        relay_host: String,
        relay_port: u16,
        transport: PortForwardTransportKind,
    ) -> Self {
        Self {
            tenant_id: DEFAULT_TENANT_ID.to_string(),
            user_id: DEFAULT_USER_ID.to_string(),
            id: Uuid::new_v4().to_string(),
            device_id: request.device_id,
            protocol: request.protocol,
            relay_host,
            relay_port,
            target_host: request.target_host,
            target_port: request.target_port,
            transport,
            status: PortForwardStatus::Pending,
            created_at_epoch_ms: now_epoch_millis(),
            started_at_epoch_ms: None,
            finished_at_epoch_ms: None,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePortForwardRequest {
    pub device_id: String,
    pub protocol: PortForwardProtocol,
    pub target_host: String,
    pub target_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreatePortForwardResponse {
    pub forward: PortForwardRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaimPortForwardResponse {
    pub forward: Option<PortForwardRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PortForwardBridgeRequest {
    Start {
        token: Option<String>,
        forward_id: String,
        target_host: String,
        target_port: u16,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PortForwardBridgeEvent {
    Ready,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PortForwardTunnelControl {
    ClientConnected,
    ClientClosed,
    TargetConnected,
    TargetConnectFailed { message: String },
    TargetClosed { message: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReportPortForwardStateRequest {
    pub device_id: String,
    pub status: Option<PortForwardStatus>,
    pub error: Option<String>,
    pub clear_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardDetailResponse {
    pub forward: PortForwardRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServiceHealth {
    pub service: String,
    pub status: String,
    pub device_count: usize,
    pub online_device_count: usize,
    pub task_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub app_name: String,
    pub default_relay_base_url: String,
    pub requires_auth: bool,
    pub supported_targets: Vec<String>,
    pub control_clients: Vec<String>,
    pub feature_flags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelayEventType {
    DeviceUpdated,
    TaskUpdated,
    TaskEvent,
}

impl RelayEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeviceUpdated => "device_updated",
            Self::TaskUpdated => "task_updated",
            Self::TaskEvent => "task_event",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RelayEventEnvelope {
    pub event_type: RelayEventType,
    pub device: Option<DeviceRecord>,
    pub task: Option<TaskRecord>,
    pub task_event: Option<TaskEvent>,
}

pub fn default_app_config(
    default_relay_base_url: impl Into<String>,
    requires_auth: bool,
) -> AppConfig {
    AppConfig {
        app_name: "Vibe Everywhere".to_string(),
        default_relay_base_url: default_relay_base_url.into(),
        requires_auth,
        supported_targets: vec![
            "Windows".to_string(),
            "macOS".to_string(),
            "Linux".to_string(),
        ],
        control_clients: vec![
            "Desktop".to_string(),
            "iOS".to_string(),
            "Android".to_string(),
        ],
        feature_flags: vec![
            "device_registry".to_string(),
            "task_streaming".to_string(),
            "provider_adapters".to_string(),
            "provider_protocol_reporting".to_string(),
            "opencode_acp_mvp".to_string(),
            "easytier_embedded_ready".to_string(),
            "easytier_embedded_agent".to_string(),
            "easytier_embedded_relay".to_string(),
            "relay_shell_sessions".to_string(),
            "relay_tcp_forwarding_control_plane".to_string(),
        ],
    }
}

pub fn now_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
