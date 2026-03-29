import { defineStore } from "pinia";
import {
  archiveConversation,
  cancelTask,
  closePortForward,
  closeShellSession,
  createConversation,
  createPortForward,
  createShellSession,
  createTask,
  fetchAppConfig,
  fetchAuditEvents,
  fetchConversationDetail,
  fetchConversations,
  fetchDevices,
  fetchHealth,
  fetchPortForwards,
  fetchShellSessionDetail,
  fetchShellSessions,
  fetchTaskDetail,
  fetchTasks,
  respondTaskInputRequest,
  sendConversationMessage,
  sendShellInput,
} from "../lib/api";
import {
  buildEventStreamUrl,
  loadStoredProjectFolder,
  buildWebSocketUrl,
  loadTauriConfig,
  normalizeRelayBaseUrl,
  persistProjectFolder,
  persistRelayAccessToken,
  persistRelayBaseUrl,
  resolveInitialRelayAccessToken,
  resolveInitialRelayBaseUrl,
} from "../lib/runtime";
import { APP_FEATURE_FLAGS, hasAppFeatureFlag } from "../lib/features";
import { i18n } from "../lib/i18n";
import {
  buildPreviewActivity,
  buildTaskActivity,
  publishSystemActivity,
  readNotificationPermission,
  type ActivityItem,
} from "../lib/notifications";
import {
  resolveCurrentPlatformCapability,
  supportsSystemNotifications,
} from "../lib/platform";
import type {
  AppConfig,
  AuditRecord,
  ConversationDetailResponse,
  ConversationInputOption,
  ConversationInputRequest,
  ConversationRecord,
  CreateConversationPayload,
  CreatePortForwardPayload,
  CreateTaskPayload,
  DeviceRecord,
  ExecutionProtocol,
  PortForwardRecord,
  PortForwardStatus,
  ProviderKind,
  RelayEventEnvelope,
  RespondConversationInputPayload,
  ServiceHealth,
  ShellSessionDetailResponse,
  ShellSessionRecord,
  ShellSessionStatus,
  TaskDetailResponse,
  TaskEvent,
  TaskRecord,
  TaskStatus,
} from "../types";

type DraftTask = {
  title: string;
  provider: ProviderKind | "";
  cwd: string;
  model: string;
  prompt: string;
};

type DraftShell = {
  cwd: string;
  input: string;
};

type DraftPortForward = {
  targetHost: string;
  targetPort: string;
};

type PendingConversationInputDraft = {
  optionId: string | null;
  text: string;
};

let activeEventSource: EventSource | null = null;
let activeShellPollTimer: number | null = null;
let activePortForwardPollTimer: number | null = null;
let activeShellSocket: WebSocket | null = null;
let activeShellSocketSessionId: string | null = null;
const CONVERSATION_HISTORY_LIMIT = 200;
const TASK_HISTORY_LIMIT = 200;
const SHELL_HISTORY_LIMIT = 100;
const PORT_FORWARD_HISTORY_LIMIT = 100;
const AUDIT_HISTORY_LIMIT = 200;
const ACTIVITY_HISTORY_LIMIT = 120;
const SHELL_POLL_INTERVAL_MS = 1_500;
const PORT_FORWARD_POLL_INTERVAL_MS = 1_500;

