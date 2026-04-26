<template>
  <div class="flex flex-col h-screen items-center justify-center bg-bg-secondary p-6">
    <div class="w-full max-w-md">
      <!-- Logo -->
      <div class="flex flex-col items-center mb-8">
        <div class="w-16 h-16 rounded-2xl bg-gradient-to-br from-accent to-purple flex items-center justify-center mb-4">
          <span class="text-2xl font-bold text-white">V</span>
        </div>
        <h1 class="text-2xl font-bold">{{ $t('setup.title') }}</h1>
        <p class="text-sm text-text-secondary mt-1">{{ $t('setup.description') }}</p>
      </div>

      <!-- Form -->
      <div class="bg-bg-primary rounded-xl border border-border p-6 space-y-5">
        <!-- Server URL -->
        <div>
          <label class="block text-sm font-medium mb-2">{{ $t('setup.serverUrl') }}</label>
          <input
            v-model="serverUrl"
            type="url"
            placeholder="https://ve-server.example.com"
            class="w-full px-4 py-2.5 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent transition-colors"
            :disabled="isLoading"
            @keyup.enter="testConnection"
          />
          <p v-if="connectionStatus === 'success'" class="text-xs text-green mt-2 flex items-center gap-1">
            <svg class="w-3.5 h-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12"/>
            </svg>
            {{ $t('setup.connectionSuccess') }}
          </p>
          <p v-if="connectionStatus === 'error'" class="text-xs text-red mt-2">
            {{ connectionError }}
          </p>
        </div>

        <!-- Test Connection Button -->
        <button
          @click="testConnection"
          :disabled="!serverUrl || isTesting"
          class="w-full py-2.5 px-4 rounded-lg text-sm font-medium transition-colors border border-border hover:bg-bg-hover disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          <svg v-if="isTesting" class="animate-spin w-4 h-4" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ isTesting ? $t('setup.testing') : $t('setup.testConnection') }}
        </button>

        <!-- Divider -->
        <div class="border-t border-border"></div>

        <!-- Device Name -->
        <div>
          <label class="block text-sm font-medium mb-2">{{ $t('setup.deviceName') }}</label>
          <input
            v-model="deviceName"
            type="text"
            :placeholder="defaultDeviceName"
            class="w-full px-4 py-2.5 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent transition-colors"
            :disabled="isLoading"
          />
        </div>

        <!-- Connect Button -->
        <button
          @click="connect"
          :disabled="!canConnect || isLoading"
          class="w-full py-2.5 px-4 bg-accent hover:bg-accent/90 text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
        >
          <svg v-if="isLoading" class="animate-spin w-4 h-4" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ isLoading ? $t('setup.connecting') : $t('setup.connect') }}
        </button>

        <!-- Error -->
        <p v-if="error" class="text-xs text-red text-center">
          {{ error }}
        </p>
      </div>

      <!-- Hint -->
      <p class="text-xs text-text-muted text-center mt-6">
        {{ $t('setup.hint') }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

// State
const serverUrl = ref('')
const deviceName = ref('')
const connectionStatus = ref<'idle' | 'testing' | 'success' | 'error'>('idle')
const connectionError = ref('')
const isTesting = ref(false)
const isLoading = ref(false)
const error = ref('')

// Computed
const defaultDeviceName = computed(() => {
  // In a real app, get the actual device name from Tauri OS API
  return `Desktop ${new Date().toLocaleDateString()}`
})

const canConnect = computed(() => {
  return serverUrl.value && connectionStatus.value === 'success' && deviceName.value
})

// Methods
async function testConnection() {
  if (!serverUrl.value) return

  isTesting.value = true
  connectionStatus.value = 'testing'
  connectionError.value = ''

  const success = await authStore.testConnection(serverUrl.value)

  if (success) {
    connectionStatus.value = 'success'
  } else {
    connectionStatus.value = 'error'
    connectionError.value = authStore.error || 'Connection failed'
  }

  isTesting.value = false
}

async function connect() {
  if (!canConnect.value) return

  isLoading.value = true
  error.value = ''

  const name = deviceName.value || defaultDeviceName.value
  const success = await authStore.login(serverUrl.value, name)

  if (success) {
    router.push('/')
  } else {
    error.value = authStore.error || 'Failed to connect'
  }

  isLoading.value = false
}

// Check if already authenticated
onMounted(async () => {
  await authStore.loadCredentials()
  if (authStore.isAuthenticated) {
    router.push('/')
  }
})
</script>
