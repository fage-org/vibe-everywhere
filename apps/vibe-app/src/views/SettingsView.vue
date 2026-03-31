<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatRelativeTime } from "@/lib/format";
import { getSupportedLocales, setAppLocale } from "@/lib/i18n";
import { getSupportedThemeModes, setThemeMode, useTheme } from "@/lib/theme";
import { getRelayBaseUrlPlaceholder, isLoopbackRelayBaseUrl } from "@/lib/runtime";
import { useAppStore } from "@/stores/app";
import type { ProviderKind } from "@/types";

const store = useAppStore();
const { themeMode } = useTheme();
const { t } = useI18n();
const locales = getSupportedLocales();
const themeModes = getSupportedThemeModes();
const currentLocale = computed(() => document.documentElement.lang || "en");

const projectForm = ref({
  id: "",
  name: "",
  deviceId: "",
  cwd: ""
});

const modelForm = ref({
  id: "",
  name: "",
  provider: "codex" as ProviderKind,
  modelId: ""
});

const hostOptions = computed(() => store.hostSummaries);
const providerOptions: ProviderKind[] = ["codex", "claude_code", "open_code"];
const relayPlaceholder = computed(() => getRelayBaseUrlPlaceholder());
const relayNeedsMobileHint = computed(() => isLoopbackRelayBaseUrl(store.relayBaseUrlInput));

async function saveRelaySettings() {
  await store.saveRelaySettings();
}

function editProject(projectId: string) {
  const project = store.configuredProjects.find((entry) => entry.id === projectId);
  if (!project) {
    return;
  }

  projectForm.value = {
    id: project.id,
    name: project.name,
    deviceId: project.deviceId,
    cwd: project.cwd
  };
}

function resetProjectForm() {
  projectForm.value = {
    id: "",
    name: "",
    deviceId: hostOptions.value[0]?.device.id ?? "",
    cwd: ""
  };
}

function submitProject() {
  if (!projectForm.value.name.trim() || !projectForm.value.deviceId || !projectForm.value.cwd.trim()) {
    return;
  }

  if (projectForm.value.id) {
    store.updateConfiguredProject(projectForm.value.id, {
      name: projectForm.value.name,
      deviceId: projectForm.value.deviceId,
      cwd: projectForm.value.cwd
    });
  } else {
    store.createConfiguredProject({
      name: projectForm.value.name,
      deviceId: projectForm.value.deviceId,
      cwd: projectForm.value.cwd
    });
  }

  resetProjectForm();
}

function editModel(modelId: string) {
  const profile = store.modelProfiles.find((entry) => entry.id === modelId);
  if (!profile) {
    return;
  }

  modelForm.value = {
    id: profile.id,
    name: profile.name,
    provider: profile.provider,
    modelId: profile.modelId
  };
}

function resetModelForm() {
  modelForm.value = {
    id: "",
    name: "",
    provider: store.selectedProvider || "codex",
    modelId: ""
  };
}

function submitModel() {
  if (!modelForm.value.name.trim() || !modelForm.value.modelId.trim()) {
    return;
  }

  if (modelForm.value.id) {
    store.updateModelProfile(modelForm.value.id, {
      name: modelForm.value.name,
      provider: modelForm.value.provider,
      modelId: modelForm.value.modelId
    });
  } else {
    store.createModelProfile({
      name: modelForm.value.name,
      provider: modelForm.value.provider,
      modelId: modelForm.value.modelId
    });
  }

  resetModelForm();
}

resetProjectForm();
resetModelForm();
</script>

