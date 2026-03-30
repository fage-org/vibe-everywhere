import { defineStore } from "pinia";
import {
  archiveConversation,
  browseWorkspace,
  cancelTask,
  createConversation,
  fetchAppConfig,
  fetchConversationDetail,
  fetchConversations,
  fetchDevices,
  fetchHealth,
  fetchTasks,
  inspectGitWorkspace,
  respondTaskInputRequest,
  sendConversationMessage
} from "@/lib/api";
import {
  buildProjectKey,
  deriveHostSummaries,
  deriveProjectSummaries,
  inferProjectTitle,
  type DiscoveredProjectRecord,
  type ProjectDiscoverySource,
  type ProjectSummary
} from "@/lib/projects";
import {
  clearRediscoveredHiddenProjectKeys,
  filterVisibleProjectKeys,
  suppressProjectKey
} from "@/lib/projectInventory";
import {
  loadStoredProjectFolder,
  normalizeRelayBaseUrl,
  persistProjectFolder,
  persistRelayAccessToken,
  persistRelayBaseUrl,
  resolveInitialRelayAccessToken,
  resolveInitialRelayBaseUrl
} from "@/lib/runtime";
import type {
  AppConfig,
  ConversationDetailResponse,
  ConversationRecord,
  CreateConversationPayload,
  DeviceRecord,
  GitInspectResponse,
  SendConversationMessagePayload,
  ServiceHealth,
  TaskExecutionMode,
  TaskRecord
} from "@/types";

type NotificationItem = {
  id: string;
  kind: "waiting_input" | "failed" | "completed";
  targetTab: "conversation" | "changes" | "logs";
  title: string;
  summary: string;
  deviceId: string;
  conversationId: string | null;
  taskId: string;
  cwd: string | null;
  timestampEpochMs: number;
  seen: boolean;
};

const RECENT_PROJECTS_STORAGE_KEY = "vibe.everywhere.recentProjects";
const NOTIFICATION_PREFS_STORAGE_KEY = "vibe.everywhere.notificationPrefs";
const NOTIFICATION_DEFAULT_PREF_STORAGE_KEY = "vibe.everywhere.notificationDefaultPref";
const NOTIFICATION_SEEN_STORAGE_KEY = "vibe.everywhere.notificationSeen";
const DEFAULT_EXECUTION_MODE_STORAGE_KEY = "vibe.everywhere.defaultExecutionMode";
const SENSITIVE_CONFIRM_STORAGE_KEY = "vibe.everywhere.sensitiveConfirmEnabled";
const HIDDEN_PROJECT_KEYS_STORAGE_KEY = "vibe.everywhere.hiddenProjectKeys";
const AUTO_REFRESH_MS = 10_000;
const PROJECT_DISCOVERY_TTL_MS = 120_000;
const IGNORED_PROJECT_DIRS = new Set([
  ".git",
  ".vibe-agent",
  "node_modules",
  "target",
  "dist",
  "build",
  ".next",
  ".turbo",
  ".venv",
  "venv"
]);

function canPromoteGitInspectToProject(git: GitInspectResponse) {
  return git.state === "ready" && Boolean(git.repoRoot) && git.repoRoot === git.workspaceRoot;
}

function buildDiscoveredProjectRecord(
  device: DeviceRecord,
  providers: DeviceRecord["providers"][number]["kind"][],
  git: GitInspectResponse,
  discoverySource: ProjectDiscoverySource
): DiscoveredProjectRecord {
  return {
    deviceId: device.id,
    cwd: git.workspaceRoot,
    repoRoot: git.repoRoot,
    repoCommonDir: git.repoCommonDir,
    pathLabel: git.workspaceRoot,
    title: inferProjectTitle(git.workspaceRoot),
    updatedAtEpochMs: git.recentCommits[0]?.committedAtEpochMs ?? 0,
    branchName: git.branchName,
    changedFilesCount: git.diffStats.changedFiles,
    providers,
    discoverySource,
    lastVerifiedAtEpochMs: Date.now(),
    availabilityState: "available"
  };
}

function updateProjectAvailability(
  project: DiscoveredProjectRecord,
  availabilityState: DiscoveredProjectRecord["availabilityState"]
): DiscoveredProjectRecord {
  return {
    ...project,
    availabilityState
  };
}

