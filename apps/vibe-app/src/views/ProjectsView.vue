<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import ProjectCard from "@/components/app/ProjectCard.vue";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { buildProjectRouteParam } from "@/lib/projects";
import { useAppStore } from "@/stores/app";

const store = useAppStore();
const router = useRouter();
const { t } = useI18n();
const search = ref("");
const filter = ref<"all" | "running" | "recent">("all");

const projects = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  return store.projectSummaries.filter((project) => {
    if (filter.value === "running" && project.runningTaskCount === 0) {
      return false;
    }
    if (filter.value === "recent" && !store.recentProjectKeys.includes(project.key)) {
      return false;
    }
    if (!keyword) {
      return true;
    }
    return [project.title, project.deviceName, project.pathLabel].some((field) =>
      field.toLowerCase().includes(keyword)
    );
  });
});

function hostProjects(deviceId: string) {
  return projects.value.filter((entry) => entry.deviceId === deviceId);
}

function hostProjectEmptyKey(host: (typeof store.hostSummaries)[number]) {
  if (!host.device.online) {
    return "projects.hostEmptyOffline";
  }

  const workingRoot =
    host.device.metadata.workingRoot ??
    host.device.metadata.workspace_root ??
    host.device.metadata.working_root;
  if (!workingRoot) {
    return "projects.hostEmptyNoWorkspace";
  }

  return "projects.hostEmptyNoProjects";
}

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
    <div class="rounded-[1.8rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div class="space-y-2">
          <StatusBadge>{{ t("nav.projects") }}</StatusBadge>
          <h2 class="text-xl font-semibold">{{ t("projects.title") }}</h2>
          <p class="max-w-2xl text-sm text-muted-foreground">
            {{ t("projects.summary") }}
          </p>
        </div>
        <div class="grid gap-3 sm:grid-cols-[1fr_auto_auto_auto]">
          <input
            v-model="search"
            type="search"
            :placeholder="t('projects.searchPlaceholder')"
            class="rounded-2xl border border-border bg-background px-4 py-2.5 text-sm"
          />
          <button
            class="rounded-full border px-4 py-2 text-sm"
            :class="filter === 'all' ? 'border-primary bg-primary text-primary-foreground' : 'border-border'"
            @click="filter = 'all'"
          >
            {{ t("common.all") }}
          </button>
          <button
            class="rounded-full border px-4 py-2 text-sm"
            :class="filter === 'running' ? 'border-primary bg-primary text-primary-foreground' : 'border-border'"
            @click="filter = 'running'"
          >
            {{ t("projectCard.running") }}
          </button>
          <button
            class="rounded-full border px-4 py-2 text-sm"
            :class="filter === 'recent' ? 'border-primary bg-primary text-primary-foreground' : 'border-border'"
            @click="filter = 'recent'"
          >
            {{ t("projects.recentFilter") }}
          </button>
        </div>
      </div>
    </div>

    <section v-for="host in store.hostSummaries" :key="host.device.id" class="space-y-3">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold">{{ host.device.name }}</h3>
          <p class="text-sm text-muted-foreground">
            {{ host.device.platform }} · {{ t("common.projectsCount", { count: host.projectCount }) }}
          </p>
        </div>
        <StatusBadge :tone="host.device.online ? 'success' : 'muted'">
          {{ host.device.online ? t("common.online") : t("common.offline") }}
        </StatusBadge>
      </div>

      <div v-if="hostProjects(host.device.id).length" class="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        <button
          v-for="project in hostProjects(host.device.id)"
          :key="project.key"
          class="w-full text-left"
          @click="openProject(project.deviceId, project.cwd)"
        >
          <ProjectCard :project="project" />
        </button>
      </div>
      <div
        v-else
        class="rounded-[1.4rem] border border-dashed border-border bg-background/65 p-4 text-sm text-muted-foreground"
      >
        {{ t(hostProjectEmptyKey(host)) }}
      </div>
    </section>

    <div
      v-if="!projects.length"
      class="rounded-[1.6rem] border border-dashed border-border bg-background/65 p-6 text-sm text-muted-foreground"
    >
      {{ t("projects.empty") }}
    </div>
  </section>
</template>
