export type ExecutionProtocol = "cli" | "acp";
export type TaskExecutionMode =
  | "read_only"
  | "workspace_write"
  | "workspace_write_and_test";
export type OverlayState =
  | "connected"
  | "degraded"
  | "disabled"
  | "unavailable";
export type ProviderKind = "codex" | "claude_code" | "open_code";
export type TaskStatus =
  | "pending"
  | "assigned"
  | "running"
  | "waiting_input"
  | "cancel_requested"
  | "succeeded"
  | "failed"
  | "canceled";
export type ShellSessionStatus =
  | "pending"
  | "active"
  | "close_requested"
  | "succeeded"
  | "failed"
  | "closed";
export type PortForwardStatus =
  | "pending"
  | "active"
  | "close_requested"
  | "closed"
  | "failed";
export type UserRole = "owner" | "admin" | "member" | "viewer" | "agent";
export type AuthMode = "disabled" | "access_token";
export type StorageKind = "file" | "memory" | "external";
export type DeploymentMode = "self_hosted" | "hosted_compatible";
export type NotificationChannel = "in_app" | "system";
export type ControlClientKind = "web" | "tauri_desktop" | "android";
export type ShellStreamKind = "stdout" | "stderr" | "system";
export type PortForwardProtocol = "tcp";
export type PortForwardTransportKind = "relay_tunnel" | "overlay_proxy";

export type TaskEventKind =
  | "system"
  | "status"
  | "provider_stdout"
  | "provider_stderr"
  | "assistant_delta"
  | "tool_call"
  | "tool_output";

export type ProviderStatus = {
  kind: ProviderKind;
  command: string;
  available: boolean;
  version: string | null;
  executionProtocol: ExecutionProtocol;
  supportsAcp: boolean;
  error: string | null;
};

export type OverlayNetworkStatus = {
  mode: "easytier_embedded" | "easytier_sidecar" | "disabled";
  state: OverlayState;
  networkName: string | null;
  nodeIp: string | null;
  relayUrl: string | null;
  binaryPath: string | null;
  lastError: string | null;
};

export type DeviceRecord = {
  tenantId: string;
  userId: string;
  id: string;
  name: string;
  platform: string;
  capabilities: string[];
  metadata: Record<string, string>;
  providers: ProviderStatus[];
  overlay: OverlayNetworkStatus;
  online: boolean;
  lastSeenEpochMs: number;
  currentTaskId: string | null;
};

export type TaskRecord = {
  tenantId: string;
  userId: string;
  id: string;
  deviceId: string;
  conversationId: string | null;
  title: string;
  provider: ProviderKind;
  executionProtocol: ExecutionProtocol;
  executionMode: TaskExecutionMode;
  prompt: string;
  cwd: string | null;
  model: string | null;
  providerSessionId: string | null;
  pendingInputRequestId: string | null;
  status: TaskStatus;
  cancelRequested: boolean;
  createdAtEpochMs: number;
  startedAtEpochMs: number | null;
  finishedAtEpochMs: number | null;
  exitCode: number | null;
  error: string | null;
  lastEventSeq: number;
};

export type TaskEvent = {
  seq: number;
  taskId: string;
  deviceId: string;
  kind: TaskEventKind;
  message: string;
  timestampEpochMs: number;
};

export type TaskDetailResponse = {
  task: TaskRecord;
  events: TaskEvent[];
  pendingInputRequest: ConversationInputRequest | null;
};

export type ConversationRecord = {
  tenantId: string;
  userId: string;
  id: string;
  deviceId: string;
  title: string;
  provider: ProviderKind;
  executionProtocol: ExecutionProtocol;
  executionMode: TaskExecutionMode;
  cwd: string | null;
  model: string | null;
  providerSessionId: string | null;
  latestTaskId: string | null;
  pendingInputRequestId: string | null;
  archived: boolean;
  createdAtEpochMs: number;
  updatedAtEpochMs: number;
};

export type ConversationInputOption = {
  id: string;
  label: string;
  description: string | null;
  requiresTextInput: boolean;
};

export type ConversationInputRequestStatus = "pending" | "answered" | "canceled";

export type ConversationInputRequest = {
  id: string;
  conversationId: string;
  taskId: string;
  prompt: string;
  options: ConversationInputOption[];
  allowCustomInput: boolean;
  customInputPlaceholder: string | null;
  status: ConversationInputRequestStatus;
  selectedOptionId: string | null;
  responseText: string | null;
  createdAtEpochMs: number;
  answeredAtEpochMs: number | null;
};

