<template>
  <div class="bg-bg-primary rounded-lg border border-border p-4 hover:border-accent/50 transition-colors">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-3">
        <!-- Folder Icon -->
        <div class="w-10 h-10 rounded-lg bg-bg-secondary flex items-center justify-center">
          <svg class="w-5 h-5 text-text-secondary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
          </svg>
        </div>

        <div>
          <h4 class="font-medium text-sm">{{ workspace.display_name }}</h4>
          <p class="text-xs text-text-muted truncate max-w-[200px]">{{ workspace.path }}</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <!-- Favorite Button -->
        <button
          @click.stop="toggleFavorite"
          :class="[
            'p-2 rounded-lg transition-colors',
            workspace.is_favorited ? 'text-warning' : 'text-text-muted hover:text-warning'
          ]"
        >
          <svg class="w-4 h-4" viewBox="0 0 24 24" :fill="workspace.is_favorited ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
          </svg>
        </button>

        <!-- New Session Button -->
        <button
          @click.stop="$emit('newSession', workspace.workspace_id)"
          class="px-3 py-1.5 text-xs bg-accent hover:bg-accent/90 text-white rounded-md transition-colors"
        >
          {{ $t('hosts.newSessionFromWorkspace') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Workspace } from '@/types'

const props = defineProps<{
  workspace: Workspace
}>()

const emit = defineEmits<{
  toggleFavorite: [workspaceId: string, isFavorited: boolean]
  newSession: [workspaceId: string]
}>()

function toggleFavorite() {
  emit('toggleFavorite', props.workspace.workspace_id, !props.workspace.is_favorited)
}
</script>
