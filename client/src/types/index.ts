// Re-export all generated types
export type * from './generated/ArchiveStatistics'
export type * from './generated/ClientDevice'
export type * from './generated/CloseReason'
export type * from './generated/ConnectionStatus'
export type * from './generated/CreateSessionRequest'
export type * from './generated/CreateSessionResponse'
export type * from './generated/CreateWorkspaceRequest'
export type * from './generated/DaemonStatus'
export type * from './generated/DeviceType'
export type * from './generated/FileTreeNode'
export type * from './generated/FileType'
export type * from './generated/Host'
export type * from './generated/HostListResponse'
export type * from './generated/NotificationPreference'
export type * from './generated/OnlineStatus'
export type * from './generated/Paginated'
export type * from './generated/Pagination'
export type * from './generated/PairResponse'
export type * from './generated/PairStatus'
export type * from './generated/PermissionRequest'
export type * from './generated/PermissionResponseRequest'
export type * from './generated/PermissionStatus'
export type * from './generated/Platform'
export type * from './generated/RegisterDeviceRequest'
export type * from './generated/RegisterDeviceResponse'
export type * from './generated/RiskType'
export type * from './generated/Session'
export type * from './generated/SessionArchive'
export type * from './generated/SessionControlAction'
export type * from './generated/SessionMessage'
export type * from './generated/SessionMessageType'
export type * from './generated/SessionStatus'
export type * from './generated/Workspace'
export type * from './generated/BatchDeleteResponse'
export type * from './generated/CloseSessionResponse'
export type * from './generated/ControlSessionResponse'
export type * from './generated/ErrorPayload'
export type * from './generated/FileContent'
export type * from './generated/SendMessageResponse'
export type * from './generated/SuccessResponse'

// Export enums as values - need export type for verbatimModuleSyntax
export type { SessionStatus } from './generated/SessionStatus'
export type { PermissionStatus } from './generated/PermissionStatus'
export type { RiskType } from './generated/RiskType'
export type { DaemonStatus } from './generated/DaemonStatus'
export type { OnlineStatus } from './generated/OnlineStatus'
export type { CloseReason } from './generated/CloseReason'
export type { ConnectionStatus } from './generated/ConnectionStatus'
export type { DeviceType } from './generated/DeviceType'
export type { FileType } from './generated/FileType'
export type { PairStatus } from './generated/PairStatus'
export type { Platform } from './generated/Platform'
export type { SessionControlAction } from './generated/SessionControlAction'
export type { SessionMessageType } from './generated/SessionMessageType'
export type { PermissionDecision } from './generated/PermissionDecision'

// Use the generated PermissionDecision type
export type ClientPermissionDecision = 'approve_once' | 'deny_once' | 'approve_session'

// Extended types for API usage

// Extended Pagination with cursor support
export interface Pagination {
  page?: number
  limit?: number
  cursor?: string
}

// Extended SessionArchive with agent_type
export interface SessionArchive {
  archive_id: string
  session_id: string
  title: string
  closed_at: string
  close_reason: import('./generated/CloseReason').CloseReason
  host_id: string
  workspace_id: string
  agent_type: string
  created_at: string
}

// Extended Host with workspace_count
export interface Host {
  host_id: string
  host_name: string
  platform: import('./generated/Platform').Platform
  online_status: import('./generated/OnlineStatus').OnlineStatus
  daemon_status: import('./generated/DaemonStatus').DaemonStatus
  last_active_at: string | null
  pair_status: import('./generated/PairStatus').PairStatus
  pair_code: string | null
  qr_payload: string | null
  created_at: string
  updated_at: string
  workspace_count?: number
}

// Extended NotificationPreference with optional device_id
export interface NotificationPreference {
  device_id?: string
  enabled: boolean
  permission_request_enabled: boolean
  task_completed_enabled: boolean
  task_failed_enabled: boolean
  session_error_enabled: boolean
  [key: string]: boolean | string | undefined
}

// CreateSessionRequest for client usage
export interface CreateSessionRequest {
  host_id: string
  workspace_id: string
  title?: string
  initial_message?: string
}

// CreateWorkspaceRequest for client usage
export interface CreateWorkspaceRequest {
  host_id: string
  path: string
  display_name?: string | null
}

// Extended PermissionResponseRequest
export interface PermissionResponseRequest {
  decision: 'approve_once' | 'deny_once' | 'approve_session'
  remember_in_session?: boolean
}
