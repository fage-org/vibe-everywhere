<template>
  <div class="select-none">
    <div
      class="flex items-center gap-1 py-1 px-1 rounded cursor-pointer hover:bg-bg-secondary"
      :style="{ paddingLeft: `${level * 12 + 4}px` }"
      @click="handleClick"
    >
      <!-- Expand/Collapse Icon for directories -->
      <span v-if="node.is_dir" class="w-4 h-4 flex items-center justify-center">
        <svg
          v-if="isExpanded"
          class="w-3 h-3 text-text-muted"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
        <svg
          v-else
          class="w-3 h-3 text-text-muted"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </span>
      <span v-else class="w-4" />

      <!-- File/Folder Icon -->
      <svg
        v-if="node.is_dir"
        class="w-4 h-4 text-accent shrink-0"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
      </svg>

      <svg
        v-else
        class="w-4 h-4 text-text-muted shrink-0"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
        <polyline points="14 2 14 8 20 8" />
      </svg>

      <!-- Name -->
      <span class="text-sm truncate ml-1" :class="{ 'font-medium': node.is_dir }">{{ node.name }}</span>
    </div>

    <!-- Children -->
    <div v-if="node.is_dir && isExpanded && node.children">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :level="level + 1"
        @select="$emit('select', $event)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { FileTreeNode } from '@/types'

const props = withDefaults(defineProps<{
  node: FileTreeNode
  level?: number
}>(), {
  level: 0
})

const emit = defineEmits<{
  select: [path: string]
}>()

const isExpanded = ref(true)

function handleClick() {
  if (props.node.is_dir) {
    isExpanded.value = !isExpanded.value
  } else {
    emit('select', props.node.path)
  }
}
</script>
