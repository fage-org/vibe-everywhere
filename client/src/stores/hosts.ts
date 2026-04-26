import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { hostsApi } from '@/api'
import type { Host, Workspace } from '@/types'

export const useHostStore = defineStore('hosts', () => {
  // State
  const hosts = ref<Host[]>([])
  const selectedHostId = ref<string | null>(null)
  const workspaces = ref<Workspace[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const selectedHost = computed(() =>
    hosts.value.find(h => h.host_id === selectedHostId.value) || null
  )

  const favoriteWorkspaces = computed(() =>
    workspaces.value.filter(w => w.is_favorited)
  )

  const normalWorkspaces = computed(() =>
    workspaces.value.filter(w => !w.is_favorited)
      .sort((a, b) => {
        // Sort by last used time, most recent first
        const aTime = a.last_used_at ? new Date(a.last_used_at).getTime() : 0
        const bTime = b.last_used_at ? new Date(b.last_used_at).getTime() : 0
        return bTime - aTime
      })
  )

  const onlineHosts = computed(() =>
    hosts.value.filter(h => h.online_status === 'online')
  )

  // Actions
  async function fetchHosts() {
    isLoading.value = true
    error.value = null

    try {
      const data = await hostsApi.listHosts()
      hosts.value = data
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch hosts'
    } finally {
      isLoading.value = false
    }
  }

  function selectHost(hostId: string | null) {
    selectedHostId.value = hostId
  }

  async function unpairHost(hostId: string) {
    try {
      await hostsApi.deleteHost(hostId)
      // Remove from list
      hosts.value = hosts.value.filter(h => h.host_id !== hostId)
      if (selectedHostId.value === hostId) {
        selectedHostId.value = null
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to unpair host'
    }
  }

  function setWorkspaces(data: Workspace[]) {
    workspaces.value = data
  }

  function updateWorkspaceFavorite(workspaceId: string, isFavorited: boolean) {
    const workspace = workspaces.value.find(w => w.workspace_id === workspaceId)
    if (workspace) {
      workspace.is_favorited = isFavorited
    }
  }

  return {
    // State
    hosts,
    selectedHostId,
    workspaces,
    isLoading,
    error,
    // Getters
    selectedHost,
    favoriteWorkspaces,
    normalWorkspaces,
    onlineHosts,
    // Actions
    fetchHosts,
    selectHost,
    unpairHost,
    setWorkspaces,
    updateWorkspaceFavorite,
  }
})
