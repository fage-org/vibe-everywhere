import { ref } from 'vue'
import { defineStore } from 'pinia'
import { Store } from '@tauri-apps/plugin-store'
import { settingsApi } from '@/api'
import { useToastStore } from './toast'
import type { NotificationPreference } from '@/types'

const STORE_NAME = 've-settings.json'

export interface ServerConfig {
  url: string
  name: string
}

export const useSettingsStore = defineStore('settings', () => {
  // State
  const serverConfig = ref<ServerConfig | null>(null)
  const notificationPrefs = ref<NotificationPreference>({
    device_id: '',
    enabled: true,
    permission_request_enabled: true,
    task_completed_enabled: true,
    task_failed_enabled: true,
    session_error_enabled: true,
  })
  const language = ref<string>('zh-CN')
  const isLoading = ref(false)
  const serverInfo = ref<{ version: string; build: string } | null>(null)

  const toast = useToastStore()

  // Actions
  async function loadSettings() {
    isLoading.value = true
    try {
      const store = await Store.load(STORE_NAME)
      const savedConfig = await store.get<ServerConfig>('serverConfig')
      if (savedConfig) {
        serverConfig.value = savedConfig
      }
      const savedLang = await store.get<string>('language')
      if (savedLang) {
        language.value = savedLang
      }

      // Load notification prefs from server
      await fetchNotificationPrefs()
    } catch (err) {
      console.error('Failed to load settings:', err)
    } finally {
      isLoading.value = false
    }
  }

  async function fetchNotificationPrefs() {
    try {
      const prefs = await settingsApi.getNotificationPrefs()
      notificationPrefs.value = prefs
    } catch (err) {
      console.error('Failed to fetch notification prefs:', err)
      // Use local fallback if server request fails
    }
  }

  async function saveServerConfig(config: ServerConfig) {
    try {
      const store = await Store.load(STORE_NAME)
      serverConfig.value = config
      await store.set('serverConfig', config)
      await store.save()
    } catch (err) {
      console.error('Failed to save server config:', err)
    }
  }

  async function saveNotificationPrefs(prefs: NotificationPreference) {
    isLoading.value = true
    try {
      // Save to server
      const updated = await settingsApi.updateNotificationPrefs(prefs)
      notificationPrefs.value = updated

      // Also save locally as backup
      const store = await Store.load(STORE_NAME)
      await store.set('notificationPrefs', prefs)
      await store.save()

      toast.success('Notification preferences saved')
    } catch (err) {
      console.error('Failed to save notification prefs:', err)
      toast.error('Failed to save notification preferences', err instanceof Error ? err.message : undefined)

      // Fallback: save locally only
      try {
        const store = await Store.load(STORE_NAME)
        notificationPrefs.value = prefs
        await store.set('notificationPrefs', prefs)
        await store.save()
      } catch (localErr) {
        console.error('Failed to save locally:', localErr)
      }
    } finally {
      isLoading.value = false
    }
  }

  async function setLanguage(lang: string) {
    try {
      const store = await Store.load(STORE_NAME)
      language.value = lang
      await store.set('language', lang)
      await store.save()
    } catch (err) {
      console.error('Failed to save language:', err)
    }
  }

  async function fetchServerInfo() {
    try {
      const info = await settingsApi.getServerInfo()
      serverInfo.value = info
    } catch (err) {
      console.error('Failed to fetch server info:', err)
    }
  }

  return {
    // State
    serverConfig,
    notificationPrefs,
    language,
    isLoading,
    serverInfo,
    // Actions
    loadSettings,
    fetchNotificationPrefs,
    saveServerConfig,
    saveNotificationPrefs,
    setLanguage,
    fetchServerInfo,
  }
})
