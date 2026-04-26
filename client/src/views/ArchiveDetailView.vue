<template>
  <div v-if="archive" class="h-full overflow-y-auto p-6">
    <div class="max-w-4xl mx-auto">
      <!-- Header -->
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-xl font-semibold">{{ archive.title }}</h1>
          <div class="flex items-center gap-2 text-sm text-text-muted mt-1">
            <span>{{ formatTime(archive.closed_at) }}</span>
            <span>•</span>
            <span :class="closeReasonClass">{{ closeReasonText }}</span>
          </div>
        </div>

        <button
          @click="deleteArchive"
          class="px-4 py-2 text-sm text-danger hover:bg-danger-bg rounded-lg transition-colors"
        >
          {{ $t('common.delete') }}
        </button>
      </div>

      <!-- Content Tabs -->
      <div class="bg-bg-secondary rounded-lg border border-border">
        <!-- Tab Headers -->
        <div class="flex border-b border-border">
          <button
            v-for="tab in tabs"
            :key="tab.value"
            @click="currentTab = tab.value"
            :class="[
              'px-6 py-3 text-sm font-medium border-b-2 transition-colors',
              currentTab === tab.value
                ? 'border-accent text-accent'
                : 'border-transparent text-text-secondary hover:text-text-primary'
            ]"
          >
            {{ tab.label }}
          </button>
        </div>

        <!-- Tab Content -->
        <div class="p-6">
          <!-- Overview Tab -->
          <div v-if="currentTab === 'overview'" class="space-y-4">
            <div class="grid grid-cols-2 gap-4 text-sm">
              <div>
                <span class="text-text-muted">Host:</span>
                <span class="ml-2">{{ archive.host_id }}</span>
              </div>
              <div>
                <span class="text-text-muted">Workspace:</span>
                <span class="ml-2">{{ archive.workspace_id }}</span>
              </div>
              <div>
                <span class="text-text-muted">Agent:</span>
                <span class="ml-2">{{ archive.agent_type }}</span>
              </div>
              <div>
                <span class="text-text-muted">Close Reason:</span>
                <span class="ml-2" :class="closeReasonClass">{{ closeReasonText }}</span>
              </div>
            </div>
          </div>

          <!-- Messages Tab -->
          <div v-else-if="currentTab === 'messages'" class="space-y-4">
            <div v-if="isLoading" class="flex items-center justify-center py-8 text-text-muted">
              <svg class="animate-spin w-6 h-6 mr-2" viewBox="0 0 24 24" fill="none">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ $t('common.loading') }}
            </div>
            <div v-else-if="messages.length === 0" class="text-center py-8 text-text-muted">
              {{ $t('common.noData') }}
            </div>
            <div v-else class="space-y-3">
              <div
                v-for="msg in messages"
                :key="msg.message_id"
                class="p-3 rounded-lg"
                :class="String(msg.message_type) === 'user_input' ? 'bg-accent-bg ml-8' : 'bg-bg-tertiary mr-8'"
              >
                <div class="text-xs text-text-muted mb-1">{{ msg.message_type }}</div>
                <div class="text-sm whitespace-pre-wrap">{{ msg.content }}</div>
                <div class="text-xs text-text-muted mt-1">{{ formatTime(msg.created_at) }}</div>
              </div>
            </div>
          </div>

          <!-- Logs Tab -->
          <div v-else-if="currentTab === 'logs'" class="space-y-4">
            <div v-if="isLoading" class="flex items-center justify-center py-8 text-text-muted">
              <svg class="animate-spin w-6 h-6 mr-2" viewBox="0 0 24 24" fill="none">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ $t('common.loading') }}
            </div>
            <div v-else-if="logs.length === 0" class="text-center py-8 text-text-muted">
              {{ $t('common.noData') }}
            </div>
            <div v-else class="font-mono text-sm space-y-1">
              <div
                v-for="(log, index) in logs"
                :key="index"
                class="py-1 px-2 rounded hover:bg-bg-tertiary"
              >
                <span class="text-text-muted">{{ formatTime(log.timestamp || log.created_at) }}</span>
                <span
                  class="ml-2 text-xs px-1.5 py-0.5 rounded"
                  :class="logLevelClass(log.level)"
                >
                  {{ log.level || 'INFO' }}
                </span>
                <span class="ml-2">{{ log.message || log.content }}</span>
              </div>
            </div>
          </div>

          <!-- Diff Tab -->
          <div v-else-if="currentTab === 'diff'" class="space-y-4">
            <div v-if="isLoading" class="flex items-center justify-center py-8 text-text-muted">
              <svg class="animate-spin w-6 h-6 mr-2" viewBox="0 0 24 24" fill="none">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              {{ $t('common.loading') }}
            </div>
            <div v-else-if="diffs.length === 0" class="text-center py-8 text-text-muted">
              {{ $t('common.noData') }}
            </div>
            <div v-else class="space-y-4">
              <div
                v-for="diff in diffs"
                :key="diff.path"
                class="border border-border rounded-lg overflow-hidden"
              >
                <div class="bg-bg-tertiary px-4 py-2 text-sm font-medium border-b border-border">
                  {{ diff.path || diff.name }}
                </div>
                <div class="font-mono text-xs overflow-x-auto">
                  <div
                    v-for="(line, idx) in diff.lines || diff.hunks"
                    :key="idx"
                    class="py-0.5 px-4"
                    :class="diffLineClass(line.type)"
                  >
                    <span class="text-text-muted select-none w-8 inline-block">
                      {{ line.oldLine || line.newLine || ' ' }}
                    </span>
                    <span
                      :class="{
                        'text-success': line.type === 'add',
                        'text-danger': line.type === 'remove'
                      }"
                    >
                      {{ line.content || line.line }}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div v-else class="flex items-center justify-center h-full text-text-muted">
    Select an archive to view details
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { archivesApi } from '@/api'
import { useArchiveStore } from '@/stores/archives'
import { useToastStore } from '@/stores/toast'
import type { SessionArchive, SessionMessage } from '@/types'

