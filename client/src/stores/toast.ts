import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface Toast {
  id: string
  type: ToastType
  title: string
  message?: string
  duration: number
  createdAt: number
}

export const useToastStore = defineStore('toast', () => {
  // State
  const toasts = ref<Toast[]>([])
  const maxToasts = 5

  // Getters
  const visibleToasts = computed(() => toasts.value.slice(0, maxToasts))

  // Actions
  function addToast(
    type: ToastType,
    title: string,
    message?: string,
    duration: number = 5000
  ): string {
    const id = `toast_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`

    const toast: Toast = {
      id,
      type,
      title,
      message,
      duration,
      createdAt: Date.now(),
    }

    toasts.value.unshift(toast)

    // Auto-remove after duration
    if (duration > 0) {
      setTimeout(() => {
        removeToast(id)
      }, duration)
    }

    // Keep only max toasts
    if (toasts.value.length > maxToasts) {
      toasts.value = toasts.value.slice(0, maxToasts)
    }

    return id
  }

  function removeToast(id: string) {
    const index = toasts.value.findIndex((t) => t.id === id)
    if (index > -1) {
      toasts.value.splice(index, 1)
    }
  }

  function success(title: string, message?: string, duration?: number) {
    return addToast('success', title, message, duration)
  }

  function error(title: string, message?: string, duration?: number) {
    return addToast('error', title, message, duration)
  }

  function warning(title: string, message?: string, duration?: number) {
    return addToast('warning', title, message, duration)
  }

  function info(title: string, message?: string, duration?: number) {
    return addToast('info', title, message, duration)
  }

  function clearAll() {
    toasts.value = []
  }

  return {
    // State
    toasts: visibleToasts,
    // Actions
    addToast,
    removeToast,
    success,
    error,
    warning,
    info,
    clearAll,
  }
})
