import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { permissionsApi } from '@/api'
import type { PermissionRequest } from '@/types'
import type { PermissionDecision } from '@/types/generated/PermissionDecision'

interface SessionMemory {
  [rule: string]: PermissionDecision
}

export const usePermissionStore = defineStore('permissions', () => {
  // State
  const pendingRequests = ref<PermissionRequest[]>([])
  const sessionMemory = ref<Map<string, SessionMemory>>(new Map())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const totalPendingCount = computed(() => pendingRequests.value.length)

  const pendingBySession = computed(() => {
    const grouped = new Map<string, PermissionRequest[]>()
    pendingRequests.value.forEach(req => {
      const list = grouped.get(req.session_id) || []
      list.push(req)
      grouped.set(req.session_id, list)
    })
    return grouped
  })

  const getSessionPendingCount = computed(() => (sessionId: string) => {
    return pendingRequests.value.filter(r => r.session_id === sessionId).length
  })

  // Actions
  async function fetchPendingPermissions(sessionId?: string) {
    isLoading.value = true
    error.value = null

    try {
      const data = await permissionsApi.listPermissions(sessionId)
      if (sessionId) {
        // Replace requests for this session
        const otherRequests = pendingRequests.value.filter(r => r.session_id !== sessionId)
        pendingRequests.value = [...otherRequests, ...data]
      } else {
        pendingRequests.value = data
      }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch permissions'
    } finally {
      isLoading.value = false
    }
  }

  async function respondToPermission(
    permissionId: string,
    decision: PermissionDecision,
    rememberInSession: boolean = false
  ) {
    isLoading.value = true
    error.value = null

    try {
      await permissionsApi.respondPermission(permissionId, decision, rememberInSession)

      // Remove from pending list
      pendingRequests.value = pendingRequests.value.filter(r => r.permission_id !== permissionId)

      // If remember in session, add to session memory
      if (rememberInSession && decision === 'approve_once') {
        const request = pendingRequests.value.find(r => r.permission_id === permissionId)
        if (request) {
          const memory = sessionMemory.value.get(request.session_id) || {}
          memory[request.risk_type] = decision
          sessionMemory.value.set(request.session_id, memory)
        }
      }

      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to respond to permission'
      return false
    } finally {
      isLoading.value = false
    }
  }

  function addPendingRequest(request: PermissionRequest) {
    // Check if we have a remembered decision for this session and risk type
    const memory = sessionMemory.value.get(request.session_id)
    if (memory?.[request.risk_type] === 'approve_once') {
      // Auto-approve
      respondToPermission(request.permission_id, 'approve_once' as PermissionDecision, false)
      return
    }

    // Add to pending list
    const existing = pendingRequests.value.find(r => r.permission_id === request.permission_id)
    if (!existing) {
      pendingRequests.value.push(request)
    }
  }

  function getSessionMemory(sessionId: string): SessionMemory {
    return sessionMemory.value.get(sessionId) || {}
  }

  function clearSessionMemory(sessionId: string) {
    sessionMemory.value.delete(sessionId)
  }

  return {
    // State
    pendingRequests,
    sessionMemory,
    isLoading,
    error,
    // Getters
    totalPendingCount,
    pendingBySession,
    getSessionPendingCount,
    // Actions
    fetchPendingPermissions,
    respondToPermission,
    addPendingRequest,
    getSessionMemory,
    clearSessionMemory,
  }
})
