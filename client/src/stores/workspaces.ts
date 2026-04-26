import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { workspacesApi } from '@/api'
import type { Workspace, CreateWorkspaceRequest } from '@/types'

export const useWorkspaceStore = defineStore('workspaces', () => {
  // State
  const workspaces = ref<Workspace[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const favoriteWorkspaces = computed(() =>
    workspaces.value.filter(w => w.is_favorited)
      .sort((a, b) => {
        const aTime = a.last_used_at ? new Date(a.last_used_at).getTime() : 0
        const bTime = b.last_used_at ? new Date(b.last_used_at).getTime() : 0
        return bTime - aTime
      })
  )

  const normalWorkspaces = computed(() =>
    workspaces.value.filter(w => !w.is_favorited)
      .sort((a, b) => {
        const aTime = a.last_used_at ? new Date(a.last_used_at).getTime() : 0
        const bTime = b.last_used_at ? new Date(b.last_used_at).getTime() : 0
        return bTime - aTime
      })
  )

  // Actions
  async function fetchWorkspaces(hostId: string) {
    isLoading.value = true
    error.value = null

    try {
      const data = await workspacesApi.listWorkspaces(hostId)
      workspaces.value = data
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch workspaces'
    } finally {
      isLoading.value = false
    }
  }

  async function createWorkspace(data: CreateWorkspaceRequest): Promise<Workspace | null> {
    isLoading.value = true
    error.value = null

    try {
      const workspace = await workspacesApi.createWorkspace(data)
      workspaces.value.push(workspace)
      return workspace
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create workspace'
      return null
    } finally {
      isLoading.value = false
    }
  }

  async function toggleFavorite(workspaceId: string, isFavorited: boolean) {
    try {
      const updated = await workspacesApi.toggleFavorite(workspaceId, isFavorited)
      const index = workspaces.value.findIndex(w => w.workspace_id === workspaceId)
      if (index !== -1) {
        workspaces.value[index] = updated
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to update workspace'
    }
  }

  return {
    // State
    workspaces,
    isLoading,
    error,
    // Getters
    favoriteWorkspaces,
    normalWorkspaces,
    // Actions
    fetchWorkspaces,
    createWorkspace,
    toggleFavorite,
  }
})
