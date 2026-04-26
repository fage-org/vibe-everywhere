import { apiClient } from './client'
import type { Host, HostListResponse } from './types'

export const hostsApi = {
  // List all paired hosts
  async listHosts(): Promise<Host[]> {
    const response = await apiClient.get<HostListResponse>('/api/hosts')
    return response.hosts
  },

  // Get host details
  async getHost(id: string): Promise<Host> {
    return apiClient.get<Host>(`/api/hosts/${id}`)
  },

  // Delete/unpair a host
  async deleteHost(id: string): Promise<void> {
    return apiClient.delete(`/api/hosts/${id}`)
  },
}
