<template>
  <div class="h-full overflow-y-auto p-6">
    <div class="max-w-3xl mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <h1 class="text-2xl font-semibold">{{ $t('notifications.title') }}</h1>

        <button
          @click="markAllRead"
          class="text-sm text-accent hover:underline"
        >
          {{ $t('notifications.markAllRead') }}
        </button>
      </div>

      <!-- Notification List -->
      <div class="bg-bg-secondary rounded-lg border border-border">
        <div v-if="notifications.length === 0" class="p-8 text-center text-text-muted">
          <svg class="w-12 h-12 mx-auto mb-3 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
            <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
          </svg>
          <p>{{ $t('common.noData') }}</p>
        </div>

        <div v-else class="divide-y divide-border">
          <div
            v-for="notification in notifications"
            :key="notification.id"
            :class="[
              'p-4 cursor-pointer transition-colors',
              notification.read ? 'opacity-60' : 'bg-accent-bg/30'
            ]"
            @click="handleNotification(notification)"
          >
            <div class="flex items-start gap-3">
              <div
                :class="[
                  'w-2 h-2 rounded-full mt-2 shrink-0',
                  notificationIconClass(notification.type)
                ]"
              />

              <div class="flex-1">
                <p class="font-medium text-sm">{{ notification.title }}</p>
                <p class="text-sm text-text-secondary mt-1">{{ notification.message }}</p>
                <p class="text-xs text-text-muted mt-2">{{ formatTime(notification.time) }}</p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useEventStore } from '@/stores/events'
import type { EventItem } from '@/stores/events'

interface Notification {
  id: string
  type: 'permission' | 'completed' | 'failed' | 'error'
  title: string
  message: string
  time: string
  read: boolean
  sessionId?: string
}

const router = useRouter()
const eventStore = useEventStore()

// Convert events to notifications
const notifications = computed<Notification[]>(() => {
  return eventStore.recentEvents.map((event: EventItem) => {
    let type: Notification['type'] = 'error'
    let title = ''
    let message = ''

    switch (event.type) {
      case 'permission_request':
        type = 'permission'
        title = 'Permission Request'
        message = (event.payload as { summary?: string })?.summary || 'New permission request'
        break
      case 'session_status_changed':
        const status = (event.payload as { status?: string })?.status
        type = status === 'error' ? 'error' : 'completed'
        title = 'Session Status Changed'
        message = `Session is now ${status}`
        break
      case 'host_status_changed':
        type = 'completed'
        title = 'Host Status Changed'
        message = 'Host connection status updated'
        break
      case 'notification':
        type = 'completed'
        title = 'Notification'
        message = (event.payload as { message?: string })?.message || 'New notification'
        break
      default:
        type = 'completed'
        title = event.type
        message = 'New event'
    }

    return {
      id: event.id,
      type,
      title,
      message,
      time: event.timestamp.toISOString(),
      read: event.read,
      sessionId: event.sessionId,
    }
  })
})

function notificationIconClass(type: string): string {
  switch (type) {
    case 'permission': return 'bg-warning'
    case 'completed': return 'bg-success'
    case 'failed': return 'bg-danger'
    case 'error': return 'bg-danger'
    default: return 'bg-text-muted'
  }
}

function formatTime(time: string): string {
  return new Date(time).toLocaleString()
}

function handleNotification(notification: Notification) {
  eventStore.markAsRead(notification.id)
  if (notification.sessionId) {
    router.push(`/sessions/${notification.sessionId}`)
  }
}

function markAllRead() {
  eventStore.markAllAsRead()
}
</script>
