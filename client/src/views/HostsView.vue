<template>
  <div class="flex flex-col h-full bg-bg-secondary">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-border bg-bg-primary">
      <div class="flex items-center gap-4">
        <h2 class="font-semibold">{{ $t('hosts.title') }}</h2>
      </div>
      <button
        @click="showPairDialog = true"
        class="px-3 py-1.5 bg-accent hover:bg-accent/90 text-white text-sm rounded-md transition-colors flex items-center gap-1.5"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"/>
          <line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        {{ $t('hosts.pair') }}
      </button>
    </div>

    <!-- Host List -->
    <div class="flex-1 overflow-y-auto p-4">
      <!-- Loading -->
      <div v-if="hostStore.isLoading" class="flex items-center justify-center h-32">
        <svg class="animate-spin w-6 h-6 text-accent" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>

      <!-- Empty -->
      <div v-else-if="hostStore.hosts.length === 0" class="flex flex-col items-center justify-center h-64 text-text-muted">
        <svg class="w-16 h-16 mb-4 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <rect x="2" y="3" width="20" height="14" rx="2"/>
          <line x1="8" y1="21" x2="16" y2="21"/>
          <line x1="12" y1="17" x2="12" y2="21"/>
        </svg>
        <p class="text-sm mb-4">{{ $t('common.noData') }}</p>
        <button
          @click="showPairDialog = true"
          class="px-4 py-2 bg-accent hover:bg-accent/90 text-white text-sm rounded-lg transition-colors"
        >
          {{ $t('hosts.pairHost') }}
        </button>
      </div>

      <!-- Host Cards -->
      <div v-else class="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <div
          v-for="host in hostStore.hosts"
          :key="host.host_id"
          class="bg-bg-primary rounded-lg border border-border p-4 hover:border-accent/50 transition-colors cursor-pointer"
          @click="viewWorkspaces(host.host_id)"
        >
          <div class="flex items-start justify-between mb-3">
            <div class="flex items-center gap-3">
              <!-- Platform Icon -->
              <div class="w-10 h-10 rounded-lg bg-bg-secondary flex items-center justify-center">
                <svg class="w-5 h-5 text-text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="2" y="3" width="20" height="14" rx="2"/>
                  <line x1="8" y1="21" x2="16" y2="21"/>
                  <line x1="12" y1="17" x2="12" y2="21"/>
                </svg>
              </div>
              <div>
                <h3 class="font-medium">{{ host.host_name }}</h3>
                <p class="text-xs text-text-muted">{{ host.platform }}</p>
              </div>
            </div>

            <!-- Status Badge -->
            <span
              :class="[
                'text-[10px] px-2 py-0.5 rounded-full',
                host.online_status === 'online' ? 'bg-success-bg text-success' : 'bg-bg-tertiary text-text-muted'
              ]"
            >
              {{ host.online_status === 'online' ? $t('status.online') : $t('status.offline') }}
            </span>
          </div>

          <div class="flex items-center justify-between text-sm">
            <span class="text-text-secondary">
              {{ host.workspace_count || 0 }} {{ $t('hosts.workspaces') }}
            </span>
            <span
              :class="[
                'text-[10px] px-2 py-0.5 rounded-full',
                daemonStatusClass(host.daemon_status)
              ]"
            >
              {{ daemonStatusText(host.daemon_status) }}
            </span>
          </div>

          <!-- Last Active -->
          <p v-if="host.last_active_at" class="text-xs text-text-muted mt-2">
            {{ $t('hosts.lastActive') }}: {{ formatTime(host.last_active_at) }}
          </p>
        </div>
      </div>
    </div>

    <!-- Pair Host Dialog -->
    <PairHostDialog v-model="showPairDialog" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useHostStore } from '@/stores/hosts'
import type { DaemonStatus } from '@/types'
import PairHostDialog from '@/components/hosts/PairHostDialog.vue'

const router = useRouter()
const hostStore = useHostStore()

// State
const showPairDialog = ref(false)

// Methods
function viewWorkspaces(hostId: string) {
  router.push(`/hosts/${hostId}/workspaces`)
}

function daemonStatusClass(status: DaemonStatus): string {
  switch (status) {
    case 'healthy': return 'bg-success-bg text-success'
    case 'connecting': return 'bg-warning-bg text-warning'
    case 'disconnected': return 'bg-bg-tertiary text-text-muted'
    case 'error': return 'bg-danger-bg text-danger'
    default: return 'bg-bg-tertiary text-text-muted'
  }
}

function daemonStatusText(status: DaemonStatus): string {
  switch (status) {
    case 'healthy': return 'Healthy'
    case 'connecting': return 'Connecting'
    case 'disconnected': return 'Disconnected'
    case 'error': return 'Error'
    default: return status
  }
}

function formatTime(time: string): string {
  const date = new Date(time)
  return date.toLocaleString()
}

// Lifecycle
onMounted(() => {
  hostStore.fetchHosts()
})
</script>
