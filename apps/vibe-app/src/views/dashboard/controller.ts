import { computed, inject, onBeforeUnmount, onMounted, provide, ref, watch } from "vue"
import type { InjectionKey } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import { browseWorkspace, inspectGitWorkspace, previewWorkspaceFile } from "@/lib/api"
import { APP_FEATURE_FLAGS, hasAppFeatureFlag } from "@/lib/features"
import { getSupportedLocales, setAppLocale, type AppLocale } from "@/lib/i18n"
import { detectControlClientKind, prefersExplicitRemoteRelayUrl } from "@/lib/platform"
import { getSupportedThemeModes, setThemeMode, type ThemeMode, useTheme } from "@/lib/theme"
import { getRelayBaseUrlPlaceholder, isLoopbackRelayBaseUrl } from "@/lib/runtime"
import { useControlStore } from "@/stores/control"
import type {
  AuditRecord,
  ControlClientKind,
  GitChangedFile,
  GitInspectResponse,
  WorkspaceBrowseResponse,
  WorkspaceEntry,
  WorkspaceFilePreviewResponse
} from "@/types"

const TASK_SCOPE_OPTIONS = ["all", "selected_device"] as const
const TASK_STATUS_OPTIONS = [
  "all",
  "pending",
  "assigned",
  "running",
  "waiting_input",
  "cancel_requested",
  "succeeded",
  "failed",
  "canceled"
] as const
const SHELL_STATUS_OPTIONS = [
  "all",
  "pending",
  "active",
  "close_requested",
  "succeeded",
  "failed",
  "closed"
] as const
const PORT_FORWARD_STATUS_OPTIONS = [
  "all",
  "pending",
  "active",
  "close_requested",
  "closed",
  "failed"
] as const

const DASHBOARD_CONTROLLER_KEY: InjectionKey<DashboardController> = Symbol("dashboard-controller")

