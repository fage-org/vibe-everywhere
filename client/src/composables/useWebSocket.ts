import { ref, computed, onUnmounted } from 'vue'
import { useAuthStore } from '@/stores/auth'

// Message types from server and client
export type WsMessageType =
  | 'session_event'
  | 'permission_request'
  | 'session_status_changed'
  | 'host_status_changed'
  | 'notification'
  | 'ping'
  | 'pong'
  | 'subscribe_session'
  | 'unsubscribe_session'

export interface WsMessage {
  type: WsMessageType
  payload: unknown
  timestamp: string
  request_id?: string
}

// Subscription callback type
type MessageHandler = (message: WsMessage) => void

// Subscription key structure
interface SubscriptionKey {
  messageType?: WsMessageType
  sessionId?: string
}

// Single WebSocket manager (singleton pattern)
class WebSocketManager {
  private ws: WebSocket | null = null
  private reconnectTimer: number | null = null
  private heartbeatTimer: number | null = null
  private subscriptions = new Map<string, Set<MessageHandler>>()
  private pendingMessages: WsMessage[] = []
  private isConnected = ref(false)
  private reconnectAttempts = 0
  private maxReconnectAttempts = 5
  private reconnectDelay = 1000 // Start with 1s, exponential backoff

  // Public state
  public readonly connected = computed(() => this.isConnected.value)

  // Generate subscription key
  private getSubKey(key: SubscriptionKey): string {
    return `${key.messageType || '*'}:${key.sessionId || '*'}`
  }

  // Connect to WebSocket
  connect(url: string, token: string) {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return
    }

    try {
      this.ws = new WebSocket(`${url}?token=${token}`)

      this.ws.onopen = () => {
        console.log('[WebSocket] Connected')
        this.isConnected.value = true
        this.reconnectAttempts = 0
        this.reconnectDelay = 1000
        this.startHeartbeat()
        this.flushPendingMessages()
      }

      this.ws.onmessage = (event) => {
        try {
          const message: WsMessage = JSON.parse(event.data)
          this.handleMessage(message)
        } catch (err) {
          console.error('[WebSocket] Failed to parse message:', err)
        }
      }

      this.ws.onclose = () => {
        console.log('[WebSocket] Disconnected')
        this.isConnected.value = false
        this.stopHeartbeat()
        this.scheduleReconnect()
      }

      this.ws.onerror = (error) => {
        console.error('[WebSocket] Error:', error)
      }
    } catch (err) {
      console.error('[WebSocket] Failed to connect:', err)
      this.scheduleReconnect()
    }
  }

  // Disconnect
  disconnect() {
    this.stopHeartbeat()
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }

  // Subscribe to messages
  subscribe(key: SubscriptionKey, handler: MessageHandler): () => void {
    const subKey = this.getSubKey(key)

    if (!this.subscriptions.has(subKey)) {
      this.subscriptions.set(subKey, new Set())
    }

    this.subscriptions.get(subKey)!.add(handler)

    // If subscribing to a specific session, send subscribe message
    if (key.sessionId && this.isConnected.value) {
      this.send({
        type: 'subscribe_session',
        payload: { session_id: key.sessionId },
        timestamp: new Date().toISOString(),
      })
    }

    // Return unsubscribe function
    return () => this.unsubscribe(key, handler)
  }

  // Unsubscribe from messages
  unsubscribe(key: SubscriptionKey, handler: MessageHandler) {
    const subKey = this.getSubKey(key)
    const handlers = this.subscriptions.get(subKey)

    if (handlers) {
      handlers.delete(handler)
      if (handlers.size === 0) {
        this.subscriptions.delete(subKey)
      }
    }

    // If unsubscribing from a specific session, send unsubscribe message
    if (key.sessionId && this.isConnected.value) {
      this.send({
        type: 'unsubscribe_session',
        payload: { session_id: key.sessionId },
        timestamp: new Date().toISOString(),
      })
    }
  }

  // Send message
  send(message: Omit<WsMessage, 'timestamp'> & { timestamp?: string }) {
    const fullMessage = {
      ...message,
      timestamp: message.timestamp || new Date().toISOString(),
    }

    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(fullMessage))
    } else {
      this.pendingMessages.push(fullMessage as WsMessage)
    }
  }

  // Handle incoming message
  private handleMessage(message: WsMessage) {
    // Handle pong
    if (message.type === 'pong') {
      return
    }

    // Route to specific subscribers
    const routes: string[] = []

    if (message.type) {
      // Match by type + session
      const sessionId = (message.payload as { session_id?: string })?.session_id
      if (sessionId) {
        routes.push(this.getSubKey({ messageType: message.type, sessionId }))
        routes.push(this.getSubKey({ sessionId }))
      }
      // Match by type only
      routes.push(this.getSubKey({ messageType: message.type }))
    }

    // Match all
    routes.push(this.getSubKey({}))

    // Notify all matching handlers
    const notified = new Set<MessageHandler>()
    routes.forEach((route) => {
      const handlers = this.subscriptions.get(route)
      handlers?.forEach((handler) => {
        if (!notified.has(handler)) {
          notified.add(handler)
          handler(message)
        }
      })
    })
  }

  // Flush pending messages
  private flushPendingMessages() {
    while (this.pendingMessages.length > 0) {
      const msg = this.pendingMessages.shift()
      if (msg) this.send(msg)
    }
  }

  // Heartbeat
  private startHeartbeat() {
    this.heartbeatTimer = window.setInterval(() => {
      this.send({
        type: 'ping',
        payload: {},
        timestamp: new Date().toISOString(),
      })
    }, 30000) // 30s heartbeat
  }

  private stopHeartbeat() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
  }

  // Reconnection with exponential backoff
  private scheduleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[WebSocket] Max reconnection attempts reached')
      return
    }

    this.reconnectAttempts++
    const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)

    console.log(`[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`)

    this.reconnectTimer = window.setTimeout(() => {
      const authStore = useAuthStore()
      if (authStore.serverUrl && authStore.token) {
        this.connect(authStore.serverUrl.replace('http', 'ws') + '/ws/client', authStore.token)
      }
    }, delay)
  }
}

