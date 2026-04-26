import { apiClient } from './client'
import type { SessionArchive, ArchiveStatistics, ArchiveFilters, Paginated, Pagination, SessionMessage } from './types'

export interface BatchDeleteRequest {
  ids: string[]
}

export interface BatchDeleteResponse {
  deleted_count: number
}

export const archivesApi = {
  // List archived sessions with filters
  async listArchives(filters?: ArchiveFilters, pagination?: Pagination): Promise<Paginated<SessionArchive>> {
    const queryParams = new URLSearchParams()

    if (filters?.host_id) queryParams.append('host_id', filters.host_id)
    if (filters?.workspace_id) queryParams.append('workspace_id', filters.workspace_id)
    if (filters?.agent_type) queryParams.append('agent_type', filters.agent_type)
    if (filters?.from_date) queryParams.append('from_date', filters.from_date)
    if (filters?.to_date) queryParams.append('to_date', filters.to_date)

    if (pagination?.page) queryParams.append('page', pagination.page.toString())
    if (pagination?.limit) queryParams.append('limit', pagination.limit.toString())

    const query = queryParams.toString()
    const path = query ? `/api/archives?${query}` : '/api/archives'

    return apiClient.get<Paginated<SessionArchive>>(path)
  },

  // Get archive details
  async getArchive(id: string): Promise<SessionArchive> {
    return apiClient.get<SessionArchive>(`/api/archives/${id}`)
  },

  // Delete archives in batch
  async deleteArchives(ids: string[]): Promise<BatchDeleteResponse> {
    return apiClient.post<BatchDeleteResponse>('/api/archives/delete', { ids })
  },

  // Get archive statistics
  async getArchiveStats(): Promise<ArchiveStatistics> {
    return apiClient.get<ArchiveStatistics>('/api/archives/stats')
  },

  // Get archive messages
  async getArchiveMessages(archiveId: string, pagination?: Pagination): Promise<Paginated<SessionMessage>> {
    const queryParams = new URLSearchParams()
    if (pagination?.page) queryParams.append('page', pagination.page.toString())
    if (pagination?.limit) queryParams.append('limit', pagination.limit.toString())

    const query = queryParams.toString()
    const path = query
      ? `/api/archives/${archiveId}/messages?${query}`
      : `/api/archives/${archiveId}/messages`

    return apiClient.get<Paginated<SessionMessage>>(path)
  },

  // Get archive events/logs
  async getArchiveEvents(archiveId: string, pagination?: Pagination): Promise<Paginated<unknown>> {
    const queryParams = new URLSearchParams()
    if (pagination?.page) queryParams.append('page', pagination.page.toString())
    if (pagination?.limit) queryParams.append('limit', pagination.limit.toString())

    const query = queryParams.toString()
    const path = query
      ? `/api/archives/${archiveId}/events?${query}`
      : `/api/archives/${archiveId}/events`

    return apiClient.get<Paginated<unknown>>(path)
  },

  // Get archive diff
  async getArchiveDiff(archiveId: string): Promise<unknown[]> {
    return apiClient.get<unknown[]>(`/api/archives/${archiveId}/diff`)
  },
}
