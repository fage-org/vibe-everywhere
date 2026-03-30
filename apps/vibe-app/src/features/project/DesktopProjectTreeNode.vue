<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { RouterLink } from "vue-router";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { buildProjectRouteParam, type ProjectTreeNode } from "@/lib/projects";

const props = defineProps<{
  node: ProjectTreeNode;
  currentProjectKey?: string | null;
  depth?: number;
}>();

const depth = computed(() => props.depth ?? 0);
const { t } = useI18n();

function availabilityTone(state: NonNullable<ProjectTreeNode["project"]>["availabilityState"]) {
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
  <div class="space-y-2">
    <div
      v-if="node.kind === 'group'"
      class="rounded-2xl border border-border/70 bg-background/55 px-3 py-2"
      :style="{ marginLeft: `${depth * 10}px` }"
    >
      <p class="text-xs font-semibold uppercase tracking-[0.16em] text-muted-foreground">
        {{ node.label }}
      </p>
    </div>

    <RouterLink
      v-else-if="node.project"
      :to="{
        name: 'project-workspace',
        params: {
          deviceId: node.project.deviceId,
          projectPath: buildProjectRouteParam(node.project.cwd)
        }
      }"
      class="block rounded-2xl border px-3 py-3 text-sm"
      :class="
        node.project.key === currentProjectKey
          ? 'border-primary bg-primary/10'
          : 'border-border bg-background/75'
      "
      :style="{ marginLeft: `${depth * 10}px` }"
    >
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0">
          <p class="truncate font-medium text-foreground">{{ node.project.title }}</p>
          <p class="mt-1 truncate text-xs text-muted-foreground">{{ node.project.pathLabel }}</p>
          <div class="mt-2 flex flex-wrap gap-2">
            <StatusBadge :tone="availabilityTone(node.project.availabilityState)">
              {{ t(`projectCard.availability.${node.project.availabilityState}`) }}
            </StatusBadge>
          </div>
        </div>
        <StatusBadge
          v-if="node.project.failedTaskCount || node.project.waitingInputCount || node.project.runningTaskCount"
          :tone="
            node.project.failedTaskCount
              ? 'danger'
              : node.project.waitingInputCount
                ? 'warning'
                : 'default'
          "
        >
          {{
            node.project.failedTaskCount
              ? node.project.failedTaskCount
              : node.project.waitingInputCount
                ? node.project.waitingInputCount
                : node.project.runningTaskCount
          }}
        </StatusBadge>
      </div>
    </RouterLink>

    <div v-if="node.children.length" class="space-y-2">
      <DesktopProjectTreeNode
        v-for="child in node.children"
        :key="child.id"
        :node="child"
        :current-project-key="currentProjectKey"
        :depth="depth + 1"
      />
    </div>
  </div>
</template>