// Singleton instance
const wsManager = new WebSocketManager()

// Composable for components
export function useWebSocket() {
  const authStore = useAuthStore()

  // Auto-connect when auth is available
  if (authStore.serverUrl && authStore.token && !wsManager.connected.value) {
    const wsUrl = authStore.serverUrl.replace(/^http/, 'ws')
    wsManager.connect(`${wsUrl}/ws/client`, authStore.token)
  }

  return {
    connected: wsManager.connected,
    subscribe: wsManager.subscribe.bind(wsManager),
    unsubscribe: wsManager.unsubscribe.bind(wsManager),
    send: wsManager.send.bind(wsManager),
    disconnect: wsManager.disconnect.bind(wsManager),
  }
}

// Hook for subscribing to specific message types
export function useWsMessage(
  messageType: WsMessageType | undefined,
  sessionId: string | undefined,
  handler: MessageHandler
) {
  const ws = useWebSocket()

  const unsubscribe = ws.subscribe({ messageType, sessionId }, handler)

  onUnmounted(() => {
    unsubscribe()
  })

  return unsubscribe
}

// Convenience hooks for specific message types
export function useSessionEvents(sessionId: string, handler: (payload: unknown) => void) {
  return useWsMessage('session_event', sessionId, (msg) => handler(msg.payload))
}

export function usePermissionRequests(sessionId: string, handler: (payload: unknown) => void) {
  return useWsMessage('permission_request', sessionId, (msg) => handler(msg.payload))
}

export function useSessionStatusChanges(handler: (payload: unknown) => void) {
  return useWsMessage('session_status_changed', undefined, (msg) => handler(msg.payload))
}

export function useHostStatusChanges(handler: (payload: unknown) => void) {
  return useWsMessage('host_status_changed', undefined, (msg) => handler(msg.payload))
}

export function useNotifications(handler: (payload: unknown) => void) {
  return useWsMessage('notification', undefined, (msg) => handler(msg.payload))
}
