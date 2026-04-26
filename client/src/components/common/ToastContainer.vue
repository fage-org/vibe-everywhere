<template>
  <div class="fixed top-4 right-4 z-50 flex flex-col gap-2 pointer-events-none">
    <transition-group name="toast">
      <div
        v-for="toast in toastStore.toasts"
        :key="toast.id"
        class="pointer-events-auto min-w-[320px] max-w-[480px] bg-bg-primary border border-border rounded-lg shadow-lg p-4 flex items-start gap-3"
        :class="toastBorderClass(toast.type)"
      >
        <!-- Icon -->
        <div class="shrink-0 mt-0.5">
          <svg
            v-if="toast.type === 'success'"
            class="w-5 h-5 text-success"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/>
            <polyline points="22 4 12 14.01 9 11.01"/>
          </svg>
          <svg
            v-else-if="toast.type === 'error'"
            class="w-5 h-5 text-danger"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="10"/>
            <line x1="15" y1="9" x2="9" y2="15"/>
            <line x1="9" y1="9" x2="15" y2="15"/>
          </svg>
          <svg
            v-else-if="toast.type === 'warning'"
            class="w-5 h-5 text-warning"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
            <line x1="12" y1="9" x2="12" y2="13"/>
            <line x1="12" y1="17" x2="12.01" y2="17"/>
          </svg>
          <svg
            v-else
            class="w-5 h-5 text-accent"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="16" x2="12" y2="12"/>
            <line x1="12" y1="8" x2="12.01" y2="8"/>
          </svg>
        </div>

        <!-- Content -->
        <div class="flex-1 min-w-0">
          <p class="font-medium text-sm text-text-primary">{{ toast.title }}</p>
          <p v-if="toast.message" class="text-sm text-text-secondary mt-1">{{ toast.message }}</p>
        </div>

        <!-- Close Button -->
        <button
          @click="toastStore.removeToast(toast.id)"
          class="shrink-0 text-text-muted hover:text-text-primary transition-colors"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </transition-group>
  </div>
</template>

<script setup lang="ts">
import { useToastStore, type ToastType } from '@/stores/toast'

const toastStore = useToastStore()

function toastBorderClass(type: ToastType): string {
  switch (type) {
    case 'success':
      return 'border-l-4 border-l-success'
    case 'error':
      return 'border-l-4 border-l-danger'
    case 'warning':
      return 'border-l-4 border-l-warning'
    case 'info':
      return 'border-l-4 border-l-accent'
    default:
      return ''
  }
}
</script>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}
</style>
