import { apiClient } from './client'
import type { NotificationPreference } from './types'

export interface ServerInfo {
  version: string
  build: string
}

export const settingsApi = {
  // Get notification preferences
  async getNotificationPrefs(): Promise<NotificationPreference> {
    return apiClient.get<NotificationPreference>('/api/settings/notifications')
  },

  // Update notification preferences
  async updateNotificationPrefs(prefs: NotificationPreference): Promise<NotificationPreference> {
    return apiClient.put<NotificationPreference>('/api/settings/notifications', prefs)
  },

  // Get server info
  async getServerInfo(): Promise<ServerInfo> {
    return apiClient.get<ServerInfo>('/api/settings/server')
  },
}
