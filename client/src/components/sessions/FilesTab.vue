<template>
  <div class="flex h-full">
    <!-- File Tree -->
    <div class="w-64 border-r border-border flex flex-col">
      <div class="p-3 border-b border-border">
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="$t('sessionDetail.fileSearch')"
          class="w-full px-3 py-1.5 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent"
        />
      </div>

      <div class="flex-1 overflow-y-auto p-2">
        <div v-if="isLoading" class="flex items-center justify-center py-4 text-text-muted">
          <svg class="animate-spin w-5 h-5 mr-2" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ $t('common.loading') }}
        </div>
        <FileTreeNode
          v-else-if="fileTree"
          :node="fileTree"
          @select="selectFile"
        />
        <div v-else class="text-center py-4 text-text-muted">
          {{ $t('common.noData') }}
        </div>
      </div>
    </div>

    <!-- File Preview -->
    <div class="flex-1 overflow-auto">
      <div v-if="!selectedFile" class="flex flex-col items-center justify-center h-full text-text-muted">
        <svg class="w-16 h-16 mb-4 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
        </svg>
        <p>{{ $t('common.selectFile') || 'Select a file to preview' }}</p>
      </div>

      <div v-else-if="fileContent" class="h-full">
        <!-- Text File -->
        <pre v-if="isTextFile" class="p-4 font-mono text-sm whitespace-pre-wrap">{{ fileContent }}</pre>

        <!-- Binary File -->
        <div v-else class="flex flex-col items-center justify-center h-full text-text-muted">
          <svg class="w-16 h-16 mb-4 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 2 14 8 20 8"/>
          </svg>
          <p>{{ $t('common.binaryFile') || 'Binary file' }}</p>
          <p class="text-sm">{{ selectedFile }}</p>
        </div>
      </div>

      <div v-else class="flex items-center justify-center h-full text-text-muted">
        {{ $t('common.loading') }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { sessionsApi } from '@/api'
import { useToastStore } from '@/stores/toast'
import FileTreeNode from './FileTreeNode.vue'
import type { FileTreeNode as FileTreeNodeType, FileContent } from '@/types'

const props = defineProps<{
  sessionId: string
}>()

const toast = useToastStore()

const searchQuery = ref('')
const selectedFile = ref('')
const fileTree = ref<FileTreeNodeType | null>(null)
const fileContent = ref('')
const isTextFile = ref(true)
const isLoading = ref(false)

async function loadFileTree() {
  isLoading.value = true
  try {
    const tree = await sessionsApi.getFileTree(props.sessionId)
    fileTree.value = tree
  } catch (err) {
    toast.error('Failed to load file tree', err instanceof Error ? err.message : undefined)
  } finally {
    isLoading.value = false
  }
}

async function selectFile(path: string) {
  selectedFile.value = path
  fileContent.value = ''

  try {
    const content: FileContent = await sessionsApi.getFileContent(props.sessionId, path)
    isTextFile.value = content.file_type === 'text'
    fileContent.value = content.content || ''
  } catch (err) {
    toast.error('Failed to load file content', err instanceof Error ? err.message : undefined)
  }
}

onMounted(() => {
  loadFileTree()
})
</script>
