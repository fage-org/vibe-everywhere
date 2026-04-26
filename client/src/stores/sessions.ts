import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { sessionsApi } from '@/api'
import { useToastStore } from './toast'
import type {
  Session,
  CreateSessionRequest,
  SessionControlAction,
} from '@/types'
import type { SessionFilters } from '@/api/types'

export const useSessionStore = defineStore('sessions', () => {
  // State
  const sessions = ref<Session[]>([])
  const activeSessionId = ref<string | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const filterStatus = ref<string | null>(null)

  // Getters
  const activeSession = computed(() =>
    sessions.value.find(s => s.session_id === activeSessionId.value) || null
  )

  const filteredSessions = computed(() => {
    let result = sessions.value

    if (filterStatus.value) {
      result = result.filter(s => s.status === filterStatus.value)
    }

    // Sort by: pending permission first, then by last activity
    return result.sort((a, b) => {
      // Pending permission sessions first
      if (a.pending_permission_count > 0 && b.pending_permission_count === 0) return -1
      if (b.pending_permission_count > 0 && a.pending_permission_count === 0) return 1

      // Then by last activity (most recent first)
      const aTime = a.last_activity_at ? new Date(a.last_activity_at).getTime() : 0
      const bTime = b.last_activity_at ? new Date(b.last_activity_at).getTime() : 0
      return bTime - aTime
    })
  })

  const attentionSessions = computed(() =>
    sessions.value.filter(s => s.pending_permission_count > 0 || s.status === 'error')
  )

  const runningSessions = computed(() =>
    sessions.value.filter(s => s.status === 'running')
  )

  const pausedSessions = computed(() =>
    sessions.value.filter(s => s.status === 'paused')
  )

  // Actions
  async function fetchSessions(filters?: SessionFilters) {
    isLoading.value = true
    error.value = null

    try {
      const data = await sessionsApi.listSessions(filters)
      sessions.value = data
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch sessions'
      const toast = useToastStore()
      toast.error('Failed to fetch sessions', error.value)
    } finally {
      isLoading.value = false
    }
  }

  async function createSession(data: CreateSessionRequest): Promise<string | null> {
    isLoading.value = true
    error.value = null

    try {
      const response = await sessionsApi.createSession(data)
      // Refresh sessions list
      await fetchSessions()
      const toast = useToastStore()
      toast.success('Session created successfully')
      return response.session_id
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create session'
      const toast = useToastStore()
      toast.error('Failed to create session', error.value)
      return null
    } finally {
      isLoading.value = false
    }
  }

  function selectSession(sessionId: string | null) {
    activeSessionId.value = sessionId
  }

  async function sendMessage(sessionId: string, content: string) {
    try {
      await sessionsApi.sendMessage(sessionId, content)
      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to send message'
      const toast = useToastStore()
      toast.error('Failed to send message', error.value)
      return false
    }
  }

  async function controlSession(sessionId: string, action: SessionControlAction) {
    try {
      await sessionsApi.controlSession(sessionId, action)
      // Refresh session data
      await fetchSessions()
      const toast = useToastStore()
      toast.success(`Session ${action} successfully`)
      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to control session'
      const toast = useToastStore()
      toast.error('Failed to control session', error.value)
      return false
    }
  }

  async function closeSession(sessionId: string) {
    try {
      await sessionsApi.closeSession(sessionId)
      // Remove from list and refresh
      sessions.value = sessions.value.filter(s => s.session_id !== sessionId)
      if (activeSessionId.value === sessionId) {
        activeSessionId.value = null
      }
      const toast = useToastStore()
      toast.success('Session closed and archived')
      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to close session'
      const toast = useToastStore()
      toast.error('Failed to close session', error.value)
      return false
    }
  }

  function updateSessionStatus(sessionId: string, status: string) {
    const session = sessions.value.find(s => s.session_id === sessionId)
    if (session) {
      session.status = status as Session['status']
    }
  }

  function setFilterStatus(status: string | null) {
    filterStatus.value = status
  }

  return {
    // State
    sessions,
    activeSessionId,
    isLoading,
    error,
    filterStatus,
    // Getters
    activeSession,
    filteredSessions,
    attentionSessions,
    runningSessions,
    pausedSessions,
    // Actions
    fetchSessions,
    createSession,
    selectSession,
    sendMessage,
    controlSession,
    closeSession,
    updateSessionStatus,
    setFilterStatus,
  }
})
