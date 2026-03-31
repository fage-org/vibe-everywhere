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
  buildConversationScopeKey,
  loadConfiguredProjects,
  loadLastConversationIdByScope,
  loadModelProfiles,
  normalizeModelId,
  loadSelectedModelProfileId,
  loadSelectedProjectId,
  loadSelectedProvider,
  persistConfiguredProjects,
  persistLastConversationIdByScope,
  persistModelProfiles,
  persistSelectedModelProfileId,
  persistSelectedProjectId,
  persistSelectedProvider,
  type ConfiguredProject,
  type ModelProfile
} from "@/lib/clientConfig";
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
  ProviderKind,
  RelayEventEnvelope,
  SendConversationMessagePayload,
  ServiceHealth,
  TaskExecutionMode,
  TaskRecord
} from "@/types";

const RECENT_PROJECTS_STORAGE_KEY = "vibe.everywhere.recentProjects";
const DEFAULT_EXECUTION_MODE_STORAGE_KEY = "vibe.everywhere.defaultExecutionMode";
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

type ConfiguredProjectView = {
  id: string;
  name: string;
  deviceId: string;
  deviceName: string;
  cwd: string;
  pathLabel: string;
  projectSummary: ProjectSummary | null;
  availableProviders: ProviderKind[];
  online: boolean;
  updatedAtEpochMs: number;
};

const APP_SUPPORTED_PROVIDERS: ProviderKind[] = ["codex", "claude_code", "open_code"];

function listVisibleProjectSummaries(state: {
  devices: DeviceRecord[];
  conversations: ConversationRecord[];
  tasks: TaskRecord[];
  discoveredProjects: DiscoveredProjectRecord[];
  hiddenProjectKeys: string[];
}) {
  return filterVisibleProjectKeys(
    deriveProjectSummaries(
      state.devices,
      state.conversations,
      state.tasks,
      state.discoveredProjects
    ),
    state.hiddenProjectKeys
  );
}

