<template>
  <div class="flex flex-col h-full">
    <!-- Toolbar -->
    <div class="flex items-center justify-between px-6 py-3 border-b border-border">
      <div class="relative flex-1 max-w-md">
        <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="$t('sessionDetail.logSearch')"
          class="w-full pl-9 pr-3 py-1.5 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent"
        />
      </div>

      <div class="flex items-center gap-3">
        <label class="flex items-center gap-2 text-sm cursor-pointer">
          <input v-model="autoScroll" type="checkbox" class="rounded border-border" />
          {{ $t('sessionDetail.autoScroll') }}
        </label>

        <button
          @click="clearLogs"
          class="p-1.5 text-text-muted hover:text-text-primary transition-colors"
          title="Clear"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Log List -->
    <div ref="logContainer" class="flex-1 overflow-y-auto p-4 font-mono text-sm">
      <div v-if="logs.length === 0" class="flex items-center justify-center h-full text-text-muted">
        {{ $t('common.noData') }}
      </div>

      <div v-else class="space-y-1">
        <div
          v-for="log in filteredLogs"
          :key="log.id"
          class="flex items-start gap-3 py-1 hover:bg-bg-secondary rounded px-2"
        >
          <span class="text-text-muted shrink-0 w-16">{{ formatTime(log.timestamp) }}</span>
          <span
            :class="[
              'shrink-0 w-14 text-center text-[10px] px-1.5 py-0.5 rounded',
              levelClass(log.level)
            ]"
          >
            {{ log.level }}
          </span>
          <span class="text-text-primary whitespace-pre-wrap">{{ log.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { useSessionEvents } from '@/composables/useWebSocket'

interface LogEntry {
  id: string
  timestamp: string
  level: 'INFO' | 'WARN' | 'ERROR' | 'TOOL'
  message: string
}

const props = defineProps<{
  sessionId: string
}>()

const searchQuery = ref('')
const autoScroll = ref(true)
const logs = ref<LogEntry[]>([])
const logContainer = ref<HTMLDivElement>()

// WebSocket subscription
let unsubscribe: (() => void) | null = null

const filteredLogs = computed(() => {
  if (!searchQuery.value) return logs.value
  const query = searchQuery.value.toLowerCase()
  return logs.value.filter(log =>
    log.message.toLowerCase().includes(query)
  )
})

function levelClass(level: string): string {
  switch (level) {
    case 'INFO': return 'bg-success-bg text-success'
    case 'WARN': return 'bg-warning-bg text-warning'
    case 'ERROR': return 'bg-danger-bg text-danger'
    case 'TOOL': return 'bg-accent-bg text-accent'
    default: return 'bg-bg-tertiary text-text-muted'
  }
}

function formatTime(timestamp: string): string {
  const date = new Date(timestamp)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function clearLogs() {
  logs.value = []
}

function scrollToBottom() {
  if (autoScroll.value && logContainer.value) {
    nextTick(() => {
      logContainer.value!.scrollTop = logContainer.value!.scrollHeight
    })
  }
}

// Handle incoming session events
function handleSessionEvent(payload: unknown) {
  const event = payload as {
    event_type: string
    data: {
      level?: 'INFO' | 'WARN' | 'ERROR' | 'TOOL'
      message?: string
      timestamp?: string
    }
  }

  // Convert session events to log entries
  if (event.event_type === 'log' || event.event_type === 'tool_call') {
    const level = event.event_type === 'tool_call' ? 'TOOL' : (event.data.level || 'INFO')
    const message = event.data.message || JSON.stringify(event.data)

    logs.value.push({
      id: `${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      timestamp: event.data.timestamp || new Date().toISOString(),
      level,
      message,
    })

    // Trim to last 1000 logs
    if (logs.value.length > 1000) {
      logs.value = logs.value.slice(-1000)
    }
  }
}

// Watch for new logs and auto-scroll
watch(() => logs.value.length, scrollToBottom)

onMounted(() => {
  // Subscribe to WebSocket events for this session
  unsubscribe = useSessionEvents(props.sessionId, handleSessionEvent)
})

onUnmounted(() => {
  // Clean up WebSocket subscription
  if (unsubscribe) {
    unsubscribe()
  }
})
</script>
