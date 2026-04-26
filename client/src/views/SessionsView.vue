<template>
  <div class="flex h-full">
    <!-- Session List -->
    <div class="w-80 min-w-80 border-r border-border flex flex-col bg-bg-secondary">
      <!-- Header -->
      <div class="p-4 border-b border-border">
        <div class="flex items-center justify-between mb-3">
          <h2 class="font-semibold">{{ $t('sessions.title') }}</h2>
          <button
            @click="showNewSession = true"
            class="px-3 py-1.5 bg-accent hover:bg-accent/90 text-white text-sm rounded-md transition-colors flex items-center gap-1.5"
          >
            <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"/>
              <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
            {{ $t('sessions.newSession') }}
          </button>
        </div>

        <!-- Search -->
        <div class="relative">
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
          </svg>
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="$t('sessions.searchPlaceholder')"
            class="w-full pl-9 pr-3 py-2 bg-bg-primary border border-border rounded-lg text-sm focus:outline-none focus:border-accent transition-colors"
          />
        </div>
      </div>

      <!-- Filters -->
      <div class="flex gap-1 p-2 border-b border-border">
        <button
          v-for="filter in filters"
          :key="filter.value"
          @click="currentFilter = filter.value"
          :class="[
            'px-3 py-1 text-xs rounded-md transition-colors',
            currentFilter === filter.value
              ? 'bg-accent text-white'
              : 'bg-bg-primary hover:bg-bg-hover text-text-secondary'
          ]"
        >
          {{ filter.label }}
        </button>
      </div>

      <!-- Session List Content -->
      <div class="flex-1 overflow-y-auto">
        <!-- Loading -->
        <div v-if="sessionStore.isLoading" class="flex items-center justify-center h-32">
          <svg class="animate-spin w-5 h-5 text-accent" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>

        <!-- Empty -->
        <div v-else-if="filteredSessions.length === 0" class="flex flex-col items-center justify-center h-32 text-text-muted">
          <svg class="w-10 h-10 mb-2 opacity-50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="3" y="3" width="18" height="18" rx="2"/>
            <path d="M9 9h6v6H9z"/>
          </svg>
          <p class="text-sm">{{ $t('common.noData') }}</p>
        </div>

        <!-- Session Groups -->
        <template v-else>
          <!-- Attention Group -->
          <div v-if="attentionSessions.length > 0" class="py-2">
            <div class="px-4 py-1 text-xs font-medium text-danger">{{ $t('sessions.groupAttention') }}</div>
            <SessionCard
              v-for="session in attentionSessions"
              :key="session.session_id"
              :session="session"
              :is-active="sessionStore.activeSessionId === session.session_id"
              @click="selectSession(session.session_id)"
            />
          </div>

          <!-- Running Group -->
          <div v-if="runningSessions.length > 0" class="py-2">
            <div class="px-4 py-1 text-xs font-medium text-success">{{ $t('sessions.groupRunning') }}</div>
            <SessionCard
              v-for="session in runningSessions"
              :key="session.session_id"
              :session="session"
              :is-active="sessionStore.activeSessionId === session.session_id"
              @click="selectSession(session.session_id)"
            />
          </div>

          <!-- Paused Group -->
          <div v-if="pausedSessions.length > 0" class="py-2">
            <div class="px-4 py-1 text-xs font-medium text-text-muted">{{ $t('sessions.groupPaused') }}</div>
            <SessionCard
              v-for="session in pausedSessions"
              :key="session.session_id"
              :session="session"
              :is-active="sessionStore.activeSessionId === session.session_id"
              @click="selectSession(session.session_id)"
            />
          </div>
        </template>
      </div>
    </div>

    <!-- Detail Area -->
    <div class="flex-1 bg-bg-primary">
      <router-view v-slot="{ Component }">
        <component :is="Component" v-if="Component" />
        <div v-else class="flex flex-col items-center justify-center h-full text-text-muted">
          <svg class="w-16 h-16 mb-4 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
          </svg>
          <p class="text-sm">{{ $t('common.selectSession') || 'Select a session to view details' }}</p>
        </div>
      </router-view>
    </div>
  </div>

  <!-- New Session Dialog -->
  <NewSessionDialog v-model="showNewSession" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionStore } from '@/stores/sessions'
import { useI18n } from 'vue-i18n'
import SessionCard from '@/components/sessions/SessionCard.vue'
import NewSessionDialog from '@/components/sessions/NewSessionDialog.vue'

const router = useRouter()
const sessionStore = useSessionStore()
const { t } = useI18n()

// State
const searchQuery = ref('')
const currentFilter = ref<'all' | 'attention' | 'running' | 'paused'>('all')
const showNewSession = ref(false)

// Filters
const filters: { value: 'all' | 'attention' | 'running' | 'paused', label: string }[] = [
  { value: 'all', label: t('sessions.filterAll') },
  { value: 'attention', label: t('sessions.filterAttention') },
  { value: 'running', label: t('sessions.filterRunning') },
  { value: 'paused', label: t('sessions.filterPaused') },
]

// Computed
const filteredSessions = computed(() => {
  let sessions = sessionStore.sessions

  // Apply status filter
  if (currentFilter.value === 'attention') {
    sessions = sessionStore.attentionSessions
  } else if (currentFilter.value === 'running') {
    sessions = sessionStore.runningSessions
  } else if (currentFilter.value === 'paused') {
    sessions = sessionStore.pausedSessions
  }

  // Apply search filter
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    sessions = sessions.filter(s =>
      s.title.toLowerCase().includes(query) ||
      s.host_id.toLowerCase().includes(query)
    )
  }

  return sessions
})

const attentionSessions = computed(() =>
  filteredSessions.value.filter(s => s.pending_permission_count > 0 || s.status === 'error')
)

const runningSessions = computed(() =>
  filteredSessions.value.filter(s => s.status === 'running' && s.pending_permission_count === 0)
)

const pausedSessions = computed(() =>
  filteredSessions.value.filter(s => s.status === 'paused')
)

// Methods
function selectSession(sessionId: string) {
  sessionStore.selectSession(sessionId)
  router.push(`/sessions/${sessionId}`)
}

// Lifecycle
onMounted(() => {
  sessionStore.fetchSessions()
})
</script>
