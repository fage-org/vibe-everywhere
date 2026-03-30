<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { RouterLink, useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatRelativeTime } from "@/lib/format";
import { buildProjectRouteParam, buildProjectTree } from "@/lib/projects";
import { useAppStore } from "@/stores/app";
import ProjectChangesPanel from "@/features/project/ProjectChangesPanel.vue";
import ProjectConversationPanel from "@/features/project/ProjectConversationPanel.vue";
import DesktopProjectTreeNode from "@/features/project/DesktopProjectTreeNode.vue";
import ProjectFilesPanel from "@/features/project/ProjectFilesPanel.vue";
import ProjectLogsPanel from "@/features/project/ProjectLogsPanel.vue";
import { useProjectWorkspace, type ProjectTab } from "@/features/project/useProjectWorkspace";

const route = useRoute();
const store = useAppStore();
const { t } = useI18n();
const deviceId = computed(() => String(route.params.deviceId));
const projectPath = computed(() => route.params.projectPath);
const workspace = useProjectWorkspace(deviceId, projectPath);
const {
  project,
  cwd,
  conversations,
  activeConversationId,
  conversationDetail,
  auditEvents,
  gitInspect,
  gitDiff,
  activeDiffRepoPath,
  workspace: workspaceBrowse,
  filePreview,
  activeTab,
  errorMessage
} = workspace;

const tabs: { id: ProjectTab; key: string }[] = [
  { id: "conversation", key: "workspace.tabs.conversation" },
  { id: "changes", key: "workspace.tabs.changes" },
  { id: "files", key: "workspace.tabs.files" },
  { id: "logs", key: "workspace.tabs.logs" }
];
const currentWorktree = computed(
  () => gitInspect.value?.worktrees.find((entry) => entry.isCurrent) ?? null
);
const worktreeBranchName = ref("");
const worktreeDirectoryName = ref("");
const worktreeDirectoryDirty = ref(false);
const worktreeCreateError = ref("");
const worktreeCreateSuccess = ref("");
const worktreeRemoveError = ref("");
const worktreeRemoveSuccess = ref("");
const isCreatingWorktree = ref(false);
const removingWorktreePath = ref<string | null>(null);
const worktreeRemoveFailureByPath = ref<Record<string, string>>({});
type WorktreeLifecycleState =
  | "current"
  | "detached"
  | "inventory_missing"
  | "offline"
  | "unreachable"
  | "available"
  | "remove_failed";
const desktopHostTree = computed(() =>
  store.hostSummaries.map((host) => ({
    ...host,
    projectTree: buildProjectTree(
      store.projectSummaries.filter((entry) => entry.deviceId === host.device.id),
      host.device.metadata.workingRoot ??
        host.device.metadata.workspace_root ??
        host.device.metadata.working_root ??
        null
    )
  }))
);
const worktreeProjects = computed(() =>
  (gitInspect.value?.worktrees ?? []).map((entry) => ({
    ...entry,
    project:
      store.projectSummaries.find(
        (projectSummary) =>
          projectSummary.deviceId === deviceId.value && projectSummary.cwd === entry.path
      ) ?? null
  }))
);
const worktreeLifecycleItems = computed(() =>
  worktreeProjects.value.map((entry) => {
    const removeFailure = worktreeRemoveFailureByPath.value[entry.path] ?? null;
    const inventoryState = entry.project?.availabilityState ?? null;
    const lifecycleState: WorktreeLifecycleState = removeFailure
      ? "remove_failed"
      : entry.isCurrent
        ? "current"
        : entry.isDetached
          ? "detached"
          : !entry.project
            ? "inventory_missing"
            : inventoryState === "offline"
              ? "offline"
              : inventoryState === "unreachable"
                ? "unreachable"
                : "available";

    return {
      ...entry,
      lifecycleState,
      removeFailure
    };
  })
);
const worktreeBaseName = computed(() => {
  const path = gitInspect.value?.repoRoot ?? project.value?.cwd ?? project.value?.pathLabel ?? "";
  const segments = path.split(/[/\\]+/).filter(Boolean);
  return segments[segments.length - 1] ?? "worktree";
});
const suggestedWorktreeDirectory = computed(() => {
  const branchSlug = worktreeBranchName.value
    .trim()
    .replace(/[^a-zA-Z0-9._-]+/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "");
  return branchSlug
    ? `${worktreeBaseName.value}-${branchSlug}`
    : `${worktreeBaseName.value}-worktree`;
});

