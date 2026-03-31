import { computed, ref, toValue, watch, type MaybeRefOrGetter } from "vue";
import {
  browseWorkspace,
  fetchGitDiffFile,
  inspectGitWorkspace,
  previewWorkspaceFile
} from "@/lib/api";
import { preferredProjectProvider } from "@/lib/policy";
import { parseProjectRouteParam } from "@/lib/projects";
import { useAppStore } from "@/stores/app";
import type {
  ConversationDetailResponse,
  GitDiffFileResponse,
  GitInspectResponse,
  ProviderKind,
  TaskExecutionMode,
  WorkspaceBrowseResponse,
  WorkspaceFilePreviewResponse
} from "@/types";

export type ProjectTab = "conversation" | "changes" | "files";

export function useProjectWorkspace(
  deviceIdSource: MaybeRefOrGetter<string>,
  projectPathSource: MaybeRefOrGetter<string | string[] | undefined>,
  providerSource?: MaybeRefOrGetter<ProviderKind | null | undefined>,
  enabledSource?: MaybeRefOrGetter<boolean>
) {
  const store = useAppStore();
  const deviceId = computed(() => toValue(deviceIdSource));
  const cwd = computed(() => parseProjectRouteParam(toValue(projectPathSource)));
  const provider = computed(() => toValue(providerSource));
  const enabled = computed(() => toValue(enabledSource) ?? true);
  const project = computed(() => store.findProject(deviceId.value, cwd.value));
  const conversations = computed(() =>
    enabled.value ? store.listProjectConversations(deviceId.value, cwd.value, provider.value) : []
  );
  const tasks = computed(() =>
    enabled.value ? store.listProjectTasks(deviceId.value, cwd.value, provider.value) : []
  );
  const activeConversationId = ref<string | null>(null);
  const conversationDetail = ref<ConversationDetailResponse | null>(null);
  const gitInspect = ref<GitInspectResponse | null>(null);
  const gitDiff = ref<GitDiffFileResponse | null>(null);
  const activeDiffRepoPath = ref<string | null>(null);
  const workspace = ref<WorkspaceBrowseResponse | null>(null);
  const filePreview = ref<WorkspaceFilePreviewResponse | null>(null);
  const browserPath = ref<string>("");
  const activeTab = ref<ProjectTab>("conversation");
  const isDraftConversation = ref(false);
  const isLoading = ref(false);
  const errorMessage = ref("");

  watch(
    conversations,
    (value) => {
      if (isDraftConversation.value) {
        conversationDetail.value = null;
        return;
      }

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
    if (!enabled.value || !deviceId.value || !cwd.value) {
      conversationDetail.value = null;
      gitInspect.value = null;
      gitDiff.value = null;
      activeDiffRepoPath.value = null;
      workspace.value = null;
      filePreview.value = null;
      errorMessage.value = "";
      return;
    }

    isLoading.value = true;
    errorMessage.value = "";
    console.info("[vibe-app] project refresh start", {
      deviceId: deviceId.value,
      cwd: cwd.value
    });

    try {
      await store.refreshAll();

      if (activeConversationId.value) {
        await loadConversationContext(activeConversationId.value);
      } else {
        conversationDetail.value = null;
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
      console.info("[vibe-app] project refresh success", {
        deviceId: deviceId.value,
        cwd: cwd.value,
        conversations: conversations.value.length,
        changedFiles: gitInspect.value?.changedFiles.length ?? 0,
        entries: workspace.value?.entries.length ?? 0
      });
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : String(error);
      console.error("[vibe-app] project refresh failed", error);
    } finally {
      isLoading.value = false;
    }
  }

  async function selectConversation(conversationId: string) {
    isDraftConversation.value = false;
    activeConversationId.value = conversationId;
    await loadConversationContext(conversationId);
  }

  async function loadConversationContext(conversationId: string) {
    conversationDetail.value = await store.loadConversation(conversationId);
  }

  function startNewConversation() {
    isDraftConversation.value = true;
    activeConversationId.value = null;
    conversationDetail.value = null;
    activeTab.value = "conversation";
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
    const preferredProvider = preferredProjectProvider(project.value?.providers);
    const response = await store.createProjectConversation({
      deviceId: deviceId.value,
      provider: provider.value ?? preferredProvider,
      executionMode,
      prompt,
      cwd: cwd.value ?? undefined,
      model: model || undefined,
      title: prompt.slice(0, 60)
    });
    isDraftConversation.value = false;
    activeConversationId.value = response.conversation.id;
    store.markProjectVisited(deviceId.value, cwd.value);
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

    isDraftConversation.value = false;
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
      return;
    }

    await loadConversationContext(value);
  }, { immediate: true });

  watch(enabled, async (value) => {
    if (!value) {
      conversationDetail.value = null;
      gitInspect.value = null;
      gitDiff.value = null;
      activeDiffRepoPath.value = null;
      workspace.value = null;
      filePreview.value = null;
      activeConversationId.value = null;
      return;
    }

    await refreshProject();
  }, { immediate: false });

  return {
    cwd,
    project,
    tasks,
    conversations,
    activeConversationId,
    isDraftConversation,
    conversationDetail,
    gitInspect,
    gitDiff,
    activeDiffRepoPath,
    workspace,
    filePreview,
    activeTab,
    isLoading,
    errorMessage,
    refreshProject,
    startNewConversation,
    selectChangeFile,
    selectConversation,
    openEntry,
    createTopic,
    sendFollowUp,
    respondToInput,
    cancelTask
  };
}