function providerLabel(provider: ProviderKind) {
  switch (provider) {
    case "claude_code":
      return "Claude Code";
    case "open_code":
      return "OpenCode";
    default:
      return "Codex";
  }
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
    configuredProjects: [] as ConfiguredProject[],
    modelProfiles: [] as ModelProfile[],
    selectedProjectId: "",
    selectedProvider: "" as ProviderKind | "",
    selectedModelProfileId: "",
    lastConversationIdByScope: {} as Record<string, string>,
    defaultExecutionMode: "workspace_write" as TaskExecutionMode,
    refreshTimer: null as number | null
  }),
  getters: {
    hasRelayConfig(state) {
      return Boolean(state.relayBaseUrl.trim());
    },
    activeServerLabel(state) {
      return state.appConfig?.appName ?? "Current server";
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
      return listVisibleProjectSummaries(state);
    },
    hostSummaries(state) {
      return deriveHostSummaries(state.devices, listVisibleProjectSummaries(state));
    },
    recentProjects(state): ProjectSummary[] {
      const summaries = listVisibleProjectSummaries(state);
      const byKey = new Map(summaries.map((project) => [project.key, project]));
      return state.recentProjectKeys
        .map((key) => byKey.get(key))
        .filter((value): value is ProjectSummary => Boolean(value));
    },
    runningProjects(state): ProjectSummary[] {
      return listVisibleProjectSummaries(state).filter((project) => project.runningTaskCount > 0);
    },
    reviewProjects(state): ProjectSummary[] {
      return listVisibleProjectSummaries(state).filter(
        (project) => project.failedTaskCount > 0 || project.waitingInputCount > 0
      );
    },
    configuredProjectViews(state): ConfiguredProjectView[] {
      const deviceById = new Map(state.devices.map((device) => [device.id, device]));
      const summaryByKey = new Map(
        listVisibleProjectSummaries(state).map((project) => [project.key, project])
      );

      return [...state.configuredProjects]
        .map((project) => {
          const device = deviceById.get(project.deviceId);
          const key = buildProjectKey(project.deviceId, project.cwd);
          const summary = summaryByKey.get(key) ?? null;
          const availableProviders = (device?.providers ?? [])
            .filter((provider) => provider.available && APP_SUPPORTED_PROVIDERS.includes(provider.kind))
            .map((provider) => provider.kind);

          return {
            id: project.id,
            name: project.name,
            deviceId: project.deviceId,
            deviceName: device?.name ?? project.deviceId,
            cwd: project.cwd,
            pathLabel: summary?.pathLabel ?? project.cwd,
            projectSummary: summary,
            availableProviders,
            online: device?.online ?? false,
            updatedAtEpochMs: Math.max(project.updatedAtEpochMs, summary?.updatedAtEpochMs ?? 0)
          };
        })
        .sort((left, right) => right.updatedAtEpochMs - left.updatedAtEpochMs);
    },
    selectedConfiguredProject(): ConfiguredProjectView | null {
      return this.configuredProjectViews.find((project) => project.id === this.selectedProjectId) ?? null;
    },
    selectedModelProfile(state): ModelProfile | null {
      return state.modelProfiles.find((profile) => profile.id === state.selectedModelProfileId) ?? null;
    },
    availableProvidersForSelectedProject(): ProviderKind[] {
      return this.selectedConfiguredProject?.availableProviders ?? [];
    },
    availableModelProfiles(): ModelProfile[] {
      if (!this.selectedProvider) {
        return [];
      }

      return this.modelProfiles.filter((profile) => profile.provider === this.selectedProvider);
    },
    currentConversationScopeKey(): string | null {
      if (!this.selectedProjectId || !this.selectedProvider) {
        return null;
      }
      return buildConversationScopeKey(this.selectedProjectId, this.selectedProvider);
    }
  },
  actions: {
    async bootstrap() {
      console.info("[vibe-app] bootstrap start");
      this.isBootstrapping = true;
      this.recentProjectKeys = loadRecentProjectKeys();
      this.defaultExecutionMode = loadDefaultExecutionMode();
      this.hiddenProjectKeys = loadHiddenProjectKeys();
      this.configuredProjects = loadConfiguredProjects();
      this.modelProfiles = loadModelProfiles();
      this.selectedProjectId = loadSelectedProjectId();
      this.selectedProvider = loadSelectedProvider();
      this.selectedModelProfileId = loadSelectedModelProfileId();
      this.lastConversationIdByScope = loadLastConversationIdByScope();
      this.relayBaseUrl = await resolveInitialRelayBaseUrl();
      this.relayBaseUrlInput = this.relayBaseUrl;
      this.relayAccessToken = resolveInitialRelayAccessToken();
      this.relayAccessTokenInput = this.relayAccessToken;
      await this.refreshAll(true);
      this.ensureSelections();
      this.startAutoRefresh();
      this.isBootstrapping = false;
      console.info("[vibe-app] bootstrap complete");
    },
    async saveRelaySettings() {
      this.relayBaseUrl = normalizeRelayBaseUrl(this.relayBaseUrlInput);
      this.relayAccessToken = this.relayAccessTokenInput.trim();
      console.info("[vibe-app] save relay settings", {
        relayBaseUrl: this.relayBaseUrl,
        hasAccessToken: Boolean(this.relayAccessToken)
      });
      persistRelayBaseUrl(this.relayBaseUrl);
      persistRelayAccessToken(this.relayAccessToken);
      await this.refreshAll(true);
      this.ensureSelections();
    },
    async refreshAll(forceProjectDiscovery = false) {
      if (!this.relayBaseUrl.trim()) {
        this.health = null;
        this.devices = [];
        this.conversations = [];
        this.tasks = [];
        this.discoveredProjects = [];
        this.projectInventoryUpdatedAtByDevice = {};
        this.ensureSelections();
        return;
      }

      this.isRefreshing = true;
      this.errorMessage = "";
      console.info("[vibe-app] refresh start", {
        relayBaseUrl: this.relayBaseUrl,
        forceProjectDiscovery
      });

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
        this.ensureSelections();
        this.lastRefreshEpochMs = Date.now();
        console.info("[vibe-app] refresh success", {
          devices: this.devices.length,
          conversations: this.conversations.length,
          tasks: this.tasks.length,
          projects: this.discoveredProjects.length
        });
      } catch (error) {
        this.errorMessage = error instanceof Error ? error.message : String(error);
        console.error("[vibe-app] refresh failed", error);
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
    listProjectConversations(deviceId: string, cwd: string | null, provider?: ProviderKind | null) {
      return this.conversations
        .filter(
          (conversation) =>
            conversation.deviceId === deviceId &&
            conversation.cwd === cwd &&
            (!provider || conversation.provider === provider)
        )
        .sort((left, right) => right.updatedAtEpochMs - left.updatedAtEpochMs);
    },
    listProjectTasks(deviceId: string, cwd: string | null, provider?: ProviderKind | null) {
      return this.tasks
        .filter(
          (task) =>
            task.deviceId === deviceId &&
            task.cwd === cwd &&
            (!provider || task.provider === provider)
        )
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
    getPreferredProjectPath(deviceId: string) {
      return loadStoredProjectFolder(deviceId) || null;
    },
    setDefaultExecutionMode(executionMode: TaskExecutionMode) {
      this.defaultExecutionMode = executionMode;
      persistDefaultExecutionMode(executionMode);
    },
    setSelectedProject(projectId: string) {
      this.selectedProjectId = projectId;
      persistSelectedProjectId(projectId);
      this.ensureSelections();
    },
    setSelectedProvider(provider: ProviderKind | "") {
      this.selectedProvider = provider;
      persistSelectedProvider(provider);
      this.ensureSelections();
    },
    setSelectedModelProfile(modelProfileId: string) {
      this.selectedModelProfileId = modelProfileId;
      persistSelectedModelProfileId(modelProfileId);
      this.ensureSelections();
    },
    rememberConversationForCurrentScope(conversationId: string) {
      const scopeKey = this.currentConversationScopeKey;
      if (!scopeKey) {
        return;
      }

      this.lastConversationIdByScope = {
        ...this.lastConversationIdByScope,
        [scopeKey]: conversationId
      };
      persistLastConversationIdByScope(this.lastConversationIdByScope);
    },
    getRememberedConversationId(projectId: string, provider: ProviderKind) {
      return this.lastConversationIdByScope[buildConversationScopeKey(projectId, provider)] ?? null;
    },
    createConfiguredProject(payload: { name: string; deviceId: string; cwd: string }) {
      const now = Date.now();
      const project: ConfiguredProject = {
        id: crypto.randomUUID(),
        name: payload.name.trim(),
        deviceId: payload.deviceId,
        cwd: payload.cwd.trim(),
        createdAtEpochMs: now,
        updatedAtEpochMs: now
      };
      this.configuredProjects = [project, ...this.configuredProjects];
      persistConfiguredProjects(this.configuredProjects);
      this.setSelectedProject(project.id);
      return project;
    },
    updateConfiguredProject(projectId: string, payload: { name: string; deviceId: string; cwd: string }) {
      this.configuredProjects = this.configuredProjects.map((project) =>
        project.id === projectId
          ? {
              ...project,
              name: payload.name.trim(),
              deviceId: payload.deviceId,
              cwd: payload.cwd.trim(),
              updatedAtEpochMs: Date.now()
            }
          : project
      );
      persistConfiguredProjects(this.configuredProjects);
      this.ensureSelections();
    },
    deleteConfiguredProject(projectId: string) {
      this.configuredProjects = this.configuredProjects.filter((project) => project.id !== projectId);
      persistConfiguredProjects(this.configuredProjects);

      const nextLastConversationIdByScope = Object.fromEntries(
        Object.entries(this.lastConversationIdByScope).filter(
          ([scopeKey]) => !scopeKey.startsWith(`${projectId}::`)
        )
      );
      this.lastConversationIdByScope = nextLastConversationIdByScope;
      persistLastConversationIdByScope(this.lastConversationIdByScope);

      if (this.selectedProjectId === projectId) {
        this.selectedProjectId = "";
        persistSelectedProjectId("");
      }

      this.ensureSelections();
    },
    createModelProfile(payload: { name: string; provider: ProviderKind; modelId: string }) {
      const now = Date.now();
      const profile: ModelProfile = {
        id: crypto.randomUUID(),
        name: payload.name.trim(),
        provider: payload.provider,
        modelId: normalizeModelId(payload.provider, payload.modelId),
        createdAtEpochMs: now,
        updatedAtEpochMs: now
      };
      this.modelProfiles = [profile, ...this.modelProfiles];
      persistModelProfiles(this.modelProfiles);
      if (this.selectedProvider === profile.provider) {
        this.setSelectedModelProfile(profile.id);
      }
      return profile;
    },
    updateModelProfile(modelProfileId: string, payload: { name: string; provider: ProviderKind; modelId: string }) {
      this.modelProfiles = this.modelProfiles.map((profile) =>
        profile.id === modelProfileId
          ? {
              ...profile,
              name: payload.name.trim(),
              provider: payload.provider,
              modelId: normalizeModelId(payload.provider, payload.modelId),
              updatedAtEpochMs: Date.now()
            }
          : profile
      );
      persistModelProfiles(this.modelProfiles);
      this.ensureSelections();
    },
    deleteModelProfile(modelProfileId: string) {
      this.modelProfiles = this.modelProfiles.filter((profile) => profile.id !== modelProfileId);
      persistModelProfiles(this.modelProfiles);
      if (this.selectedModelProfileId === modelProfileId) {
        this.selectedModelProfileId = "";
        persistSelectedModelProfileId("");
      }
      this.ensureSelections();
    },
    ensureSelections() {
      const availableProjectIds = new Set(this.configuredProjectViews.map((project) => project.id));
      if (!this.selectedProjectId || !availableProjectIds.has(this.selectedProjectId)) {
        this.selectedProjectId = this.configuredProjectViews[0]?.id ?? "";
        persistSelectedProjectId(this.selectedProjectId);
      }

      const availableProviders = this.availableProvidersForSelectedProject;
      if (!this.selectedProvider || !availableProviders.includes(this.selectedProvider)) {
        this.selectedProvider = availableProviders[0] ?? "";
        persistSelectedProvider(this.selectedProvider);
      }

      const availableModelIds = new Set(this.availableModelProfiles.map((profile) => profile.id));
      if (!this.selectedModelProfileId || !availableModelIds.has(this.selectedModelProfileId)) {
        this.selectedModelProfileId = this.availableModelProfiles[0]?.id ?? "";
        persistSelectedModelProfileId(this.selectedModelProfileId);
      }
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
    applyRelayEvent(event: RelayEventEnvelope) {
      if (event.eventType !== "task_updated" || !event.task) {
        return;
      }

      const task = event.task;
      const existingTask = this.tasks.find((entry) => entry.id === task.id);
      if (existingTask) {
        Object.assign(existingTask, task);
      } else {
        this.tasks = [task, ...this.tasks].sort(
          (left, right) => right.createdAtEpochMs - left.createdAtEpochMs
        );
      }

      if (!task.conversationId) {
        return;
      }

      const conversation = this.conversations.find((entry) => entry.id === task.conversationId);
      if (!conversation) {
        return;
      }

      conversation.latestTaskId = task.id;
      conversation.updatedAtEpochMs = Math.max(
        conversation.updatedAtEpochMs,
        task.finishedAtEpochMs ?? task.startedAtEpochMs ?? task.createdAtEpochMs
      );
      if (task.providerSessionId) {
        conversation.providerSessionId = task.providerSessionId;
      }
      conversation.pendingInputRequestId = task.pendingInputRequestId;
    },
    async createProjectConversation(payload: CreateConversationPayload) {
      const response = await createConversation(this.relayBaseUrl, payload, this.relayAccessToken);
      await this.refreshAll();
      if (this.selectedProjectId && response.conversation.provider === this.selectedProvider) {
        this.rememberConversationForCurrentScope(response.conversation.id);
      }
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
      this.rememberConversationForCurrentScope(conversationId);
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
    },
    getSelectedModelId() {
      return this.selectedModelProfile?.modelId ?? "";
    },
    getProviderLabel(provider: ProviderKind) {
      return providerLabel(provider);
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

function persistDefaultExecutionMode(executionMode: TaskExecutionMode) {
  window.localStorage.setItem(DEFAULT_EXECUTION_MODE_STORAGE_KEY, executionMode);
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