export function createDashboardController() {
  const store = useControlStore()
  const { t, locale } = useI18n()
  const supportedLocales = getSupportedLocales()
  const supportedThemeModes = getSupportedThemeModes()
  const { themeMode } = useTheme()
  const {
    activities,
    appConfig,
    auditRecords,
    devices,
    draft,
    errorCode,
    errorMessage,
    eventState,
    health,
    portForwardDraft,
    portForwardScope,
    portForwardStatusFilter,
    portForwards,
    relayAccessToken,
    relayAccessTokenInput,
    relayBaseUrl,
    relayInput,
    selectedDevice,
    selectedPortForward,
    selectedShellSession,
    selectedShellSessionDetail,
    selectedTask,
    selectedTaskDetail,
    shellDraft,
    shellScope,
    shellSessions,
    shellSocketState,
    shellStatusFilter,
    taskScope,
    taskStatusFilter,
    tasks,
    unreadActivityCount,
    visiblePortForwards,
    visibleShellSessions,
    visibleTasks
  } = storeToRefs(store)

  const appName = computed(() => appConfig.value?.appName ?? t("app.name"))
  const localeOptions = computed(() =>
    supportedLocales.map((value) => ({
      value,
      label: t(localeLabelKey(value))
    }))
  )
  const themeOptions = computed(() =>
    supportedThemeModes.map((value) => ({
      value,
      label: t(themeLabelKey(value))
    }))
  )
  const aiSessions = computed(() => visibleTasks.value)
  const workspaceListing = ref<WorkspaceBrowseResponse | null>(null)
  const workspacePreview = ref<WorkspaceFilePreviewResponse | null>(null)
  const workspaceLoading = ref(false)
  const workspacePreviewLoading = ref(false)
  const workspaceError = ref("")
  const gitInspect = ref<GitInspectResponse | null>(null)
  const gitLoading = ref(false)
  const gitError = ref("")
  const selectedWorkspacePath = ref<string | null>(null)
  const localizedErrorMessage = computed(() => {
    if (errorCode.value) {
      return t(`error.${errorCode.value}`)
    }

    return errorMessage.value
  })
  const selectedDeviceAvailableProviders = computed(
    () => selectedDevice.value?.providers.filter((provider) => provider.available) ?? []
  )
  const selectedDeviceUnavailableProviders = computed(
    () => selectedDevice.value?.providers.filter((provider) => !provider.available) ?? []
  )
  const sessionLaunchState = computed(() => {
    if (!relayBaseUrl.value) {
      return "needs_relay"
    }

    if (!selectedDevice.value) {
      return "needs_device"
    }

    if (!selectedDevice.value.online) {
      return "device_offline"
    }

    if (!selectedDeviceAvailableProviders.value.length) {
      return "needs_provider"
    }

    return "ready"
  })
  const canSubmit = computed(
    () =>
      sessionLaunchState.value === "ready" &&
      Boolean(draft.value.provider) &&
      Boolean(draft.value.prompt.trim())
  )
  const canCancel = computed(() =>
    Boolean(
      selectedTask.value &&
        !["succeeded", "failed", "canceled"].includes(selectedTask.value.status)
    )
  )
  const selectedDeviceSupportsShell = computed(
    () => selectedDevice.value?.capabilities.includes("shell") ?? false
  )
  const selectedDeviceSupportsWorkspace = computed(
    () => selectedDevice.value?.capabilities.includes("workspace_browse") ?? false
  )
  const selectedDeviceSupportsGitInspect = computed(
    () => selectedDevice.value?.capabilities.includes("git_inspect") ?? false
  )
  const canOpenShell = computed(
    () => Boolean(selectedDevice.value) && selectedDeviceSupportsShell.value
  )
  const canSendShellInput = computed(() => {
    if (!selectedShellSession.value) {
      return false
    }

    return (
      Boolean(shellDraft.value.input.trim()) &&
      !isShellTerminal(selectedShellSession.value.status) &&
      !selectedShellSession.value.closeRequested
    )
  })
  const canCloseShell = computed(() => {
    if (!selectedShellSession.value) {
      return false
    }

    return (
      !isShellTerminal(selectedShellSession.value.status) &&
      !selectedShellSession.value.closeRequested
    )
  })
  const canCreatePortForward = computed(() => {
    const targetPort = Number.parseInt(portForwardDraft.value.targetPort.trim(), 10)
    return (
      Boolean(selectedDevice.value) &&
      Boolean(portForwardDraft.value.targetHost.trim()) &&
      Number.isInteger(targetPort) &&
      targetPort > 0 &&
      targetPort <= 65_535
    )
  })
  const canClosePortForward = computed(() =>
    Boolean(
      selectedPortForward.value &&
        !["closed", "failed", "close_requested"].includes(selectedPortForward.value.status)
    )
  )
  const shellTimeline = computed(() => {
    const detail = selectedShellSessionDetail.value
    if (!detail) {
      return []
    }

    return [
      ...detail.inputs.map((input) => ({
        key: `input-${input.seq}`,
        stream: "stdin",
        data: input.data,
        timestampEpochMs: input.timestampEpochMs,
        order: input.seq
      })),
      ...detail.outputs.map((output) => ({
        key: `output-${output.seq}`,
        stream: output.stream,
        data: output.data,
        timestampEpochMs: output.timestampEpochMs,
        order: output.seq + 1_000_000
      }))
    ].sort((left, right) => {
      if (left.timestampEpochMs === right.timestampEpochMs) {
        return left.order - right.order
      }

      return left.timestampEpochMs - right.timestampEpochMs
    })
  })
  const currentClientKind = computed(() => detectControlClientKind())
  const platformMatrix = computed(() => appConfig.value?.platformMatrix ?? [])
  const activePlatformCapability = computed(
    () =>
      platformMatrix.value.find((capability) => capability.client === currentClientKind.value) ??
      null
  )
  const currentActor = computed(() => appConfig.value?.currentActor ?? null)
  const showMobileRelayHint = computed(() => prefersExplicitRemoteRelayUrl(appConfig.value))
  const relayPlaceholder = computed(() => getRelayBaseUrlPlaceholder(showMobileRelayHint.value))
  const selectedDeviceSessionCount = computed(() => {
    const deviceId = selectedDevice.value?.id
    return deviceId ? tasks.value.filter((task) => task.deviceId === deviceId).length : 0
  })
  const selectedDeviceShellCount = computed(() => {
    const deviceId = selectedDevice.value?.id
    return deviceId
      ? shellSessions.value.filter((session) => session.deviceId === deviceId).length
      : 0
  })
  const selectedDevicePreviewCount = computed(() => {
    const deviceId = selectedDevice.value?.id
    return deviceId
      ? portForwards.value.filter((forward) => forward.deviceId === deviceId).length
      : 0
  })
  const selectedDeviceWorkingRoot = computed(
    () => selectedDevice.value?.metadata.workingRoot ?? t("common.useAgentWorkingRoot")
  )
  const selectedDeviceAvailableProviderCount = computed(
    () => selectedDevice.value?.providers.filter((provider) => provider.available).length ?? 0
  )
  const selectedPreviewUrl = computed(() =>
    selectedPortForward.value ? buildPreviewUrl(selectedPortForward.value) : ""
  )
  const selectedPreviewIsReady = computed(() => selectedPortForward.value?.status === "active")
  const selectedPreviewLoopbackWarning = computed(
    () =>
      showMobileRelayHint.value &&
      Boolean(selectedPreviewUrl.value) &&
      isLoopbackRelayBaseUrl(selectedPreviewUrl.value)
  )
  const showShellTools = computed(
    () => !appConfig.value || hasAppFeatureFlag(appConfig.value, APP_FEATURE_FLAGS.relayShellSessions)
  )
  const showPreviewTools = computed(
    () =>
      !appConfig.value ||
      hasAppFeatureFlag(appConfig.value, APP_FEATURE_FLAGS.relayTcpForwardingControlPlane)
  )
  const showGitInspect = computed(
    () => !appConfig.value || hasAppFeatureFlag(appConfig.value, APP_FEATURE_FLAGS.sessionGitInspect)
  )
  const showGovernanceSurface = computed(() =>
    hasAppFeatureFlag(appConfig.value, APP_FEATURE_FLAGS.governanceAuditConsole)
  )
  const activityItems = computed(() => activities.value.slice(0, 8))
  const auditTrail = computed(() => auditRecords.value.slice(0, 8))
  const deploymentDocsUrl = computed(() => appConfig.value?.deployment.documentationUrl ?? "")
  const sessionWorkspaceTitle = computed(() => {
    if (selectedTask.value) {
      return selectedTask.value.title
    }

    if (selectedDevice.value) {
      return t("dashboard.sessions.newOnDevice", { name: selectedDevice.value.name })
    }

    return t("dashboard.sessions.title")
  })
  const workspaceSessionCwd = computed(() => selectedTask.value?.cwd ?? undefined)
  const sessionEventCounts = computed(() => {
    const events = selectedTaskDetail.value?.events ?? []
    return {
      assistant: events.filter((event) => event.kind === "assistant_delta").length,
      tool: events.filter((event) => event.kind === "tool_call" || event.kind === "tool_output")
        .length,
      stderr: events.filter((event) => event.kind === "provider_stderr").length
    }
  })
  const visibleChangedFileCount = computed(() =>
    gitInspect.value?.state === "ready" ? gitInspect.value.changedFiles.length : 0
  )
  const supervisionSummary = computed(() => {
    if (!selectedTaskDetail.value) {
      return ""
    }

    const taskStatus = selectedTaskDetail.value.task.status
    const changed = visibleChangedFileCount.value

    if (gitInspect.value?.state !== "ready") {
      return t("dashboard.workspace.supervision.summary.noGit")
    }

    if (["pending", "assigned", "running", "cancel_requested"].includes(taskStatus)) {
      return t("dashboard.workspace.supervision.summary.running", { changed })
    }

    if (taskStatus === "succeeded") {
      return changed
        ? t("dashboard.workspace.supervision.summary.succeededWithChanges", {
            changed,
            branch: gitInspect.value.branchName ?? "HEAD"
          })
        : t("dashboard.workspace.supervision.summary.succeededClean")
    }

    return changed
      ? t("dashboard.workspace.supervision.summary.failedWithChanges", { changed })
      : t("dashboard.workspace.supervision.summary.failedClean")
  })

  const nativeSelectClass =
    "border-input bg-input/30 text-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:border-destructive h-9 w-full rounded-md border px-3 py-2 text-sm shadow-xs outline-none transition focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50"

  function switchLocale(nextLocale: string) {
    setAppLocale(nextLocale)
  }

  function switchTheme(nextThemeMode: string) {
    setThemeMode(nextThemeMode)
  }

  function formatScopeOption(value: string) {
    return value === "all" ? t("common.allDevices") : t("common.selectedDevice")
  }

  function formatTaskStatus(value: string) {
    return translateWithFallback(t, "taskStatus", value)
  }

  function formatTaskStatusOption(value: string) {
    return value === "all" ? t("common.allStatus") : formatTaskStatus(value)
  }

  function formatShellStatus(value: string, closeRequested: boolean) {
    const effectiveValue = closeRequested && value === "active" ? "close_requested" : value
    return translateWithFallback(t, "shellStatus", effectiveValue)
  }

  function formatShellStatusOption(value: string) {
    return value === "all" ? t("common.allStatus") : formatShellStatus(value, false)
  }

  function formatPortForwardStatus(value: string) {
    return translateWithFallback(t, "portForwardStatus", value)
  }

  function formatPortForwardStatusOption(value: string) {
    return value === "all" ? t("common.allStatus") : formatPortForwardStatus(value)
  }

  function formatPortForwardTransport(value: string) {
    return translateWithFallback(t, "transport", value)
  }

  function formatPortForwardProtocol(value: string) {
    return value.toUpperCase()
  }

  function buildPreviewUrl(forward: { relayHost: string; relayPort: number }) {
    try {
      const url = new URL(relayBaseUrl.value || window.location.origin)
      url.protocol = "http:"
      url.hostname = forward.relayHost
      url.port = String(forward.relayPort)
      url.pathname = "/"
      url.search = ""
      url.hash = ""
      return url.toString()
    } catch {
      return `http://${formatEndpoint(forward.relayHost, forward.relayPort)}/`
    }
  }

  function formatDevicePresence(online: boolean) {
    return online ? t("common.online") : t("common.offline")
  }

  function formatConnectionState(value: string) {
    return translateWithFallback(t, "connectionState", value)
  }

  function formatEventKind(value: string) {
    return translateWithFallback(t, "eventKind", value)
  }

  function formatGitFileStatus(value: string) {
    return translateWithFallback(t, "gitFileStatus", value)
  }

  function formatStreamLabel(value: string) {
    return translateWithFallback(t, "stream", value)
  }

  function formatGitDrift(aheadCount: number, behindCount: number) {
    return t("dashboard.workspace.git.driftSummary", {
      ahead: aheadCount,
      behind: behindCount
    })
  }

  function canPreviewGitChangedFile(file: GitChangedFile) {
    return selectedDeviceSupportsWorkspace.value && file.status !== "deleted"
  }

  function formatAuthRequirement(requiresAuth?: boolean | null) {
    return t(requiresAuth ? "auth.required" : "auth.optional")
  }

  function formatUserRole(value: string) {
    return translateWithFallback(t, "role", value)
  }

  function formatAuthMode(value?: string | null) {
    if (!value) {
      return t("common.pending")
    }

    return translateWithFallback(t, "authMode", value)
  }

  function formatStorageKind(value?: string | null) {
    if (!value) {
      return t("common.pending")
    }

    return translateWithFallback(t, "storageKind", value)
  }

  function formatDeploymentMode(value?: string | null) {
    if (!value) {
      return t("common.pending")
    }

    return translateWithFallback(t, "deploymentMode", value)
  }

  function formatControlClient(value: ControlClientKind) {
    return translateWithFallback(t, "platform.client", value)
  }

  function formatNotificationChannel(value: string) {
    return translateWithFallback(t, "notificationChannel", value)
  }

  function formatAuditAction(value: string) {
    return translateWithFallback(t, "auditAction", value)
  }

  function formatAuditOutcome(value: string) {
    return translateWithFallback(t, "auditOutcome", value)
  }

  function formatTimestamp(value: number | null | undefined) {
    if (!value) {
      return t("common.pending")
    }

    return new Date(value).toLocaleString(locale.value)
  }

  function deviceSessionCount(deviceId: string) {
    return tasks.value.filter((task) => task.deviceId === deviceId).length
  }

  function deviceShellCount(deviceId: string) {
    return shellSessions.value.filter((session) => session.deviceId === deviceId).length
  }

  function devicePreviewCount(deviceId: string) {
    return portForwards.value.filter((forward) => forward.deviceId === deviceId).length
  }

  async function loadWorkspace(path?: string | null) {
    if (!selectedDevice.value || !relayBaseUrl.value || !selectedDeviceSupportsWorkspace.value) {
      workspaceListing.value = null
      workspacePreview.value = null
      selectedWorkspacePath.value = null
      workspaceError.value = ""
      return
    }

    workspaceLoading.value = true
    workspaceError.value = ""

    try {
      const response = await browseWorkspace(
        relayBaseUrl.value,
        {
          deviceId: selectedDevice.value.id,
          sessionCwd: workspaceSessionCwd.value,
          path: path ?? undefined
        },
        relayAccessToken.value
      )
      workspaceListing.value = response

      if (
        selectedWorkspacePath.value &&
        !response.entries.some((entry) => entry.path === selectedWorkspacePath.value)
      ) {
        selectedWorkspacePath.value = null
        workspacePreview.value = null
      }
    } catch (error) {
      workspaceError.value = formatClientError(error)
    } finally {
      workspaceLoading.value = false
    }
  }

  async function previewWorkspacePath(path: string) {
    if (!selectedDevice.value || !relayBaseUrl.value) {
      return
    }

    selectedWorkspacePath.value = path
    workspacePreviewLoading.value = true
    workspaceError.value = ""

    try {
      workspacePreview.value = await previewWorkspaceFile(
        relayBaseUrl.value,
        {
          deviceId: selectedDevice.value.id,
          sessionCwd: workspaceSessionCwd.value,
          path,
          line: 1,
          limit: 200
        },
        relayAccessToken.value
      )
    } catch (error) {
      workspaceError.value = formatClientError(error)
    } finally {
      workspacePreviewLoading.value = false
    }
  }

  async function openWorkspaceEntry(entry: WorkspaceEntry) {
    if (entry.kind === "directory") {
      selectedWorkspacePath.value = null
      workspacePreview.value = null
      await loadWorkspace(entry.path)
      return
    }

    await previewWorkspacePath(entry.path)
  }

  async function refreshWorkspace() {
    await loadWorkspace(workspaceListing.value?.path)
  }

  async function navigateWorkspaceUp() {
    if (!workspaceListing.value?.parentPath) {
      return
    }

    await loadWorkspace(workspaceListing.value.parentPath)
  }

  async function loadGitInspect() {
    if (
      !selectedDevice.value ||
      !relayBaseUrl.value ||
      !showGitInspect.value ||
      !selectedDeviceSupportsGitInspect.value
    ) {
      gitInspect.value = null
      gitLoading.value = false
      gitError.value = ""
      return
    }

    gitLoading.value = true
    gitError.value = ""

    try {
      gitInspect.value = await inspectGitWorkspace(
        relayBaseUrl.value,
        {
          deviceId: selectedDevice.value.id,
          sessionCwd: workspaceSessionCwd.value
        },
        relayAccessToken.value
      )
    } catch (error) {
      gitError.value = formatClientError(error)
    } finally {
      gitLoading.value = false
    }
  }

  async function refreshGitInspect() {
    await loadGitInspect()
  }

  async function openGitChangedFile(file: GitChangedFile) {
    if (!canPreviewGitChangedFile(file)) {
      return
    }

    await previewWorkspacePath(file.path)
  }

  watch(
    [
      () => selectedDevice.value?.id ?? "",
      () => selectedTask.value?.id ?? "",
      () => selectedTask.value?.cwd ?? "",
      () => selectedTask.value?.status ?? "",
      () => relayBaseUrl.value,
      () => selectedDeviceSupportsWorkspace.value,
      () => selectedDeviceSupportsGitInspect.value,
      () => showGitInspect.value
    ],
    async ([deviceId]) => {
      workspacePreview.value = null
      selectedWorkspacePath.value = null

      if (!deviceId) {
        workspaceListing.value = null
        workspaceError.value = ""
        gitInspect.value = null
        gitError.value = ""
        return
      }

      await Promise.all([loadWorkspace(), loadGitInspect()])
    },
    { immediate: true }
  )

  onMounted(() => {
    void store.initialize()
  })

  onBeforeUnmount(() => {
    store.disposeRealtime()
  })

  return {
    TASK_SCOPE_OPTIONS,
    TASK_STATUS_OPTIONS,
    SHELL_STATUS_OPTIONS,
    PORT_FORWARD_STATUS_OPTIONS,
    store,
    locale,
    themeMode,
    activities,
    activityItems,
    activePlatformCapability,
    aiSessions,
    appConfig,
    appName,
    auditRecords,
    auditTrail,
    canCancel,
    canClosePortForward,
    canCloseShell,
    canCreatePortForward,
    canOpenShell,
    canSendShellInput,
    canSubmit,
    currentActor,
    currentClientKind,
    deploymentDocsUrl,
    devices,
    draft,
    errorCode,
    errorMessage,
    eventState,
    gitError,
    gitInspect,
    gitLoading,
    health,
    localeOptions,
    localizedErrorMessage,
    nativeSelectClass,
    portForwardDraft,
    portForwardScope,
    portForwardStatusFilter,
    portForwards,
    relayAccessToken,
    relayAccessTokenInput,
    relayBaseUrl,
    relayInput,
    relayPlaceholder,
    selectedDevice,
    selectedDeviceAvailableProviders,
    selectedDeviceAvailableProviderCount,
    selectedDevicePreviewCount,
    selectedDeviceSessionCount,
    selectedDeviceShellCount,
    selectedDeviceSupportsGitInspect,
    selectedDeviceSupportsShell,
    selectedDeviceUnavailableProviders,
    selectedDeviceSupportsWorkspace,
    selectedDeviceWorkingRoot,
    selectedPortForward,
    selectedPreviewIsReady,
    selectedPreviewLoopbackWarning,
    selectedPreviewUrl,
    selectedShellSession,
    selectedShellSessionDetail,
    selectedTask,
    selectedTaskDetail,
    selectedWorkspacePath,
    sessionEventCounts,
    sessionWorkspaceTitle,
    shellDraft,
    shellScope,
    shellSessions,
    shellSocketState,
    shellStatusFilter,
    shellTimeline,
    showGitInspect,
    showGovernanceSurface,
    showMobileRelayHint,
    showPreviewTools,
    showShellTools,
    supervisionSummary,
    taskScope,
    taskStatusFilter,
    tasks,
    themeOptions,
    unreadActivityCount,
    visibleChangedFileCount,
    visiblePortForwards,
    visibleShellSessions,
    visibleTasks,
    workspaceError,
    workspaceListing,
    workspaceLoading,
    workspacePreview,
    workspacePreviewLoading,
    activitySeverityClass,
    auditOutcomeClass,
    buildPreviewUrl,
    canPreviewGitChangedFile,
    devicePreviewCount,
    deviceSessionCount,
    deviceShellCount,
    eventKindClass,
    formatAuditAction,
    formatAuditOutcome,
    formatAuthMode,
    formatAuthRequirement,
    formatConnectionState,
    formatControlClient,
    formatDeploymentMode,
    formatDevicePresence,
    formatEndpoint,
    formatEventKind,
    formatExecutionProtocol,
    formatGitDrift,
    formatGitFileStatus,
    formatNotificationChannel,
    formatPortForwardProtocol,
    formatPortForwardStatus,
    formatPortForwardStatusOption,
    formatPortForwardTransport,
    formatProviderKind,
    formatProviderSummary,
    formatScopeOption,
    formatShellStatus,
    formatShellStatusOption,
    formatStorageKind,
    formatStreamLabel,
    formatTaskStatus,
    formatTaskStatusOption,
    formatTimestamp,
    formatUserRole,
    navigateWorkspaceUp,
    openGitChangedFile,
    openWorkspaceEntry,
    platformCapabilityStateClass,
    portForwardStatusClass,
    previewWorkspacePath,
    providerBadgeClass,
    refreshGitInspect,
    refreshWorkspace,
    selectedStateClass,
    shellStatusClass,
    statusBadgeClass,
    switchLocale,
    switchTheme,
    sessionLaunchState,
    taskStatusClass,
    terminalStreamClass,
    workspaceSessionCwd
  }
}