export type ConversationDetailResponse = {
  conversation: ConversationRecord;
  tasks: TaskDetailResponse[];
  pendingInputRequest: ConversationInputRequest | null;
};

export type ShellTransportKind = "relay_polling" | "overlay_proxy";

export type ShellSessionRecord = {
  tenantId: string;
  userId: string;
  id: string;
  deviceId: string;
  cwd: string | null;
  transport: ShellTransportKind;
  status: ShellSessionStatus;
  closeRequested: boolean;
  createdAtEpochMs: number;
  startedAtEpochMs: number | null;
  finishedAtEpochMs: number | null;
  exitCode: number | null;
  error: string | null;
  lastInputSeq: number;
  lastOutputSeq: number;
};

export type ShellInputRecord = {
  seq: number;
  sessionId: string;
  data: string;
  timestampEpochMs: number;
};

export type ShellOutputChunk = {
  seq: number;
  sessionId: string;
  stream: ShellStreamKind;
  data: string;
  timestampEpochMs: number;
};

export type ShellSessionDetailResponse = {
  session: ShellSessionRecord;
  inputs: ShellInputRecord[];
  outputs: ShellOutputChunk[];
};

export type PortForwardRecord = {
  tenantId: string;
  userId: string;
  id: string;
  deviceId: string;
  protocol: PortForwardProtocol;
  relayHost: string;
  relayPort: number;
  targetHost: string;
  targetPort: number;
  transport: PortForwardTransportKind;
  status: PortForwardStatus;
  createdAtEpochMs: number;
  startedAtEpochMs: number | null;
  finishedAtEpochMs: number | null;
  error: string | null;
};

export type PortForwardDetailResponse = {
  forward: PortForwardRecord;
};

export type WorkspaceEntryKind = "directory" | "file";

export type WorkspaceEntry = {
  path: string;
  name: string;
  kind: WorkspaceEntryKind;
  sizeBytes: number | null;
  modifiedAtEpochMs: number | null;
};

export type WorkspaceBrowseRequest = {
  deviceId: string;
  sessionCwd?: string;
  path?: string;
};

export type WorkspaceBrowseResponse = {
  deviceId: string;
  rootPath: string;
  path: string;
  parentPath: string | null;
  entries: WorkspaceEntry[];
};

export type WorkspacePreviewKind = "text" | "binary" | "directory";

export type WorkspaceFilePreviewRequest = {
  deviceId: string;
  sessionCwd?: string;
  path: string;
  line?: number;
  limit?: number;
};

export type WorkspaceFilePreviewResponse = {
  deviceId: string;
  rootPath: string;
  path: string;
  kind: WorkspacePreviewKind;
  content: string | null;
  truncated: boolean;
  line: number | null;
  totalLines: number | null;
  sizeBytes: number;
};

export type GitInspectState = "ready" | "not_repository" | "git_unavailable";

export type GitFileStatus =
  | "modified"
  | "added"
  | "deleted"
  | "renamed"
  | "copied"
  | "updated_but_unmerged"
  | "untracked"
  | "type_changed"
  | "unknown";

export type GitInspectRequest = {
  deviceId: string;
  sessionCwd?: string;
};

export type GitDiffFileRequest = {
  deviceId: string;
  sessionCwd?: string;
  repoPath: string;
};

export type GitCreateWorktreeRequest = {
  deviceId: string;
  sessionCwd?: string;
  branchName: string;
  destinationPath: string;
};

export type GitRemoveWorktreeRequest = {
  deviceId: string;
  sessionCwd?: string;
  worktreePath: string;
};

export type GitChangedFile = {
  path: string;
  repoPath: string;
  status: GitFileStatus;
  staged: boolean;
  unstaged: boolean;
};

export type GitCommitSummary = {
  id: string;
  shortId: string;
  summary: string;
  authorName: string;
  committedAtEpochMs: number;
};

export type GitDiffStats = {
  changedFiles: number;
  stagedFiles: number;
  unstagedFiles: number;
  untrackedFiles: number;
  conflictedFiles: number;
  stagedAdditions: number;
  stagedDeletions: number;
  unstagedAdditions: number;
  unstagedDeletions: number;
};

export type GitWorktreeSummary = {
  path: string;
  branchName: string | null;
  headId: string | null;
  isCurrent: boolean;
  isDetached: boolean;
};