export const useControlStore = defineStore("control", {
  state: () => ({
    relayBaseUrl: "",
    relayInput: "",
    relayAccessToken: "",
    relayAccessTokenInput: "",
    localAppConfig: null as AppConfig | null,
    appConfig: null as AppConfig | null,
    health: null as ServiceHealth | null,
    auditRecords: [] as AuditRecord[],
    activities: [] as ActivityItem[],
    devices: [] as DeviceRecord[],
    conversations: [] as ConversationRecord[],
    tasks: [] as TaskRecord[],
    shellSessions: [] as ShellSessionRecord[],
    portForwards: [] as PortForwardRecord[],
    taskScope: "selected_device" as "all" | "selected_device",
    taskStatusFilter: "all" as TaskStatus | "all",
    shellScope: "selected_device" as "all" | "selected_device",
    shellStatusFilter: "all" as ShellSessionStatus | "all",
    portForwardScope: "selected_device" as "all" | "selected_device",
    portForwardStatusFilter: "all" as PortForwardStatus | "all",
    selectedDeviceId: null as string | null,
    selectedConversationId: null as string | null,
    selectedTaskId: null as string | null,
    selectedShellSessionId: null as string | null,
    selectedPortForwardId: null as string | null,
    selectedConversationDetail: null as ConversationDetailResponse | null,
    selectedTaskDetail: null as TaskDetailResponse | null,
    selectedShellSessionDetail: null as ShellSessionDetailResponse | null,
    eventState: "disconnected" as "disconnected" | "connecting" | "connected",
    isBootstrapping: false,
    isShellPolling: false,
    isPortForwardPolling: false,
    isAuditLoading: false,
    shellSocketState: "disconnected" as
      | "disconnected"
      | "connecting"
      | "connected",
    notificationPermission: "default" as NotificationPermission,
    errorCode: null as string | null,
    errorMessage: "",
    draft: {
      title: "",
      provider: "",
      cwd: "",
      model: "",
      prompt: "",
    } as DraftTask,
    projectFolder: "",
    shellDraft: {
      cwd: "",
      input: "",
    } as DraftShell,
    portForwardDraft: {
      targetHost: "127.0.0.1",
      targetPort: "",
    } as DraftPortForward,
    pendingConversationInputDraft: {
      optionId: null,
      text: "",
    } as PendingConversationInputDraft,
  }),
  getters: {
    selectedDevice(state) {
      return (
        state.devices.find((device) => device.id === state.selectedDeviceId) ??
        null
      );
    },
    selectedConversation(state) {
      return (
        state.selectedConversationDetail?.conversation ??
        state.conversations.find(
          (conversation) => conversation.id === state.selectedConversationId,
        ) ??
        null
      );
    },
    selectedTask(state) {
      return (
        state.selectedTaskDetail?.task ??
        state.tasks.find((task) => task.id === state.selectedTaskId) ??
        null
      );
    },
    selectedShellSession(state) {
      return (
        state.selectedShellSessionDetail?.session ??
        state.shellSessions.find(
          (session) => session.id === state.selectedShellSessionId,
        ) ??
        null
      );
    },
    selectedPortForward(state) {
      return (
        state.portForwards.find(
          (forward) => forward.id === state.selectedPortForwardId,
        ) ?? null
      );
    },
    conversationPendingInput(state) {
      return state.selectedConversationDetail?.pendingInputRequest ?? null;
    },
    conversationCurrentTask(state) {
      const latestTaskId =
        state.selectedConversationDetail?.conversation.latestTaskId;
      if (!latestTaskId) {
        return null;
      }

      return (
        state.selectedConversationDetail?.tasks.find(
          (detail) => detail.task.id === latestTaskId,
        )?.task ??
        state.tasks.find((task) => task.id === latestTaskId) ??
        null
      );
    },
    visibleTasks(state) {
      return state.tasks.filter((task) => {
        if (
          state.taskScope === "selected_device" &&
          task.deviceId !== state.selectedDeviceId
        ) {
          return false;
        }
        if (
          state.taskStatusFilter !== "all" &&
          task.status !== state.taskStatusFilter
        ) {
          return false;
        }
        return true;
      });
    },
    visibleShellSessions(state) {
      return state.shellSessions.filter((session) => {
        if (
          state.shellScope === "selected_device" &&
          session.deviceId !== state.selectedDeviceId
        ) {
          return false;
        }
        if (
          state.shellStatusFilter !== "all" &&
          session.status !== state.shellStatusFilter
        ) {
          return false;
        }
        return true;
      });
    },
    visiblePortForwards(state) {
      return state.portForwards.filter((forward) => {
        if (
          state.portForwardScope === "selected_device" &&
          forward.deviceId !== state.selectedDeviceId
        ) {
          return false;
        }
        if (
          state.portForwardStatusFilter !== "all" &&
          forward.status !== state.portForwardStatusFilter
        ) {
          return false;
        }
        return true;
      });
    },
    visibleConversations(state) {
      return state.conversations.filter(
        (conversation) => !conversation.archived,
      );
    },
    availableProviders(): {
      kind: ProviderKind;
      label: string;
      executionProtocol: ExecutionProtocol;
    }[] {
      const device = this.selectedDevice;
      if (!device) {
        return [];
      }

      return device.providers
        .filter((provider) => provider.available)
        .map((provider) => ({
          kind: provider.kind,
          label: formatProviderLabel(provider.kind, provider.executionProtocol),
          executionProtocol: provider.executionProtocol,
        }));
    },
    unreadActivityCount(state) {
      return state.activities.filter((activity) => activity.unread).length;
    },
    recentActivities(state) {
      return [...state.activities].sort(
        (left, right) => right.timestampEpochMs - left.timestampEpochMs,
      );
    },
    currentPlatformCapability(state) {
      return resolveCurrentPlatformCapability(state.appConfig);
    },
  },
  actions: {
    async initialize() {
      if (this.isBootstrapping) {
        return;
      }

      this.isBootstrapping = true;
      this.errorCode = null;
      this.errorMessage = "";

      try {
        this.notificationPermission = readNotificationPermission();
        this.localAppConfig = await loadTauriConfig();
        this.appConfig = this.localAppConfig;
        this.relayBaseUrl = await resolveInitialRelayBaseUrl();
        this.relayAccessToken = resolveInitialRelayAccessToken();
        this.relayInput = this.relayBaseUrl;
        this.relayAccessTokenInput = this.relayAccessToken;
        if (!this.relayBaseUrl) {
          this.resetRemoteState();
          return;
        }
        await this.reloadAll();
        this.syncProjectFolder();
        this.connectEvents();
        this.startShellPolling();
        this.startPortForwardPolling();
        this.connectShellSocket();
      } catch (error) {
        this.errorCode = null;
        this.errorMessage = formatError(error);
      } finally {
        this.isBootstrapping = false;
      }
    },
    async applyRelayBaseUrl() {
      await runStoreAction(this, async () => {
        this.relayBaseUrl = normalizeRelayBaseUrl(this.relayInput);
        this.relayAccessToken = this.relayAccessTokenInput.trim();
        persistRelayBaseUrl(this.relayBaseUrl);
        persistRelayAccessToken(this.relayAccessToken);
        if (!this.relayBaseUrl) {
          this.errorCode = null;
          this.errorMessage = "";
          this.resetRemoteState();
          return;
        }
        await this.reloadAll();
        this.connectEvents();
        this.startShellPolling();
        this.startPortForwardPolling();
        this.connectShellSocket();
      });
    },
    async reloadAll() {
      await runStoreAction(this, async () => {
        if (!this.relayBaseUrl) {
          this.resetRemoteState();
          return;
        }

        const [appConfig, health] = await Promise.all([
          fetchAppConfig(this.relayBaseUrl),
          fetchHealth(this.relayBaseUrl),
        ]);

        this.appConfig = appConfig;
        this.health = health;
        const shouldLoadAuditEvents = hasAppFeatureFlag(
          appConfig,
          APP_FEATURE_FLAGS.governanceAuditConsole,
        );
        const [
          devices,
          conversations,
          tasks,
          shellSessions,
          portForwards,
          auditRecords,
        ] = await Promise.all([
          fetchDevices(this.relayBaseUrl, this.relayAccessToken),
          fetchConversations(this.relayBaseUrl, this.relayAccessToken, {
            archived: false,
          }),
          fetchTasks(this.relayBaseUrl, this.relayAccessToken, {
            limit: TASK_HISTORY_LIMIT,
          }),
          fetchShellSessions(this.relayBaseUrl, this.relayAccessToken, {
            limit: SHELL_HISTORY_LIMIT,
          }),
          fetchPortForwards(this.relayBaseUrl, this.relayAccessToken, {
            limit: PORT_FORWARD_HISTORY_LIMIT,
          }),
          shouldLoadAuditEvents
            ? fetchAuditEvents(this.relayBaseUrl, this.relayAccessToken, {
                limit: AUDIT_HISTORY_LIMIT,
              })
            : Promise.resolve([]),
        ]);
        this.devices = devices;
        this.conversations = conversations;
        this.tasks = tasks;
        this.shellSessions = shellSessions;
        this.portForwards = portForwards;
        this.auditRecords = auditRecords;
        this.ensureSelections();
        this.syncTaskSelectionToDevice();
        this.syncShellSelectionToDevice();
        this.syncPortForwardSelectionToDevice();

        await Promise.all([
          this.selectedConversationId
            ? this.loadConversationDetail(this.selectedConversationId)
            : this.selectedTaskId
              ? this.loadTaskDetail(this.selectedTaskId)
              : Promise.resolve(),
          this.selectedShellSessionId
            ? this.loadShellSessionDetail(this.selectedShellSessionId)
            : Promise.resolve(),
        ]);
        this.connectShellSocket();
      });
    },
    ensureSelections() {
      if (
        !this.conversations.some(
          (conversation) => conversation.id === this.selectedConversationId,
        )
      ) {
        this.selectedConversationId = this.conversations[0]?.id ?? null;
      }

      const selectedConversation = this.conversations.find(
        (conversation) => conversation.id === this.selectedConversationId,
      );
      if (!this.devices.some((device) => device.id === this.selectedDeviceId)) {
        this.selectedDeviceId =
          this.devices.find(
            (device) => device.id === selectedConversation?.deviceId,
          )?.id ??
          this.devices[0]?.id ??
          null;
      }

      if (selectedConversation?.deviceId) {
        this.selectedDeviceId = selectedConversation.deviceId;
      }

      if (!this.tasks.some((task) => task.id === this.selectedTaskId)) {
        this.selectedTaskId =
          this.tasks.find(
            (task) => task.id === selectedConversation?.latestTaskId,
          )?.id ??
          this.tasks[0]?.id ??
          null;
      }

      if (
        !this.shellSessions.some(
          (session) => session.id === this.selectedShellSessionId,
        )
      ) {
        this.selectedShellSessionId = this.shellSessions[0]?.id ?? null;
      }

      if (
        !this.portForwards.some(
          (forward) => forward.id === this.selectedPortForwardId,
        )
      ) {
        this.selectedPortForwardId = this.portForwards[0]?.id ?? null;
      }

      const availableProviders = this.availableProviders;
      if (
        !availableProviders.some(
          (provider) => provider.kind === this.draft.provider,
        )
      ) {
        this.draft.provider = availableProviders[0]?.kind ?? "";
      }

      this.syncProjectFolder();
    },
    async selectDevice(deviceId: string) {
      await runStoreAction(this, async () => {
        this.selectedDeviceId = deviceId;
        this.ensureSelections();
        this.syncTaskSelectionToDevice();
        this.syncShellSelectionToDevice();
        this.syncPortForwardSelectionToDevice();
        if (
          this.selectedTaskId &&
          this.selectedTaskDetail?.task.id !== this.selectedTaskId
        ) {
          await this.loadTaskDetail(this.selectedTaskId);
        }
        if (
          this.selectedShellSessionId &&
          this.selectedShellSessionDetail?.session.id !==
            this.selectedShellSessionId
        ) {
          await this.loadShellSessionDetail(this.selectedShellSessionId);
        }
        this.syncProjectFolder();
        this.connectShellSocket();
      });
    },
    async selectConversation(conversationId: string) {
      await runStoreAction(this, async () => {
        this.selectedConversationId = conversationId;
        await this.loadConversationDetail(conversationId);
        const conversation = this.selectedConversationDetail?.conversation;
        if (conversation) {
          this.selectedDeviceId = conversation.deviceId;
        }
        this.ensureSelections();
        this.syncTaskSelectionToDevice();
        this.syncShellSelectionToDevice();
        this.syncPortForwardSelectionToDevice();
        this.syncProjectFolder();
      });
    },
    startNewConversationDraft(
      options?: {
        deviceId?: string | null;
        cwd?: string | null;
        provider?: ProviderKind | "";
        model?: string | null;
      },
    ) {
      if (typeof options?.deviceId !== "undefined") {
        this.selectedDeviceId = options.deviceId;
      }

      this.selectedConversationId = null;
      this.selectedConversationDetail = null;
      this.selectedTaskId = null;
      this.selectedTaskDetail = null;
      this.pendingConversationInputDraft = {
        optionId: null,
        text: "",
      };
      this.draft.title = "";
      this.draft.prompt = "";

      if (typeof options?.cwd !== "undefined") {
        this.projectFolder = options.cwd?.trim() ?? "";
      } else {
        this.syncProjectFolder();
      }

      if (typeof options?.model !== "undefined") {
        this.draft.model = options.model?.trim() ?? "";
      }

      if (typeof options?.provider !== "undefined") {
        this.draft.provider = options.provider;
      }

      const availableProviders = this.availableProviders;
      if (
        !availableProviders.some(
          (provider) => provider.kind === this.draft.provider,
        )
      ) {
        this.draft.provider = availableProviders[0]?.kind ?? "";
      }
    },
    async selectTask(taskId: string) {
      await runStoreAction(this, async () => {
        this.selectedTaskId = taskId;
        const task = this.tasks.find((item) => item.id === taskId);
        if (task?.conversationId) {
          this.selectedConversationId = task.conversationId;
          await this.loadConversationDetail(task.conversationId);
          return;
        }
        await this.loadTaskDetail(taskId);
      });
    },
    async selectShellSession(sessionId: string) {
      await runStoreAction(this, async () => {
        this.selectedShellSessionId = sessionId;
        await this.loadShellSessionDetail(sessionId);
        this.connectShellSocket();
      });
    },
    selectPortForward(forwardId: string) {
      this.selectedPortForwardId = forwardId;
    },
    async openActivity(activityId: string) {
      const activity = this.activities.find((item) => item.id === activityId);
      if (!activity) {
        return;
      }

      activity.unread = false;

      if (activity.deviceId) {
        this.selectedDeviceId = activity.deviceId;
        this.ensureSelections();
        this.syncTaskSelectionToDevice();
        this.syncShellSelectionToDevice();
        this.syncPortForwardSelectionToDevice();
      }

      if (activity.resourceKind === "task") {
        await this.selectTask(activity.resourceId);
        return;
      }

      if (activity.resourceKind === "preview") {
        await this.refreshPortForwardsFromPoll();
        this.selectPortForward(activity.resourceId);
      }
    },
    markAllActivitiesRead() {
      this.activities = this.activities.map((activity) => ({
        ...activity,
        unread: false,
      }));
    },
    async loadConversationDetail(conversationId: string) {
      const detail = await fetchConversationDetail(
        this.relayBaseUrl,
        conversationId,
        this.relayAccessToken,
      );
      this.selectedConversationDetail = detail;
      this.upsertConversation(detail.conversation);
      for (const taskDetail of detail.tasks) {
        this.upsertTask(taskDetail.task);
      }
      const latestTaskId =
        detail.conversation.latestTaskId ??
        detail.tasks[detail.tasks.length - 1]?.task.id ??
        null;
      this.selectedTaskId = latestTaskId;
      this.selectedTaskDetail =
        detail.tasks.find(
          (taskDetail) => taskDetail.task.id === latestTaskId,
        ) ?? null;
      this.selectedDeviceId = detail.conversation.deviceId;
      this.draft.provider = detail.conversation.provider;
      this.draft.model = detail.conversation.model ?? "";
      this.draft.title = detail.conversation.title;
      this.syncProjectFolder();
      this.resetPendingConversationInputDraft(detail.pendingInputRequest);
    },
    async loadTaskDetail(taskId: string) {
      this.selectedTaskDetail = await fetchTaskDetail(
        this.relayBaseUrl,
        taskId,
        this.relayAccessToken,
      );
    },
    async startConversation() {
      await runStoreAction(this, async () => {
        if (
          !this.selectedDeviceId ||
          !this.draft.provider ||
          !this.draft.prompt.trim()
        ) {
          return;
        }

        const payload: CreateConversationPayload = {
          deviceId: this.selectedDeviceId,
          provider: this.draft.provider,
          prompt: this.draft.prompt.trim(),
          title: this.draft.title.trim() || undefined,
          cwd: this.projectFolder.trim() || undefined,
          model: this.draft.model.trim() || undefined,
        };

        const response = await createConversation(
          this.relayBaseUrl,
          payload,
          this.relayAccessToken,
        );
        this.upsertConversation(response.conversation);
        this.upsertTask(response.task);
        this.selectedConversationId = response.conversation.id;
        this.selectedDeviceId = response.conversation.deviceId;
        await this.loadConversationDetail(response.conversation.id);
        this.draft.prompt = "";
        this.draft.title = "";
      });
    },
    async sendConversationPrompt() {
      await runStoreAction(this, async () => {
        const prompt = this.draft.prompt.trim();
        if (!prompt) {
          return;
        }

        if (!this.selectedConversationId) {
          await this.startConversation();
          return;
        }

        const response = await sendConversationMessage(
          this.relayBaseUrl,
          this.selectedConversationId,
          {
            prompt,
            title: this.draft.title.trim() || undefined,
            model: this.draft.model.trim() || undefined,
          },
          this.relayAccessToken,
        );
        this.upsertConversation(response.conversation);
        this.upsertTask(response.task);
        await this.loadConversationDetail(response.conversation.id);
        this.draft.prompt = "";
      });
    },
    async archiveSelectedConversation() {
      await runStoreAction(this, async () => {
        if (!this.selectedConversationId) {
          return;
        }

        const archivedConversation = await archiveConversation(
          this.relayBaseUrl,
          this.selectedConversationId,
          this.relayAccessToken,
        );
        this.conversations = this.conversations.filter(
          (conversation) => conversation.id !== archivedConversation.id,
        );
        this.selectedConversationId = this.conversations[0]?.id ?? null;
        if (this.selectedConversationId) {
          await this.loadConversationDetail(this.selectedConversationId);
        } else {
          this.selectedConversationDetail = null;
          this.selectedTaskDetail = null;
          this.selectedTaskId = null;
        }
      });
    },
    async respondToConversationInput(payload: RespondConversationInputPayload) {
      await runStoreAction(this, async () => {
        const request = this.selectedConversationDetail?.pendingInputRequest;
        if (!request) {
          return;
        }

        await respondTaskInputRequest(
          this.relayBaseUrl,
          request.taskId,
          request.id,
          payload,
          this.relayAccessToken,
        );
        await this.loadConversationDetail(request.conversationId);
        this.pendingConversationInputDraft = {
          optionId: null,
          text: "",
        };
      });
    },
    resetPendingConversationInputDraft(
      request: ConversationInputRequest | null,
    ) {
      this.pendingConversationInputDraft = {
        optionId: request?.selectedOptionId ?? null,
        text: request?.responseText ?? "",
      };
    },
    async loadShellSessionDetail(sessionId: string) {
      this.selectedShellSessionDetail = await fetchShellSessionDetail(
        this.relayBaseUrl,
        sessionId,
        this.relayAccessToken,
      );
    },
    async submitTask() {
      await runStoreAction(this, async () => {
        if (
          !this.selectedDeviceId ||
          !this.draft.provider ||
          !this.draft.prompt.trim()
        ) {
          return;
        }

        const payload: CreateTaskPayload = {
          deviceId: this.selectedDeviceId,
          provider: this.draft.provider,
          prompt: this.draft.prompt.trim(),
          title: this.draft.title.trim() || undefined,
          cwd: this.projectFolder.trim() || undefined,
          model: this.draft.model.trim() || undefined,
        };

        const task = await createTask(
          this.relayBaseUrl,
          payload,
          this.relayAccessToken,
        );
        this.tasks = [
          task,
          ...this.tasks.filter((item) => item.id !== task.id),
        ].slice(0, TASK_HISTORY_LIMIT);
        this.selectedTaskId = task.id;
        await this.loadTaskDetail(task.id);
        this.draft.prompt = "";
      });
    },
    async createShellSession() {
      await runStoreAction(this, async () => {
        if (!this.selectedDeviceId) {
          return;
        }

        const session = await createShellSession(
          this.relayBaseUrl,
          {
            deviceId: this.selectedDeviceId,
            cwd: this.shellDraft.cwd.trim() || undefined,
          },
          this.relayAccessToken,
        );
        this.upsertShellSession(session);
        this.selectedShellSessionId = session.id;
        await this.loadShellSessionDetail(session.id);
        this.connectShellSocket();
        this.shellDraft.input = "";
      });
    },
    setProjectFolder(folder: string) {
      this.projectFolder = folder;

      if (this.selectedDeviceId) {
        persistProjectFolder(this.selectedDeviceId, folder);
      }
    },
    syncProjectFolder() {
      if (!this.selectedDeviceId) {
        this.projectFolder = "";
        return;
      }

      this.projectFolder = loadStoredProjectFolder(this.selectedDeviceId);
    },
    async createPortForward() {
      await runStoreAction(this, async () => {
        if (
          !this.selectedDeviceId ||
          !this.portForwardDraft.targetHost.trim()
        ) {
          return;
        }

        const targetPort = parsePort(this.portForwardDraft.targetPort);
        if (targetPort === null) {
          this.errorCode = "targetPortInvalid";
          this.errorMessage = "";
          return;
        }

        const payload: CreatePortForwardPayload = {
          deviceId: this.selectedDeviceId,
          protocol: "tcp",
          targetHost: this.portForwardDraft.targetHost.trim(),
          targetPort,
        };

        const forward = await createPortForward(
          this.relayBaseUrl,
          payload,
          this.relayAccessToken,
        );
        this.upsertPortForward(forward);
        this.selectedPortForwardId = forward.id;
      });
    },
    async submitShellInput() {
      await runStoreAction(this, async () => {
        if (!this.selectedShellSessionId || !this.shellDraft.input.trim()) {
          return;
        }

        const detail = await sendShellInput(
          this.relayBaseUrl,
          this.selectedShellSessionId,
          this.normalizeShellInput(this.shellDraft.input),
          this.relayAccessToken,
        );
        this.selectedShellSessionDetail = detail;
        this.upsertShellSession(detail.session);
        this.shellDraft.input = "";
      });
    },
    async closeSelectedShellSession() {
      await runStoreAction(this, async () => {
        if (!this.selectedShellSessionId) {
          return;
        }

        const detail = await closeShellSession(
          this.relayBaseUrl,
          this.selectedShellSessionId,
          this.relayAccessToken,
        );
        this.selectedShellSessionDetail = detail;
        this.upsertShellSession(detail.session);
      });
    },
    async closeSelectedPortForward() {
      await runStoreAction(this, async () => {
        if (!this.selectedPortForwardId) {
          return;
        }

        const detail = await closePortForward(
          this.relayBaseUrl,
          this.selectedPortForwardId,
          this.relayAccessToken,
        );
        this.upsertPortForward(detail.forward);
      });
    },
    async cancelSelectedTask() {
      await runStoreAction(this, async () => {
        if (!this.selectedTaskId) {
          return;
        }

        const detail = await cancelTask(
          this.relayBaseUrl,
          this.selectedTaskId,
          this.relayAccessToken,
        );
        this.selectedTaskDetail = detail;
        this.upsertTask(detail.task);
      });
    },
    disconnectEvents() {
      if (activeEventSource) {
        activeEventSource.close();
        activeEventSource = null;
      }
      this.eventState = "disconnected";
    },
    resetRemoteState() {
      this.disconnectEvents();
      this.stopShellPolling();
      this.stopPortForwardPolling();
      this.disconnectShellSocket();
      this.appConfig = this.localAppConfig;
      this.health = null;
      this.auditRecords = [];
      this.activities = [];
      this.devices = [];
      this.conversations = [];
      this.tasks = [];
      this.shellSessions = [];
      this.portForwards = [];
      this.selectedDeviceId = null;
      this.selectedConversationId = null;
      this.selectedTaskId = null;
      this.selectedShellSessionId = null;
      this.selectedPortForwardId = null;
      this.selectedConversationDetail = null;
      this.selectedTaskDetail = null;
      this.selectedShellSessionDetail = null;
      this.pendingConversationInputDraft = {
        optionId: null,
        text: "",
      };
    },
    connectEvents() {
      this.disconnectEvents();

      if (
        !this.relayBaseUrl ||
        (this.appConfig?.requiresAuth && !this.relayAccessToken)
      ) {
        return;
      }

      this.eventState = "connecting";
      activeEventSource = new EventSource(
        buildEventStreamUrl(this.relayBaseUrl, this.relayAccessToken),
      );

      activeEventSource.addEventListener("open", () => {
        this.eventState = "connected";
      });

      activeEventSource.addEventListener("error", () => {
        this.eventState = "disconnected";
      });

      activeEventSource.addEventListener("device_updated", (event) => {
        this.handleEvent(event as MessageEvent<string>);
      });
      activeEventSource.addEventListener("task_updated", (event) => {
        this.handleEvent(event as MessageEvent<string>);
      });
      activeEventSource.addEventListener("task_event", (event) => {
        this.handleEvent(event as MessageEvent<string>);
      });
    },
    startShellPolling() {
      this.stopShellPolling();
      if (
        !this.relayBaseUrl ||
        (this.appConfig?.requiresAuth && !this.relayAccessToken)
      ) {
        return;
      }

      activeShellPollTimer = window.setInterval(() => {
        void this.refreshShellSessionsFromPoll();
      }, SHELL_POLL_INTERVAL_MS);
      void this.refreshShellSessionsFromPoll();
    },
    stopShellPolling() {
      if (activeShellPollTimer !== null) {
        window.clearInterval(activeShellPollTimer);
        activeShellPollTimer = null;
      }
    },
    startPortForwardPolling() {
      this.stopPortForwardPolling();
      if (
        !this.relayBaseUrl ||
        (this.appConfig?.requiresAuth && !this.relayAccessToken)
      ) {
        return;
      }

      activePortForwardPollTimer = window.setInterval(() => {
        void this.refreshPortForwardsFromPoll();
      }, PORT_FORWARD_POLL_INTERVAL_MS);
      void this.refreshPortForwardsFromPoll();
    },
    stopPortForwardPolling() {
      if (activePortForwardPollTimer !== null) {
        window.clearInterval(activePortForwardPollTimer);
        activePortForwardPollTimer = null;
      }
    },
    connectShellSocket() {
      const sessionId = this.selectedShellSessionId;
      if (
        !sessionId ||
        !this.relayBaseUrl ||
        (this.appConfig?.requiresAuth && !this.relayAccessToken)
      ) {
        this.disconnectShellSocket();
        return;
      }

      if (
        activeShellSocket &&
        activeShellSocketSessionId === sessionId &&
        (activeShellSocket.readyState === WebSocket.CONNECTING ||
          activeShellSocket.readyState === WebSocket.OPEN)
      ) {
        return;
      }

      this.disconnectShellSocket();
      this.shellSocketState = "connecting";
      const socket = new WebSocket(
        buildWebSocketUrl(
          this.relayBaseUrl,
          `/api/shell/sessions/${sessionId}/ws`,
          this.relayAccessToken,
        ),
      );
      activeShellSocket = socket;
      activeShellSocketSessionId = sessionId;

      socket.addEventListener("open", () => {
        if (activeShellSocket !== socket) {
          return;
        }
        this.shellSocketState = "connected";
      });

      socket.addEventListener("message", (event) => {
        if (activeShellSocket !== socket) {
          return;
        }

        try {
          const detail = JSON.parse(
            event.data as string,
          ) as ShellSessionDetailResponse;
          this.selectedShellSessionDetail = detail;
          this.upsertShellSession(detail.session);
        } catch (error) {
          this.errorCode = null;
          this.errorMessage = formatError(error);
        }
      });

      socket.addEventListener("error", () => {
        if (activeShellSocket !== socket) {
          return;
        }
        this.shellSocketState = "disconnected";
      });

      socket.addEventListener("close", () => {
        if (activeShellSocket !== socket) {
          return;
        }
        activeShellSocket = null;
        activeShellSocketSessionId = null;
        this.shellSocketState = "disconnected";
      });
    },
    disconnectShellSocket() {
      if (activeShellSocket) {
        const socket = activeShellSocket;
        activeShellSocket = null;
        activeShellSocketSessionId = null;
        socket.close();
      }
      this.shellSocketState = "disconnected";
    },
    disposeRealtime() {
      this.disconnectEvents();
      this.stopShellPolling();
      this.stopPortForwardPolling();
      this.disconnectShellSocket();
    },
    async refreshShellSessionsFromPoll() {
      if (this.isShellPolling) {
        return;
      }
      if (
        !this.relayBaseUrl ||
        (this.appConfig?.requiresAuth && !this.relayAccessToken)
      ) {
        return;
      }

      this.isShellPolling = true;
      try {
        const sessions = await fetchShellSessions(
          this.relayBaseUrl,
          this.relayAccessToken,
          {
            limit: SHELL_HISTORY_LIMIT,
          },
        );
        this.shellSessions = sessions;
        this.ensureSelections();
        this.syncShellSelectionToDevice();
        if (
          this.selectedShellSessionId &&
          this.shellSocketState !== "connected"
        ) {
          this.selectedShellSessionDetail = await fetchShellSessionDetail(
            this.relayBaseUrl,
            this.selectedShellSessionId,
            this.relayAccessToken,
          );
          this.upsertShellSession(this.selectedShellSessionDetail.session);
        }
      } catch (error) {
        this.errorCode = null;
        this.errorMessage = formatError(error);
      } finally {
        this.isShellPolling = false;
      }
    },
    async refreshPortForwardsFromPoll() {
      if (this.isPortForwardPolling) {
        return;
      }
      if (
        !this.relayBaseUrl ||
        (this.appConfig?.requiresAuth && !this.relayAccessToken)
      ) {
        return;
      }

      this.isPortForwardPolling = true;
      try {
        const portForwards = await fetchPortForwards(
          this.relayBaseUrl,
          this.relayAccessToken,
          {
            limit: PORT_FORWARD_HISTORY_LIMIT,
          },
        );
        this.reconcilePortForwards(portForwards);
        this.ensureSelections();
        this.syncPortForwardSelectionToDevice();
      } catch (error) {
        this.errorCode = null;
        this.errorMessage = formatError(error);
      } finally {
        this.isPortForwardPolling = false;
      }
    },
    handleEvent(event: MessageEvent<string>) {
      const envelope = JSON.parse(event.data) as RelayEventEnvelope;

      if (envelope.device) {
        this.upsertDevice(envelope.device);
      }
      if (envelope.task) {
        this.upsertTask(envelope.task);
        if (
          envelope.task.conversationId &&
          this.selectedConversationId === envelope.task.conversationId &&
          (!this.selectedConversationDetail?.tasks.some(
            (detail) => detail.task.id === envelope.task?.id,
          ) ||
            this.selectedConversationDetail?.pendingInputRequest?.id !==
              envelope.task.pendingInputRequestId)
        ) {
          void this.loadConversationDetail(envelope.task.conversationId);
        }
      }
      if (envelope.taskEvent) {
        this.upsertTaskEvent(envelope.taskEvent);
      }

      this.ensureSelections();
      this.syncTaskSelectionToDevice();
      this.syncShellSelectionToDevice();
      this.syncPortForwardSelectionToDevice();
    },
    upsertConversation(conversation: ConversationRecord) {
      const existingIndex = this.conversations.findIndex(
        (item) => item.id === conversation.id,
      );
      if (existingIndex >= 0) {
        this.conversations.splice(existingIndex, 1, conversation);
      } else {
        this.conversations.unshift(conversation);
      }
      this.conversations.sort(
        (left, right) => right.updatedAtEpochMs - left.updatedAtEpochMs,
      );
      if (this.conversations.length > CONVERSATION_HISTORY_LIMIT) {
        this.conversations = this.conversations.slice(
          0,
          CONVERSATION_HISTORY_LIMIT,
        );
      }

      if (
        this.selectedConversationDetail?.conversation.id === conversation.id
      ) {
        this.selectedConversationDetail = {
          ...this.selectedConversationDetail,
          conversation,
        };
      }
    },
    upsertDevice(device: DeviceRecord) {
      const existingIndex = this.devices.findIndex(
        (item) => item.id === device.id,
      );
      if (existingIndex >= 0) {
        this.devices.splice(existingIndex, 1, device);
      } else {
        this.devices.unshift(device);
      }
      this.devices.sort((left, right) => {
        if (left.online === right.online) {
          return left.name.localeCompare(right.name);
        }
        return Number(right.online) - Number(left.online);
      });
    },
    upsertTask(task: TaskRecord) {
      const existingIndex = this.tasks.findIndex((item) => item.id === task.id);
      const previous = existingIndex >= 0 ? this.tasks[existingIndex] : null;
      if (existingIndex >= 0) {
        this.tasks.splice(existingIndex, 1, task);
      } else {
        this.tasks.unshift(task);
      }
      this.tasks.sort(
        (left, right) => right.createdAtEpochMs - left.createdAtEpochMs,
      );
      if (this.tasks.length > TASK_HISTORY_LIMIT) {
        this.tasks = this.tasks.slice(0, TASK_HISTORY_LIMIT);
      }

      if (this.selectedTaskDetail?.task.id === task.id) {
        this.selectedTaskDetail = {
          task,
          events: this.selectedTaskDetail.events,
          pendingInputRequest:
            this.selectedTaskDetail.pendingInputRequest ??
            this.selectedConversationDetail?.pendingInputRequest ??
            null,
        };
      }

      if (task.conversationId) {
        const existingConversation = this.conversations.find(
          (conversation) => conversation.id === task.conversationId,
        );
        if (existingConversation) {
          this.upsertConversation({
            ...existingConversation,
            deviceId: task.deviceId,
            provider: task.provider,
            executionProtocol: task.executionProtocol,
            cwd: task.cwd,
            model: task.model,
            providerSessionId:
              task.providerSessionId ?? existingConversation.providerSessionId,
            latestTaskId:
              task.createdAtEpochMs >= existingConversation.updatedAtEpochMs
                ? task.id
                : existingConversation.latestTaskId,
            pendingInputRequestId:
              task.pendingInputRequestId ??
              existingConversation.pendingInputRequestId,
            updatedAtEpochMs: Math.max(
              existingConversation.updatedAtEpochMs,
              task.createdAtEpochMs,
              task.startedAtEpochMs ?? 0,
              task.finishedAtEpochMs ?? 0,
            ),
          });
        }

        if (
          this.selectedConversationDetail?.conversation.id ===
          task.conversationId
        ) {
          const taskDetails = [...this.selectedConversationDetail.tasks];
          const taskIndex = taskDetails.findIndex(
            (detail) => detail.task.id === task.id,
          );
          if (taskIndex >= 0) {
            taskDetails.splice(taskIndex, 1, {
              ...taskDetails[taskIndex],
              task,
            });
          } else {
            taskDetails.push({
              task,
              events: [],
              pendingInputRequest: null,
            });
            taskDetails.sort(
              (left, right) =>
                left.task.createdAtEpochMs - right.task.createdAtEpochMs,
            );
          }

          this.selectedConversationDetail = {
            ...this.selectedConversationDetail,
            conversation: {
              ...this.selectedConversationDetail.conversation,
              providerSessionId:
                task.providerSessionId ??
                this.selectedConversationDetail.conversation.providerSessionId,
              latestTaskId:
                task.createdAtEpochMs >=
                this.selectedConversationDetail.conversation.updatedAtEpochMs
                  ? task.id
                  : this.selectedConversationDetail.conversation.latestTaskId,
              pendingInputRequestId: task.pendingInputRequestId,
              updatedAtEpochMs: Math.max(
                this.selectedConversationDetail.conversation.updatedAtEpochMs,
                task.createdAtEpochMs,
                task.startedAtEpochMs ?? 0,
                task.finishedAtEpochMs ?? 0,
              ),
            },
            tasks: taskDetails,
            pendingInputRequest:
              task.pendingInputRequestId === null
                ? null
                : this.selectedConversationDetail.pendingInputRequest,
          };
        }
      }

      const activity = buildTaskActivity(previous, task, i18n.global.t);
      if (activity) {
        this.pushActivity(activity);
      }
    },
    upsertShellSession(session: ShellSessionRecord) {
      const existingIndex = this.shellSessions.findIndex(
        (item) => item.id === session.id,
      );
      if (existingIndex >= 0) {
        this.shellSessions.splice(existingIndex, 1, session);
      } else {
        this.shellSessions.unshift(session);
      }
      this.shellSessions.sort(
        (left, right) => right.createdAtEpochMs - left.createdAtEpochMs,
      );
      if (this.shellSessions.length > SHELL_HISTORY_LIMIT) {
        this.shellSessions = this.shellSessions.slice(0, SHELL_HISTORY_LIMIT);
      }

      if (this.selectedShellSessionDetail?.session.id === session.id) {
        this.selectedShellSessionDetail = {
          session,
          inputs: this.selectedShellSessionDetail.inputs,
          outputs: this.selectedShellSessionDetail.outputs,
        };
      }
    },
    upsertPortForward(forward: PortForwardRecord) {
      const existingIndex = this.portForwards.findIndex(
        (item) => item.id === forward.id,
      );
      const previous =
        existingIndex >= 0 ? this.portForwards[existingIndex] : null;
      if (existingIndex >= 0) {
        this.portForwards.splice(existingIndex, 1, forward);
      } else {
        this.portForwards.unshift(forward);
      }
      this.portForwards.sort(
        (left, right) => right.createdAtEpochMs - left.createdAtEpochMs,
      );
      if (this.portForwards.length > PORT_FORWARD_HISTORY_LIMIT) {
        this.portForwards = this.portForwards.slice(
          0,
          PORT_FORWARD_HISTORY_LIMIT,
        );
      }

      const activity = buildPreviewActivity(previous, forward, i18n.global.t);
      if (activity) {
        this.pushActivity(activity);
      }
    },
    upsertTaskEvent(taskEvent: TaskEvent) {
      if (this.selectedTaskDetail?.task.id === taskEvent.taskId) {
        this.selectedTaskDetail = {
          task: this.selectedTaskDetail.task,
          events: [...this.selectedTaskDetail.events, taskEvent],
          pendingInputRequest: this.selectedTaskDetail.pendingInputRequest,
        };
      }
      if (this.selectedConversationDetail) {
        const taskIndex = this.selectedConversationDetail.tasks.findIndex(
          (detail) => detail.task.id === taskEvent.taskId,
        );
        if (taskIndex >= 0) {
          const tasks = [...this.selectedConversationDetail.tasks];
          const detail = tasks[taskIndex];
          tasks.splice(taskIndex, 1, {
            ...detail,
            events: [...detail.events, taskEvent],
          });
          this.selectedConversationDetail = {
            ...this.selectedConversationDetail,
            tasks,
          };
        }
      }
    },
    reconcilePortForwards(forwards: PortForwardRecord[]) {
      const nextIds = new Set(forwards.map((forward) => forward.id));
      for (const forward of forwards) {
        this.upsertPortForward(forward);
      }
      this.portForwards = this.portForwards
        .filter((forward) => nextIds.has(forward.id))
        .slice(0, PORT_FORWARD_HISTORY_LIMIT);
    },
    pushActivity(activity: ActivityItem) {
      const existingIndex = this.activities.findIndex(
        (item) => item.fingerprint === activity.fingerprint,
      );
      if (existingIndex >= 0) {
        this.activities.splice(existingIndex, 1, activity);
      } else {
        this.activities.unshift(activity);
      }
      this.activities.sort(
        (left, right) => right.timestampEpochMs - left.timestampEpochMs,
      );
      if (this.activities.length > ACTIVITY_HISTORY_LIMIT) {
        this.activities = this.activities.slice(0, ACTIVITY_HISTORY_LIMIT);
      }

      void this.publishActivity(activity);
    },
    async publishActivity(activity: ActivityItem) {
      const enabled =
        supportsSystemNotifications(this.appConfig) &&
        (this.appConfig?.notificationChannels ?? []).includes("system");
      this.notificationPermission = await publishSystemActivity(
        activity,
        enabled,
      );
    },
    normalizeShellInput(value: string) {
      return value.endsWith("\n") ? value : `${value}\n`;
    },
    syncTaskSelectionToDevice() {
      if (this.taskScope !== "selected_device" || !this.selectedDeviceId) {
        return;
      }

      const current = this.tasks.find(
        (task) => task.id === this.selectedTaskId,
      );
      if (current?.deviceId === this.selectedDeviceId) {
        return;
      }

      const fallback = this.tasks.find(
        (task) => task.deviceId === this.selectedDeviceId,
      );
      this.selectedTaskId = fallback?.id ?? null;
      if (!fallback) {
        this.selectedTaskDetail = null;
      }
    },
    syncShellSelectionToDevice() {
      if (this.shellScope !== "selected_device" || !this.selectedDeviceId) {
        return;
      }

      const current = this.shellSessions.find(
        (session) => session.id === this.selectedShellSessionId,
      );
      if (current?.deviceId === this.selectedDeviceId) {
        return;
      }

      const fallback = this.shellSessions.find(
        (session) => session.deviceId === this.selectedDeviceId,
      );
      this.selectedShellSessionId = fallback?.id ?? null;
      if (!fallback) {
        this.selectedShellSessionDetail = null;
        this.disconnectShellSocket();
      }
    },
    syncPortForwardSelectionToDevice() {
      if (
        this.portForwardScope !== "selected_device" ||
        !this.selectedDeviceId
      ) {
        return;
      }

      const current = this.portForwards.find(
        (forward) => forward.id === this.selectedPortForwardId,
      );
      if (current?.deviceId === this.selectedDeviceId) {
        return;
      }

      const fallback = this.portForwards.find(
        (forward) => forward.deviceId === this.selectedDeviceId,
      );
      this.selectedPortForwardId = fallback?.id ?? null;
    },
  },
});

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

async function runStoreAction(
  store: { errorCode: string | null; errorMessage: string },
  operation: () => Promise<void>,
) {
  try {
    store.errorCode = null;
    store.errorMessage = "";
    await operation();
  } catch (error) {
    store.errorCode = null;
    store.errorMessage = formatError(error);
  }
}

function formatProviderKind(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function formatProviderLabel(value: string, executionProtocol: string) {
  return `${formatProviderKind(value)} · ${executionProtocol.toUpperCase()}`;
}

function parsePort(value: string) {
  const normalized = value.trim();
  if (!normalized) {
    return null;
  }

  const port = Number.parseInt(normalized, 10);
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    return null;
  }

  return port;
}
