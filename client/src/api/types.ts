// API request/response types extending generated types
import type {
  RegisterDeviceRequest,
  RegisterDeviceResponse,
  Host,
  Workspace,
  CreateWorkspaceRequest,
  Session,
  CreateSessionRequest,
  CreateSessionResponse,
  SessionMessage,
  PermissionRequest,
  PermissionResponseRequest,
  SessionArchive,
  SessionControlAction,
  ControlSessionResponse,
  CloseSessionResponse,
  SendMessageResponse,
  ArchiveStatistics,
  NotificationPreference,
  Paginated,
  Pagination,
  PairResponse,
  HostListResponse,
  PermissionDecision,
} from '@/types'

// API Error response
export interface ApiError {
  error: string
  code?: string
}

// Server connection test
export interface ConnectionTestResult {
  success: boolean
  message?: string
}

// Auth API
export interface AuthApi {
  registerDevice: (serverUrl: string, deviceName: string, deviceType: 'mobile' | 'desktop') => Promise<RegisterDeviceResponse>
  testConnection: (serverUrl: string) => Promise<ConnectionTestResult>
}

// Host pairing
export interface PairHostRequest {
  pair_code: string
}

export interface DaemonHelloRequest {
  pair_code: string
  host_info: {
    name: string
    platform: string
    version: string
  }
}

// Archive filters
export interface ArchiveFilters {
  host_id?: string
  workspace_id?: string
  agent_type?: string
  from_date?: string
  to_date?: string
}

// Session filters
export interface SessionFilters {
  status?: string
  host_id?: string
  workspace_id?: string
}

// Re-export generated types for convenience
export type {
  RegisterDeviceRequest,
  RegisterDeviceResponse,
  Host,
  Workspace,
  CreateWorkspaceRequest,
  Session,
  CreateSessionRequest,
  CreateSessionResponse,
  SessionMessage,
  PermissionRequest,
  PermissionResponseRequest,
  SessionArchive,
  SessionControlAction,
  ControlSessionResponse,
  CloseSessionResponse,
  SendMessageResponse,
  ArchiveStatistics,
  NotificationPreference,
  Paginated,
  Pagination,
  PairResponse,
  HostListResponse,
  PermissionDecision,
}