<template>
  <section class="mx-auto flex min-h-screen w-full max-w-[1600px] flex-col gap-4 px-3 py-3 md:px-4">
    <header class="rounded-[1.25rem] border border-white/60 bg-white/90 px-4 py-4 shadow-[0_18px_50px_-36px_rgba(15,23,42,0.55)] backdrop-blur dark:border-white/10 dark:bg-slate-950/80">
      <div class="flex flex-wrap items-center gap-2">
        <StatusBadge>{{ t("settings.badge") }}</StatusBadge>
        <StatusBadge :tone="store.onlineHostCount ? 'success' : 'muted'">
          {{ t("dashboard.hostsOnline", { count: store.onlineHostCount }) }}
        </StatusBadge>
        <StatusBadge tone="muted">
          {{ t("home.configuredProjectsCount", { count: store.configuredProjectViews.length }) }}
        </StatusBadge>
        <StatusBadge tone="muted">
          {{ t("settings.modelsCount", { count: store.modelProfiles.length }) }}
        </StatusBadge>
      </div>
      <h2 class="mt-2 text-2xl font-semibold">{{ t("settings.title") }}</h2>
      <p class="mt-1 text-sm text-muted-foreground">{{ t("settings.summary") }}</p>
    </header>

    <div class="grid gap-4 xl:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]">
      <article class="rounded-[1.25rem] border border-white/60 bg-white/90 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/80">
        <div class="flex items-center justify-between gap-3">
          <div>
            <h3 class="text-lg font-semibold">{{ t("settings.serverTitle") }}</h3>
            <p class="mt-1 text-sm text-muted-foreground">{{ t("settings.serverSummary") }}</p>
          </div>
          <button class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground" @click="saveRelaySettings">
            {{ t("settings.save") }}
          </button>
        </div>

        <div class="mt-4 grid gap-3">
          <label class="grid gap-2 text-sm">
            <span class="font-medium">{{ t("settings.relayUrl") }}</span>
            <input
              v-model="store.relayBaseUrlInput"
              type="url"
              class="w-full rounded-2xl border border-border bg-background px-4 py-3"
              :placeholder="relayPlaceholder"
            />
          </label>

          <label class="grid gap-2 text-sm">
            <span class="font-medium">{{ t("settings.accessToken") }}</span>
            <input
              v-model="store.relayAccessTokenInput"
              type="password"
              class="w-full rounded-2xl border border-border bg-background px-4 py-3"
              :placeholder="t('settings.accessTokenPlaceholder')"
            />
          </label>

          <div class="rounded-2xl border border-border bg-background/70 px-4 py-3 text-xs text-muted-foreground">
            {{ t("settings.currentServer", { value: store.relayBaseUrl || t("settings.notConfigured") }) }}
            <span v-if="relayNeedsMobileHint" class="mt-1 block">
              {{ t("settings.mobileRelayHint") }}
            </span>
          </div>
        </div>
      </article>

      <article class="rounded-[1.25rem] border border-white/60 bg-white/90 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/80">
        <h3 class="text-lg font-semibold">{{ t("settings.hostsTitle") }}</h3>
        <div class="mt-4 space-y-2">
          <div
            v-for="host in hostOptions"
            :key="host.device.id"
            class="grid gap-2 rounded-2xl border border-border bg-background/75 px-4 py-3"
          >
            <div class="flex items-center justify-between gap-3">
              <p class="truncate text-sm font-medium">{{ host.device.name }}</p>
              <StatusBadge :tone="host.device.online ? 'success' : 'muted'">
                {{ host.device.online ? t("common.online") : t("common.offline") }}
              </StatusBadge>
            </div>
            <p class="text-xs text-muted-foreground">
              {{ host.device.platform }} · {{ t("common.projectsCount", { count: host.projectCount }) }}
            </p>
            <p class="text-xs text-muted-foreground">
              {{ t("settings.hostProviders", { value: host.device.providers.filter((provider) => provider.available).map((provider) => store.getProviderLabel(provider.kind)).join(", ") || "—" }) }}
            </p>
          </div>
        </div>
      </article>
    </div>

    <div class="grid gap-4 xl:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]">
      <article class="rounded-[1.25rem] border border-white/60 bg-white/90 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/80">
        <div class="flex items-center justify-between gap-3">
          <div>
            <h3 class="text-lg font-semibold">{{ t("settings.projectsTitle") }}</h3>
            <p class="mt-1 text-sm text-muted-foreground">{{ t("settings.projectsSummary") }}</p>
          </div>
          <button class="rounded-full border border-border px-4 py-2 text-sm" @click="resetProjectForm">
            {{ t("settings.newProject") }}
          </button>
        </div>

        <div class="mt-4 grid gap-3 rounded-[1.1rem] border border-border bg-background/60 p-3">
          <input
            v-model="projectForm.name"
            type="text"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :placeholder="t('settings.projectName')"
          />
          <select
            v-model="projectForm.deviceId"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
          >
            <option value="">{{ t("settings.projectHost") }}</option>
            <option
              v-for="host in hostOptions"
              :key="host.device.id"
              :value="host.device.id"
            >
              {{ host.device.name }}
            </option>
          </select>
          <input
            v-model="projectForm.cwd"
            type="text"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :placeholder="t('settings.projectPath')"
          />
          <div class="flex gap-2">
            <button class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground" @click="submitProject">
              {{ projectForm.id ? t("settings.updateProject") : t("settings.createProject") }}
            </button>
            <button class="rounded-full border border-border px-4 py-2 text-sm" @click="resetProjectForm">
              {{ t("settings.cancelEdit") }}
            </button>
          </div>
        </div>

        <div class="mt-4 space-y-2">
          <div
            v-for="project in store.configuredProjectViews"
            :key="project.id"
            class="rounded-2xl border border-border bg-background/75 px-4 py-3"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-medium">{{ project.name }}</p>
                <p class="mt-1 truncate text-xs text-muted-foreground">
                  {{ project.deviceName }} · {{ project.pathLabel }}
                </p>
                <p class="mt-1 text-[11px] text-muted-foreground">
                  {{ t("settings.projectAcps", { value: project.availableProviders.map((provider) => store.getProviderLabel(provider)).join(", ") || "—" }) }}
                </p>
              </div>
              <div class="flex items-center gap-2">
                <StatusBadge :tone="project.online ? 'success' : 'muted'">
                  {{ project.online ? t("common.online") : t("common.offline") }}
                </StatusBadge>
                <button class="rounded-full border border-border px-3 py-1 text-xs" @click="editProject(project.id)">
                  {{ t("settings.edit") }}
                </button>
                <button class="rounded-full border border-rose-500/20 px-3 py-1 text-xs text-rose-700 dark:text-rose-300" @click="store.deleteConfiguredProject(project.id)">
                  {{ t("settings.delete") }}
                </button>
              </div>
            </div>
          </div>
          <div
            v-if="!store.configuredProjectViews.length"
            class="rounded-2xl border border-dashed border-border bg-background/70 px-4 py-5 text-sm text-muted-foreground"
          >
            {{ t("settings.projectsEmpty") }}
          </div>
        </div>
      </article>

      <article class="rounded-[1.25rem] border border-white/60 bg-white/90 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/80">
        <div class="flex items-center justify-between gap-3">
          <div>
            <h3 class="text-lg font-semibold">{{ t("settings.modelsTitle") }}</h3>
            <p class="mt-1 text-sm text-muted-foreground">{{ t("settings.modelsSummary") }}</p>
          </div>
          <button class="rounded-full border border-border px-4 py-2 text-sm" @click="resetModelForm">
            {{ t("settings.newModel") }}
          </button>
        </div>

        <div class="mt-4 grid gap-3 rounded-[1.1rem] border border-border bg-background/60 p-3">
          <input
            v-model="modelForm.name"
            type="text"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :placeholder="t('settings.modelName')"
          />
          <select
            v-model="modelForm.provider"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
          >
            <option
              v-for="provider in providerOptions"
              :key="provider"
              :value="provider"
            >
              {{ store.getProviderLabel(provider) }}
            </option>
          </select>
          <input
            v-model="modelForm.modelId"
            type="text"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :placeholder="t('settings.modelId')"
          />
          <div class="flex gap-2">
            <button class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground" @click="submitModel">
              {{ modelForm.id ? t("settings.updateModel") : t("settings.createModel") }}
            </button>
            <button class="rounded-full border border-border px-4 py-2 text-sm" @click="resetModelForm">
              {{ t("settings.cancelEdit") }}
            </button>
          </div>
        </div>

        <div class="mt-4 space-y-2">
          <div
            v-for="profile in store.modelProfiles"
            :key="profile.id"
            class="rounded-2xl border border-border bg-background/75 px-4 py-3"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-medium">{{ profile.name }}</p>
                <p class="mt-1 text-xs text-muted-foreground">
                  {{ store.getProviderLabel(profile.provider) }} · {{ profile.modelId }}
                </p>
                <p class="mt-1 text-[11px] text-muted-foreground">
                  {{ formatRelativeTime(profile.updatedAtEpochMs) }}
                </p>
              </div>
              <div class="flex items-center gap-2">
                <button class="rounded-full border border-border px-3 py-1 text-xs" @click="editModel(profile.id)">
                  {{ t("settings.edit") }}
                </button>
                <button class="rounded-full border border-rose-500/20 px-3 py-1 text-xs text-rose-700 dark:text-rose-300" @click="store.deleteModelProfile(profile.id)">
                  {{ t("settings.delete") }}
                </button>
              </div>
            </div>
          </div>
          <div
            v-if="!store.modelProfiles.length"
            class="rounded-2xl border border-dashed border-border bg-background/70 px-4 py-5 text-sm text-muted-foreground"
          >
            {{ t("settings.modelsEmpty") }}
          </div>
        </div>
      </article>
    </div>

    <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
      <article class="rounded-[1.25rem] border border-white/60 bg-white/90 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/80">
        <h3 class="text-lg font-semibold">{{ t("settings.language") }}</h3>
        <div class="mt-4 flex flex-wrap gap-2">
          <button
            v-for="locale in locales"
            :key="locale"
            class="rounded-full border px-4 py-2 text-sm"
            :class="currentLocale === locale ? 'border-primary bg-primary text-primary-foreground' : 'border-border'"
            @click="setAppLocale(locale)"
          >
            {{ locale }}
          </button>
        </div>
      </article>

      <article class="rounded-[1.25rem] border border-white/60 bg-white/90 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/80">
        <h3 class="text-lg font-semibold">{{ t("settings.theme") }}</h3>
        <div class="mt-4 flex flex-wrap gap-2">
          <button
            v-for="mode in themeModes"
            :key="mode"
            class="rounded-full border px-4 py-2 text-sm capitalize"
            :class="themeMode === mode ? 'border-primary bg-primary text-primary-foreground' : 'border-border'"
            @click="setThemeMode(mode)"
          >
            {{ mode }}
          </button>
        </div>
      </article>
    </div>
  </section>
</template>
