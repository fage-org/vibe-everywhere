import { computed, ref, toValue, watch, type MaybeRefOrGetter } from "vue";
import {
  browseWorkspace,
  createGitWorktree,
  fetchAuditEvents,
  fetchGitDiffFile,
  inspectGitWorkspace,
  removeGitWorktree,
  previewWorkspaceFile
} from "@/lib/api";
import { preferredProjectProvider } from "@/lib/policy";
import { parseProjectRouteParam } from "@/lib/projects";
import { useAppStore } from "@/stores/app";
import type {
  AuditRecord,
  ConversationDetailResponse,
  GitDiffFileResponse,
  GitInspectResponse,
  TaskExecutionMode,
  WorkspaceBrowseResponse,
  WorkspaceFilePreviewResponse
} from "@/types";

export type ProjectTab = "conversation" | "changes" | "files" | "logs";

export function useProjectWorkspace(
  deviceIdSource: MaybeRefOrGetter<string>,
  projectPathSource: MaybeRefOrGetter<string | string[] | undefined>
) {
  const store = useAppStore();
  const deviceId = computed(() => toValue(deviceIdSource));
  const cwd = computed(() => parseProjectRouteParam(toValue(projectPathSource)));
  const project = computed(() => store.findProject(deviceId.value, cwd.value));
  const conversations = computed(() => store.listProjectConversations(deviceId.value, cwd.value));
  const tasks = computed(() => store.listProjectTasks(deviceId.value, cwd.value));
  const activeConversationId = ref<string | null>(null);
  const conversationDetail = ref<ConversationDetailResponse | null>(null);
  const auditEvents = ref<AuditRecord[]>([]);
  const gitInspect = ref<GitInspectResponse | null>(null);
  const gitDiff = ref<GitDiffFileResponse | null>(null);
  const activeDiffRepoPath = ref<string | null>(null);
  const workspace = ref<WorkspaceBrowseResponse | null>(null);
  const filePreview = ref<WorkspaceFilePreviewResponse | null>(null);
  const browserPath = ref<string>("");
  const activeTab = ref<ProjectTab>("conversation");
  const isLoading = ref(false);
  const errorMessage = ref("");

  watch(
    conversations,
    (value) => {
      if (!value.length) {
        activeConversationId.value = null;
        conversationDetail.value = null;
        return;
      }

      if (!activeConversationId.value || !value.some((entry) => entry.id === activeConversationId.value)) {
        activeConversationId.value = value[0].id;
      }
    },
    { immediate: true }
  );

  async function refreshProject() {
    isLoading.value = true;
    errorMessage.value = "";

    try {
      await store.refreshAll();

      if (activeConversationId.value) {
        await loadConversationContext(activeConversationId.value);
      } else {
        conversationDetail.value = null;
        auditEvents.value = [];
      }

      gitInspect.value = await inspectGitWorkspace(
        store.relayBaseUrl,
        {
          deviceId: deviceId.value,
          sessionCwd: cwd.value ?? undefined
        },
        store.relayAccessToken
      );
      if (gitInspect.value?.changedFiles.length) {
        const selectedRepoPath = gitInspect.value.changedFiles.some(
          (file) => file.repoPath === activeDiffRepoPath.value
        )
          ? activeDiffRepoPath.value
          : gitInspect.value.changedFiles[0]?.repoPath;
        if (selectedRepoPath) {
          await selectChangeFile(selectedRepoPath);
        }
      } else {
        activeDiffRepoPath.value = null;
        gitDiff.value = null;
      }

      workspace.value = await browseWorkspace(
        store.relayBaseUrl,
        {
          deviceId: deviceId.value,
          sessionCwd: cwd.value ?? undefined,
          path: browserPath.value || undefined
        },
        store.relayAccessToken
      );
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
    } finally {
      isLoading.value = false;
    }
  }

  async function selectConversation(conversationId: string) {
    activeConversationId.value = conversationId;
    await loadConversationContext(conversationId);
  }

  async function loadConversationContext(conversationId: string) {
    conversationDetail.value = await store.loadConversation(conversationId);
    const taskIds = new Set(conversationDetail.value.tasks.map((entry) => entry.task.id));
    auditEvents.value = (
      await fetchAuditEvents(store.relayBaseUrl, store.relayAccessToken, { limit: 200 })
    ).filter(
      (record) =>
        (record.resourceKind === "task" && taskIds.has(record.resourceId)) ||
        (record.resourceKind === "conversation" &&
          record.resourceId === conversationDetail.value?.conversation.id)
    );
  }

  async function openEntry(path: string, kind: "directory" | "file") {
    if (kind === "directory") {
      browserPath.value = path;
      workspace.value = await browseWorkspace(
        store.relayBaseUrl,
        {
          deviceId: deviceId.value,
          sessionCwd: cwd.value ?? undefined,
          path
        },
        store.relayAccessToken
      );
      return;
    }

    filePreview.value = await previewWorkspaceFile(
      store.relayBaseUrl,
      {
        deviceId: deviceId.value,
        sessionCwd: cwd.value ?? undefined,
        path
      },
      store.relayAccessToken
    );
  }

  async function selectChangeFile(repoPath: string) {
    activeDiffRepoPath.value = repoPath;
    gitDiff.value = await fetchGitDiffFile(
      store.relayBaseUrl,
      {
        deviceId: deviceId.value,
        sessionCwd: cwd.value ?? undefined,
        repoPath
      },
      store.relayAccessToken
    );
  }

  async function createTopic(prompt: string, model?: string, executionMode?: TaskExecutionMode) {
    const provider = preferredProjectProvider(project.value?.providers);
    const response = await store.createProjectConversation({
      deviceId: deviceId.value,
      provider,
      executionMode,
      prompt,
      cwd: cwd.value ?? undefined,
      model: model || undefined,
      title: prompt.slice(0, 60)
    });
    activeConversationId.value = response.conversation.id;
    store.markProjectVisited(deviceId.value, cwd.value);
    await refreshProject();
  }

  async function createWorktree(branchName: string, destinationPath: string) {
    await createGitWorktree(
      store.relayBaseUrl,
      {
        deviceId: deviceId.value,
        sessionCwd: cwd.value ?? undefined,
        branchName,
        destinationPath
      },
      store.relayAccessToken
    );
    await store.refreshAll(true);
    await refreshProject();
  }

  async function removeWorktree(worktreePath: string) {
    await removeGitWorktree(
      store.relayBaseUrl,
      {
        deviceId: deviceId.value,
        sessionCwd: cwd.value ?? undefined,
        worktreePath
      },
      store.relayAccessToken
    );
    store.suppressProject(deviceId.value, worktreePath);
    await store.refreshAll(true);
    await refreshProject();
  }

  async function sendFollowUp(
    prompt: string,
    model?: string,
    executionMode?: TaskExecutionMode
  ) {
    if (!activeConversationId.value) {
      await createTopic(prompt, model, executionMode);
      return;
    }

    await store.sendProjectMessage(activeConversationId.value, {
      prompt,
      executionMode,
      model: model || undefined
    });
    await refreshProject();
  }

  async function respondToInput(optionId?: string, text?: string) {
    const request = conversationDetail.value?.pendingInputRequest;
    if (!request) {
      return;
    }

    await store.respondToInput(request.taskId, request.id, optionId, text);
    await refreshProject();
  }

  async function cancelTask(taskId: string) {
    await store.cancelProjectTask(taskId);
    await refreshProject();
  }

  watch(activeConversationId, async (value) => {
    if (!value) {
      auditEvents.value = [];
      return;
    }

    await loadConversationContext(value);
  }, { immediate: true });

  return {
    cwd,
    project,
    tasks,
    conversations,
    activeConversationId,
    conversationDetail,
    auditEvents,
    gitInspect,
    gitDiff,
    activeDiffRepoPath,
    workspace,
    filePreview,
    activeTab,
    isLoading,
    errorMessage,
    refreshProject,
    selectChangeFile,
    selectConversation,
    openEntry,
    createWorktree,
    removeWorktree,
    createTopic,
    sendFollowUp,
    respondToInput,
    cancelTask
  };
}
