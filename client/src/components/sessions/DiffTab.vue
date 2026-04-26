<template>
  <div class="flex h-full">
    <!-- File List -->
    <div class="w-64 border-r border-border flex flex-col">
      <div class="p-3 border-b border-border">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search files..."
          class="w-full px-3 py-1.5 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent"
        />
      </div>

      <div class="flex-1 overflow-y-auto">
        <div v-if="isLoading" class="flex items-center justify-center py-4 text-text-muted">
          <svg class="animate-spin w-5 h-5 mr-2" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ $t('common.loading') }}
        </div>

        <div
          v-for="file in filteredFiles"
          :key="file.path"
          :class="[
            'px-3 py-2 text-sm cursor-pointer border-b border-border',
            selectedFile === file.path ? 'bg-accent-bg text-accent' : 'hover:bg-bg-secondary'
          ]"
          @click="selectFile(file.path)"
        >
          <div class="flex items-center justify-between">
            <span class="truncate">{{ file.name }}</span>
            <span class="text-xs text-text-muted">+{{ file.additions }} -{{ file.deletions }}</span>
          </div>
        </div>

        <div v-if="!isLoading && files.length === 0" class="text-center py-4 text-text-muted">
          {{ $t('common.noData') }}
        </div>
      </div>
    </div>

    <!-- Diff Viewer -->
    <div class="flex-1 overflow-auto p-4">
      <div v-if="!selectedFile" class="flex items-center justify-center h-full text-text-muted">
        Select a file to view diff
      </div>

      <div v-else-if="isLoadingDiff" class="flex items-center justify-center h-full text-text-muted">
        <svg class="animate-spin w-6 h-6 mr-2" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Loading diff...
      </div>

      <div v-else-if="currentDiff" class="font-mono text-sm">
        <div class="mb-4 pb-4 border-b border-border">
          <div class="text-text-muted mb-2">{{ selectedFile }}</div>
          <div class="flex gap-4 text-xs">
            <span class="text-success">+{{ currentDiff.stats.additions }} additions</span>
            <span class="text-danger">-{{ currentDiff.stats.deletions }} deletions</span>
          </div>
        </div>

        <div class="space-y-1">
          <div
            v-for="(line, index) in currentDiff.lines"
            :key="index"
            :class="[
              'flex',
              line.type === 'add' ? 'bg-success-bg/50' : '',
              line.type === 'remove' ? 'bg-danger-bg/50' : ''
            ]"
          >
            <span class="w-12 text-right text-text-muted select-none shrink-0 pr-3">
              {{ line.oldLine || ' ' }}
            </span>
            <span class="w-12 text-right text-text-muted select-none shrink-0 pr-3">
              {{ line.newLine || ' ' }}
            </span>
            <span
              :class="[
                'pl-3 whitespace-pre',
                line.type === 'add' ? 'text-success' : '',
                line.type === 'remove' ? 'text-danger' : ''
              ]"
            >
              {{ line.content }}
            </span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { sessionsApi, type DiffFile, type DiffData } from '@/api/sessions'
import { useToastStore } from '@/stores/toast'

const props = defineProps<{
  sessionId: string
}>()

const toast = useToastStore()

const searchQuery = ref('')
const selectedFile = ref('')
const files = ref<DiffFile[]>([])
const diffs = ref<Map<string, DiffData>>(new Map())
const isLoading = ref(false)
const isLoadingDiff = ref(false)

const filteredFiles = computed(() => {
  if (!searchQuery.value) return files.value
  const query = searchQuery.value.toLowerCase()
  return files.value.filter(f => f.path.toLowerCase().includes(query))
})

const currentDiff = computed(() => {
  if (!selectedFile.value) return null
  return diffs.value.get(selectedFile.value) || null
})

async function loadDiffList() {
  isLoading.value = true
  try {
    const diffFiles = await sessionsApi.getDiff(props.sessionId)
    files.value = diffFiles
  } catch (err) {
    toast.error('Failed to load diff list', err instanceof Error ? err.message : undefined)
  } finally {
    isLoading.value = false
  }
}

async function selectFile(path: string) {
  selectedFile.value = path

  // Load diff if not cached
  if (!diffs.value.has(path)) {
    isLoadingDiff.value = true
    try {
      const diffData = await sessionsApi.getFileDiff(props.sessionId, path)
      diffs.value.set(path, diffData)
    } catch (err) {
      toast.error('Failed to load diff', err instanceof Error ? err.message : undefined)
    } finally {
      isLoadingDiff.value = false
    }
  }
}

onMounted(() => {
  loadDiffList()
})
</script>
