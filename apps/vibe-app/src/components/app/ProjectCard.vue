<script setup lang="ts">
import { useI18n } from "vue-i18n";
import type { ProjectSummary } from "@/lib/projects";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatRelativeTime } from "@/lib/format";

defineProps<{
  project: ProjectSummary;
  compact?: boolean;
}>();

const { t } = useI18n();

function inventoryTone(state: ProjectSummary["availabilityState"]) {
  if (state === "offline") {
    return "muted" as const;
  }
  if (state === "unreachable" || state === "history_only") {
    return "warning" as const;
  }
  return "success" as const;
}
</script>

<template>
  <article
    class="rounded-[1.4rem] border border-white/55 bg-white/80 p-4 shadow-[0_18px_50px_-28px_rgba(15,23,42,0.45)] backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
  >
    <div class="flex items-start justify-between gap-3">
      <div class="min-w-0">
        <p class="truncate text-sm font-semibold text-foreground">{{ project.title }}</p>
        <p class="mt-1 truncate text-xs text-muted-foreground">{{ project.deviceName }}</p>
      </div>
      <StatusBadge
        :tone="
          project.failedTaskCount > 0
            ? 'danger'
            : project.waitingInputCount > 0
              ? 'warning'
              : project.runningTaskCount > 0
                ? 'default'
                : 'success'
        "
      >
        {{
          project.failedTaskCount > 0
            ? t("projectCard.failed")
            : project.waitingInputCount > 0
              ? t("projectCard.waiting")
              : project.runningTaskCount > 0
                ? t("projectCard.running")
                : t("projectCard.ready")
        }}
      </StatusBadge>
    </div>
    <p class="mt-3 line-clamp-2 text-xs text-muted-foreground">{{ project.pathLabel }}</p>
    <div class="mt-3 flex flex-wrap items-center gap-2 text-[11px] text-muted-foreground">
      <span v-if="project.branchName" class="rounded-full bg-background px-2 py-1">
        {{ project.branchName }}
      </span>
      <span v-if="project.changedFilesCount > 0" class="rounded-full bg-background px-2 py-1">
        {{ t("projectCard.changedFiles", { count: project.changedFilesCount }) }}
      </span>
      <StatusBadge :tone="inventoryTone(project.availabilityState)">
        {{ t(`projectCard.availability.${project.availabilityState}`) }}
      </StatusBadge>
      <span
        v-if="project.discoverySource"
        class="rounded-full bg-background px-2 py-1"
      >
        {{ t(`projectCard.discovery.${project.discoverySource}`) }}
      </span>
    </div>
    <div
      class="mt-4 grid gap-2 text-[11px] text-muted-foreground"
      :class="compact ? 'grid-cols-2' : 'grid-cols-4'"
    >
      <div>
        <p class="font-medium text-foreground">{{ project.conversationCount }}</p>
        <p>{{ t("projectCard.topics") }}</p>
      </div>
      <div>
        <p class="font-medium text-foreground">{{ project.runningTaskCount }}</p>
        <p>{{ t("projectCard.running") }}</p>
      </div>
      <div>
        <p class="font-medium text-foreground">{{ project.waitingInputCount }}</p>
        <p>{{ t("projectCard.waiting") }}</p>
      </div>
      <div>
        <p class="font-medium text-foreground">{{ formatRelativeTime(project.updatedAtEpochMs) }}</p>
        <p>{{ t("projectCard.updated") }}</p>
      </div>
    </div>
  </article>
</template>
