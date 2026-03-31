<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import ProjectChangesPanel from "@/features/project/ProjectChangesPanel.vue";
import ProjectConversationPanel from "@/features/project/ProjectConversationPanel.vue";
import ProjectFilesPanel from "@/features/project/ProjectFilesPanel.vue";
import { useProjectWorkspace, type ProjectTab } from "@/features/project/useProjectWorkspace";
import { formatRelativeTime } from "@/lib/format";
import { useAppStore } from "@/stores/app";
import type { ProviderKind } from "@/types";

const router = useRouter();
const store = useAppStore();
const { t } = useI18n();
const sidebarOpen = ref(false);
const activeTab = ref<ProjectTab>("conversation");

const selectedProject = computed(() => store.selectedConfiguredProject);
const selectedProvider = computed(() => store.selectedProvider || null);
const selectedModelValue = computed({
  get: () => store.getSelectedModelId(),
  set: (value: string) => {
    const matched = store.availableModelProfiles.find((profile) => profile.modelId === value);
    store.setSelectedModelProfile(matched?.id ?? "");
  }
});
const modelOptions = computed(() =>
  store.availableModelProfiles.map((profile) => ({
    label: `${profile.name} · ${profile.modelId}`,
    value: profile.modelId
  }))
);
const scopeReady = computed(() => Boolean(selectedProject.value && selectedProvider.value));
const providerLabel = computed(() =>
  selectedProvider.value ? store.getProviderLabel(selectedProvider.value) : t("chat.noAcpSelected")
);

const workspace = useProjectWorkspace(
  computed(() => selectedProject.value?.deviceId ?? ""),
  computed(() => selectedProject.value?.cwd),
  computed(() => selectedProvider.value ?? undefined),
  scopeReady
);

const projectSummary = computed(() => selectedProject.value?.projectSummary ?? null);
const recentConversationList = computed(() => workspace.conversations.value.slice(0, 8));
const workspaceErrorMessage = computed(() => workspace.errorMessage.value || store.errorMessage);
const emptySummary = computed(() => {
  if (!store.hasRelayConfig) {
    return t("home.emptyServerSummary");
  }
  if (!store.configuredProjectViews.length) {
    return t("home.emptyConfiguredProjectsSummary");
  }
  if (!selectedProvider.value) {
    return t("home.emptyProviderSummary");
  }
  return t("conversation.firstTurnSummary", {
    project: selectedProject.value?.name ?? t("workspace.title")
  });
});

async function syncWorkspaceConversation() {
  if (!scopeReady.value || !selectedProject.value || !selectedProvider.value) {
    activeTab.value = "conversation";
    workspace.startNewConversation();
    return;
  }

  await workspace.refreshProject();
  const rememberedId = store.getRememberedConversationId(
    selectedProject.value.id,
    selectedProvider.value
  );
  const rememberedConversation = workspace.conversations.value.find(
    (conversation) => conversation.id === rememberedId
  );
  if (rememberedConversation) {
    await workspace.selectConversation(rememberedConversation.id);
    return;
  }

  const latestConversation = workspace.conversations.value[0];
  if (latestConversation) {
    await workspace.selectConversation(latestConversation.id);
    return;
  }

  workspace.startNewConversation();
}

function onProjectChange(event: Event) {
  store.setSelectedProject((event.target as HTMLSelectElement).value);
}

function onProviderChange(event: Event) {
  store.setSelectedProvider((event.target as HTMLSelectElement).value as ProviderKind);
}

function refreshWorkspace() {
  void syncWorkspaceConversation();
}

watch(
  () => [store.selectedProjectId, store.selectedProvider],
  async () => {
    await syncWorkspaceConversation();
  },
  { immediate: true }
);

watch(
  () => workspace.activeConversationId.value,
  (conversationId) => {
    if (!conversationId) {
      return;
    }
    store.rememberConversationForCurrentScope(conversationId);
  }
);
</script>