const route = useRoute()
const router = useRouter()
const archiveStore = useArchiveStore()
const toast = useToastStore()

const currentTab = ref('overview')
const isLoading = ref(false)
const messages = ref<SessionMessage[]>([])
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const logs = ref<any[]>([])
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const diffs = ref<any[]>([])

const tabs = [
  { value: 'overview', label: 'Overview' },
  { value: 'messages', label: 'Messages' },
  { value: 'logs', label: 'Logs' },
  { value: 'diff', label: 'Changes' },
]

const archive = computed(() => {
  const id = route.params.id as string
  return archiveStore.archives.find(
    (a: SessionArchive) => a.archive_id === id || a.session_id === id
  )
})

const closeReasonClass = computed(() => {
  if (!archive.value) return ''
  switch (archive.value.close_reason) {
    case 'completed': return 'text-success'
    case 'failed': return 'text-danger'
    case 'user_closed': return 'text-text-muted'
    case 'terminated': return 'text-warning'
    default: return 'text-text-muted'
  }
})

const closeReasonText = computed(() => {
  if (!archive.value) return ''
  switch (archive.value.close_reason) {
    case 'completed': return 'Completed'
    case 'failed': return 'Failed'
    case 'user_closed': return 'Closed'
    case 'terminated': return 'Terminated'
    default: return archive.value.close_reason
  }
})

function formatTime(time: string | undefined): string {
  if (!time) return 'N/A'
  return new Date(time).toLocaleString()
}

function logLevelClass(level: string): string {
  switch (level) {
    case 'ERROR': return 'bg-danger-bg text-danger'
    case 'WARN': return 'bg-warning-bg text-warning'
    case 'INFO': return 'bg-accent-bg text-accent'
    default: return 'bg-bg-tertiary text-text-muted'
  }
}

function diffLineClass(type: string): string {
  switch (type) {
    case 'add': return 'bg-success-bg/30'
    case 'remove': return 'bg-danger-bg/30'
    default: return ''
  }
}

async function loadMessages() {
  if (!archive.value) return
  isLoading.value = true
  try {
    const response = await archivesApi.getArchiveMessages(archive.value.archive_id)
    messages.value = response.items
  } catch (err) {
    toast.error('Failed to load messages', err instanceof Error ? err.message : undefined)
  } finally {
    isLoading.value = false
  }
}

async function loadLogs() {
  if (!archive.value) return
  isLoading.value = true
  try {
    const response = await archivesApi.getArchiveEvents(archive.value.archive_id)
    logs.value = response.items
  } catch (err) {
    toast.error('Failed to load logs', err instanceof Error ? err.message : undefined)
  } finally {
    isLoading.value = false
  }
}

async function loadDiffs() {
  if (!archive.value) return
  isLoading.value = true
  try {
    diffs.value = await archivesApi.getArchiveDiff(archive.value.archive_id)
  } catch (err) {
    toast.error('Failed to load diff', err instanceof Error ? err.message : undefined)
  } finally {
    isLoading.value = false
  }
}

async function deleteArchive() {
  if (!archive.value) return
  if (confirm('Are you sure you want to delete this archive?')) {
    await archiveStore.deleteArchives([archive.value.archive_id])
    router.push('/archives')
  }
}

// Watch for tab changes to load data
watch(currentTab, (tab) => {
  if (tab === 'messages' && messages.value.length === 0) {
    loadMessages()
  } else if (tab === 'logs' && logs.value.length === 0) {
    loadLogs()
  } else if (tab === 'diff' && diffs.value.length === 0) {
    loadDiffs()
  }
})

// Fetch archive data when route changes
watch(() => route.params.id, () => {
  if (archiveStore.archives.length === 0) {
    archiveStore.fetchArchives()
  }
}, { immediate: true })
</script>
