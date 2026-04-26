<template>
  <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click="close">
    <div class="bg-bg-primary rounded-xl border border-border shadow-lg w-full max-w-md mx-4" @click.stop>
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-border">
        <h2 class="text-lg font-semibold">{{ $t('hosts.pairHost') }}</h2>
        <button @click="close" class="text-text-muted hover:text-text-primary transition-colors">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="px-6 py-5 space-y-4">
        <p class="text-sm text-text-secondary">{{ $t('hosts.pairInstructionsDesc') }}</p>

        <!-- QR Scan Button -->
        <button
          @click="showQrScanner = true"
          class="w-full py-3 border-2 border-dashed border-border rounded-lg flex items-center justify-center gap-2 text-text-secondary hover:border-accent hover:text-accent transition-colors"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7" />
            <rect x="14" y="3" width="7" height="7" />
            <rect x="14" y="14" width="7" height="7" />
            <rect x="3" y="14" width="7" height="7" />
          </svg>
          {{ $t('hosts.scanPair') || 'Scan QR Code' }}
        </button>

        <div class="relative">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-border"></div>
          </div>
          <div class="relative flex justify-center text-sm">
            <span class="px-2 bg-bg-primary text-text-muted">OR</span>
          </div>
        </div>

        <div>
          <label class="block text-sm font-medium mb-2">{{ $t('hosts.pairCode') }}</label>
          <input
            v-model="pairCode"
            type="text"
            maxlength="6"
            placeholder="000000"
            class="w-full px-4 py-3 bg-bg-secondary border border-border rounded-lg text-center text-2xl font-mono tracking-widest focus:outline-none focus:border-accent transition-colors"
            @keyup.enter="pair"
          />
        </div>

        <p v-if="error" class="text-sm text-danger text-center">{{ error }}</p>
        <p v-if="success" class="text-sm text-success text-center">{{ $t('hosts.pairSuccess') }}</p>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-border">
        <button
          @click="close"
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          @click="pair"
          :disabled="!canSubmit || isPairing"
          class="px-4 py-2 bg-accent hover:bg-accent/90 text-white text-sm rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          <svg v-if="isPairing" class="animate-spin w-4 h-4" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ isPairing ? $t('common.pairing') || 'Pairing...' : $t('hosts.pairButton') }}
        </button>
      </div>
    </div>

    <!-- QR Scanner Modal (Placeholder) -->
    <div
      v-if="showQrScanner"
      class="fixed inset-0 z-[60] flex items-center justify-center bg-black/80"
      @click="showQrScanner = false"
    >
      <div class="bg-bg-primary rounded-xl border border-border shadow-lg w-full max-w-sm mx-4 p-6" @click.stop>
        <div class="text-center">
          <div class="w-16 h-16 mx-auto mb-4 rounded-full bg-accent-bg flex items-center justify-center">
            <svg class="w-8 h-8 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="3" y="3" width="7" height="7" />
              <rect x="14" y="3" width="7" height="7" />
              <rect x="14" y="14" width="7" height="7" />
              <rect x="3" y="14" width="7" height="7" />
            </svg>
          </div>
          <h3 class="text-lg font-semibold mb-2">QR Code Scanner</h3>
          <p class="text-sm text-text-secondary mb-4">
            QR code scanning will be available when running in the Tauri desktop app with camera access.
          </p>
          <div class="text-xs text-text-muted bg-bg-secondary rounded-lg p-3 mb-4">
            <p class="mb-1"><strong>Current workaround:</strong></p>
            <p>Please enter the pairing code manually above.</p>
          </div>
          <button
            @click="showQrScanner = false"
            class="px-4 py-2 bg-accent hover:bg-accent/90 text-white rounded-lg transition-colors"
          >
            Got it
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { authApi } from '@/api'
import { useHostStore } from '@/stores/hosts'

defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const hostStore = useHostStore()

// State
const pairCode = ref('')
const isPairing = ref(false)
const error = ref('')
const success = ref(false)
const showQrScanner = ref(false)

// Computed
const canSubmit = computed(() => pairCode.value.length === 6)

// Methods
function close() {
  emit('update:modelValue', false)
  resetForm()
}

function resetForm() {
  pairCode.value = ''
  error.value = ''
  success.value = false
}

async function pair() {
  if (!canSubmit.value) return

  isPairing.value = true
  error.value = ''
  success.value = false

  try {
    await authApi.completePair({ pair_code: pairCode.value })
    success.value = true
    // Refresh hosts list
    await hostStore.fetchHosts()
    // Close dialog after a delay
    setTimeout(() => {
      close()
    }, 1500)
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Pairing failed'
  } finally {
    isPairing.value = false
  }
}
</script>
