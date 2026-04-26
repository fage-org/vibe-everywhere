<template>
  <div v-if="hasError" class="min-h-screen flex items-center justify-center p-6">
    <div class="max-w-md w-full text-center">
      <!-- Error Icon -->
      <div class="w-20 h-20 mx-auto mb-6 rounded-full bg-danger-bg flex items-center justify-center">
        <svg class="w-10 h-10 text-danger" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"/>
          <line x1="12" y1="8" x2="12" y2="12"/>
          <line x1="12" y1="16" x2="12.01" y2="16"/>
        </svg>
      </div>

      <!-- Error Message -->
      <h2 class="text-xl font-semibold mb-2">{{ $t('error.title') || 'Something went wrong' }}</h2>
      <p class="text-text-secondary mb-6">
        {{ $t('error.message') || 'An unexpected error occurred. Please try again.' }}
      </p>

      <!-- Error Details (development only) -->
      <div
        v-if="isDev && errorDetails"
        class="bg-bg-secondary rounded-lg p-4 mb-6 text-left overflow-auto max-h-48"
      >
        <p class="text-sm font-mono text-danger mb-2">{{ errorDetails.message }}</p>
        <pre class="text-xs text-text-muted">{{ errorDetails.stack }}</pre>
      </div>

      <!-- Action Buttons -->
      <div class="flex gap-3 justify-center">
        <button
          @click="resetError"
          class="px-4 py-2 bg-accent hover:bg-accent/90 text-white rounded-lg transition-colors"
        >
          {{ $t('error.retry') || 'Try Again' }}
        </button>

        <button
          @click="goHome"
          class="px-4 py-2 border border-border hover:bg-bg-secondary rounded-lg transition-colors"
        >
          {{ $t('error.goHome') || 'Go Home' }}
        </button>
      </div>
    </div>
  </div>

  <slot v-else />
</template>

<script setup lang="ts">
import { ref, onErrorCaptured, onMounted } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

// State
const hasError = ref(false)
const errorDetails = ref<Error | null>(null)
const isDev = import.meta.env.DEV

// Reset error state
function resetError() {
  hasError.value = false
  errorDetails.value = null
}

// Navigate to home
function goHome() {
  resetError()
  router.push('/')
}

// Capture errors from child components
onErrorCaptured((err, instance, info) => {
  console.error('ErrorBoundary caught error:', err)
  console.error('Component:', instance)
  console.error('Info:', info)

  hasError.value = true
  errorDetails.value = err instanceof Error ? err : new Error(String(err))

  // Prevent error from propagating
  return false
})

// Handle global errors
onMounted(() => {
  const handler = (event: ErrorEvent) => {
    console.error('Global error:', event.error)
    hasError.value = true
    errorDetails.value = event.error
  }

  const rejectionHandler = (event: PromiseRejectionEvent) => {
    console.error('Unhandled promise rejection:', event.reason)
    hasError.value = true
    errorDetails.value = event.reason instanceof Error
      ? event.reason
      : new Error(String(event.reason))
  }

  window.addEventListener('error', handler)
  window.addEventListener('unhandledrejection', rejectionHandler)

  // Cleanup
  return () => {
    window.removeEventListener('error', handler)
    window.removeEventListener('unhandledrejection', rejectionHandler)
  }
})
</script>