export const useAppStore = defineStore("app", {
  state: () => ({
    relayBaseUrl: "",
    relayBaseUrlInput: "",
    relayAccessToken: "",
    relayAccessTokenInput: "",
    appConfig: null as AppConfig | null,
    health: null as ServiceHealth | null,
    devices: [] as DeviceRecord[],
    conversations: [] as ConversationRecord[],
    tasks: [] as TaskRecord[],
    discoveredProjects: [] as DiscoveredProjectRecord[],
    projectInventoryUpdatedAtByDevice: {} as Record<string, number>,
    isBootstrapping: false,
    isRefreshing: false,
    errorMessage: "",
    lastRefreshEpochMs: 0,
    recentProjectKeys: [] as string[],
    hiddenProjectKeys: [] as string[],
    defaultExecutionMode: "workspace_write" as TaskExecutionMode,
    sensitiveConfirmationEnabled: true,
    defaultNotificationPreference: "important" as "all" | "important",
    notificationPreferenceByProjectKey: {} as Record<string, "all" | "important">,
    seenNotificationIds: [] as string[],
    refreshTimer: null as number | null
  }),
  getters: {
    hasRelayConfig(state) {
      return Boolean(state.relayBaseUrl.trim());
    },
    activeServerLabel(state) {
      return state.appConfig?.deployment.displayName ?? "Current server";
    },
    onlineHostCount(state) {
      return state.devices.filter((device) => device.online).length;
    },
    runningTaskCount(state) {
      return state.tasks.filter((task) => task.status === "running").length;
    },
    attentionCount(state) {
      return state.tasks.filter(
        (task) => task.status === "waiting_input" || task.status === "failed"
      ).length;
    },
    projectSummaries(state): ProjectSummary[] {
      return filterVisibleProjectKeys(
        deriveProjectSummaries(
          state.devices,
          state.conversations,
          state.tasks,
          state.discoveredProjects
        ),
        state.hiddenProjectKeys
      );
    },
    hostSummaries(state) {
      const visibleProjects = filterVisibleProjectKeys(
        deriveProjectSummaries(
          state.devices,
          state.conversations,
          state.tasks,
          state.discoveredProjects
        ),
        state.hiddenProjectKeys
      );
      return deriveHostSummaries(
        state.devices,
        visibleProjects
      );
    },
    recentProjects(state): ProjectSummary[] {
      const summaries = filterVisibleProjectKeys(
        deriveProjectSummaries(
          state.devices,
          state.conversations,
          state.tasks,
          state.discoveredProjects
        ),
        state.hiddenProjectKeys
      );
      const byKey = new Map(summaries.map((project) => [project.key, project]));
      return state.recentProjectKeys
        .map((key) => byKey.get(key))
        .filter((value): value is ProjectSummary => Boolean(value));
    },
    runningProjects(state): ProjectSummary[] {
      return filterVisibleProjectKeys(
        deriveProjectSummaries(
          state.devices,
          state.conversations,
          state.tasks,
          state.discoveredProjects
        ),
        state.hiddenProjectKeys
      ).filter((project) => project.runningTaskCount > 0);
    },
    reviewProjects(state): ProjectSummary[] {
      return filterVisibleProjectKeys(
        deriveProjectSummaries(
          state.devices,
          state.conversations,
          state.tasks,
          state.discoveredProjects
        ),
        state.hiddenProjectKeys
      ).filter((project) => project.failedTaskCount > 0 || project.waitingInputCount > 0);
    },
    notificationItems(state): NotificationItem[] {
      return [...state.tasks]
        .filter(
          (task) => {
            const preference =
              state.notificationPreferenceByProjectKey[
                buildProjectKey(task.deviceId, task.cwd)
              ] ?? state.defaultNotificationPreference;
            if (task.status === "waiting_input" || task.status === "failed") {
              return true;
            }
            return preference === "all" && task.status === "succeeded";
          }
        )
        .sort((left, right) => {
          const leftTime = left.finishedAtEpochMs ?? left.startedAtEpochMs ?? left.createdAtEpochMs;
          const rightTime =
            right.finishedAtEpochMs ?? right.startedAtEpochMs ?? right.createdAtEpochMs;
          return rightTime - leftTime;
        })
        .slice(0, 20)
        .map((task) => ({
          id: task.id,
          kind:
            task.status === "waiting_input"
              ? "waiting_input"
              : task.status === "failed"
                ? "failed"
                : "completed",
          targetTab:
            task.status === "waiting_input"
              ? "conversation"
              : task.status === "failed"
                ? "logs"
                : "changes",
          title: task.title || task.prompt.slice(0, 40),
          summary:
            task.status === "waiting_input"
              ? "Provider needs your reply."
              : task.status === "failed"
                ? task.error || "Task failed."
                : "Task completed and is ready for review.",
          deviceId: task.deviceId,
          conversationId: task.conversationId,
          taskId: task.id,
          cwd: task.cwd,
          seen: state.seenNotificationIds.includes(task.id),
          timestampEpochMs:
            task.finishedAtEpochMs ?? task.startedAtEpochMs ?? task.createdAtEpochMs
        }));
    },
    unreadNotificationCount(state): number {
      return state.tasks.filter((task) => {
        const preference =
          state.notificationPreferenceByProjectKey[
            buildProjectKey(task.deviceId, task.cwd)
          ] ?? state.defaultNotificationPreference;
        const visible =
          task.status === "waiting_input" ||
          task.status === "failed" ||
          (preference === "all" && task.status === "succeeded");
        return visible && !state.seenNotificationIds.includes(task.id);
      }).length;
    }
  },
  actions: {
    async bootstrap() {
      this.isBootstrapping = true;
      this.recentProjectKeys = loadRecentProjectKeys();
      this.defaultExecutionMode = loadDefaultExecutionMode();
      this.sensitiveConfirmationEnabled = loadSensitiveConfirmationEnabled();
      this.defaultNotificationPreference = loadDefaultNotificationPreference();
      this.notificationPreferenceByProjectKey = loadNotificationPreferences();
      this.seenNotificationIds = loadSeenNotificationIds();
      this.hiddenProjectKeys = loadHiddenProjectKeys();
      this.relayBaseUrl = await resolveInitialRelayBaseUrl();
      this.relayBaseUrlInput = this.relayBaseUrl;
      this.relayAccessToken = resolveInitialRelayAccessToken();
      this.relayAccessTokenInput = this.relayAccessToken;
      await this.refreshAll(true);
      this.startAutoRefresh();
      this.isBootstrapping = false;
    },
    async saveRelaySettings() {
      this.relayBaseUrl = normalizeRelayBaseUrl(this.relayBaseUrlInput);
      this.relayAccessToken = this.relayAccessTokenInput.trim();
      persistRelayBaseUrl(this.relayBaseUrl);
      persistRelayAccessToken(this.relayAccessToken);
      await this.refreshAll(true);
    },
    async refreshAll(forceProjectDiscovery = false) {
      if (!this.relayBaseUrl.trim()) {
        this.health = null;
        this.devices = [];
        this.conversations = [];
        this.tasks = [];
        this.discoveredProjects = [];
        this.projectInventoryUpdatedAtByDevice = {};
        return;
      }

      this.isRefreshing = true;
      this.errorMessage = "";

      try {
        const [health, appConfig, devices, conversations, tasks] = await Promise.all([
          fetchHealth(this.relayBaseUrl),
          fetchAppConfig(this.relayBaseUrl),
          fetchDevices(this.relayBaseUrl, this.relayAccessToken),
          fetchConversations(this.relayBaseUrl, this.relayAccessToken, { archived: false }),
          fetchTasks(this.relayBaseUrl, this.relayAccessToken, { limit: 200 })
        ]);

        this.health = health;
        this.appConfig = appConfig;
        this.devices = devices;
        this.conversations = conversations;
        this.tasks = tasks;
        await this.refreshProjectInventory(forceProjectDiscovery);
        this.lastRefreshEpochMs = Date.now();
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error);
      } finally {
        this.isRefreshing = false;
      }
    },
    startAutoRefresh() {
      if (this.refreshTimer !== null) {
        return;
      }

      this.refreshTimer = window.setInterval(() => {
        void this.refreshAll(false);
      }, AUTO_REFRESH_MS);
    },
    stopAutoRefresh() {
      if (this.refreshTimer !== null) {
        window.clearInterval(this.refreshTimer);
        this.refreshTimer = null;
      }
    },
    findProject(deviceId: string, cwd: string | null) {
      return (
        this.projectSummaries.find(
          (project) => project.deviceId === deviceId && project.cwd === cwd
        ) ?? null
      );
    },
    listProjectConversations(deviceId: string, cwd: string | null) {
      return this.conversations
        .filter((conversation) => conversation.deviceId === deviceId && conversation.cwd === cwd)
        .sort((left, right) => right.updatedAtEpochMs - left.updatedAtEpochMs);
    },
    listProjectTasks(deviceId: string, cwd: string | null) {
      return this.tasks
        .filter((task) => task.deviceId === deviceId && task.cwd === cwd)
        .sort((left, right) => right.createdAtEpochMs - left.createdAtEpochMs);
    },
    markProjectVisited(deviceId: string, cwd: string | null) {
      const project = this.findProject(deviceId, cwd);
      if (!project) {
        return;
      }

      this.recentProjectKeys = [project.key, ...this.recentProjectKeys.filter((key) => key !== project.key)].slice(0, 8);
      persistRecentProjectKeys(this.recentProjectKeys);
      persistProjectFolder(deviceId, cwd ?? "");
    },
    suppressProject(deviceId: string, cwd: string | null) {
      if (!cwd) {
        return;
      }

      const key = buildProjectKey(deviceId, cwd);
      this.hiddenProjectKeys = suppressProjectKey(this.hiddenProjectKeys, key);
      this.discoveredProjects = this.discoveredProjects.filter(
        (project) => buildProjectKey(project.deviceId, project.cwd) !== key
      );
      this.recentProjectKeys = this.recentProjectKeys.filter((entry) => entry !== key);
      persistRecentProjectKeys(this.recentProjectKeys);
      persistHiddenProjectKeys(this.hiddenProjectKeys);
    },
    notificationPreferenceOverrideForProject(deviceId: string, cwd: string | null) {
      return this.notificationPreferenceByProjectKey[buildProjectKey(deviceId, cwd)] ?? null;
    },
    getPreferredProjectPath(deviceId: string) {
      return loadStoredProjectFolder(deviceId) || null;
    },
    setDefaultExecutionMode(executionMode: TaskExecutionMode) {
      this.defaultExecutionMode = executionMode;
      persistDefaultExecutionMode(executionMode);
    },
    setSensitiveConfirmationEnabled(enabled: boolean) {
      this.sensitiveConfirmationEnabled = enabled;
      persistSensitiveConfirmationEnabled(enabled);
    },
    notificationPreferenceForProject(deviceId: string, cwd: string | null) {
      return (
        this.notificationPreferenceByProjectKey[buildProjectKey(deviceId, cwd)] ??
        this.defaultNotificationPreference
      );
    },
    setDefaultNotificationPreference(preference: "all" | "important") {
      this.defaultNotificationPreference = preference;
      persistDefaultNotificationPreference(preference);
    },
    setNotificationPreference(
      deviceId: string,
      cwd: string | null,
      preference: "all" | "important"
    ) {
      this.notificationPreferenceByProjectKey = {
        ...this.notificationPreferenceByProjectKey,
        [buildProjectKey(deviceId, cwd)]: preference
      };
      persistNotificationPreferences(this.notificationPreferenceByProjectKey);
    },
    clearNotificationPreference(deviceId: string, cwd: string | null) {
      const key = buildProjectKey(deviceId, cwd);
      const nextPreferences = { ...this.notificationPreferenceByProjectKey };
      delete nextPreferences[key];
      this.notificationPreferenceByProjectKey = nextPreferences;
      persistNotificationPreferences(nextPreferences);
    },
    markNotificationSeen(taskId: string) {
      if (this.seenNotificationIds.includes(taskId)) {
        return;
      }
      this.seenNotificationIds = [taskId, ...this.seenNotificationIds].slice(0, 200);
      persistSeenNotificationIds(this.seenNotificationIds);
    },
    async refreshProjectInventory(force = false) {
      const inventory = new Map<string, DiscoveredProjectRecord>();
      const now = Date.now();

      for (const device of this.devices) {
        const lastUpdated = this.projectInventoryUpdatedAtByDevice[device.id] ?? 0;
        const shouldRefresh =
          force ||
          !lastUpdated ||
          now - lastUpdated >= PROJECT_DISCOVERY_TTL_MS;
        if (!shouldRefresh) {
          for (const project of this.discoveredProjects.filter(
            (entry) => entry.deviceId === device.id
          )) {
            inventory.set(`${project.deviceId}::${project.cwd}`, project);
          }
          continue;
        }

        const discovered = await this.discoverProjectsForDevice(device);
        this.projectInventoryUpdatedAtByDevice[device.id] = now;
        for (const project of discovered) {
          inventory.set(`${project.deviceId}::${project.cwd}`, project);
        }
      }

      this.hiddenProjectKeys = clearRediscoveredHiddenProjectKeys(
        this.hiddenProjectKeys,
        inventory.keys()
      );
      persistHiddenProjectKeys(this.hiddenProjectKeys);
      this.discoveredProjects = [...inventory.values()];
    },
    async discoverProjectsForDevice(device: DeviceRecord) {
      const existingProjects = this.discoveredProjects.filter((entry) => entry.deviceId === device.id);
      if (!device.online) {
        return existingProjects.map((entry) => updateProjectAvailability(entry, "offline"));
      }

      const workingRoot =
        device.metadata.workingRoot ??
        device.metadata.workspace_root ??
        device.metadata.working_root ??
        null;
      if (!workingRoot) {
        return existingProjects.map((entry) => updateProjectAvailability(entry, "unreachable"));
      }

      const providers = device.providers
        .filter((provider) => provider.available)
        .map((provider) => provider.kind);
      const candidates = new Map<
        string,
        { sessionCwd?: string; source: ProjectDiscoverySource }
      >();
      const inspectedPaths = new Set<string>();
      candidates.set(workingRoot, { sessionCwd: undefined, source: "working_root" });
      for (const project of existingProjects) {
        candidates.set(project.cwd, {
          sessionCwd: project.cwd,
          source: project.discoverySource ?? "known_project"
        });
      }

      try {
        const rootBrowse = await browseWorkspace(
          this.relayBaseUrl,
          {
            deviceId: device.id
          },
          this.relayAccessToken
        );

        for (const entry of rootBrowse.entries) {
          if (entry.kind !== "directory") {
            continue;
          }
          if (IGNORED_PROJECT_DIRS.has(entry.name)) {
            continue;
          }
          candidates.set(entry.path, {
            sessionCwd: entry.path,
            source: "working_root"
          });
        }
      } catch {
        return existingProjects.map((entry) => updateProjectAvailability(entry, "unreachable"));
      }

      const discovered = new Map<string, DiscoveredProjectRecord>();
      const pendingCandidates = [...candidates.entries()];
      let inspectSucceeded = false;
      while (pendingCandidates.length) {
        const [cwd, candidate] = pendingCandidates.shift()!;
        if (inspectedPaths.has(cwd)) {
          continue;
        }
        inspectedPaths.add(cwd);

        try {
          inspectSucceeded = true;
          const git = await inspectGitWorkspace(
            this.relayBaseUrl,
            {
              deviceId: device.id,
              sessionCwd: candidate.sessionCwd
            },
            this.relayAccessToken
          );

          if (canPromoteGitInspectToProject(git)) {
            discovered.set(
              `${device.id}::${git.workspaceRoot}`,
              buildDiscoveredProjectRecord(device, providers, git, candidate.source)
            );
          }

          for (const worktree of git.worktrees) {
            if (!worktree.path.trim() || inspectedPaths.has(worktree.path)) {
              continue;
            }
            if (!candidates.has(worktree.path)) {
              candidates.set(worktree.path, {
                sessionCwd: worktree.path,
                source: "git_worktree"
              });
              pendingCandidates.push([
                worktree.path,
                {
                  sessionCwd: worktree.path,
                  source: "git_worktree"
                }
              ]);
            }
          }
        } catch {
          // Skip directories that fail git inspection.
        }
      }

      if (!inspectSucceeded && existingProjects.length) {
        return existingProjects.map((entry) => updateProjectAvailability(entry, "unreachable"));
      }

      for (const existingProject of existingProjects) {
        const key = `${existingProject.deviceId}::${existingProject.cwd}`;
        if (!discovered.has(key)) {
          discovered.set(key, updateProjectAvailability(existingProject, "unreachable"));
        }
      }

      return [...discovered.values()].sort((left, right) => left.title.localeCompare(right.title));
    },
    async loadConversation(conversationId: string) {
      return fetchConversationDetail(this.relayBaseUrl, conversationId, this.relayAccessToken);
    },
    async createProjectConversation(payload: CreateConversationPayload) {
      const response = await createConversation(this.relayBaseUrl, payload, this.relayAccessToken);
      await this.refreshAll();
      return response;
    },
    async sendProjectMessage(conversationId: string, payload: SendConversationMessagePayload) {
      const response = await sendConversationMessage(
        this.relayBaseUrl,
        conversationId,
        payload,
        this.relayAccessToken
      );
      await this.refreshAll();
      return response;
    },
    async cancelProjectTask(taskId: string) {
      const response = await cancelTask(this.relayBaseUrl, taskId, this.relayAccessToken);
      await this.refreshAll();
      return response;
    },
    async replyToInput(detail: ConversationDetailResponse) {
      const request = detail.pendingInputRequest;
      if (!request) {
        return null;
      }

      const response = await respondTaskInputRequest(
        this.relayBaseUrl,
        request.taskId,
        request.id,
        {
          optionId: request.selectedOptionId ?? undefined,
          text: request.responseText ?? undefined
        },
        this.relayAccessToken
      );
      await this.refreshAll();
      return response;
    },
    async respondToInput(taskId: string, requestId: string, optionId?: string, text?: string) {
      const response = await respondTaskInputRequest(
        this.relayBaseUrl,
        taskId,
        requestId,
        { optionId, text },
        this.relayAccessToken
      );
      await this.refreshAll();
      return response;
    },
    async archiveProjectConversation(conversationId: string) {
      const response = await archiveConversation(
        this.relayBaseUrl,
        conversationId,
        this.relayAccessToken
      );
      await this.refreshAll();
      return response;
    }
  }
});

