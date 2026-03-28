import { defineStore } from "pinia";
import {
  cancelTask,
  closePortForward,
  closeShellSession,
  createPortForward,
  createShellSession,
  createTask,
  fetchAppConfig,
  fetchAuditEvents,
  fetchDevices,
  fetchHealth,
  fetchPortForwards,
  fetchShellSessionDetail,
  fetchShellSessions,
  fetchTaskDetail,
  fetchTasks,
  sendShellInput
} from "../lib/api";
import {
  buildEventStreamUrl,
  buildWebSocketUrl,
  loadTauriConfig,
  normalizeRelayBaseUrl,
  persistRelayAccessToken,
  persistRelayBaseUrl,
  resolveInitialRelayAccessToken,
  resolveInitialRelayBaseUrl
} from "../lib/runtime";
import { i18n } from "../lib/i18n";
import {
  buildPreviewActivity,
  buildTaskActivity,
  publishSystemActivity,
  readNotificationPermission,
  type ActivityItem
} from "../lib/notifications";
import {
  resolveCurrentPlatformCapability,
  supportsSystemNotifications
} from "../lib/platform";
import type {
  AppConfig,
  AuditRecord,
  CreatePortForwardPayload,
  CreateTaskPayload,
  DeviceRecord,
  ExecutionProtocol,
  PortForwardRecord,
  PortForwardStatus,
  ProviderKind,
  RelayEventEnvelope,
  ServiceHealth,
  ShellSessionDetailResponse,
  ShellSessionRecord,
  ShellSessionStatus,
  TaskDetailResponse,
  TaskEvent,
  TaskRecord,
  TaskStatus
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

let activeEventSource: EventSource | null = null;
let activeShellPollTimer: number | null = null;
let activePortForwardPollTimer: number | null = null;
let activeShellSocket: WebSocket | null = null;
let activeShellSocketSessionId: string | null = null;
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
    selectedTaskId: null as string | null,
    selectedShellSessionId: null as string | null,
    selectedPortForwardId: null as string | null,
    selectedTaskDetail: null as TaskDetailResponse | null,
    selectedShellSessionDetail: null as ShellSessionDetailResponse | null,
    eventState: "disconnected" as "disconnected" | "connecting" | "connected",
    isBootstrapping: false,
    isShellPolling: false,
    isPortForwardPolling: false,
    isAuditLoading: false,
    shellSocketState: "disconnected" as "disconnected" | "connecting" | "connected",
    notificationPermission: "default" as NotificationPermission,
    errorCode: null as string | null,
    errorMessage: "",
    draft: {
      title: "",
      provider: "",
      cwd: "",
      model: "",
      prompt: ""
    } as DraftTask,
    shellDraft: {
      cwd: "",
      input: ""
    } as DraftShell,
    portForwardDraft: {
      targetHost: "127.0.0.1",
      targetPort: ""
    } as DraftPortForward
  }),
  getters: {
    selectedDevice(state) {
      return state.devices.find((device) => device.id === state.selectedDeviceId) ?? null;
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
        state.shellSessions.find((session) => session.id === state.selectedShellSessionId) ??
        null
      );
    },
    selectedPortForward(state) {
      return state.portForwards.find((forward) => forward.id === state.selectedPortForwardId) ?? null;
    },
    visibleTasks(state) {
      return state.tasks.filter((task) => {
        if (state.taskScope === "selected_device" && task.deviceId !== state.selectedDeviceId) {
          return false;
        }
        if (state.taskStatusFilter !== "all" && task.status !== state.taskStatusFilter) {
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
    availableProviders(): { kind: ProviderKind; label: string; executionProtocol: ExecutionProtocol }[] {
      const device = this.selectedDevice;
      if (!device) {
        return [];
      }

      return device.providers
        .filter((provider) => provider.available)
        .map((provider) => ({
          kind: provider.kind,
          label: formatProviderLabel(provider.kind, provider.executionProtocol),
          executionProtocol: provider.executionProtocol
        }));
    },
    unreadActivityCount(state) {
      return state.activities.filter((activity) => activity.unread).length;
    },
    recentActivities(state) {
      return [...state.activities].sort(
        (left, right) => right.timestampEpochMs - left.timestampEpochMs
      );
    },
    currentPlatformCapability(state) {
      return resolveCurrentPlatformCapability(state.appConfig);
    }
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
          fetchHealth(this.relayBaseUrl)
        ]);

        this.appConfig = appConfig;
        this.health = health;
        const [devices, tasks, shellSessions, portForwards, auditRecords] = await Promise.all([
          fetchDevices(this.relayBaseUrl, this.relayAccessToken),
          fetchTasks(this.relayBaseUrl, this.relayAccessToken, { limit: TASK_HISTORY_LIMIT }),
          fetchShellSessions(this.relayBaseUrl, this.relayAccessToken, {
            limit: SHELL_HISTORY_LIMIT
          }),
          fetchPortForwards(this.relayBaseUrl, this.relayAccessToken, {
            limit: PORT_FORWARD_HISTORY_LIMIT
          }),
          fetchAuditEvents(this.relayBaseUrl, this.relayAccessToken, {
            limit: AUDIT_HISTORY_LIMIT
          })
        ]);
        this.devices = devices;
        this.tasks = tasks;
        this.shellSessions = shellSessions;
        this.portForwards = portForwards;
        this.auditRecords = auditRecords;
        this.ensureSelections();
        this.syncTaskSelectionToDevice();
        this.syncShellSelectionToDevice();
        this.syncPortForwardSelectionToDevice();

        await Promise.all([
          this.selectedTaskId ? this.loadTaskDetail(this.selectedTaskId) : Promise.resolve(),
          this.selectedShellSessionId
            ? this.loadShellSessionDetail(this.selectedShellSessionId)
            : Promise.resolve()
        ]);
        this.connectShellSocket();
      });
    },
    ensureSelections() {
      if (!this.devices.some((device) => device.id === this.selectedDeviceId)) {
        this.selectedDeviceId = this.devices[0]?.id ?? null;
      }

      if (!this.tasks.some((task) => task.id === this.selectedTaskId)) {
        this.selectedTaskId = this.tasks[0]?.id ?? null;
      }

      if (!this.shellSessions.some((session) => session.id === this.selectedShellSessionId)) {
        this.selectedShellSessionId = this.shellSessions[0]?.id ?? null;
      }

      if (!this.portForwards.some((forward) => forward.id === this.selectedPortForwardId)) {
        this.selectedPortForwardId = this.portForwards[0]?.id ?? null;
      }

      const availableProviders = this.availableProviders;
      if (!availableProviders.some((provider) => provider.kind === this.draft.provider)) {
        this.draft.provider = availableProviders[0]?.kind ?? "";
      }
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
          this.selectedShellSessionDetail?.session.id !== this.selectedShellSessionId
        ) {
          await this.loadShellSessionDetail(this.selectedShellSessionId);
        }
        this.connectShellSocket();
      });
    },
    async selectTask(taskId: string) {
      await runStoreAction(this, async () => {
        this.selectedTaskId = taskId;
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
        unread: false
      }));
    },
    async loadTaskDetail(taskId: string) {
      this.selectedTaskDetail = await fetchTaskDetail(
        this.relayBaseUrl,
        taskId,
        this.relayAccessToken
      );
    },
    async loadShellSessionDetail(sessionId: string) {
      this.selectedShellSessionDetail = await fetchShellSessionDetail(
        this.relayBaseUrl,
        sessionId,
        this.relayAccessToken
      );
    },
    async submitTask() {
      await runStoreAction(this, async () => {
        if (!this.selectedDeviceId || !this.draft.provider || !this.draft.prompt.trim()) {
          return;
        }

        const payload: CreateTaskPayload = {
          deviceId: this.selectedDeviceId,
          provider: this.draft.provider,
          prompt: this.draft.prompt.trim(),
          title: this.draft.title.trim() || undefined,
          cwd: this.draft.cwd.trim() || undefined,
          model: this.draft.model.trim() || undefined
        };

        const task = await createTask(this.relayBaseUrl, payload, this.relayAccessToken);
        this.tasks = [task, ...this.tasks.filter((item) => item.id !== task.id)].slice(
          0,
          TASK_HISTORY_LIMIT
        );
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
            cwd: this.shellDraft.cwd.trim() || undefined
          },
          this.relayAccessToken
        );
        this.upsertShellSession(session);
        this.selectedShellSessionId = session.id;
        await this.loadShellSessionDetail(session.id);
        this.connectShellSocket();
        this.shellDraft.input = "";
      });
    },
    async createPortForward() {
      await runStoreAction(this, async () => {
        if (!this.selectedDeviceId || !this.portForwardDraft.targetHost.trim()) {
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
          targetPort
        };

        const forward = await createPortForward(
          this.relayBaseUrl,
          payload,
          this.relayAccessToken
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
          this.relayAccessToken
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
          this.relayAccessToken
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
          this.relayAccessToken
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
          this.relayAccessToken
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
      this.tasks = [];
      this.shellSessions = [];
      this.portForwards = [];
      this.selectedDeviceId = null;
      this.selectedTaskId = null;
      this.selectedShellSessionId = null;
      this.selectedPortForwardId = null;
      this.selectedTaskDetail = null;
      this.selectedShellSessionDetail = null;
    },
    connectEvents() {
      this.disconnectEvents();

      if (!this.relayBaseUrl || (this.appConfig?.requiresAuth && !this.relayAccessToken)) {
        return;
      }

      this.eventState = "connecting";
      activeEventSource = new EventSource(
        buildEventStreamUrl(this.relayBaseUrl, this.relayAccessToken)
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
      if (!this.relayBaseUrl || (this.appConfig?.requiresAuth && !this.relayAccessToken)) {
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
      if (!this.relayBaseUrl || (this.appConfig?.requiresAuth && !this.relayAccessToken)) {
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
          this.relayAccessToken
        )
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
          const detail = JSON.parse(event.data as string) as ShellSessionDetailResponse;
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
      if (!this.relayBaseUrl || (this.appConfig?.requiresAuth && !this.relayAccessToken)) {
        return;
      }

      this.isShellPolling = true;
      try {
        const sessions = await fetchShellSessions(this.relayBaseUrl, this.relayAccessToken, {
          limit: SHELL_HISTORY_LIMIT
        });
        this.shellSessions = sessions;
        this.ensureSelections();
        this.syncShellSelectionToDevice();
        if (this.selectedShellSessionId && this.shellSocketState !== "connected") {
          this.selectedShellSessionDetail = await fetchShellSessionDetail(
            this.relayBaseUrl,
            this.selectedShellSessionId,
            this.relayAccessToken
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
      if (!this.relayBaseUrl || (this.appConfig?.requiresAuth && !this.relayAccessToken)) {
        return;
      }

      this.isPortForwardPolling = true;
      try {
        const portForwards = await fetchPortForwards(this.relayBaseUrl, this.relayAccessToken, {
          limit: PORT_FORWARD_HISTORY_LIMIT
        });
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
      }
      if (envelope.taskEvent) {
        this.upsertTaskEvent(envelope.taskEvent);
      }

      this.ensureSelections();
      this.syncTaskSelectionToDevice();
      this.syncShellSelectionToDevice();
      this.syncPortForwardSelectionToDevice();
    },
    upsertDevice(device: DeviceRecord) {
      const existingIndex = this.devices.findIndex((item) => item.id === device.id);
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
      this.tasks.sort((left, right) => right.createdAtEpochMs - left.createdAtEpochMs);
      if (this.tasks.length > TASK_HISTORY_LIMIT) {
        this.tasks = this.tasks.slice(0, TASK_HISTORY_LIMIT);
      }

      if (this.selectedTaskDetail?.task.id === task.id) {
        this.selectedTaskDetail = {
          task,
          events: this.selectedTaskDetail.events
        };
      }

      const activity = buildTaskActivity(previous, task, i18n.global.t);
      if (activity) {
        this.pushActivity(activity);
      }
    },
    upsertShellSession(session: ShellSessionRecord) {
      const existingIndex = this.shellSessions.findIndex((item) => item.id === session.id);
      if (existingIndex >= 0) {
        this.shellSessions.splice(existingIndex, 1, session);
      } else {
        this.shellSessions.unshift(session);
      }
      this.shellSessions.sort((left, right) => right.createdAtEpochMs - left.createdAtEpochMs);
      if (this.shellSessions.length > SHELL_HISTORY_LIMIT) {
        this.shellSessions = this.shellSessions.slice(0, SHELL_HISTORY_LIMIT);
      }

      if (this.selectedShellSessionDetail?.session.id === session.id) {
        this.selectedShellSessionDetail = {
          session,
          inputs: this.selectedShellSessionDetail.inputs,
          outputs: this.selectedShellSessionDetail.outputs
        };
      }
    },
    upsertPortForward(forward: PortForwardRecord) {
      const existingIndex = this.portForwards.findIndex((item) => item.id === forward.id);
      const previous = existingIndex >= 0 ? this.portForwards[existingIndex] : null;
      if (existingIndex >= 0) {
        this.portForwards.splice(existingIndex, 1, forward);
      } else {
        this.portForwards.unshift(forward);
      }
      this.portForwards.sort((left, right) => right.createdAtEpochMs - left.createdAtEpochMs);
      if (this.portForwards.length > PORT_FORWARD_HISTORY_LIMIT) {
        this.portForwards = this.portForwards.slice(0, PORT_FORWARD_HISTORY_LIMIT);
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
          events: [...this.selectedTaskDetail.events, taskEvent]
        };
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
        (item) => item.fingerprint === activity.fingerprint
      );
      if (existingIndex >= 0) {
        this.activities.splice(existingIndex, 1, activity);
      } else {
        this.activities.unshift(activity);
      }
      this.activities.sort((left, right) => right.timestampEpochMs - left.timestampEpochMs);
      if (this.activities.length > ACTIVITY_HISTORY_LIMIT) {
        this.activities = this.activities.slice(0, ACTIVITY_HISTORY_LIMIT);
      }

      void this.publishActivity(activity);
    },
    async publishActivity(activity: ActivityItem) {
      const enabled =
        supportsSystemNotifications(this.appConfig) &&
        (this.appConfig?.notificationChannels ?? []).includes("system");
      this.notificationPermission = await publishSystemActivity(activity, enabled);
    },
    normalizeShellInput(value: string) {
      return value.endsWith("\n") ? value : `${value}\n`;
    },
    syncTaskSelectionToDevice() {
      if (this.taskScope !== "selected_device" || !this.selectedDeviceId) {
        return;
      }

      const current = this.tasks.find((task) => task.id === this.selectedTaskId);
      if (current?.deviceId === this.selectedDeviceId) {
        return;
      }

      const fallback = this.tasks.find((task) => task.deviceId === this.selectedDeviceId);
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
        (session) => session.id === this.selectedShellSessionId
      );
      if (current?.deviceId === this.selectedDeviceId) {
        return;
      }

      const fallback = this.shellSessions.find(
        (session) => session.deviceId === this.selectedDeviceId
      );
      this.selectedShellSessionId = fallback?.id ?? null;
      if (!fallback) {
        this.selectedShellSessionDetail = null;
        this.disconnectShellSocket();
      }
    },
    syncPortForwardSelectionToDevice() {
      if (this.portForwardScope !== "selected_device" || !this.selectedDeviceId) {
        return;
      }

      const current = this.portForwards.find(
        (forward) => forward.id === this.selectedPortForwardId
      );
      if (current?.deviceId === this.selectedDeviceId) {
        return;
      }

      const fallback = this.portForwards.find(
        (forward) => forward.deviceId === this.selectedDeviceId
      );
      this.selectedPortForwardId = fallback?.id ?? null;
    }
  }
});

function formatError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

async function runStoreAction(
  store: { errorCode: string | null; errorMessage: string },
  operation: () => Promise<void>
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