export type DashboardController = ReturnType<typeof createDashboardController>

export function provideDashboardController() {
  const controller = createDashboardController()
  provide(DASHBOARD_CONTROLLER_KEY, controller)
  return controller
}

export function useDashboardController() {
  const controller = inject(DASHBOARD_CONTROLLER_KEY)

  if (!controller) {
    throw new Error("Dashboard controller is not available")
  }

  return controller
}

function localeLabelKey(value: AppLocale) {
  return value === "zh-CN" ? "locale.zhCN" : "locale.en"
}

function themeLabelKey(value: ThemeMode) {
  return `theme.${value}`
}

function formatClientError(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

function translateWithFallback(
  t: (key: string, named?: Record<string, unknown>) => string,
  prefix: string,
  value: string
) {
  const key = `${prefix}.${value}`
  const translated = t(key)
  return translated === key ? value.replaceAll("_", " ") : translated
}

function formatProviderKind(value: string) {
  return value
    .split("_")
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ")
}

function formatExecutionProtocol(value: string) {
  return value.toUpperCase()
}

function formatProviderSummary(kind: string, executionProtocol: string) {
  return `${formatProviderKind(kind)} · ${formatExecutionProtocol(executionProtocol)}`
}

function formatEndpoint(host: string, port: number) {
  return `${host}:${port}`
}

function isShellTerminal(value: string) {
  return ["succeeded", "failed", "closed"].includes(value)
}

function selectedStateClass(active: boolean) {
  return active
    ? "border-primary/60 bg-primary/10 shadow-lg shadow-primary/10"
    : "border-border/60 bg-background/35 hover:bg-accent/25"
}

function statusBadgeClass(kind: "online" | "offline" | "running" | "warning" | "done" | "error") {
  switch (kind) {
    case "online":
      return "border-emerald-500/30 bg-emerald-500/12 text-emerald-700 dark:text-emerald-200"
    case "offline":
      return "border-rose-500/30 bg-rose-500/12 text-rose-700 dark:text-rose-200"
    case "running":
      return "border-sky-500/30 bg-sky-500/12 text-sky-700 dark:text-sky-200"
    case "warning":
      return "border-amber-500/30 bg-amber-500/12 text-amber-700 dark:text-amber-200"
    case "done":
      return "border-emerald-500/30 bg-emerald-500/12 text-emerald-700 dark:text-emerald-200"
    case "error":
      return "border-rose-500/30 bg-rose-500/12 text-rose-700 dark:text-rose-200"
  }
}

function taskStatusClass(status: string) {
  if (status === "running" || status === "assigned") {
    return statusBadgeClass("running")
  }
  if (status === "pending" || status === "cancel_requested" || status === "waiting_input") {
    return statusBadgeClass("warning")
  }
  if (status === "succeeded") {
    return statusBadgeClass("done")
  }
  return statusBadgeClass("error")
}

function portForwardStatusClass(status: string) {
  if (status === "active") {
    return statusBadgeClass("running")
  }
  if (status === "pending" || status === "close_requested") {
    return statusBadgeClass("warning")
  }
  if (status === "closed") {
    return statusBadgeClass("done")
  }
  return statusBadgeClass("error")
}

function shellStatusClass(status: string) {
  if (status === "active") {
    return statusBadgeClass("running")
  }
  if (status === "pending" || status === "close_requested") {
    return statusBadgeClass("warning")
  }
  if (status === "succeeded" || status === "closed") {
    return statusBadgeClass("done")
  }
  return statusBadgeClass("error")
}

function providerBadgeClass(available: boolean) {
  return available
    ? "border-sky-500/30 bg-sky-500/12 text-sky-700 dark:text-sky-100"
    : "border-rose-500/30 bg-rose-500/12 text-rose-700 dark:text-rose-200"
}

function activitySeverityClass(severity: "info" | "success" | "warning" | "error") {
  switch (severity) {
    case "success":
      return statusBadgeClass("done")
    case "warning":
      return statusBadgeClass("warning")
    case "error":
      return statusBadgeClass("error")
    default:
      return statusBadgeClass("running")
  }
}

function auditOutcomeClass(outcome: AuditRecord["outcome"]) {
  switch (outcome) {
    case "succeeded":
      return statusBadgeClass("done")
    case "rejected":
      return statusBadgeClass("warning")
    case "failed":
      return statusBadgeClass("error")
  }
}

function platformCapabilityStateClass(enabled: boolean) {
  return enabled
    ? "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-100"
    : "border-border/70 bg-background/55 text-muted-foreground"
}

function eventKindClass(kind: string) {
  switch (kind) {
    case "assistant_delta":
      return "border-sky-500/30 bg-sky-500/12 text-sky-700 dark:text-sky-200"
    case "tool_call":
    case "tool_output":
      return "border-violet-500/30 bg-violet-500/12 text-violet-700 dark:text-violet-200"
    case "provider_stderr":
      return "border-rose-500/30 bg-rose-500/12 text-rose-700 dark:text-rose-200"
    case "status":
      return "border-amber-500/30 bg-amber-500/12 text-amber-700 dark:text-amber-200"
    default:
      return "border-border/70 bg-background/55 text-foreground"
  }
}

function terminalStreamClass(stream: string) {
  switch (stream) {
    case "stdin":
      return "border-amber-500/20 bg-amber-500/5"
    case "stdout":
      return "border-emerald-500/20 bg-emerald-500/5"
    case "stderr":
      return "border-rose-500/20 bg-rose-500/5"
    default:
      return "border-sky-500/20 bg-sky-500/5"
  }
}