export type GitInspectResponse = {
  deviceId: string;
  workspaceRoot: string;
  repoRoot: string | null;
  repoCommonDir: string | null;
  scopePath: string | null;
  state: GitInspectState;
  branchName: string | null;
  upstreamBranch: string | null;
  aheadCount: number;
  behindCount: number;
  hasCommits: boolean;
  changedFiles: GitChangedFile[];
  recentCommits: GitCommitSummary[];
  worktrees: GitWorktreeSummary[];
  diffStats: GitDiffStats;
};

export type GitDiffFileResponse = {
  deviceId: string;
  workspaceRoot: string;
  repoRoot: string | null;
  repoCommonDir: string | null;
  repoPath: string;
  path: string;
  state: GitInspectState;
  status: GitFileStatus | null;
  staged: boolean;
  unstaged: boolean;
  isBinary: boolean;
  truncated: boolean;
  stagedPatch: string | null;
  unstagedPatch: string | null;
};

export type GitCreateWorktreeResponse = {
  deviceId: string;
  workspaceRoot: string;
  repoRoot: string | null;
  repoCommonDir: string | null;
  branchName: string;
  destinationPath: string;
};

export type GitRemoveWorktreeResponse = {
  deviceId: string;
  workspaceRoot: string;
  repoRoot: string | null;
  repoCommonDir: string | null;
  removedPath: string;
};

export type ServiceHealth = {
  service: string;
  status: string;
  deviceCount: number;
  onlineDeviceCount: number;
  taskCount: number;
};

export type ActorIdentity = {
  tenantId: string;
  userId: string;
  role: UserRole;
};

export type PlatformCapability = {
  client: ControlClientKind;
  mobileOptimized: boolean;
  supportsSystemNotifications: boolean;
  supportsPersistedRuntimeConfig: boolean;
  prefersExplicitRemoteRelayUrl: boolean;
};

export type DeploymentMetadata = {
  mode: DeploymentMode;
  displayName: string;
  relayPublicOrigin: string;
  documentationUrl: string | null;
};

export type AuditAction =
  | "device_registered"
  | "task_created"
  | "task_canceled"
  | "shell_session_created"
  | "shell_session_closed"
  | "preview_created"
  | "preview_closed";

export type AuditOutcome = "succeeded" | "rejected" | "failed";

export type AuditRecord = {
  id: string;
  tenantId: string;
  userId: string;
  actorRole: UserRole;
  action: AuditAction;
  resourceKind: string;
  resourceId: string;
  outcome: AuditOutcome;
  message: string | null;
  timestampEpochMs: number;
};

export type AppConfig = {
  appName: string;
  defaultRelayBaseUrl: string;
  requiresAuth: boolean;
  authMode: AuthMode;
  storageKind: StorageKind;
  deployment: DeploymentMetadata;
  currentActor: ActorIdentity;
  notificationChannels: NotificationChannel[];
  platformMatrix: PlatformCapability[];
  supportedTargets: string[];
  controlClients: string[];
  featureFlags: string[];
};

export type RelayEventEnvelope = {
  eventType: "device_updated" | "task_updated" | "task_event";
  device: DeviceRecord | null;
  task: TaskRecord | null;
  taskEvent: TaskEvent | null;
};

export type CreateTaskPayload = {
  deviceId: string;
  provider: ProviderKind;
  executionMode?: TaskExecutionMode;
  prompt: string;
  cwd?: string;
  model?: string;
  title?: string;
};

export type CreateConversationPayload = {
  deviceId: string;
  provider: ProviderKind;
  executionMode?: TaskExecutionMode;
  prompt: string;
  cwd?: string;
  model?: string;
  title?: string;
};

export type CreateConversationResponse = {
  conversation: ConversationRecord;
  task: TaskRecord;
};

export type SendConversationMessagePayload = {
  prompt: string;
  executionMode?: TaskExecutionMode;
  model?: string;
  title?: string;
};

export type SendConversationMessageResponse = {
  conversation: ConversationRecord;
  task: TaskRecord;
};

export type RespondConversationInputPayload = {
  optionId?: string;
  text?: string;
};

export type CreateShellSessionPayload = {
  deviceId: string;
  cwd?: string;
};

export type CreatePortForwardPayload = {
  deviceId: string;
  protocol?: PortForwardProtocol;
  targetHost: string;
  targetPort: number;
};