<template>
  <section class="mx-auto flex min-h-screen w-full max-w-[1700px] gap-0 px-0 xl:px-4">
    <div
      v-if="sidebarOpen"
      class="fixed inset-0 z-30 bg-black/40 xl:hidden"
      @click="sidebarOpen = false"
    />

    <aside
      class="fixed inset-y-0 left-0 z-40 flex w-[90vw] max-w-[360px] flex-col border-r border-border/60 bg-[#f5f2ea]/96 px-3 py-3 shadow-[0_28px_80px_-40px_rgba(15,23,42,0.6)] backdrop-blur transition-transform dark:bg-slate-950/96 xl:sticky xl:top-0 xl:z-0 xl:h-screen xl:w-[330px] xl:max-w-none xl:translate-x-0 xl:border xl:border-white/10 xl:shadow-none"
      :class="sidebarOpen ? 'translate-x-0' : '-translate-x-full'"
    >
      <div class="flex items-center justify-between xl:hidden">
        <p class="text-xs font-semibold uppercase tracking-[0.2em] text-muted-foreground">
          {{ t("chat.context") }}
        </p>
        <button class="rounded-full border border-border px-3 py-1 text-xs" @click="sidebarOpen = false">
          {{ t("common.close") }}
        </button>
      </div>

      <div class="mt-3 space-y-3 rounded-[1.25rem] border border-white/60 bg-white/85 p-3 dark:border-white/10 dark:bg-slate-900/75">
        <div class="grid gap-2">
          <label class="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
            {{ t("chat.project") }}
          </label>
          <select
            :value="store.selectedProjectId"
            class="w-full rounded-2xl border border-border bg-background px-3 py-2.5 text-sm"
            @change="onProjectChange"
          >
            <option value="">{{ t("chat.noProjectSelected") }}</option>
            <option
              v-for="project in store.configuredProjectViews"
              :key="project.id"
              :value="project.id"
            >
              {{ project.name }} · {{ project.deviceName }}
            </option>
          </select>
        </div>

        <div class="grid gap-2">
          <label class="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
            {{ t("chat.acp") }}
          </label>
          <select
            :value="store.selectedProvider"
            class="w-full rounded-2xl border border-border bg-background px-3 py-2.5 text-sm"
            :disabled="!selectedProject"
            @change="onProviderChange"
          >
            <option value="">{{ t("chat.noAcpSelected") }}</option>
            <option
              v-for="provider in store.availableProvidersForSelectedProject"
              :key="provider"
              :value="provider"
            >
              {{ store.getProviderLabel(provider) }}
            </option>
          </select>
        </div>

        <div class="grid gap-2">
          <label class="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
            {{ t("chat.model") }}
          </label>
          <select
            v-model="selectedModelValue"
            class="w-full rounded-2xl border border-border bg-background px-3 py-2.5 text-sm"
            :disabled="!selectedProvider"
          >
            <option value="">{{ t("chat.noModelSelected") }}</option>
            <option
              v-for="profile in store.availableModelProfiles"
              :key="profile.id"
              :value="profile.modelId"
            >
              {{ profile.name }}
            </option>
          </select>
        </div>
      </div>

      <button
        class="mt-3 flex w-full items-center justify-between rounded-[1.15rem] border border-border bg-background px-4 py-3 text-left text-sm font-medium"
        :disabled="!scopeReady"
        @click="workspace.startNewConversation"
      >
        <span>{{ t("conversation.newChat") }}</span>
        <span class="text-muted-foreground">+</span>
      </button>

      <div class="mt-3 flex-1 overflow-auto">
        <p class="px-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
          {{ t("conversation.recentChats") }}
        </p>
        <div class="mt-2 space-y-2">
          <button
            v-for="conversation in recentConversationList"
            :key="conversation.id"
            class="w-full rounded-[1.15rem] border px-3 py-3 text-left"
            :class="
              conversation.id === workspace.activeConversationId.value && !workspace.isDraftConversation.value
                ? 'border-primary bg-primary/10'
                : 'border-border bg-background/75'
            "
            @click="workspace.selectConversation(conversation.id)"
          >
            <p class="truncate text-sm font-medium">{{ conversation.title }}</p>
            <p class="mt-1 flex items-center gap-2 text-[11px] text-muted-foreground">
              <span>{{ formatRelativeTime(conversation.updatedAtEpochMs) }}</span>
              <span>{{ store.getProviderLabel(conversation.provider) }}</span>
            </p>
          </button>
          <div
            v-if="!recentConversationList.length"
            class="rounded-[1.15rem] border border-dashed border-border bg-background/65 px-3 py-4 text-sm text-muted-foreground"
          >
            {{ t("conversation.emptyTitle") }}
          </div>
        </div>
      </div>

      <button
        class="mt-3 flex w-full items-center justify-between rounded-[1.15rem] border border-border bg-background px-4 py-3 text-left text-sm font-medium"
        @click="router.push({ name: 'settings' })"
      >
        <span>{{ t("nav.settings") }}</span>
        <span class="text-xs text-muted-foreground">{{ store.activeServerLabel }}</span>
      </button>
    </aside>

    <div class="flex min-h-screen min-w-0 flex-1 flex-col px-3 py-3 md:px-4 xl:pl-4">
      <header class="sticky top-0 z-20 rounded-[1.3rem] border border-white/60 bg-white/90 px-4 py-3 shadow-[0_18px_50px_-36px_rgba(15,23,42,0.55)] backdrop-blur dark:border-white/10 dark:bg-slate-950/80">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <button
                class="rounded-full border border-border px-3 py-1.5 text-sm xl:hidden"
                @click="sidebarOpen = true"
              >
                {{ t("common.menu") }}
              </button>
              <StatusBadge>{{ t("dashboard.badge") }}</StatusBadge>
              <StatusBadge :tone="selectedProject?.online ? 'success' : 'muted'">
                {{ selectedProject?.deviceName ?? t("chat.noProjectSelected") }}
              </StatusBadge>
              <StatusBadge tone="muted">{{ providerLabel }}</StatusBadge>
              <StatusBadge v-if="store.selectedModelProfile" tone="muted">
                {{ store.selectedModelProfile.name }}
              </StatusBadge>
            </div>
            <h1 class="mt-2 truncate text-xl font-semibold">
              {{ selectedProject?.name ?? t("home.chatTitle") }}
            </h1>
            <p class="mt-1 truncate text-sm text-muted-foreground">
              {{
                selectedProject
                  ? `${selectedProject.pathLabel} · ${selectedProject.deviceName}`
                  : t("home.chatSummary")
              }}
            </p>
          </div>

          <div class="flex flex-wrap gap-2">
            <button
              v-for="tab in ['changes', 'files']"
              :key="tab"
              class="rounded-full border px-4 py-2 text-sm"
              :class="
                activeTab === tab
                  ? 'border-primary bg-primary text-primary-foreground'
                  : 'border-border bg-background/70'
              "
              :disabled="!scopeReady"
              @click="activeTab = tab as ProjectTab"
            >
              {{ t(`workspace.tabs.${tab}`) }}
            </button>
            <button class="rounded-full border border-border px-4 py-2 text-sm" @click="refreshWorkspace">
              {{ t("common.refresh") }}
            </button>
          </div>
        </div>

        <div class="mt-3 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
          <span>{{ t("home.configuredProjectsCount", { count: store.configuredProjectViews.length }) }}</span>
          <span>{{ t("dashboard.hostsOnline", { count: store.onlineHostCount }) }}</span>
          <span>{{ t("workspace.metrics.topics", { count: projectSummary?.conversationCount ?? recentConversationList.length }) }}</span>
          <span>{{ t("workspace.metrics.running", { count: projectSummary?.runningTaskCount ?? 0 }) }}</span>
          <span v-if="projectSummary?.branchName">{{ t("workspace.metrics.branch", { value: projectSummary.branchName }) }}</span>
          <span v-if="projectSummary">{{ t("workspace.metrics.changedFiles", { count: projectSummary.changedFilesCount }) }}</span>
        </div>
      </header>

      <div
        v-if="workspaceErrorMessage"
        class="mt-3 rounded-[1.1rem] border border-rose-500/20 bg-rose-500/8 px-4 py-3 text-sm text-rose-700 dark:text-rose-300"
      >
        {{ workspaceErrorMessage }}
      </div>

      <ProjectConversationPanel
        class="mt-3"
        :detail="workspace.conversationDetail.value"
        :project-title="selectedProject?.name"
        :is-draft-conversation="workspace.isDraftConversation.value"
        v-model:selected-model="selectedModelValue"
        :model-options="modelOptions"
        :can-compose="scopeReady"
        :empty-summary="emptySummary"
        @send-prompt="workspace.sendFollowUp"
        @respond-input="workspace.respondToInput"
        @cancel-task="workspace.cancelTask"
        @open-tab="activeTab = $event"
      />
    </div>

    <div
      v-if="scopeReady && activeTab !== 'conversation'"
      class="fixed inset-0 z-50 bg-black/40 px-2 py-2 md:px-4 md:py-4"
      @click.self="activeTab = 'conversation'"
    >
      <div class="mx-auto flex h-full max-w-6xl flex-col overflow-hidden rounded-[1.5rem] border border-white/60 bg-[#f8f6f1] shadow-[0_28px_90px_-45px_rgba(15,23,42,0.6)] dark:border-white/10 dark:bg-slate-950">
        <div class="flex items-center justify-between border-b border-border/70 px-4 py-3">
          <div>
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              {{ activeTab === 'changes' ? t('workspace.tabs.changes') : t('workspace.tabs.files') }}
            </p>
            <p class="text-sm text-muted-foreground">
              {{ selectedProject?.name ?? t("workspace.title") }}
            </p>
          </div>
          <button class="rounded-full border border-border px-4 py-2 text-sm" @click="activeTab = 'conversation'">
            {{ t("common.close") }}
          </button>
        </div>

        <div class="flex-1 overflow-auto p-4">
          <ProjectChangesPanel
            v-if="activeTab === 'changes'"
            :git-inspect="workspace.gitInspect.value"
            :git-diff="workspace.gitDiff.value"
            :active-repo-path="workspace.activeDiffRepoPath.value"
            @select-file="workspace.selectChangeFile"
          />

          <ProjectFilesPanel
            v-else
            :workspace="workspace.workspace.value"
            :file-preview="workspace.filePreview.value"
            @open-entry="workspace.openEntry"
          />
        </div>
      </div>
    </div>
  </section>
</template>
