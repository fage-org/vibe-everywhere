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
pub enum UserRole {
    Owner,
    Admin,
    Member,
    Viewer,
    Agent,
}

impl UserRole {
    pub fn can_read_control_plane(&self) -> bool {
        matches!(
            self,
            Self::Owner | Self::Admin | Self::Member | Self::Viewer
        )
    }

    pub fn can_write_control_plane(&self) -> bool {
        matches!(self, Self::Owner | Self::Admin | Self::Member)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActorIdentity {
    pub tenant_id: String,
    pub user_id: String,
    pub role: UserRole,
}

impl ActorIdentity {
    pub fn personal_owner() -> Self {
        Self {
            tenant_id: DEFAULT_TENANT_ID.to_string(),
            user_id: DEFAULT_USER_ID.to_string(),
            role: UserRole::Owner,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TenantRecord {
    pub id: String,
    pub name: String,
    pub created_at_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    pub id: String,
    pub display_name: String,
    pub created_at_epoch_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MembershipRecord {
    pub tenant_id: String,
    pub user_id: String,
    pub role: UserRole,
    pub created_at_epoch_ms: u64,
}

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
    GitInspect,
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
pub enum TaskExecutionMode {
    ReadOnly,
    WorkspaceWrite,
    WorkspaceWriteAndTest,
}

impl TaskExecutionMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ReadOnly => "Read only",
            Self::WorkspaceWrite => "Workspace write",
            Self::WorkspaceWriteAndTest => "Workspace write + test",
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
    pub device_credential: String,
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
    WaitingInput,
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
    pub conversation_id: Option<String>,
    pub title: String,
    pub provider: ProviderKind,
    pub execution_protocol: ExecutionProtocol,
    pub execution_mode: TaskExecutionMode,
    pub prompt: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub provider_session_id: Option<String>,
    pub pending_input_request_id: Option<String>,
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
        actor: &ActorIdentity,
    ) -> Self {
        let execution_protocol = match execution_protocol {
            ExecutionProtocol::Acp => ExecutionProtocol::Acp,
            ExecutionProtocol::Cli => ExecutionProtocol::Acp,
        };
        Self {
            tenant_id: actor.tenant_id.clone(),
            user_id: actor.user_id.clone(),
            id: Uuid::new_v4().to_string(),
            device_id: request.device_id,
            conversation_id: request.conversation_id,
            title: request
                .title
                .filter(|title| !title.trim().is_empty())
                .unwrap_or_else(|| "Ad hoc AI task".to_string()),
            provider: request.provider,
            execution_protocol,
            execution_mode: request
                .execution_mode
                .unwrap_or(TaskExecutionMode::WorkspaceWrite),
            prompt: request.prompt,
            cwd: request.cwd,
            model: request.model,
            provider_session_id: request.provider_session_id,
            pending_input_request_id: None,
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
    pub conversation_id: Option<String>,
    pub provider: ProviderKind,
    pub execution_mode: Option<TaskExecutionMode>,
    pub prompt: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub title: Option<String>,
    pub provider_session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskResponse {
    pub task: TaskRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationRequest {
    pub device_id: String,
    pub provider: ProviderKind,
    pub execution_mode: Option<TaskExecutionMode>,
    pub prompt: String,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRecord {
    pub tenant_id: String,
    pub user_id: String,
    pub id: String,
    pub device_id: String,
    pub title: String,
    pub provider: ProviderKind,
    pub execution_protocol: ExecutionProtocol,
    pub execution_mode: TaskExecutionMode,
    pub cwd: Option<String>,
    pub model: Option<String>,
    pub provider_session_id: Option<String>,
    pub latest_task_id: Option<String>,
    pub pending_input_request_id: Option<String>,
    pub archived: bool,
    pub created_at_epoch_ms: u64,
    pub updated_at_epoch_ms: u64,
}

impl ConversationRecord {
    pub fn new(
        request: &CreateConversationRequest,
        execution_protocol: ExecutionProtocol,
        actor: &ActorIdentity,
    ) -> Self {
        let now = now_epoch_millis();
        let execution_protocol = match execution_protocol {
            ExecutionProtocol::Acp => ExecutionProtocol::Acp,
            ExecutionProtocol::Cli => ExecutionProtocol::Acp,
        };
        Self {
            tenant_id: actor.tenant_id.clone(),
            user_id: actor.user_id.clone(),
            id: Uuid::new_v4().to_string(),
            device_id: request.device_id.clone(),
            title: request
                .title
                .as_deref()
                .map(str::trim)
                .filter(|title| !title.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| default_conversation_title(&request.prompt)),
            provider: request.provider.clone(),
            execution_protocol,
            execution_mode: request
                .execution_mode
                .clone()
                .unwrap_or(TaskExecutionMode::WorkspaceWrite),
            cwd: request.cwd.clone(),
            model: request.model.clone(),
            provider_session_id: None,
            latest_task_id: None,
            pending_input_request_id: None,
            archived: false,
            created_at_epoch_ms: now,
            updated_at_epoch_ms: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationResponse {
    pub conversation: ConversationRecord,
    pub task: TaskRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SendConversationMessageRequest {
    pub prompt: String,
    pub execution_mode: Option<TaskExecutionMode>,
    pub model: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SendConversationMessageResponse {
    pub conversation: ConversationRecord,
    pub task: TaskRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationInputOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    #[serde(default)]
    pub requires_text_input: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConversationInputRequestStatus {
    Pending,
    Answered,
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationInputRequest {
    pub id: String,
    pub conversation_id: String,
    pub task_id: String,
    pub prompt: String,
    pub options: Vec<ConversationInputOption>,
    pub allow_custom_input: bool,
    pub custom_input_placeholder: Option<String>,
    pub status: ConversationInputRequestStatus,
    pub selected_option_id: Option<String>,
    pub response_text: Option<String>,
    pub created_at_epoch_ms: u64,
    pub answered_at_epoch_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateConversationInputRequest {
    pub prompt: String,
    pub options: Vec<ConversationInputOption>,
    pub allow_custom_input: bool,
    pub custom_input_placeholder: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RespondConversationInputRequest {
    pub option_id: Option<String>,
    pub text: Option<String>,
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
    pub provider_session_id: Option<String>,
    pub events: Vec<TaskEventInput>,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetailResponse {
    pub task: TaskRecord,
    pub events: Vec<TaskEvent>,
    pub pending_input_request: Option<ConversationInputRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConversationDetailResponse {
    pub conversation: ConversationRecord,
    pub tasks: Vec<TaskDetailResponse>,
    pub pending_input_request: Option<ConversationInputRequest>,
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
        provider_session_id: Option<String>,
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
    pub fn new(
        request: CreateShellSessionRequest,
        transport: ShellTransportKind,
        actor: &ActorIdentity,
    ) -> Self {
        Self {
            tenant_id: actor.tenant_id.clone(),
            user_id: actor.user_id.clone(),
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
        actor: &ActorIdentity,
    ) -> Self {
        Self {
            tenant_id: actor.tenant_id.clone(),
            user_id: actor.user_id.clone(),
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
#[serde(rename_all = "snake_case")]
pub enum WorkspaceEntryKind {
    Directory,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEntry {
    pub path: String,
    pub name: String,
    pub kind: WorkspaceEntryKind,
    pub size_bytes: Option<u64>,
    pub modified_at_epoch_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBrowseRequest {
    pub device_id: String,
    pub session_cwd: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceBrowseResponse {
    pub device_id: String,
    pub root_path: String,
    pub path: String,
    pub parent_path: Option<String>,
    pub entries: Vec<WorkspaceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkspacePreviewKind {
    Text,
    Binary,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFilePreviewRequest {
    pub device_id: String,
    pub session_cwd: Option<String>,
    pub path: String,
    pub line: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFilePreviewResponse {
    pub device_id: String,
    pub root_path: String,
    pub path: String,
    pub kind: WorkspacePreviewKind,
    pub content: Option<String>,
    pub truncated: bool,
    pub line: Option<u64>,
    pub total_lines: Option<u64>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitInspectState {
    Ready,
    NotRepository,
    GitUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GitFileStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    UpdatedButUnmerged,
    Untracked,
    TypeChanged,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitInspectRequest {
    pub device_id: String,
    pub session_cwd: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitDiffFileRequest {
    pub device_id: String,
    pub session_cwd: Option<String>,
    pub repo_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitCreateWorktreeRequest {
    pub device_id: String,
    pub session_cwd: Option<String>,
    pub branch_name: String,
    pub destination_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitRemoveWorktreeRequest {
    pub device_id: String,
    pub session_cwd: Option<String>,
    pub worktree_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitChangedFile {
    pub path: String,
    pub repo_path: String,
    pub status: GitFileStatus,
    pub staged: bool,
    pub unstaged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitCommitSummary {
    pub id: String,
    pub short_id: String,
    pub summary: String,
    pub author_name: String,
    pub committed_at_epoch_ms: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitDiffStats {
    pub changed_files: u64,
    pub staged_files: u64,
    pub unstaged_files: u64,
    pub untracked_files: u64,
    pub conflicted_files: u64,
    pub staged_additions: u64,
    pub staged_deletions: u64,
    pub unstaged_additions: u64,
    pub unstaged_deletions: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitWorktreeSummary {
    pub path: String,
    pub branch_name: Option<String>,
    pub head_id: Option<String>,
    pub is_current: bool,
    pub is_detached: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitInspectResponse {
    pub device_id: String,
    pub workspace_root: String,
    pub repo_root: Option<String>,
    pub repo_common_dir: Option<String>,
    pub scope_path: Option<String>,
    pub state: GitInspectState,
    pub branch_name: Option<String>,
    pub upstream_branch: Option<String>,
    pub ahead_count: u64,
    pub behind_count: u64,
    pub has_commits: bool,
    pub changed_files: Vec<GitChangedFile>,
    pub recent_commits: Vec<GitCommitSummary>,
    pub worktrees: Vec<GitWorktreeSummary>,
    pub diff_stats: GitDiffStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitDiffFileResponse {
    pub device_id: String,
    pub workspace_root: String,
    pub repo_root: Option<String>,
    pub repo_common_dir: Option<String>,
    pub repo_path: String,
    pub path: String,
    pub state: GitInspectState,
    pub status: Option<GitFileStatus>,
    pub staged: bool,
    pub unstaged: bool,
    pub is_binary: bool,
    pub truncated: bool,
    pub staged_patch: Option<String>,
    pub unstaged_patch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitCreateWorktreeResponse {
    pub device_id: String,
    pub workspace_root: String,
    pub repo_root: Option<String>,
    pub repo_common_dir: Option<String>,
    pub branch_name: String,
    pub destination_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitRemoveWorktreeResponse {
    pub device_id: String,
    pub workspace_root: String,
    pub repo_root: Option<String>,
    pub repo_common_dir: Option<String>,
    pub removed_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkspaceOperationRequest {
    Browse {
        id: String,
        device_id: String,
        session_cwd: Option<String>,
        path: Option<String>,
    },
    Preview {
        id: String,
        device_id: String,
        session_cwd: Option<String>,
        path: String,
        line: Option<u64>,
        limit: Option<u64>,
    },
}

impl WorkspaceOperationRequest {
    pub fn id(&self) -> &str {
        match self {
            Self::Browse { id, .. } | Self::Preview { id, .. } => id,
        }
    }

    pub fn device_id(&self) -> &str {
        match self {
            Self::Browse { device_id, .. } | Self::Preview { device_id, .. } => device_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaimWorkspaceOperationResponse {
    pub request: Option<WorkspaceOperationRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkspaceOperationResult {
    Browse {
        response: WorkspaceBrowseResponse,
    },
    Preview {
        response: WorkspaceFilePreviewResponse,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompleteWorkspaceOperationRequest {
    pub device_id: String,
    pub result: WorkspaceOperationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GitOperationRequest {
    Inspect {
        id: String,
        device_id: String,
        session_cwd: Option<String>,
    },
    DiffFile {
        id: String,
        device_id: String,
        session_cwd: Option<String>,
        repo_path: String,
    },
    CreateWorktree {
        id: String,
        device_id: String,
        session_cwd: Option<String>,
        branch_name: String,
        destination_path: String,
    },
    RemoveWorktree {
        id: String,
        device_id: String,
        session_cwd: Option<String>,
        worktree_path: String,
    },
}

impl GitOperationRequest {
    pub fn id(&self) -> &str {
        match self {
            Self::Inspect { id, .. }
            | Self::DiffFile { id, .. }
            | Self::CreateWorktree { id, .. }
            | Self::RemoveWorktree { id, .. } => id,
        }
    }

    pub fn device_id(&self) -> &str {
        match self {
            Self::Inspect { device_id, .. }
            | Self::DiffFile { device_id, .. }
            | Self::CreateWorktree { device_id, .. }
            | Self::RemoveWorktree { device_id, .. } => device_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaimGitOperationResponse {
    pub request: Option<GitOperationRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GitOperationResult {
    Inspect { response: GitInspectResponse },
    DiffFile { response: GitDiffFileResponse },
    CreateWorktree { response: GitCreateWorktreeResponse },
    RemoveWorktree { response: GitRemoveWorktreeResponse },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompleteGitOperationRequest {
    pub device_id: String,
    pub result: GitOperationResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMode {
    Disabled,
    AccessToken,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StorageKind {
    File,
    Memory,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    DeviceRegistered,
    TaskCreated,
    TaskCanceled,
    ShellSessionCreated,
    ShellSessionClosed,
    PreviewCreated,
    PreviewClosed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    Succeeded,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub actor_role: UserRole,
    pub action: AuditAction,
    pub resource_kind: String,
    pub resource_id: String,
    pub outcome: AuditOutcome,
    pub message: Option<String>,
    pub timestamp_epoch_ms: u64,
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
    pub auth_mode: AuthMode,
    pub storage_kind: StorageKind,
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
    pub tenant_id: String,
    pub event_type: RelayEventType,
    pub device: Option<DeviceRecord>,
    pub task: Option<TaskRecord>,
    pub task_event: Option<TaskEvent>,
}

pub fn default_app_config(
    default_relay_base_url: impl Into<String>,
    requires_auth: bool,
    storage_kind: StorageKind,
) -> AppConfig {
    AppConfig {
        app_name: "Vibe Everywhere".to_string(),
        default_relay_base_url: default_relay_base_url.into(),
        requires_auth,
        auth_mode: if requires_auth {
            AuthMode::AccessToken
        } else {
            AuthMode::Disabled
        },
        storage_kind,
        supported_targets: vec![
            "Windows".to_string(),
            "macOS".to_string(),
            "Linux".to_string(),
        ],
        control_clients: vec![
            "Web".to_string(),
            "Desktop".to_string(),
            "Android".to_string(),
        ],
        feature_flags: vec![
            "device_registry".to_string(),
            "task_streaming".to_string(),
            "provider_adapters".to_string(),
            "provider_protocol_reporting".to_string(),
            "opencode_acp_mvp".to_string(),
            "session_git_inspect".to_string(),
            "easytier_embedded_ready".to_string(),
            "easytier_embedded_agent".to_string(),
            "easytier_embedded_relay".to_string(),
            "relay_shell_sessions".to_string(),
            "relay_tcp_forwarding_control_plane".to_string(),
        ],
    }
}

fn default_conversation_title(prompt: &str) -> String {
    let trimmed = prompt.trim();
    if trimmed.is_empty() {
        return "Untitled conversation".to_string();
    }

    let mut title = trimmed
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(trimmed)
        .trim()
        .to_string();
    if title.chars().count() > 72 {
        title = title.chars().take(72).collect::<String>();
        title.push_str("...");
    }
    title
}

pub fn now_epoch_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
