<template>
  <div
    :class="[
      'px-4 py-3 cursor-pointer border-b border-border transition-colors',
      isActive ? 'bg-accent-bg border-l-4 border-l-accent' : 'hover:bg-bg-hover border-l-4 border-l-transparent'
    ]"
    @click="$emit('click')"
  >
    <div class="flex items-start justify-between mb-1">
      <h3 class="text-sm font-medium truncate pr-2" :class="isActive ? 'text-accent' : ''">
        {{ session.title }}
      </h3>
      <!-- Status Badge -->
      <span
        :class="[
          'text-[10px] px-1.5 py-0.5 rounded-full shrink-0',
          statusClass
        ]"
      >
        {{ statusText }}
      </span>
    </div>

    <div class="flex items-center gap-2 text-xs text-text-secondary mb-2">
      <span class="flex items-center gap-1">
        <svg class="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="2" y="3" width="20" height="14" rx="2"/>
          <line x1="8" y1="21" x2="16" y2="21"/>
          <line x1="12" y1="17" x2="12" y2="21"/>
        </svg>
        {{ hostName }}
      </span>
      <span class="text-text-muted">•</span>
      <span>{{ workspaceName }}</span>
    </div>

    <div class="flex items-center justify-between">
      <span class="text-xs text-text-muted">
        {{ lastActivityText }}
      </span>

      <!-- Permission Badge -->
      <span
        v-if="session.pending_permission_count > 0"
        class="text-[10px] px-2 py-0.5 bg-danger-bg text-danger rounded-full font-medium"
      >
        {{ session.pending_permission_count }} {{ $t('sessions.pendingPermission') }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Session } from '@/types'

const props = defineProps<{
  session: Session
  isActive?: boolean
}>()

defineEmits<{
  click: []
}>()

const { t } = useI18n()

const statusText = computed(() => {
  switch (props.session.status) {
    case 'running': return t('sessions.statusRunning')
    case 'waiting_approval': return t('sessions.statusWaiting')
    case 'paused': return t('sessions.statusPaused')
    case 'error': return t('sessions.statusError')
    case 'archived': return t('sessions.statusArchived')
    default: return props.session.status
  }
})

const statusClass = computed(() => {
  switch (props.session.status) {
    case 'running': return 'bg-success-bg text-success'
    case 'waiting_approval': return 'bg-warning-bg text-warning'
    case 'paused': return 'bg-bg-tertiary text-text-secondary'
    case 'error': return 'bg-danger-bg text-danger'
    default: return 'bg-bg-tertiary text-text-muted'
  }
})

const hostName = computed(() => {
  // In a real app, look up the host name from host store
  return props.session.host_id.slice(0, 8)
})

const workspaceName = computed(() => {
  // In a real app, look up the workspace name from workspace store
  return props.session.workspace_id.slice(0, 8)
})

const lastActivityText = computed(() => {
  if (!props.session.last_activity_at) return ''
  const date = new Date(props.session.last_activity_at)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)

  if (minutes < 1) return 'Just now'
  if (minutes < 60) return `${minutes}m ago`
  if (hours < 24) return `${hours}h ago`
  if (days < 7) return `${days}d ago`
  return date.toLocaleDateString()
})
</script>
