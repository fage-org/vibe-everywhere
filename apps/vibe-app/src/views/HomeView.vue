<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import ProjectCard from "@/components/app/ProjectCard.vue";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatDuration, formatRelativeTime } from "@/lib/format";
import { buildProjectRouteParam } from "@/lib/projects";
import { useAppStore } from "@/stores/app";

const store = useAppStore();
const router = useRouter();
const { t } = useI18n();

const recentProject = computed(() => store.recentProjects[0] ?? store.projectSummaries[0] ?? null);
const runningTasks = computed(() => store.tasks.filter((task) => task.status === "running").slice(0, 4));
const reviewProjects = computed(() => store.reviewProjects.slice(0, 4));

function openProject(deviceId: string, cwd: string | null) {
  store.markProjectVisited(deviceId, cwd);
  void router.push({
    name: "project-workspace",
    params: {
      deviceId,
      projectPath: buildProjectRouteParam(cwd)
    }
  });
}
</script>

<template>
  <section class="space-y-5">
    <div class="grid gap-3 sm:grid-cols-3">
      <article
        class="rounded-[1.6rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
      >
        <p class="text-xs uppercase tracking-[0.24em] text-muted-foreground">{{ t("home.stats.onlineHosts") }}</p>
        <p class="mt-3 text-3xl font-semibold">{{ store.onlineHostCount }}</p>
      </article>
      <article
        class="rounded-[1.6rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
      >
        <p class="text-xs uppercase tracking-[0.24em] text-muted-foreground">{{ t("home.stats.runningTasks") }}</p>
        <p class="mt-3 text-3xl font-semibold">{{ store.runningTaskCount }}</p>
      </article>
      <article
        class="rounded-[1.6rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
      >
        <p class="text-xs uppercase tracking-[0.24em] text-muted-foreground">{{ t("home.stats.needsAttention") }}</p>
        <p class="mt-3 text-3xl font-semibold">{{ store.attentionCount }}</p>
      </article>
    </div>

    <article
      class="rounded-[1.8rem] border border-primary/20 bg-[linear-gradient(135deg,rgba(245,158,11,0.18),rgba(14,165,233,0.14))] p-5 shadow-[0_24px_70px_-35px_rgba(15,23,42,0.55)]"
    >
      <div class="flex items-start justify-between gap-4">
        <div class="space-y-2">
          <StatusBadge>{{ t("home.continueWorkBadge") }}</StatusBadge>
          <h2 class="text-xl font-semibold">
            {{ recentProject ? recentProject.title : t("home.continueWorkEmptyTitle") }}
          </h2>
          <p class="max-w-2xl text-sm text-muted-foreground">
            {{
              recentProject
                ? `${recentProject.deviceName} · ${recentProject.pathLabel}`
                : t("home.continueWorkEmptySummary")
            }}
          </p>
        </div>
        <button
          class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50"
          :disabled="!recentProject"
          @click="recentProject && openProject(recentProject.deviceId, recentProject.cwd)"
        >
          {{ t("common.openProject") }}
        </button>
      </div>
    </article>

    <div class="grid gap-5 xl:grid-cols-[1.1fr_0.9fr]">
      <section class="space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
            {{ t("home.runningNow") }}
          </h3>
          <span class="text-xs text-muted-foreground">{{ t("common.itemsCount", { count: runningTasks.length }) }}</span>
        </div>
        <div
          v-if="runningTasks.length"
          class="space-y-3 rounded-[1.6rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
        >
          <article
            v-for="task in runningTasks"
            :key="task.id"
            class="rounded-2xl border border-border/70 bg-background/75 p-4"
          >
            <div class="flex items-center justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-semibold">{{ task.title || task.prompt }}</p>
                <p class="truncate text-xs text-muted-foreground">
                  {{ store.devices.find((device) => device.id === task.deviceId)?.name ?? task.deviceId }}
                </p>
              </div>
              <StatusBadge>{{ t("projectCard.running") }}</StatusBadge>
            </div>
            <p class="mt-3 line-clamp-2 text-sm text-muted-foreground">{{ task.prompt }}</p>
            <div class="mt-3 flex items-center justify-between text-xs text-muted-foreground">
              <span>{{ formatDuration(task.startedAtEpochMs, task.finishedAtEpochMs) }}</span>
              <button class="font-medium text-primary" @click="openProject(task.deviceId, task.cwd)">
                {{ t("common.viewDetails") }}
              </button>
            </div>
          </article>
        </div>
        <div
          v-else
          class="rounded-[1.6rem] border border-dashed border-border bg-background/65 p-6 text-sm text-muted-foreground"
        >
          {{ t("home.noRunningTasks") }}
        </div>
      </section>

      <section class="space-y-3">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
            {{ t("home.needsReview") }}
          </h3>
          <span class="text-xs text-muted-foreground">{{ t("common.projectsCount", { count: reviewProjects.length }) }}</span>
        </div>
        <div v-if="reviewProjects.length" class="space-y-3">
          <button
            v-for="project in reviewProjects"
            :key="project.key"
            class="w-full text-left"
            @click="openProject(project.deviceId, project.cwd)"
          >
            <ProjectCard :project="project" compact />
          </button>
        </div>
        <div
          v-else
          class="rounded-[1.6rem] border border-dashed border-border bg-background/65 p-6 text-sm text-muted-foreground"
        >
          {{ t("home.noReviewProjects") }}
        </div>
      </section>
    </div>

    <section class="space-y-3">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
          {{ t("home.recentProjects") }}
        </h3>
        <span class="text-xs text-muted-foreground">
          {{ t("common.refreshedAt", { value: formatRelativeTime(store.lastRefreshEpochMs) }) }}
        </span>
      </div>
      <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        <button
          v-for="project in store.recentProjects.slice(0, 6)"
          :key="project.key"
          class="w-full text-left"
          @click="openProject(project.deviceId, project.cwd)"
        >
          <ProjectCard :project="project" />
        </button>
      </div>
    </section>
  </section>
</template>