const projectAvailabilityTone = computed(() => {
  if (project.value?.availabilityState === "offline") {
    return "muted" as const;
  }
  if (
    project.value?.availabilityState === "unreachable" ||
    project.value?.availabilityState === "history_only"
  ) {
    return "warning" as const;
  }
  return "success" as const;
});

watch(
  worktreeBranchName,
  () => {
    if (!worktreeDirectoryDirty.value) {
      worktreeDirectoryName.value = suggestedWorktreeDirectory.value;
    }
  },
  { immediate: true }
);

function onWorktreeDirectoryInput(value: string) {
  worktreeDirectoryDirty.value = true;
  worktreeDirectoryName.value = value;
}

async function createDesktopWorktree() {
  const branchName = worktreeBranchName.value.trim();
  const directoryName = worktreeDirectoryName.value.trim();
  if (!branchName) {
    worktreeCreateError.value = t("workspace.desktop.worktreeBranchRequired");
    worktreeCreateSuccess.value = "";
    return;
  }
  if (!directoryName) {
    worktreeCreateError.value = t("workspace.desktop.worktreeDirectoryRequired");
    worktreeCreateSuccess.value = "";
    return;
  }

  isCreatingWorktree.value = true;
  worktreeCreateError.value = "";
  worktreeCreateSuccess.value = "";
  worktreeRemoveError.value = "";
  worktreeRemoveSuccess.value = "";
  worktreeRemoveFailureByPath.value = {};

  try {
    await workspace.createWorktree(branchName, `../${directoryName}`);
    worktreeCreateSuccess.value = t("workspace.desktop.worktreeCreated", {
      value: directoryName
    });
    worktreeBranchName.value = "";
    worktreeDirectoryDirty.value = false;
    worktreeDirectoryName.value = "";
  } catch (error) {
    worktreeCreateError.value = error instanceof Error ? error.message : String(error);
  } finally {
    isCreatingWorktree.value = false;
  }
}

async function removeDesktopWorktree(worktreePath: string, label: string) {
  if (!window.confirm(t("workspace.desktop.worktreeRemoveConfirm", { value: label }))) {
    return;
  }

  removingWorktreePath.value = worktreePath;
  worktreeRemoveError.value = "";
  worktreeRemoveSuccess.value = "";
  worktreeCreateError.value = "";
  worktreeCreateSuccess.value = "";
  worktreeRemoveFailureByPath.value = {
    ...worktreeRemoveFailureByPath.value,
    [worktreePath]: ""
  };

  try {
    await workspace.removeWorktree(worktreePath);
    worktreeRemoveSuccess.value = t("workspace.desktop.worktreeRemoved", { value: label });
    const nextFailures = { ...worktreeRemoveFailureByPath.value };
    delete nextFailures[worktreePath];
    worktreeRemoveFailureByPath.value = nextFailures;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    worktreeRemoveError.value = message;
    worktreeRemoveFailureByPath.value = {
      ...worktreeRemoveFailureByPath.value,
      [worktreePath]: message
    };
  } finally {
    removingWorktreePath.value = null;
  }
}

function worktreeStateTone(
  state: WorktreeLifecycleState
) {
  if (state === "remove_failed" || state === "unreachable") {
    return "danger" as const;
  }
  if (state === "detached" || state === "inventory_missing" || state === "offline") {
    return "warning" as const;
  }
  if (state === "current" || state === "available") {
    return "success" as const;
  }
  return "muted" as const;
}

async function restoreConversationFromRoute() {
  const routeConversationId =
    typeof route.query.conversationId === "string" && route.query.conversationId.trim()
      ? route.query.conversationId.trim()
      : null;
  if (!routeConversationId) {
    return;
  }

  const match = conversations.value.find(
    (conversation) => conversation.id === routeConversationId
  );
  if (!match) {
    return;
  }

  if (activeConversationId.value === routeConversationId) {
    return;
  }

  await workspace.selectConversation(routeConversationId);
}

