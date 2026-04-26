<template>
  <div v-if="session" class="flex flex-col h-full">
    <!-- Header -->
    <div class="flex items-center justify-between px-6 py-4 border-b border-border">
      <div class="flex items-center gap-4">
        <button @click="back" class="text-text-muted hover:text-text-primary transition-colors">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m15 18-6-6 6-6"/>
          </svg>
        </button>
        <div>
          <h2 class="font-semibold">{{ session.title }}</h2>
          <div class="flex items-center gap-2 text-sm text-text-secondary">
            <span>{{ session.host_id }}</span>
            <span class="text-text-muted">/</span>
            <span>{{ session.workspace_id }}</span>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <!-- Status Badge -->
        <span
          :class="[
            'text-xs px-2.5 py-1 rounded-full',
            statusClass
          ]"
        >
          {{ statusText }}
        </span>

        <!-- Action Menu -->
        <SessionActionsMenu :session="session" />
      </div>
    </div>

    <!-- Pending Permission Banner -->
    <div v-if="pendingPermissions.length > 0" class="bg-danger-bg border-b border-danger/20 px-6 py-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2 text-danger">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          <span class="text-sm font-medium">
            {{ pendingPermissions.length }} {{ $t('sessionDetail.pendingPermission') }}
          </span>
        </div>
        <button
          @click="showPermissions = true"
          class="text-sm text-accent hover:underline"
        >
          {{ $t('common.view') || 'View' }}
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex items-center gap-1 px-6 border-b border-border">
      <button
        v-for="tab in tabs"
        :key="tab.value"
        @click="currentTab = tab.value"
        :class="[
          'px-4 py-3 text-sm font-medium border-b-2 transition-colors',
          currentTab === tab.value
            ? 'border-accent text-accent'
            : 'border-transparent text-text-secondary hover:text-text-primary'
        ]"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 overflow-hidden">
      <OverviewTab v-if="currentTab === 'overview'" :session="session" />
      <LogsTab v-if="currentTab === 'logs'" :session-id="session.session_id" />
      <DiffTab v-if="currentTab === 'diff'" :session-id="session.session_id" />
      <FilesTab v-if="currentTab === 'files'" :session-id="session.session_id" />
    </div>

    <!-- Message Input (only for active sessions) -->
    <div v-if="canSendMessage" class="border-t border-border px-6 py-4">
      <div class="flex gap-3">
        <textarea
          v-model="messageInput"
          :placeholder="$t('sessions.taskMessagePlaceholder')"
          rows="2"
          class="flex-1 px-4 py-2 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent resize-none"
          @keyup.enter.prevent="sendMessage"
        />
        <button
          @click="sendMessage"
          :disabled="!messageInput.trim() || isSending"
          class="px-4 py-2 bg-accent hover:bg-accent/90 text-white rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg v-if="isSending" class="animate-spin w-5 h-5" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <svg v-else class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="22" y1="2" x2="11" y2="13"/>
            <polygon points="22 2 15 22 11 13 2 9 22 2"/>
          </svg>
        </button>
      </div>
    </div>
  </div>

  <!-- Permission Panel -->
  <PermissionPanel v-model="showPermissions" :permissions="pendingPermissions" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSessionStore } from '@/stores/sessions'
import { usePermissionStore } from '@/stores/permissions'
// PermissionRequest type used in component
import SessionActionsMenu from '@/components/sessions/SessionActionsMenu.vue'
import OverviewTab from '@/components/sessions/OverviewTab.vue'
import LogsTab from '@/components/sessions/LogsTab.vue'
import DiffTab from '@/components/sessions/DiffTab.vue'
import FilesTab from '@/components/sessions/FilesTab.vue'
import PermissionPanel from '@/components/permissions/PermissionPanel.vue'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const sessionStore = useSessionStore()
const permissionStore = usePermissionStore()

// State
const currentTab = ref('overview')
const messageInput = ref('')
const isSending = ref(false)
const showPermissions = ref(false)

// Tabs
const tabs = [
  { value: 'overview', label: t('sessionDetail.overview') },
  { value: 'logs', label: t('sessionDetail.logs') },
  { value: 'diff', label: t('sessionDetail.diff') },
  { value: 'files', label: t('sessionDetail.files') },
]

// Computed
const sessionId = computed(() => route.params.id as string)
const session = computed(() => sessionStore.activeSession)

const statusText = computed(() => {
  if (!session.value) return ''
  switch (session.value.status) {
    case 'running': return t('sessions.statusRunning')
    case 'waiting_approval': return t('sessions.statusWaiting')
    case 'paused': return t('sessions.statusPaused')
    case 'error': return t('sessions.statusError')
    default: return session.value.status
  }
})

const statusClass = computed(() => {
  if (!session.value) return ''
  switch (session.value.status) {
    case 'running': return 'bg-success-bg text-success'
    case 'waiting_approval': return 'bg-warning-bg text-warning'
    case 'paused': return 'bg-bg-tertiary text-text-secondary'
    case 'error': return 'bg-danger-bg text-danger'
    default: return 'bg-bg-tertiary text-text-muted'
  }
})

const canSendMessage = computed(() => {
  if (!session.value) return false
  return ['running', 'waiting_approval'].includes(session.value.status)
})

const pendingPermissions = computed(() => {
  if (!session.value) return []
  return permissionStore.pendingRequests.filter(
    r => r.session_id === session.value?.session_id
  )
})

// Methods
function back() {
  router.push('/sessions')
}

async function sendMessage() {
  if (!messageInput.value.trim() || !session.value) return

  isSending.value = true
  const success = await sessionStore.sendMessage(
    session.value.session_id,
    messageInput.value.trim()
  )

  if (success) {
    messageInput.value = ''
  }

  isSending.value = false
}

// Watch for route changes
watch(() => route.params.id, async (id) => {
  if (id) {
    await sessionStore.fetchSessions()
    sessionStore.selectSession(id as string)
    await permissionStore.fetchPendingPermissions(id as string)
  }
}, { immediate: true })

// Lifecycle
onMounted(() => {
  if (sessionId.value) {
    permissionStore.fetchPendingPermissions(sessionId.value)
  }
})
</script>
