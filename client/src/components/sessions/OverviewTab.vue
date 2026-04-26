<template>
  <div class="h-full overflow-y-auto p-6">
    <div class="max-w-2xl space-y-6">
      <!-- Session Info -->
      <div class="bg-bg-secondary rounded-lg border border-border p-4">
        <h3 class="text-sm font-medium mb-4">{{ $t('sessionDetail.sessionAttributes') }}</h3>
        <div class="grid grid-cols-2 gap-4 text-sm">
          <div>
            <span class="text-text-muted">{{ $t('sessionDetail.agent') }}:</span>
            <span class="ml-2">{{ session.agent_type }}</span>
          </div>
          <div>
            <span class="text-text-muted">{{ $t('sessionDetail.workspace') }}:</span>
            <span class="ml-2">{{ session.workspace_id.slice(0, 8) }}</span>
          </div>
          <div>
            <span class="text-text-muted">Created:</span>
            <span class="ml-2">{{ formatTime(session.created_at) }}</span>
          </div>
          <div>
            <span class="text-text-muted">Status:</span>
            <span class="ml-2">{{ session.status }}</span>
          </div>
        </div>
      </div>

      <!-- Recent Activity -->
      <div class="bg-bg-secondary rounded-lg border border-border p-4">
        <h3 class="text-sm font-medium mb-4">{{ $t('sessionDetail.recentActivity') }}</h3>
        <div v-if="recentEvents.length === 0" class="text-sm text-text-muted">
          No recent activity
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="event in recentEvents"
            :key="event.id"
            class="text-sm py-2 border-b border-border last:border-0"
          >
            <span class="text-text-muted">{{ formatTime(event.timestamp) }}</span>
            <span class="ml-3">{{ event.message }}</span>
          </div>
        </div>
      </div>

      <!-- Change Summary -->
      <div class="bg-bg-secondary rounded-lg border border-border p-4">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-sm font-medium">{{ $t('sessionDetail.changeSummary') }}</h3>
          <span class="text-xs text-accent cursor-pointer hover:underline">
            {{ $t('common.view') || 'View' }}
          </span>
        </div>
        <p class="text-sm text-text-secondary">
          {{ $t('sessionDetail.filesChanged', { count: 0 }) }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useEventStore } from '@/stores/events'
import type { Session } from '@/types'

const props = defineProps<{
  session: Session
}>()

const eventStore = useEventStore()

// Get events for this session from the event store
const recentEvents = computed(() => {
  return eventStore.getEventsForSession(props.session.session_id, 10).map(event => {
    // Format event message based on type
    let message = ''
    switch (event.type) {
      case 'session_event':
        message = 'Session event'
        break
      case 'permission_request':
        message = 'Permission requested'
        break
      case 'session_status_changed':
        message = `Status changed to ${(event.payload as { status?: string })?.status || 'unknown'}`
        break
      case 'host_status_changed':
        message = 'Host status changed'
        break
      case 'notification':
        message = (event.payload as { message?: string })?.message || 'Notification'
        break
      default:
        message = event.type
    }

    return {
      id: event.id,
      timestamp: event.timestamp.toISOString(),
      message,
    }
  })
})

function formatTime(time?: string): string {
  if (!time) return 'N/A'
  return new Date(time).toLocaleString()
}
</script>
