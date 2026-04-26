import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { Store } from '@tauri-apps/plugin-store'
import { authApi } from '@/api/auth'
import { apiClient } from '@/api/client'
import type { RegisterDeviceResponse } from '@/types'

const STORE_NAME = 've-settings.json'

export const useAuthStore = defineStore('auth', () => {
  // State
  const token = ref<string | null>(null)
  const deviceId = ref<string | null>(null)
  const deviceName = ref<string>('')
  const serverUrl = ref<string>('')
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const isAuthenticated = computed(() => !!token.value)

  // Actions
  async function loadCredentials() {
    try {
      const store = await Store.load(STORE_NAME)
      token.value = await store.get<string>('token') || null
      deviceId.value = await store.get<string>('deviceId') || null
      deviceName.value = await store.get<string>('deviceName') || ''
      serverUrl.value = await store.get<string>('serverUrl') || ''

      // Set API client base URL if we have server URL
      if (serverUrl.value) {
        apiClient.setBaseUrl(serverUrl.value)
      }
    } catch (err) {
      console.error('Failed to load credentials:', err)
    }
  }

  async function saveCredentials() {
    try {
      const store = await Store.load(STORE_NAME)
      await store.set('token', token.value)
      await store.set('deviceId', deviceId.value)
      await store.set('deviceName', deviceName.value)
      await store.set('serverUrl', serverUrl.value)
      await store.save()
    } catch (err) {
      console.error('Failed to save credentials:', err)
    }
  }

  async function clearCredentials() {
    try {
      const store = await Store.load(STORE_NAME)
      await store.delete('token')
      await store.delete('deviceId')
      await store.delete('deviceName')
      await store.delete('serverUrl')
      await store.save()
    } catch (err) {
      console.error('Failed to clear credentials:', err)
    }
  }

  async function login(serverUrlInput: string, deviceNameInput: string) {
    isLoading.value = true
    error.value = null

    try {
      const response: RegisterDeviceResponse = await authApi.registerDevice(
        serverUrlInput,
        deviceNameInput,
        'desktop'
      )

      token.value = response.token
      deviceId.value = response.device_id
      deviceName.value = deviceNameInput
      serverUrl.value = serverUrlInput

      // Set API client base URL
      apiClient.setBaseUrl(serverUrlInput)

      // Save to persistent storage
      await saveCredentials()

      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Login failed'
      return false
    } finally {
      isLoading.value = false
    }
  }

  async function logout() {
    token.value = null
    deviceId.value = null
    deviceName.value = ''
    await clearCredentials()
  }

  async function testConnection(url: string) {
    isLoading.value = true
    error.value = null

    try {
      const result = await authApi.testConnection(url)
      if (!result.success) {
        error.value = result.message || 'Connection failed'
        return false
      }
      return true
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Connection failed'
      return false
    } finally {
      isLoading.value = false
    }
  }

  return {
    // State
    token,
    deviceId,
    deviceName,
    serverUrl,
    isLoading,
    error,
    // Getters
    isAuthenticated,
    // Actions
    loadCredentials,
    saveCredentials,
    login,
    logout,
    testConnection,
  }
})
