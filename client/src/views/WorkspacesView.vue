<template>
  <div class="flex flex-col h-full bg-bg-secondary">
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-border bg-bg-primary">
      <div class="flex items-center gap-4">
        <button @click="back" class="text-text-muted hover:text-text-primary transition-colors">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="m15 18-6-6 6-6"/>
          </svg>
        </button>
        <div v-if="host">
          <h2 class="font-semibold">{{ host.host_name }}</h2>
          <p class="text-xs text-text-secondary">{{ host.platform }} • {{ host.online_status }}</p>
        </div>
      </div>

      <button
        @click="showNewSession = true"
        class="px-3 py-1.5 bg-accent hover:bg-accent/90 text-white text-sm rounded-md transition-colors"
      >
        {{ $t('sessions.newSession') }}
      </button>
    </div>

    <!-- Workspaces List -->
    <div class="flex-1 overflow-y-auto p-4">
      <!-- Loading -->
      <div v-if="workspaceStore.isLoading" class="flex items-center justify-center h-32">
        <svg class="animate-spin w-6 h-6 text-accent" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      </div>

      <!-- Empty -->
      <div v-else-if="workspaceStore.workspaces.length === 0" class="flex flex-col items-center justify-center h-64 text-text-muted">
        <svg class="w-16 h-16 mb-4 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        <p class="text-sm">{{ $t('common.noData') }}</p>
      </div>

      <!-- Favorites Section -->
      <template v-else>
        <div v-if="workspaceStore.favoriteWorkspaces.length > 0" class="mb-6">
          <h3 class="text-sm font-medium text-text-secondary mb-3">{{ $t('hosts.favoriteWorkspaces') }}</h3>
          <WorkspaceItem
            v-for="workspace in workspaceStore.favoriteWorkspaces"
            :key="workspace.workspace_id"
            :workspace="workspace"
            @toggle-favorite="toggleFavorite"
            @new-session="createSession"
          />
        </div>

        <!-- Other Workspaces -->
        <div>
          <h3 class="text-sm font-medium text-text-secondary mb-3">{{ $t('hosts.otherWorkspaces') }}</h3>
          <WorkspaceItem
            v-for="workspace in workspaceStore.normalWorkspaces"
            :key="workspace.workspace_id"
            :workspace="workspace"
            @toggle-favorite="toggleFavorite"
            @new-session="createSession"
          />
        </div>
      </template>
    </div>

    <!-- New Session Dialog -->
    <NewSessionDialog
      v-model="showNewSession"
      :preselected-host-id="hostId"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useHostStore } from '@/stores/hosts'
import { useWorkspaceStore } from '@/stores/workspaces'
import type { Host } from '@/types'
import WorkspaceItem from '@/components/workspaces/WorkspaceItem.vue'
import NewSessionDialog from '@/components/sessions/NewSessionDialog.vue'

const router = useRouter()
const route = useRoute()
const hostStore = useHostStore()
const workspaceStore = useWorkspaceStore()

// State
const showNewSession = ref(false)

// Computed
const hostId = computed(() => route.params.id as string)
const host = computed(() => hostStore.hosts.find((h: Host) => h.host_id === hostId.value))

// Methods
function back() {
  router.push('/hosts')
}

async function toggleFavorite(workspaceId: string, isFavorited: boolean) {
  await workspaceStore.toggleFavorite(workspaceId, isFavorited)
}

function createSession(_workspaceId: string) {
  // TODO: Open new session dialog with preselected workspace
  showNewSession.value = true
}

// Lifecycle
onMounted(async () => {
  if (hostStore.hosts.length === 0) {
    await hostStore.fetchHosts()
  }
  await workspaceStore.fetchWorkspaces(hostId.value)
})
</script>
