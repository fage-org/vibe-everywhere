import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { archivesApi } from '@/api'
import type { SessionArchive } from '@/types'
import type { ArchiveFilters } from '@/api/types'

export const useArchiveStore = defineStore('archives', () => {
  // State
  const archives = ref<SessionArchive[]>([])
  const selectedArchiveId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const filters = ref<ArchiveFilters>({})
  const totalCount = ref(0)

  // Getters
  const selectedArchive = computed(() =>
    archives.value.find(a => a.archive_id === selectedArchiveId.value || a.session_id === selectedArchiveId.value) || null
  )

  const filteredArchives = computed(() => {
    let result = [...archives.value]

    // Apply filters
    if (filters.value.host_id) {
      result = result.filter(a => a.host_id === filters.value.host_id)
    }
    if (filters.value.workspace_id) {
      result = result.filter(a => a.workspace_id === filters.value.workspace_id)
    }
    if (filters.value.agent_type) {
      result = result.filter(a => a.agent_type === filters.value.agent_type)
    }

    // Sort by closed_at (most recent first)
    result.sort((a, b) => {
      const aTime = a.closed_at ? new Date(a.closed_at).getTime() : 0
      const bTime = b.closed_at ? new Date(b.closed_at).getTime() : 0
      return bTime - aTime
    })

    return result
  })

  const archivesByHost = computed(() => {
    const grouped = new Map<string, SessionArchive[]>()
    archives.value.forEach(archive => {
      const list = grouped.get(archive.host_id) || []
      list.push(archive)
      grouped.set(archive.host_id, list)
    })
    return grouped
  })

  // Actions
  async function fetchArchives(newFilters?: ArchiveFilters) {
    isLoading.value = true
    error.value = null

    try {
      if (newFilters) {
        filters.value = newFilters
      }

      const response = await archivesApi.listArchives(filters.value)
      archives.value = response.items
      totalCount.value = Number(response.total)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch archives'
    } finally {
      isLoading.value = false
    }
  }

  function selectArchive(archiveId: string | null) {
    selectedArchiveId.value = archiveId
  }

  async function deleteArchives(archiveIds: string[]) {
    isLoading.value = true
    error.value = null

    try {
      const response = await archivesApi.deleteArchives(archiveIds)
      // Remove deleted archives from list
      archives.value = archives.value.filter(
        a => !archiveIds.includes(a.archive_id) && !archiveIds.includes(a.session_id)
      )
      totalCount.value -= response.deleted_count

      if (selectedArchiveId.value && archiveIds.includes(selectedArchiveId.value)) {
        selectedArchiveId.value = null
      }

      return response.deleted_count
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete archives'
      return 0
    } finally {
      isLoading.value = false
    }
  }

  function setFilters(newFilters: ArchiveFilters) {
    filters.value = newFilters
  }

  function clearFilters() {
    filters.value = {}
  }

  return {
    // State
    archives,
    selectedArchiveId,
    isLoading,
    error,
    filters,
    totalCount,
    // Getters
    selectedArchive,
    filteredArchives,
    archivesByHost,
    // Actions
    fetchArchives,
    selectArchive,
    deleteArchives,
    setFilters,
    clearFilters,
  }
})