function restoreTabFromRoute() {
  const routeTab =
    typeof route.query.tab === "string" && tabs.some((tab) => tab.id === route.query.tab)
      ? (route.query.tab as ProjectTab)
      : null;
  if (!routeTab) {
    return;
  }

  activeTab.value = routeTab;
}

onMounted(async () => {
  store.markProjectVisited(deviceId.value, workspace.cwd.value);
  await workspace.refreshProject();
  restoreTabFromRoute();
  await restoreConversationFromRoute();
});

watch(
  () => [route.params.deviceId, route.params.projectPath],
  async () => {
    store.markProjectVisited(deviceId.value, workspace.cwd.value);
    await workspace.refreshProject();
    restoreTabFromRoute();
    await restoreConversationFromRoute();
  }
);

watch(
  () => [route.query.conversationId, route.query.tab],
  async () => {
    restoreTabFromRoute();
    await restoreConversationFromRoute();
  }
);
</script>

<template>
  <section class="space-y-5">
    <div
      class="rounded-[1.9rem] border border-white/55 bg-white/80 p-5 shadow-[0_24px_70px_-35px_rgba(15,23,42,0.55)] backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
    >
      <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
        <div class="space-y-3">
          <div class="flex flex-wrap items-center gap-2">
            <StatusBadge>{{ t("workspace.badge") }}</StatusBadge>
            <StatusBadge tone="muted">{{ project?.deviceName ?? deviceId }}</StatusBadge>
            <StatusBadge v-if="project" :tone="projectAvailabilityTone">
              {{ t(`projectCard.availability.${project.availabilityState}`) }}
            </StatusBadge>
          </div>
          <div>
            <h1 class="text-2xl font-semibold">{{ project?.title ?? t("workspace.title") }}</h1>
            <p class="mt-2 text-sm text-muted-foreground">
              {{ project?.pathLabel ?? t("workspace.loadingPath") }}
            </p>
          </div>
          <div class="flex flex-wrap gap-3 text-xs text-muted-foreground">
            <span>{{ t("workspace.metrics.topics", { count: project?.conversationCount ?? 0 }) }}</span>
            <span>{{ t("workspace.metrics.running", { count: project?.runningTaskCount ?? 0 }) }}</span>
            <span>{{ t("workspace.metrics.waiting", { count: project?.waitingInputCount ?? 0 }) }}</span>
            <span v-if="project?.branchName">{{ t("workspace.metrics.branch", { value: project.branchName }) }}</span>
            <span
              v-if="gitInspect?.worktrees.length"
            >
              {{
                t("workspace.metrics.worktrees", {
                  count: gitInspect.worktrees.length,
                  current: currentWorktree?.path ?? project?.pathLabel ?? t("workspace.title")
                })
              }}
            </span>
            <span>{{ t("workspace.metrics.changedFiles", { count: project?.changedFilesCount ?? 0 }) }}</span>
            <span>{{ t("workspace.metrics.updated", { value: formatRelativeTime(project?.updatedAtEpochMs) }) }}</span>
          </div>
        </div>

        <div class="grid gap-3 text-sm xl:min-w-[260px]">
          <article class="rounded-2xl bg-background/75 p-4">
            <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">{{ t("workspace.currentState") }}</p>
            <p class="mt-2 font-semibold">
              {{
                project?.failedTaskCount
                  ? t("workspace.state.failed")
                  : project?.waitingInputCount
                    ? t("workspace.state.waiting")
                    : project?.runningTaskCount
                      ? t("workspace.state.running")
                      : t("workspace.state.ready")
              }}
            </p>
          </article>
          <button class="rounded-full border border-border px-4 py-2 text-sm font-medium" @click="workspace.refreshProject">
            {{ t("common.refresh") }}
          </button>
        </div>
      </div>
    </div>

    <div class="flex flex-wrap gap-2 xl:hidden">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="rounded-full border px-4 py-2 text-sm"
        :class="
          activeTab === tab.id
            ? 'border-primary bg-primary text-primary-foreground'
            : 'border-border bg-background/70'
        "
        @click="activeTab = tab.id"
      >
        {{ t(tab.key) }}
      </button>
    </div>

    <div
      v-if="errorMessage"
      class="rounded-[1.5rem] border border-rose-500/20 bg-rose-500/8 p-4 text-sm text-rose-700 dark:text-rose-300"
    >
      {{ errorMessage }}
    </div>

    <div class="hidden gap-5 xl:grid xl:grid-cols-[280px_minmax(0,1fr)_420px]">
      <aside
        class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
      >
        <div class="space-y-4">
          <section>
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("workspace.desktop.hostTree") }}
            </p>
            <div class="mt-3 space-y-3">
              <section
                v-for="host in desktopHostTree"
                :key="host.device.id"
                class="rounded-2xl border border-border bg-background/60 p-3"
              >
                <div class="flex items-center justify-between gap-3">
                  <div class="min-w-0">
                    <p class="truncate text-sm font-semibold text-foreground">
                      {{ host.device.name }}
                    </p>
                    <p class="truncate text-xs text-muted-foreground">
                      {{ host.device.platform }} · {{ t("common.projectsCount", { count: host.projectCount }) }}
                    </p>
                  </div>
                  <StatusBadge :tone="host.device.online ? 'success' : 'muted'">
                    {{ host.device.online ? t("common.online") : t("common.offline") }}
                  </StatusBadge>
                </div>

                <div v-if="host.projectTree.length" class="mt-3 space-y-2">
                  <DesktopProjectTreeNode
                    v-for="entry in host.projectTree"
                    :key="entry.id"
                    :node="entry"
                    :current-project-key="project?.key"
                  />
                </div>
                <p v-else class="mt-3 text-xs text-muted-foreground">
                  {{ t("workspace.desktop.hostEmpty") }}
                </p>
              </section>
            </div>
          </section>

          <section class="rounded-2xl border border-border bg-background/60 p-3">
            <div class="flex items-center justify-between gap-3">
              <div>
                <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("workspace.desktop.worktreeTitle") }}
                </p>
                <p class="mt-1 text-xs text-muted-foreground">
                  {{ t("workspace.desktop.worktreeSummary") }}
                </p>
              </div>
              <StatusBadge>{{ gitInspect?.worktrees.length ?? 0 }}</StatusBadge>
            </div>

            <label class="mt-4 block text-xs font-medium text-muted-foreground">
              {{ t("workspace.desktop.worktreeBranch") }}
            </label>
            <input
              v-model="worktreeBranchName"
              class="mt-1 w-full rounded-2xl border border-border bg-background/80 px-3 py-2 text-sm outline-none transition focus:border-primary"
              :placeholder="t('workspace.desktop.worktreeBranchPlaceholder')"
            />

            <label class="mt-3 block text-xs font-medium text-muted-foreground">
              {{ t("workspace.desktop.worktreeDirectory") }}
            </label>
            <input
              :value="worktreeDirectoryName"
              class="mt-1 w-full rounded-2xl border border-border bg-background/80 px-3 py-2 text-sm outline-none transition focus:border-primary"
              :placeholder="suggestedWorktreeDirectory"
              @input="onWorktreeDirectoryInput(($event.target as HTMLInputElement).value)"
            />

            <p class="mt-2 text-xs text-muted-foreground">
              {{ t("workspace.desktop.worktreeDestinationHint", { value: worktreeDirectoryName || suggestedWorktreeDirectory }) }}
            </p>

            <div
              v-if="worktreeCreateError"
              class="mt-3 rounded-2xl border border-rose-500/25 bg-rose-500/10 px-3 py-2 text-xs text-rose-700 dark:text-rose-300"
            >
              {{ worktreeCreateError }}
            </div>

            <div
              v-else-if="worktreeCreateSuccess"
              class="mt-3 rounded-2xl border border-emerald-500/25 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-700 dark:text-emerald-300"
            >
              {{ worktreeCreateSuccess }}
            </div>

            <div
              v-if="worktreeRemoveError"
              class="mt-3 rounded-2xl border border-rose-500/25 bg-rose-500/10 px-3 py-2 text-xs text-rose-700 dark:text-rose-300"
            >
              {{ worktreeRemoveError }}
            </div>

            <div
              v-else-if="worktreeRemoveSuccess"
              class="mt-3 rounded-2xl border border-emerald-500/25 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-700 dark:text-emerald-300"
            >
              {{ worktreeRemoveSuccess }}
            </div>

            <button
              class="mt-4 w-full rounded-full border border-primary bg-primary px-4 py-2 text-sm font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-60"
              :disabled="isCreatingWorktree"
              @click="createDesktopWorktree"
            >
              {{
                isCreatingWorktree
                  ? t("workspace.desktop.worktreeCreating")
                  : t("workspace.desktop.worktreeSubmit")
              }}
            </button>

            <div v-if="worktreeLifecycleItems.length" class="mt-4 space-y-2">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("workspace.desktop.worktreeList") }}
              </p>
              <template v-for="entry in worktreeLifecycleItems" :key="entry.path">
                <div
                  v-if="entry.project"
                  class="rounded-2xl border px-3 py-3 text-sm"
                  :class="
                    entry.isCurrent || entry.project.key === project?.key
                      ? 'border-primary bg-primary/10'
                      : 'border-border bg-background/75'
                  "
                >
                  <div class="flex items-start justify-between gap-3">
                    <RouterLink
                      :to="{
                        name: 'project-workspace',
                        params: {
                          deviceId,
                          projectPath: buildProjectRouteParam(entry.project.cwd)
                        }
                      }"
                      class="min-w-0 flex-1"
                    >
                      <p class="truncate font-medium text-foreground">
                        {{ entry.project.title }}
                      </p>
                      <p class="mt-1 truncate text-xs text-muted-foreground">
                        {{ entry.path }}
                      </p>
                    </RouterLink>
                    <div class="flex flex-col items-end gap-2">
                      <StatusBadge :tone="worktreeStateTone(entry.lifecycleState)">
                        {{ t(`workspace.desktop.worktreeStates.${entry.lifecycleState}`) }}
                      </StatusBadge>
                      <button
                        v-if="!entry.isCurrent"
                        class="rounded-full border border-rose-500/30 bg-rose-500/10 px-3 py-1 text-xs font-medium text-rose-700 disabled:cursor-not-allowed disabled:opacity-60 dark:text-rose-300"
                        :disabled="removingWorktreePath === entry.path"
                        @click="removeDesktopWorktree(entry.path, entry.project.title)"
                      >
                        {{
                          removingWorktreePath === entry.path
                            ? t("workspace.desktop.worktreeRemoving")
                            : t("workspace.desktop.worktreeRemove")
                        }}
                      </button>
                    </div>
                  </div>
                  <p
                    v-if="entry.removeFailure"
                    class="mt-3 rounded-2xl border border-rose-500/25 bg-rose-500/10 px-3 py-2 text-xs text-rose-700 dark:text-rose-300"
                  >
                    {{ entry.removeFailure }}
                  </p>
                </div>

                <div
                  v-else
                  class="rounded-2xl border border-dashed border-border bg-background/45 px-3 py-3 text-sm"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <p class="truncate font-medium text-foreground">
                        {{ entry.branchName || t("workspace.desktop.worktreeDetached") }}
                      </p>
                      <p class="mt-1 truncate text-xs text-muted-foreground">
                        {{ entry.path }}
                      </p>
                      <div class="mt-2 flex flex-wrap gap-2">
                        <StatusBadge :tone="worktreeStateTone(entry.lifecycleState)">
                          {{ t(`workspace.desktop.worktreeStates.${entry.lifecycleState}`) }}
                        </StatusBadge>
                      </div>
                    </div>
                    <button
                      v-if="!entry.isCurrent"
                      class="rounded-full border border-rose-500/30 bg-rose-500/10 px-3 py-1 text-xs font-medium text-rose-700 disabled:cursor-not-allowed disabled:opacity-60 dark:text-rose-300"
                      :disabled="removingWorktreePath === entry.path"
                      @click="removeDesktopWorktree(entry.path, entry.branchName || entry.path)"
                    >
                      {{
                        removingWorktreePath === entry.path
                          ? t("workspace.desktop.worktreeRemoving")
                          : t("workspace.desktop.worktreeRemove")
                      }}
                    </button>
                  </div>
                  <p
                    v-if="entry.removeFailure"
                    class="mt-3 rounded-2xl border border-rose-500/25 bg-rose-500/10 px-3 py-2 text-xs text-rose-700 dark:text-rose-300"
                  >
                    {{ entry.removeFailure }}
                  </p>
                </div>
              </template>
            </div>
          </section>

          <section>
            <div class="flex items-center justify-between gap-3">
              <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("conversation.topics") }}
              </p>
              <StatusBadge>{{ conversations.length }}</StatusBadge>
            </div>
            <div class="mt-3 space-y-2">
              <button
                v-for="conversation in conversations"
                :key="conversation.id"
                class="w-full rounded-2xl border px-3 py-3 text-left"
                :class="
                  conversation.id === activeConversationId
                    ? 'border-primary bg-primary/10'
                    : 'border-border bg-background/70'
                "
                @click="workspace.selectConversation(conversation.id)"
              >
                <p class="truncate text-sm font-medium text-foreground">{{ conversation.title }}</p>
                <p class="mt-1 text-xs text-muted-foreground">
                  {{ formatRelativeTime(conversation.updatedAtEpochMs) }}
                </p>
              </button>
            </div>
          </section>
        </div>
      </aside>

      <ProjectConversationPanel
        :detail="conversationDetail"
        :conversations="conversations"
        :active-conversation-id="activeConversationId"
        :project-providers="project?.providers"
        :project-title="project?.title"
        @select-conversation="workspace.selectConversation"
        @send-prompt="workspace.sendFollowUp"
        @respond-input="workspace.respondToInput"
        @cancel-task="workspace.cancelTask"
        @open-tab="activeTab = $event"
      />

      <section class="space-y-4">
        <div class="flex flex-wrap gap-2">
          <button
            v-for="tab in tabs.filter((entry) => entry.id !== 'conversation')"
            :key="tab.id"
            class="rounded-full border px-4 py-2 text-sm"
            :class="
              activeTab === tab.id
                ? 'border-primary bg-primary text-primary-foreground'
                : 'border-border bg-background/70'
            "
            @click="activeTab = tab.id"
          >
            {{ t(tab.key) }}
          </button>
        </div>

        <ProjectChangesPanel
          v-if="activeTab === 'changes'"
          :git-inspect="gitInspect"
          :git-diff="gitDiff"
          :active-repo-path="activeDiffRepoPath"
          @select-file="workspace.selectChangeFile"
        />

        <ProjectFilesPanel
          v-else-if="activeTab === 'files'"
          :workspace="workspaceBrowse"
          :file-preview="filePreview"
          @open-entry="workspace.openEntry"
        />

        <ProjectLogsPanel v-else :detail="conversationDetail" :audit-events="auditEvents" />
      </section>
    </div>

    <template v-if="activeTab === 'conversation'">
      <ProjectConversationPanel
        :detail="conversationDetail"
        :conversations="conversations"
        :active-conversation-id="activeConversationId"
        :project-providers="project?.providers"
        :project-title="project?.title"
        class="xl:hidden"
        @select-conversation="workspace.selectConversation"
        @send-prompt="workspace.sendFollowUp"
        @respond-input="workspace.respondToInput"
        @cancel-task="workspace.cancelTask"
        @open-tab="activeTab = $event"
      />
    </template>

    <ProjectChangesPanel
      v-else-if="activeTab === 'changes'"
      class="xl:hidden"
      :git-inspect="gitInspect"
      :git-diff="gitDiff"
      :active-repo-path="activeDiffRepoPath"
      @select-file="workspace.selectChangeFile"
    />

    <ProjectFilesPanel
      v-else-if="activeTab === 'files'"
      class="xl:hidden"
      :workspace="workspaceBrowse"
      :file-preview="filePreview"
      @open-entry="workspace.openEntry"
    />

    <ProjectLogsPanel v-else class="xl:hidden" :detail="conversationDetail" :audit-events="auditEvents" />
  </section>
</template>
