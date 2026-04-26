<template>
  <div class="flex h-full">
    <!-- Archive List -->
    <div class="w-80 min-w-80 border-r border-border flex flex-col bg-bg-secondary">
      <div class="p-4 border-b border-border">
        <h2 class="font-semibold mb-3">{{ $t('archives.title') }}</h2>
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="$t('archives.searchPlaceholder')"
          class="w-full px-3 py-2 bg-bg-primary border border-border rounded-lg text-sm focus:outline-none focus:border-accent"
        />
      </div>

      <div class="flex-1 overflow-y-auto">
        <div v-if="archiveStore.isLoading" class="flex items-center justify-center h-32">
          <svg class="animate-spin w-5 h-5 text-accent" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
        </div>

        <div v-else-if="archiveStore.archives.length === 0" class="flex flex-col items-center justify-center h-32 text-text-muted">
          <p class="text-sm">{{ $t('common.noData') }}</p>
        </div>

        <div v-else>
          <div
            v-for="archive in filteredArchives"
            :key="archive.archive_id"
            :class="[
              'px-4 py-3 border-b border-border cursor-pointer transition-colors',
              archiveStore.selectedArchiveId === archive.archive_id ? 'bg-accent-bg' : 'hover:bg-bg-hover'
            ]"
            @click="selectArchive(archive.archive_id)"
          >
            <h3 class="font-medium text-sm mb-1">{{ archive.title }}</h3>
            <div class="flex items-center gap-2 text-xs text-text-muted">
              <span>{{ formatTime(archive.closed_at) }}</span>
              <span>•</span>
              <span :class="closeReasonClass(archive.close_reason)">
                {{ closeReasonText(archive.close_reason) }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Detail Area -->
    <div class="flex-1 bg-bg-primary">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useArchiveStore } from '@/stores/archives'
import type { CloseReason, SessionArchive } from '@/types'

const router = useRouter()
const archiveStore = useArchiveStore()

const searchQuery = ref('')

const filteredArchives = computed(() => {
  if (!searchQuery.value) return archiveStore.archives
  const query = searchQuery.value.toLowerCase()
  return archiveStore.archives.filter((a: SessionArchive) =>
    a.title.toLowerCase().includes(query)
  )
})

function closeReasonClass(reason: CloseReason): string {
  switch (reason) {
    case 'completed': return 'text-success'
    case 'failed': return 'text-danger'
    case 'user_closed': return 'text-text-muted'
    case 'terminated': return 'text-warning'
    default: return 'text-text-muted'
  }
}

function closeReasonText(reason: CloseReason): string {
  switch (reason) {
    case 'completed': return 'Completed'
    case 'failed': return 'Failed'
    case 'user_closed': return 'Closed'
    case 'terminated': return 'Terminated'
    default: return reason
  }
}

function formatTime(time: string): string {
  return new Date(time).toLocaleDateString()
}

function selectArchive(archiveId: string) {
  archiveStore.selectArchive(archiveId)
  router.push(`/archives/${archiveId}`)
}

onMounted(() => {
  archiveStore.fetchArchives()
})
</script>
