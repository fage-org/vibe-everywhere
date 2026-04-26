import { apiClient } from './client'
import type { Workspace, CreateWorkspaceRequest } from './types'

export const workspacesApi = {
  // List workspaces for a host
  async listWorkspaces(hostId: string): Promise<Workspace[]> {
    return apiClient.get<Workspace[]>(`/api/hosts/${hostId}/workspaces`)
  },

  // Create a new workspace
  async createWorkspace(data: CreateWorkspaceRequest): Promise<Workspace> {
    return apiClient.post<Workspace>('/api/workspaces', data)
  },

  // Update workspace (e.g., toggle favorite)
  async updateWorkspace(id: string, data: Partial<Workspace>): Promise<Workspace> {
    return apiClient.put<Workspace>(`/api/workspaces/${id}`, data)
  },

  // Delete a workspace
  async deleteWorkspace(id: string): Promise<void> {
    return apiClient.delete(`/api/workspaces/${id}`)
  },

  // Toggle favorite status
  async toggleFavorite(id: string, isFavorited: boolean): Promise<Workspace> {
    return this.updateWorkspace(id, { is_favorited: isFavorited })
  },
}
