import { apiClient } from './client'
import type { PermissionRequest, PermissionResponseRequest } from '@/types'
import type { PermissionDecision } from '@/types/generated/PermissionDecision'

export const permissionsApi = {
  // List pending permission requests
  async listPermissions(sessionId?: string): Promise<PermissionRequest[]> {
    const query = sessionId ? `?session_id=${sessionId}` : ''
    return apiClient.get<PermissionRequest[]>(`/api/permissions${query}`)
  },

  // Respond to a permission request
  async respondPermission(
    permissionId: string,
    decision: PermissionDecision,
    rememberInSession: boolean = false
  ): Promise<void> {
    const payload: PermissionResponseRequest = {
      decision: decision as 'approve_once' | 'deny_once' | 'approve_session',
      remember_in_session: rememberInSession,
    }
    return apiClient.post(`/api/permissions/${permissionId}/respond`, payload)
  },

  // Approve a permission request (convenience method)
  async approvePermission(permissionId: string, rememberInSession: boolean = false): Promise<void> {
    return this.respondPermission(permissionId, 'approve_once' as PermissionDecision, rememberInSession)
  },

  // Deny a permission request (convenience method)
  async denyPermission(permissionId: string): Promise<void> {
    return this.respondPermission(permissionId, 'deny_once' as PermissionDecision, false)
  },
}
