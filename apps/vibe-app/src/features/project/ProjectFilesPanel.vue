<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { WorkspaceBrowseResponse, WorkspaceFilePreviewResponse } from "@/types";

defineProps<{
  workspace: WorkspaceBrowseResponse | null;
  filePreview: WorkspaceFilePreviewResponse | null;
}>();

const emit = defineEmits<{
  openEntry: [path: string, kind: "directory" | "file"];
}>();

const { t } = useI18n();
</script>

<template>
  <div class="grid gap-5 xl:grid-cols-[320px_minmax(0,1fr)]">
    <aside
      class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
    >
      <h3 class="text-sm font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ t("files.title") }}</h3>
      <div class="mt-4 space-y-2">
        <button
          v-for="entry in workspace?.entries ?? []"
          :key="entry.path"
          class="flex w-full items-center justify-between rounded-2xl border border-border bg-background/75 px-3 py-3 text-left"
          @click="emit('openEntry', entry.path, entry.kind)"
        >
          <span class="truncate text-sm">{{ entry.name }}</span>
          <span class="text-xs text-muted-foreground">{{ entry.kind }}</span>
        </button>
      </div>
    </aside>

    <section
      class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
    >
      <div v-if="filePreview" class="space-y-3">
        <div>
          <p class="text-sm font-semibold">{{ filePreview.path }}</p>
          <p class="text-xs text-muted-foreground">
            {{ filePreview.sizeBytes }} bytes
            <span v-if="filePreview.truncated">· {{ t("files.truncated") }}</span>
          </p>
        </div>
        <pre class="overflow-x-auto rounded-2xl bg-background p-4 text-xs leading-6">{{ filePreview.content }}</pre>
      </div>
      <div v-else class="text-sm text-muted-foreground">{{ t("files.empty") }}</div>
    </section>
  </div>
</template>
