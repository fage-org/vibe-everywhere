import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { useWebSocket } from '@/composables/useWebSocket'
import { useSessionStore } from './sessions'
import { usePermissionStore } from './permissions'
import { useHostStore } from './hosts'
import type { WsMessage, WsMessageType } from '@/composables/useWebSocket'
import type { PermissionRequest } from '@/types'

// Session event interface
interface SessionEvent {
  event_type: string
  data: unknown
}

// Event with metadata
export interface EventItem {
  id: string
  type: WsMessageType
  payload: unknown
  timestamp: Date
  sessionId?: string
  read: boolean
}

export const useEventStore = defineStore('events', () => {
  // State
  const events = ref<EventItem[]>([])
  const isConnected = ref(false)
  const lastError = ref<string | null>(null)

  // Getters
  const unreadCount = computed(() => events.value.filter((e) => !e.read).length)

  const eventsBySession = computed(() => {
    const grouped = new Map<string, EventItem[]>()
    events.value.forEach((event) => {
      if (event.sessionId) {
        const list = grouped.get(event.sessionId) || []
        list.push(event)
        grouped.set(event.sessionId, list)
      }
    })
    return grouped
  })

  const recentEvents = computed(() => {
    // Last 50 events, sorted by time desc
    return [...events.value]
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
      .slice(0, 50)
  })

  // Lazy store getters to avoid circular dependency issues
  function getSessionStore() {
    return useSessionStore()
  }
  function getPermissionStore() {
    return usePermissionStore()
  }
  function getHostStore() {
    return useHostStore()
  }

  // Actions
  function initialize() {
    const ws = useWebSocket()

    // Subscribe to all messages
    ws.subscribe({}, handleMessage)

    isConnected.value = ws.connected.value
  }

  function handleMessage(message: WsMessage) {

    // Extract session ID from payload
    const sessionId = (message.payload as { session_id?: string })?.session_id

    // Create event item
    const event: EventItem = {
      id: `${message.type}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type: message.type,
      payload: message.payload,
      timestamp: new Date(message.timestamp),
      sessionId,
      read: false,
    }

    // Store event
    events.value.push(event)

    // Trim old events (keep last 1000)
    if (events.value.length > 1000) {
      events.value = events.value.slice(-1000)
    }

    // Route to specific handlers
    switch (message.type) {
      case 'session_event':
        handleSessionEvent(message.payload as SessionEvent, sessionId)
        break

      case 'permission_request':
        handlePermissionRequest(message.payload as PermissionRequest)
        break

      case 'session_status_changed':
        handleSessionStatusChange(message.payload as { session_id: string; status: string })
        break

      case 'host_status_changed':
        handleHostStatusChange(message.payload as { host_id: string; status: string })
        break

      case 'notification':
        // Notifications are just stored, UI can poll
        break
    }
  }

  function handleSessionEvent(_payload: SessionEvent, _sessionId?: string) {
    // Session events are handled by the session store
    // This could update logs, diff, etc.
    // const sessionStore = getSessionStore()
  }

  function handlePermissionRequest(payload: PermissionRequest) {
    const permissionStore = getPermissionStore()
    permissionStore.addPendingRequest(payload)
  }

  function handleSessionStatusChange(payload: { session_id: string; status: string }) {
    const sessionStore = getSessionStore()
    sessionStore.updateSessionStatus(payload.session_id, payload.status)
  }

  function handleHostStatusChange(payload: { host_id: string; status: string }) {
    const hostStore = getHostStore()
    // Update host status in store
    const host = hostStore.hosts.find((h: { host_id: string }) => h.host_id === payload.host_id)
    if (host) {
      host.online_status = payload.status as 'online' | 'offline' | 'unknown'
    }
  }

  function markAsRead(eventId: string) {
    const event = events.value.find((e) => e.id === eventId)
    if (event) {
      event.read = true
    }
  }

  function markAllAsRead() {
    events.value.forEach((e) => (e.read = true))
  }

  function clearEvents() {
    events.value = []
  }

  function getEventsForSession(sessionId: string, limit = 50): EventItem[] {
    return events.value
      .filter((e) => e.sessionId === sessionId)
      .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
      .slice(0, limit)
  }

  return {
    // State
    events,
    isConnected,
    lastError,
    // Getters
    unreadCount,
    eventsBySession,
    recentEvents,
    // Actions
    initialize,
    markAsRead,
    markAllAsRead,
    clearEvents,
    getEventsForSession,
  }
})

// Placeholder type - should match actual SessionEvent type
interface SessionEvent {
  event_type: string
  data: unknown
}
