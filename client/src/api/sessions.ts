import { apiClient } from './client'
import type {
  Session,
  CreateSessionRequest,
  CreateSessionResponse,
  SessionMessage,
  SessionControlAction,
  ControlSessionResponse,
  CloseSessionResponse,
  SendMessageResponse,
  Paginated,
  Pagination,
} from './types'
import type { FileTreeNode, FileContent } from '@/types'
import type { SessionFilters } from './types'

export interface CreateMessageRequest {
  content: string
}

export interface DiffFile {
  path: string
  name: string
  additions: number
  deletions: number
}

export interface DiffLine {
  type: 'context' | 'add' | 'remove' | 'hunk'
  content: string
  oldLine?: number
  newLine?: number
}

export interface DiffData {
  stats: { additions: number; deletions: number }
  lines: DiffLine[]
}

export const sessionsApi = {
  // List active sessions
  async listSessions(filters?: SessionFilters): Promise<Session[]> {
    const queryParams = new URLSearchParams()
    if (filters?.status) queryParams.append('status', filters.status)
    if (filters?.host_id) queryParams.append('host_id', filters.host_id)
    if (filters?.workspace_id) queryParams.append('workspace_id', filters.workspace_id)

    const query = queryParams.toString()
    const path = query ? `/api/sessions?${query}` : '/api/sessions'

    return apiClient.get<Session[]>(path)
  },

  // Get session details
  async getSession(id: string): Promise<Session> {
    return apiClient.get<Session>(`/api/sessions/${id}`)
  },

  // Create a new session
  async createSession(data: CreateSessionRequest): Promise<CreateSessionResponse> {
    return apiClient.post<CreateSessionResponse>('/api/sessions', data)
  },

  // Send a message to session
  async sendMessage(sessionId: string, content: string): Promise<SendMessageResponse> {
    return apiClient.post<SendMessageResponse>(
      `/api/sessions/${sessionId}/messages`,
      { content }
    )
  },

  // Get session messages
  async getMessages(sessionId: string, pagination?: Pagination): Promise<Paginated<SessionMessage>> {
    const queryParams = new URLSearchParams()
    if (pagination?.page) queryParams.append('page', pagination.page.toString())
    if (pagination?.limit) queryParams.append('limit', pagination.limit.toString())

    const query = queryParams.toString()
    const path = query
      ? `/api/sessions/${sessionId}/messages?${query}`
      : `/api/sessions/${sessionId}/messages`

    return apiClient.get<Paginated<SessionMessage>>(path)
  },

  // Get session events
  async getEvents(sessionId: string, pagination?: Pagination): Promise<Paginated<unknown>> {
    const queryParams = new URLSearchParams()
    if (pagination?.page) queryParams.append('page', pagination.page.toString())
    if (pagination?.limit) queryParams.append('limit', pagination.limit.toString())

    const query = queryParams.toString()
    const path = query
      ? `/api/sessions/${sessionId}/events?${query}`
      : `/api/sessions/${sessionId}/events`

    return apiClient.get<Paginated<unknown>>(path)
  },

  // Control session (pause/resume/terminate/interrupt/rerun)
  async controlSession(sessionId: string, action: SessionControlAction): Promise<ControlSessionResponse> {
    return apiClient.post<ControlSessionResponse>(
      `/api/sessions/${sessionId}/control`,
      { action }
    )
  },

  // Close and archive session
  async closeSession(sessionId: string): Promise<CloseSessionResponse> {
    return apiClient.post<CloseSessionResponse>(`/api/sessions/${sessionId}/close`)
  },

  // Get file tree for session workspace
  async getFileTree(sessionId: string): Promise<FileTreeNode> {
    return apiClient.get<FileTreeNode>(`/api/sessions/${sessionId}/files`)
  },

  // Get file content
  async getFileContent(sessionId: string, filePath: string): Promise<FileContent> {
    return apiClient.get<FileContent>(`/api/sessions/${sessionId}/files/content?path=${encodeURIComponent(filePath)}`)
  },

  // Get session diff
  async getDiff(sessionId: string): Promise<DiffFile[]> {
    return apiClient.get<DiffFile[]>(`/api/sessions/${sessionId}/diff`)
  },

  // Get diff for specific file
  async getFileDiff(sessionId: string, filePath: string): Promise<DiffData> {
    return apiClient.get<DiffData>(`/api/sessions/${sessionId}/diff?path=${encodeURIComponent(filePath)}`)
  },
}