function loadRecentProjectKeys() {
  const raw = window.localStorage.getItem(RECENT_PROJECTS_STORAGE_KEY);
  if (!raw) {
    return [];
  }

  try {
    const parsed = JSON.parse(raw) as string[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

function persistRecentProjectKeys(keys: string[]) {
  window.localStorage.setItem(RECENT_PROJECTS_STORAGE_KEY, JSON.stringify(keys));
}

function loadNotificationPreferences() {
  const raw = window.localStorage.getItem(NOTIFICATION_PREFS_STORAGE_KEY);
  if (!raw) {
    return {};
  }

  try {
    const parsed = JSON.parse(raw) as Record<string, "all" | "important">;
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}

function loadDefaultNotificationPreference(): "all" | "important" {
  const raw = window.localStorage.getItem(NOTIFICATION_DEFAULT_PREF_STORAGE_KEY);
  return raw === "all" ? "all" : "important";
}

function loadDefaultExecutionMode(): TaskExecutionMode {
  const raw = window.localStorage.getItem(DEFAULT_EXECUTION_MODE_STORAGE_KEY);
  if (
    raw === "read_only" ||
    raw === "workspace_write" ||
    raw === "workspace_write_and_test"
  ) {
    return raw;
  }
  return "workspace_write";
}

function loadSensitiveConfirmationEnabled() {
  const raw = window.localStorage.getItem(SENSITIVE_CONFIRM_STORAGE_KEY);
  return raw === null ? true : raw !== "false";
}

function persistNotificationPreferences(preferences: Record<string, "all" | "important">) {
  window.localStorage.setItem(NOTIFICATION_PREFS_STORAGE_KEY, JSON.stringify(preferences));
}

function persistDefaultNotificationPreference(preference: "all" | "important") {
  window.localStorage.setItem(NOTIFICATION_DEFAULT_PREF_STORAGE_KEY, preference);
}

function persistDefaultExecutionMode(executionMode: TaskExecutionMode) {
  window.localStorage.setItem(DEFAULT_EXECUTION_MODE_STORAGE_KEY, executionMode);
}

function persistSensitiveConfirmationEnabled(enabled: boolean) {
  window.localStorage.setItem(SENSITIVE_CONFIRM_STORAGE_KEY, String(enabled));
}

function loadSeenNotificationIds() {
  const raw = window.localStorage.getItem(NOTIFICATION_SEEN_STORAGE_KEY);
  if (!raw) {
    return [] as string[];
  }

  try {
    const parsed = JSON.parse(raw) as string[];
    return Array.isArray(parsed) ? parsed.filter((value) => typeof value === "string") : [];
  } catch {
    return [];
  }
}

function persistSeenNotificationIds(notificationIds: string[]) {
  window.localStorage.setItem(NOTIFICATION_SEEN_STORAGE_KEY, JSON.stringify(notificationIds));
}

function loadHiddenProjectKeys() {
  const raw = window.localStorage.getItem(HIDDEN_PROJECT_KEYS_STORAGE_KEY);
  if (!raw) {
    return [] as string[];
  }

  try {
    const parsed = JSON.parse(raw) as string[];
    return Array.isArray(parsed) ? parsed.filter((value) => typeof value === "string") : [];
  } catch {
    return [];
  }
}

function persistHiddenProjectKeys(projectKeys: string[]) {
  window.localStorage.setItem(HIDDEN_PROJECT_KEYS_STORAGE_KEY, JSON.stringify(projectKeys));
}
