<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import {
  Activity,
  BellRing,
  Eye,
  FolderCode,
  GitBranch,
  Laptop2,
  RefreshCw,
  Server,
  ShieldCheck,
  Sparkles,
  TerminalSquare
} from "lucide-vue-next"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Separator } from "@/components/ui/separator"
import { Textarea } from "@/components/ui/textarea"
import { browseWorkspace, inspectGitWorkspace, previewWorkspaceFile } from "@/lib/api"
import { getSupportedLocales, setAppLocale, type AppLocale } from "@/lib/i18n"
import { detectControlClientKind, prefersExplicitRemoteRelayUrl } from "@/lib/platform"
import { getSupportedThemeModes, setThemeMode, type ThemeMode, useTheme } from "@/lib/theme"
import {
  getRelayBaseUrlPlaceholder,
  isLoopbackRelayBaseUrl
} from "@/lib/runtime"
import type {
  AuditRecord,
  ControlClientKind,
  GitChangedFile,
  PlatformCapability,
  GitInspectResponse,
  WorkspaceBrowseResponse,
  WorkspaceEntry,
  WorkspaceFilePreviewResponse
} from "@/types"
import { useControlStore } from "@/stores/control"

const TASK_SCOPE_OPTIONS = ["all", "selected_device"] as const
const TASK_STATUS_OPTIONS = [
  "all",
  "pending",
  "assigned",
  "running",
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
const canSubmit = computed(
  () =>
    Boolean(selectedDevice.value) &&
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
    targetPort <= 65535
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
      order: output.seq + 1000000
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
  () => platformMatrix.value.find((capability) => capability.client === currentClientKind.value) ?? null
)
const currentActor = computed(() => appConfig.value?.currentActor ?? null)
const relayPlaceholder = computed(() => getRelayBaseUrlPlaceholder(showMobileRelayHint.value))
const showMobileRelayHint = computed(() => prefersExplicitRemoteRelayUrl(appConfig.value))
const showLoopbackRelayWarning = computed(
  () =>
    showMobileRelayHint.value &&
    Boolean(relayInput.value.trim()) &&
    isLoopbackRelayBaseUrl(relayInput.value)
)
const selectedDeviceSessionCount = computed(() =>
  selectedDevice.value
    ? tasks.value.filter((task) => task.deviceId === selectedDevice.value?.id).length
    : 0
)
const selectedDeviceShellCount = computed(() =>
  selectedDevice.value
    ? shellSessions.value.filter((session) => session.deviceId === selectedDevice.value?.id).length
    : 0
)
const selectedDevicePreviewCount = computed(() =>
  selectedDevice.value
    ? portForwards.value.filter((forward) => forward.deviceId === selectedDevice.value?.id).length
    : 0
)
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
  () => !appConfig.value || appConfig.value.featureFlags.includes("relay_shell_sessions")
)
const showPreviewTools = computed(
  () =>
    !appConfig.value ||
    appConfig.value.featureFlags.includes("relay_tcp_forwarding_control_plane")
)
const showGitInspect = computed(
  () => !appConfig.value || appConfig.value.featureFlags.includes("session_git_inspect")
)
const activityItems = computed(() => activities.value.slice(0, 8))
const auditTrail = computed(() => auditRecords.value.slice(0, 8))
const deploymentDocsUrl = computed(() => appConfig.value?.deployment.documentationUrl ?? "")
const connectionGuidance = computed(() => {
  if (showLoopbackRelayWarning.value) {
    return t("dashboard.deployment.guidance.explicitRemote")
  }

  if (appConfig.value?.deployment.mode === "hosted_compatible") {
    return t("dashboard.deployment.guidance.hostedCompatible")
  }

  return t("dashboard.deployment.guidance.selfHosted")
})
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

function localeLabelKey(value: AppLocale) {
  return value === "zh-CN" ? "locale.zhCN" : "locale.en"
}

function switchLocale(nextLocale: string) {
  setAppLocale(nextLocale)
}

function themeLabelKey(value: ThemeMode) {
  return `theme.${value}`
}

function switchTheme(nextThemeMode: string) {
  setThemeMode(nextThemeMode)
}

function formatClientError(error: unknown) {
  return error instanceof Error ? error.message : String(error)
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

function translateWithFallback(prefix: string, value: string) {
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

function formatScopeOption(value: string) {
  return value === "all" ? t("common.allDevices") : t("common.selectedDevice")
}

function formatTaskStatusOption(value: string) {
  return value === "all" ? t("common.allStatus") : formatTaskStatus(value)
}

function formatShellStatusOption(value: string) {
  return value === "all" ? t("common.allStatus") : formatShellStatus(value, false)
}

function formatPortForwardStatusOption(value: string) {
  return value === "all" ? t("common.allStatus") : formatPortForwardStatus(value)
}

function formatTimestamp(value: number | null | undefined) {
  if (!value) {
    return t("common.pending")
  }

  return new Date(value).toLocaleString(locale.value)
}

function formatTaskStatus(value: string) {
  return translateWithFallback("taskStatus", value)
}

function formatShellStatus(value: string, closeRequested: boolean) {
  const effectiveValue = closeRequested && value === "active" ? "close_requested" : value
  return translateWithFallback("shellStatus", effectiveValue)
}

function formatPortForwardStatus(value: string) {
  return translateWithFallback("portForwardStatus", value)
}

function formatPortForwardTransport(value: string) {
  return translateWithFallback("transport", value)
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

function formatEndpoint(host: string, port: number) {
  return `${host}:${port}`
}

function formatDevicePresence(online: boolean) {
  return online ? t("common.online") : t("common.offline")
}

function formatConnectionState(value: string) {
  return translateWithFallback("connectionState", value)
}

function formatEventKind(value: string) {
  return translateWithFallback("eventKind", value)
}

function formatGitFileStatus(value: string) {
  return translateWithFallback("gitFileStatus", value)
}

function formatStreamLabel(value: string) {
  return translateWithFallback("stream", value)
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
  return translateWithFallback("role", value)
}

function formatAuthMode(value?: string | null) {
  if (!value) {
    return t("common.pending")
  }

  return translateWithFallback("authMode", value)
}

function formatStorageKind(value?: string | null) {
  if (!value) {
    return t("common.pending")
  }

  return translateWithFallback("storageKind", value)
}

function formatDeploymentMode(value?: string | null) {
  if (!value) {
    return t("common.pending")
  }

  return translateWithFallback("deploymentMode", value)
}

function formatControlClient(value: ControlClientKind) {
  return translateWithFallback("platform.client", value)
}

function formatNotificationChannel(value: string) {
  return translateWithFallback("notificationChannel", value)
}

function formatAuditAction(value: string) {
  return translateWithFallback("auditAction", value)
}

function formatAuditOutcome(value: string) {
  return translateWithFallback("auditOutcome", value)
}

function isShellTerminal(value: string) {
  return ["succeeded", "failed", "closed"].includes(value)
}

function deviceSessionCount(deviceId: string) {
  return tasks.value.filter((task) => task.deviceId === deviceId).length
}

function deviceShellCount(deviceId: string) {
  return shellSessions.value.filter((session) => session.deviceId === deviceId).length
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
  if (status === "pending" || status === "cancel_requested") {
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

function isCurrentClient(capability: PlatformCapability) {
  return capability.client === currentClientKind.value
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
</script>

<template>
  <main class="mx-auto flex w-full max-w-[1400px] flex-col gap-4 px-4 py-6 md:px-6 xl:px-8">
    <Card class="overflow-hidden border-border/70 bg-card/85 text-foreground shadow-2xl backdrop-blur-xl">
      <CardContent class="grid gap-6 p-6 lg:grid-cols-[1.35fr_1fr]">
        <div class="space-y-4">
          <Badge variant="outline" class="border-sky-500/30 bg-sky-500/12 text-sky-700 dark:text-sky-100">
            <Sparkles class="size-3.5" />
            {{ t("dashboard.heroBadge") }}
          </Badge>
          <div class="space-y-3">
            <h1 class="text-4xl font-semibold tracking-tight text-foreground md:text-6xl">
              {{ appName }}
            </h1>
            <p class="max-w-3xl text-base leading-7 text-muted-foreground md:text-lg">
              {{ t("dashboard.heroDescription") }}
            </p>
          </div>
        </div>

        <div class="rounded-3xl border border-amber-400/20 bg-gradient-to-b from-amber-400/12 via-background/70 to-transparent p-5 shadow-inner">
          <div class="mb-4 flex items-center justify-between gap-3">
            <Badge variant="outline" class="border-amber-400/30 bg-amber-400/12 text-amber-700 dark:text-amber-100">
              <Activity class="size-3.5" />
              SSE {{ formatConnectionState(eventState) }}
            </Badge>
            <span class="text-xs uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("dashboard.relayTitle") }}
            </span>
          </div>

          <div class="space-y-3">
            <div class="space-y-2">
              <label class="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.relayBaseUrl") }}
              </label>
              <div class="flex flex-col gap-2 sm:flex-row">
                <Input
                  v-model="relayInput"
                  :placeholder="relayPlaceholder"
                  class="border-border/70 bg-background/70 text-foreground placeholder:text-muted-foreground"
                />
                <Button class="sm:min-w-28" @click="store.applyRelayBaseUrl">
                  {{ t("common.connect") }}
                </Button>
              </div>
            </div>

            <div class="space-y-2">
              <label class="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.fields.accessToken") }}
              </label>
              <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
                <Input
                  v-model="relayAccessTokenInput"
                  type="password"
                  :placeholder="t('common.optionalAccessToken')"
                  class="border-border/70 bg-background/70 text-foreground placeholder:text-muted-foreground"
                />
                <span class="text-xs text-muted-foreground">
                  {{ formatAuthRequirement(appConfig?.requiresAuth) }}
                </span>
              </div>
            </div>

            <div class="space-y-2">
              <label class="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("locale.label") }}
              </label>
              <div class="flex flex-wrap gap-2">
                <Button
                  v-for="option in localeOptions"
                  :key="option.value"
                  type="button"
                  variant="outline"
                  size="sm"
                  :class="
                    option.value === locale
                      ? 'border-sky-400/40 bg-sky-400/15 text-sky-900 hover:bg-sky-400/20 dark:text-white'
                      : 'border-border/70 bg-background/55 text-foreground hover:bg-accent/70'
                  "
                  @click="switchLocale(option.value)"
                >
                  {{ option.label }}
                </Button>
              </div>
            </div>

            <div class="space-y-2">
              <label class="text-xs font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("theme.label") }}
              </label>
              <div class="flex flex-wrap gap-2">
                <Button
                  v-for="option in themeOptions"
                  :key="option.value"
                  type="button"
                  variant="outline"
                  size="sm"
                  :class="
                    option.value === themeMode
                      ? 'border-emerald-400/40 bg-emerald-400/15 text-emerald-900 hover:bg-emerald-400/20 dark:text-emerald-50'
                      : 'border-border/70 bg-background/55 text-foreground hover:bg-accent/70'
                  "
                  @click="switchTheme(option.value)"
                >
                  {{ option.label }}
                </Button>
              </div>
            </div>

            <p
              v-if="showMobileRelayHint"
              class="rounded-2xl border px-3 py-2 text-sm"
              :class="
                showLoopbackRelayWarning
                  ? 'border-rose-500/30 bg-rose-500/10 text-rose-700 dark:text-rose-100'
                  : 'border-border/70 bg-background/55 text-muted-foreground'
              "
            >
              {{
                showLoopbackRelayWarning
                  ? t("dashboard.mobileLoopbackWarning")
                  : t("dashboard.mobileRemoteHint")
              }}
            </p>

            <Separator class="bg-border/70" />

            <div class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-4">
              <div class="rounded-2xl border border-border/70 bg-background/55 p-3">
                <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("dashboard.stats.onlineDevices") }}
                </p>
                <p class="mt-2 text-2xl font-semibold text-foreground">
                  {{ health?.onlineDeviceCount ?? 0 }}
                </p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-3">
                <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("dashboard.stats.devices") }}
                </p>
                <p class="mt-2 text-2xl font-semibold text-foreground">
                  {{ health?.deviceCount ?? 0 }}
                </p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-3">
                <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("dashboard.stats.aiSessions") }}
                </p>
                <p class="mt-2 text-2xl font-semibold text-foreground">{{ tasks.length }}</p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-3">
                <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("dashboard.stats.advancedTools") }}
                </p>
                <p class="mt-2 text-2xl font-semibold text-foreground">
                  {{ shellSessions.length + portForwards.length }}
                </p>
              </div>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>

    <Card
      v-if="localizedErrorMessage"
      class="border-rose-500/25 bg-rose-500/10 text-rose-700 shadow-none dark:text-rose-100"
    >
      <CardContent class="p-4">
        <p class="text-sm">{{ localizedErrorMessage }}</p>
      </CardContent>
    </Card>

    <section class="grid gap-4 2xl:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)]">
      <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
        <CardHeader class="flex flex-row items-start justify-between gap-4 space-y-0">
          <div class="space-y-1">
            <CardTitle class="flex items-center gap-2 text-foreground">
              <BellRing class="size-4 text-sky-300" />
              {{ t("dashboard.activity.title") }}
            </CardTitle>
            <CardDescription>
              {{
                t("dashboard.activity.summary", {
                  unread: unreadActivityCount,
                  total: activities.length
                })
              }}
            </CardDescription>
          </div>
          <Button
            variant="outline"
            size="sm"
            class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
            :disabled="!activities.length"
            @click="store.markAllActivitiesRead"
          >
            {{ t("dashboard.activity.markAllRead") }}
          </Button>
        </CardHeader>
        <CardContent class="pt-0">
          <div
            v-if="!activityItems.length"
            class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
          >
            {{ t("dashboard.activity.empty") }}
          </div>
          <ScrollArea v-else class="h-[22rem] pr-3">
            <div class="space-y-3">
              <button
                v-for="activity in activityItems"
                :key="activity.id"
                type="button"
                class="w-full rounded-2xl border border-border/70 bg-background/55 p-4 text-left transition hover:bg-accent/30"
                @click="store.openActivity(activity.id)"
              >
                <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div class="space-y-2">
                    <div class="flex flex-wrap items-center gap-2">
                      <Badge variant="outline" :class="activitySeverityClass(activity.severity)">
                        {{ activity.title }}
                      </Badge>
                      <Badge
                        v-if="activity.unread"
                        variant="outline"
                        class="border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-100"
                      >
                        {{ t("dashboard.activity.unread") }}
                      </Badge>
                    </div>
                    <p class="text-sm text-foreground">{{ activity.description }}</p>
                    <p class="font-mono text-xs text-muted-foreground">
                      {{ activity.resourceKind }} · {{ activity.resourceId }}
                    </p>
                  </div>
                  <span class="text-xs text-muted-foreground">
                    {{ formatTimestamp(activity.timestampEpochMs) }}
                  </span>
                </div>
              </button>
            </div>
          </ScrollArea>
        </CardContent>
      </Card>

      <div class="grid gap-4">
        <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
          <CardHeader class="space-y-1">
            <CardTitle class="flex items-center gap-2 text-foreground">
              <Server class="size-4 text-emerald-300" />
              {{ t("dashboard.deployment.title") }}
            </CardTitle>
            <CardDescription>{{ connectionGuidance }}</CardDescription>
          </CardHeader>
          <CardContent class="grid gap-4 pt-0">
            <div class="grid gap-3 md:grid-cols-2">
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.deployment.mode") }}
                </p>
                <p class="mt-2 text-sm font-medium text-foreground">
                  {{ formatDeploymentMode(appConfig?.deployment.mode) }}
                </p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.deployment.authMode") }}
                </p>
                <p class="mt-2 text-sm font-medium text-foreground">
                  {{ formatAuthMode(appConfig?.authMode) }}
                </p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.deployment.storageKind") }}
                </p>
                <p class="mt-2 text-sm font-medium text-foreground">
                  {{ formatStorageKind(appConfig?.storageKind) }}
                </p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.deployment.currentClient") }}
                </p>
                <p class="mt-2 text-sm font-medium text-foreground">
                  {{ formatControlClient(currentClientKind) }}
                </p>
              </div>
            </div>

            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                <div class="space-y-1">
                  <p class="text-sm font-medium text-foreground">
                    {{ t("dashboard.deployment.relayOrigin") }}
                  </p>
                  <p class="font-mono text-xs text-muted-foreground">
                    {{ appConfig?.deployment.relayPublicOrigin ?? relayBaseUrl ?? t("common.pending") }}
                  </p>
                </div>
                <a
                  v-if="deploymentDocsUrl"
                  :href="deploymentDocsUrl"
                  target="_blank"
                  rel="noreferrer"
                  class="text-sm text-sky-700 underline decoration-sky-500/50 underline-offset-4 dark:text-sky-100"
                >
                  {{ t("dashboard.deployment.documentation") }}
                </a>
              </div>
            </div>

            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <div class="mb-3 flex items-center gap-2">
                <Laptop2 class="size-4 text-amber-300" />
                <p class="text-sm font-medium text-foreground">
                  {{ t("dashboard.platform.title") }}
                </p>
              </div>
              <div class="mb-3 flex flex-wrap gap-2">
                <Badge variant="outline" :class="platformCapabilityStateClass(activePlatformCapability?.mobileOptimized ?? false)">
                  {{ t("dashboard.platform.mobileOptimized") }}
                </Badge>
                <Badge variant="outline" :class="platformCapabilityStateClass(activePlatformCapability?.supportsSystemNotifications ?? false)">
                  {{ t("dashboard.platform.systemNotifications") }}
                </Badge>
                <Badge variant="outline" :class="platformCapabilityStateClass(activePlatformCapability?.supportsPersistedRuntimeConfig ?? false)">
                  {{ t("dashboard.platform.persistedConfig") }}
                </Badge>
                <Badge variant="outline" :class="platformCapabilityStateClass(activePlatformCapability?.prefersExplicitRemoteRelayUrl ?? false)">
                  {{ t("dashboard.platform.explicitRelay") }}
                </Badge>
              </div>

              <div class="grid gap-3 md:grid-cols-3">
                <div
                  v-for="capability in platformMatrix"
                  :key="capability.client"
                  class="rounded-2xl border p-4"
                  :class="isCurrentClient(capability) ? 'border-sky-500/30 bg-sky-500/10' : 'border-border/70 bg-background/70'"
                >
                  <div class="flex items-center justify-between gap-2">
                    <p class="text-sm font-medium text-foreground">
                      {{ formatControlClient(capability.client) }}
                    </p>
                    <Badge
                      variant="outline"
                      :class="isCurrentClient(capability) ? 'border-sky-500/30 bg-sky-500/12 text-sky-700 dark:text-sky-100' : 'border-border/70 bg-background/55 text-muted-foreground'"
                    >
                      {{ isCurrentClient(capability) ? t("dashboard.platform.current") : t("dashboard.platform.available") }}
                    </Badge>
                  </div>
                  <div class="mt-3 space-y-2 text-xs text-muted-foreground">
                    <p>{{ capability.mobileOptimized ? t("dashboard.platform.mobileOptimized") : t("dashboard.platform.desktopOptimized") }}</p>
                    <p>{{ capability.supportsSystemNotifications ? t("dashboard.platform.systemNotifications") : t("dashboard.platform.inAppOnly") }}</p>
                    <p>{{ capability.prefersExplicitRemoteRelayUrl ? t("dashboard.platform.explicitRelay") : t("dashboard.platform.loopbackFriendly") }}</p>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
          <CardHeader class="space-y-1">
            <CardTitle class="flex items-center gap-2 text-foreground">
              <ShieldCheck class="size-4 text-violet-300" />
              {{ t("dashboard.governance.title") }}
            </CardTitle>
            <CardDescription>{{ t("dashboard.governance.description") }}</CardDescription>
          </CardHeader>
          <CardContent class="grid gap-4 pt-0">
            <div class="grid gap-3 md:grid-cols-2">
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.governance.tenant") }}
                </p>
                <p class="mt-2 font-mono text-xs text-foreground">
                  {{ currentActor?.tenantId ?? t("common.pending") }}
                </p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.governance.user") }}
                </p>
                <p class="mt-2 font-mono text-xs text-foreground">
                  {{ currentActor?.userId ?? t("common.pending") }}
                </p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.governance.role") }}
                </p>
                <p class="mt-2 text-sm font-medium text-foreground">
                  {{ currentActor ? formatUserRole(currentActor.role) : t("common.pending") }}
                </p>
              </div>
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.governance.notificationChannels") }}
                </p>
                <div class="mt-2 flex flex-wrap gap-2">
                  <Badge
                    v-for="channel in appConfig?.notificationChannels ?? []"
                    :key="channel"
                    variant="outline"
                    class="border-border/70 bg-background/70 text-foreground"
                  >
                    {{ formatNotificationChannel(channel) }}
                  </Badge>
                </div>
              </div>
            </div>

            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <div class="mb-4 flex items-center justify-between gap-3">
                <p class="text-sm font-medium text-foreground">
                  {{ t("dashboard.governance.auditTitle") }}
                </p>
                <Badge variant="outline" class="border-border/70 bg-background/70 text-foreground">
                  {{ auditRecords.length }}
                </Badge>
              </div>

              <div
                v-if="!auditTrail.length"
                class="rounded-2xl border border-dashed border-border/70 bg-background/70 p-4 text-sm text-muted-foreground"
              >
                {{ t("dashboard.governance.auditEmpty") }}
              </div>
              <ScrollArea v-else class="h-[18rem] pr-3">
                <div class="space-y-3">
                  <div
                    v-for="record in auditTrail"
                    :key="record.id"
                    class="rounded-2xl border border-border/70 bg-background/70 p-4"
                  >
                    <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                      <div class="space-y-2">
                        <div class="flex flex-wrap items-center gap-2">
                          <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                            {{ formatAuditAction(record.action) }}
                          </Badge>
                          <Badge variant="outline" :class="auditOutcomeClass(record.outcome)">
                            {{ formatAuditOutcome(record.outcome) }}
                          </Badge>
                        </div>
                        <p class="text-sm text-foreground">
                          {{ record.resourceKind }} · {{ record.resourceId }}
                        </p>
                        <p class="font-mono text-xs text-muted-foreground">
                          {{ record.userId }} · {{ formatUserRole(record.actorRole) }}
                        </p>
                        <p v-if="record.message" class="text-xs text-muted-foreground">
                          {{ record.message }}
                        </p>
                      </div>
                      <span class="text-xs text-muted-foreground">
                        {{ formatTimestamp(record.timestampEpochMs) }}
                      </span>
                    </div>
                  </div>
                </div>
              </ScrollArea>
            </div>
          </CardContent>
        </Card>
      </div>
    </section>

    <section class="grid gap-4 xl:grid-cols-[360px_minmax(0,1fr)]">
      <div class="flex flex-col gap-4">
        <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
          <CardHeader class="flex flex-row items-start justify-between gap-4 space-y-0">
            <div class="space-y-1">
              <CardTitle class="flex items-center gap-2 text-foreground">
                <Server class="size-4 text-sky-300" />
                {{ t("dashboard.devices.title") }}
              </CardTitle>
              <CardDescription>
                {{ t("dashboard.devices.registered", { count: devices.length }) }}
              </CardDescription>
            </div>
            <Button
              variant="outline"
              size="sm"
              class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
              @click="store.reloadAll"
            >
              <RefreshCw class="size-4" />
              {{ t("common.refresh") }}
            </Button>
          </CardHeader>
          <CardContent class="pt-0">
            <ScrollArea class="h-[24rem] pr-3">
              <div class="space-y-3">
                <button
                  v-for="device in devices"
                  :key="device.id"
                  type="button"
                  class="w-full rounded-2xl border p-4 text-left transition-colors"
                  :class="selectedStateClass(selectedDevice?.id === device.id)"
                  @click="store.selectDevice(device.id)"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="space-y-1">
                      <p class="font-medium text-foreground">{{ device.name }}</p>
                      <p class="text-sm text-muted-foreground">
                        {{ device.platform }} · {{ device.metadata.arch }}
                      </p>
                    </div>
                    <Badge
                      variant="outline"
                      :class="device.online ? statusBadgeClass('online') : statusBadgeClass('offline')"
                    >
                      {{ formatDevicePresence(device.online) }}
                    </Badge>
                  </div>

                  <p class="mt-3 font-mono text-xs text-muted-foreground">
                    {{ device.metadata.workingRoot ?? t("common.useAgentWorkingRoot") }}
                  </p>

                  <div class="mt-3 flex flex-wrap gap-2">
                    <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                      {{ t("dashboard.devices.sessions", { count: deviceSessionCount(device.id) }) }}
                    </Badge>
                    <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                      {{ t("dashboard.devices.terminals", { count: deviceShellCount(device.id) }) }}
                    </Badge>
                  </div>

                  <div class="mt-3 flex flex-wrap gap-2">
                    <Badge
                      v-for="provider in device.providers"
                      :key="provider.kind"
                      variant="outline"
                      :class="providerBadgeClass(provider.available)"
                    >
                      {{ formatProviderSummary(provider.kind, provider.executionProtocol) }}
                    </Badge>
                  </div>
                </button>
              </div>
            </ScrollArea>
          </CardContent>
        </Card>

        <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
          <CardHeader class="space-y-3">
            <div class="flex items-start justify-between gap-4">
              <div class="space-y-1">
                <CardTitle class="flex items-center gap-2 text-foreground">
                  <Sparkles class="size-4 text-amber-300" />
                  {{ t("dashboard.sessions.title") }}
                </CardTitle>
                <CardDescription>
                  {{
                    t("dashboard.sessions.visibleSummary", {
                      visible: aiSessions.length,
                      total: tasks.length
                    })
                  }}
                </CardDescription>
              </div>
              <Button
                variant="outline"
                size="sm"
                class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                :disabled="!canCancel"
                @click="store.cancelSelectedTask"
              >
                {{ t("common.cancel") }}
              </Button>
            </div>

            <div class="grid gap-3 sm:grid-cols-2">
              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("common.scope") }}
                </span>
                <select v-model="taskScope" :class="nativeSelectClass">
                  <option v-for="option in TASK_SCOPE_OPTIONS" :key="option" :value="option">
                    {{ formatScopeOption(option) }}
                  </option>
                </select>
              </label>

              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("common.status") }}
                </span>
                <select v-model="taskStatusFilter" :class="nativeSelectClass">
                  <option v-for="option in TASK_STATUS_OPTIONS" :key="option" :value="option">
                    {{ formatTaskStatusOption(option) }}
                  </option>
                </select>
              </label>
            </div>
          </CardHeader>
          <CardContent class="pt-0">
            <p v-if="!aiSessions.length" class="mb-3 text-sm text-muted-foreground">
              {{ t("dashboard.sessions.empty") }}
            </p>
            <ScrollArea class="h-[28rem] pr-3">
              <div class="space-y-3">
                <button
                  v-for="task in aiSessions"
                  :key="task.id"
                  type="button"
                  class="w-full rounded-2xl border p-4 text-left transition-colors"
                  :class="selectedStateClass(selectedTask?.id === task.id)"
                  @click="store.selectTask(task.id)"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="space-y-1">
                      <p class="font-medium text-foreground">{{ task.title }}</p>
                      <p class="text-sm text-muted-foreground">
                        {{ formatProviderSummary(task.provider, task.executionProtocol) }}
                      </p>
                    </div>
                    <Badge variant="outline" :class="taskStatusClass(task.status)">
                      {{ formatTaskStatus(task.status) }}
                    </Badge>
                  </div>

                  <p class="mt-3 font-mono text-xs text-muted-foreground">
                    {{ task.cwd ?? t("common.useAgentWorkingRoot") }}
                  </p>
                  <p class="mt-2 text-xs text-muted-foreground">
                    {{ formatTimestamp(task.startedAtEpochMs ?? task.createdAtEpochMs) }}
                  </p>
                </button>
              </div>
            </ScrollArea>
          </CardContent>
        </Card>
      </div>

      <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
        <CardHeader class="space-y-4">
          <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
            <div class="space-y-1">
              <CardTitle class="flex items-center gap-2 text-foreground">
                <FolderCode class="size-4 text-emerald-300" />
                {{ t("dashboard.workspace.title") }}
              </CardTitle>
              <CardDescription>{{ sessionWorkspaceTitle }}</CardDescription>
            </div>
            <Button
              variant="outline"
              class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
              :disabled="!canCancel"
              @click="store.cancelSelectedTask"
            >
              {{ t("dashboard.sessions.cancelSession") }}
            </Button>
          </div>

          <div v-if="selectedDevice" class="grid gap-3 md:grid-cols-3">
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                {{ t("dashboard.workspace.device") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">{{ selectedDevice.name }}</p>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ selectedDevice.platform }} · {{ selectedDevice.metadata.arch }}
              </p>
            </div>
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                {{ t("dashboard.workspace.workingRoot") }}
              </p>
              <p class="mt-2 font-mono text-xs text-foreground">{{ selectedDeviceWorkingRoot }}</p>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t("dashboard.workspace.relativePathHint") }}
              </p>
            </div>
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                {{ t("dashboard.workspace.deviceCapacity") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">
                {{
                  t("dashboard.workspace.providers", {
                    count: selectedDeviceAvailableProviderCount
                  })
                }}
              </p>
              <p class="mt-1 text-xs text-muted-foreground">
                {{
                  t("dashboard.workspace.capacitySummary", {
                    sessions: selectedDeviceSessionCount,
                    terminals: selectedDeviceShellCount,
                    previews: selectedDevicePreviewCount
                  })
                }}
              </p>
            </div>
          </div>
        </CardHeader>

        <CardContent class="flex flex-col gap-5 pt-0">
          <div class="rounded-2xl border border-border/70 bg-background/60 p-4">
            <div class="mb-4 space-y-1">
              <p class="text-sm font-medium text-foreground">{{ t("dashboard.workspace.newTitle") }}</p>
              <p class="text-sm text-muted-foreground">{{ t("dashboard.workspace.newDescription") }}</p>
            </div>

            <div class="grid gap-4 md:grid-cols-2">
              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.fields.provider") }}
                </span>
                <select v-model="draft.provider" :class="nativeSelectClass">
                  <option disabled value="">{{ t("dashboard.placeholders.selectProvider") }}</option>
                  <option
                    v-for="provider in store.availableProviders"
                    :key="provider.kind"
                    :value="provider.kind"
                  >
                    {{ provider.label }}
                  </option>
                </select>
              </label>

              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.fields.title") }}
                </span>
                <Input v-model="draft.title" :placeholder="t('dashboard.placeholders.sessionTitle')" />
              </label>

              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.fields.sessionCwd") }}
                </span>
                <Input v-model="draft.cwd" :placeholder="t('dashboard.placeholders.sessionCwd')" />
              </label>

              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.fields.model") }}
                </span>
                <Input v-model="draft.model" :placeholder="t('dashboard.placeholders.model')" />
              </label>

              <label class="grid gap-2 md:col-span-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.fields.prompt") }}
                </span>
                <Textarea
                  v-model="draft.prompt"
                  class="min-h-44"
                  :placeholder="t('dashboard.placeholders.prompt')"
                />
              </label>
            </div>

            <div class="mt-4 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
              <p class="text-sm text-muted-foreground">{{ t("dashboard.composerNote") }}</p>
              <Button :disabled="!canSubmit" @click="store.submitTask">
                {{ t("dashboard.sessions.start") }}
              </Button>
            </div>
          </div>

          <Separator class="bg-border/70" />

          <div v-if="selectedTaskDetail" class="space-y-4">
            <div class="grid gap-3 md:grid-cols-3">
              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("common.status") }}
                </p>
                <div class="mt-2 flex items-center gap-2">
                  <Badge variant="outline" :class="taskStatusClass(selectedTaskDetail.task.status)">
                    {{ formatTaskStatus(selectedTaskDetail.task.status) }}
                  </Badge>
                </div>
                <p class="mt-2 text-xs text-muted-foreground">
                  {{ t("common.started") }}
                  {{ formatTimestamp(selectedTaskDetail.task.startedAtEpochMs ?? selectedTaskDetail.task.createdAtEpochMs) }}
                </p>
              </div>

              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.workspace.workingDirectory") }}
                </p>
                <p class="mt-2 font-mono text-xs text-foreground">
                  {{ selectedTaskDetail.task.cwd ?? selectedDeviceWorkingRoot }}
                </p>
                <p class="mt-2 text-xs text-muted-foreground">
                  {{ selectedTaskDetail.task.model ?? t("common.defaultModel") }}
                </p>
              </div>

              <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.workspace.sessionMetrics") }}
                </p>
                <p class="mt-2 text-sm font-medium text-foreground">
                  {{
                    t("dashboard.workspace.exitCode", {
                      code: selectedTaskDetail.task.exitCode ?? t("common.pending")
                    })
                  }}
                </p>
                <p class="mt-2 text-xs text-muted-foreground">
                  {{
                    t("dashboard.workspace.eventsSummary", {
                      count: selectedTaskDetail.events.length,
                      deviceId: selectedTaskDetail.task.deviceId
                    })
                  }}
                </p>
              </div>
            </div>

            <div class="rounded-2xl border border-border/70 bg-background/60 p-4">
              <div class="mb-4 space-y-1">
                <p class="text-sm font-medium text-foreground">
                  {{ t("dashboard.workspace.supervision.title") }}
                </p>
                <p class="text-sm text-muted-foreground">
                  {{ t("dashboard.workspace.supervision.description") }}
                </p>
              </div>

              <div
                class="rounded-2xl border border-sky-500/30 bg-sky-500/10 p-4 text-sm text-sky-700 dark:text-sky-100"
              >
                {{ supervisionSummary }}
              </div>

              <div class="mt-4 grid gap-3 md:grid-cols-4">
                <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                  <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                    {{ t("dashboard.workspace.supervision.counts.assistant") }}
                  </p>
                  <p class="mt-2 text-sm font-medium text-foreground">
                    {{ sessionEventCounts.assistant }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                  <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                    {{ t("dashboard.workspace.supervision.counts.tool") }}
                  </p>
                  <p class="mt-2 text-sm font-medium text-foreground">
                    {{ sessionEventCounts.tool }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                  <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                    {{ t("dashboard.workspace.supervision.counts.stderr") }}
                  </p>
                  <p class="mt-2 text-sm font-medium text-foreground">
                    {{ sessionEventCounts.stderr }}
                  </p>
                </div>
                <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                  <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                    {{ t("dashboard.workspace.supervision.counts.changed") }}
                  </p>
                  <p class="mt-2 text-sm font-medium text-foreground">
                    {{ visibleChangedFileCount }}
                  </p>
                </div>
              </div>
            </div>

            <div class="rounded-2xl border border-border/70 bg-background/60 p-4">
              <p class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                {{ t("dashboard.workspace.promptTitle") }}
              </p>
              <pre class="mt-3 rounded-xl border border-border/70 bg-background/70 p-4 font-mono text-sm text-foreground">{{ selectedTaskDetail.task.prompt }}</pre>
            </div>

            <div class="rounded-2xl border border-border/70 bg-background/60 p-4">
              <div class="mb-4 flex flex-col gap-1">
                <p class="text-sm font-medium text-foreground">{{ selectedTaskDetail.task.title }}</p>
                <p class="text-sm text-muted-foreground">
                  {{ formatProviderSummary(selectedTaskDetail.task.provider, selectedTaskDetail.task.executionProtocol) }}
                  · {{ formatTaskStatus(selectedTaskDetail.task.status) }}
                </p>
              </div>

              <ScrollArea class="h-[24rem] pr-3">
                <div
                  v-if="!selectedTaskDetail.events.length"
                  class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                >
                  {{ t("dashboard.workspace.waitingEvents") }}
                </div>

                <div v-else class="space-y-3">
                  <div
                    v-for="event in selectedTaskDetail.events"
                    :key="`${event.taskId}-${event.seq}`"
                    class="rounded-2xl border p-4"
                    :class="eventKindClass(event.kind)"
                  >
                    <div class="mb-3 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                      <Badge variant="outline" class="border-current/30 bg-transparent text-current">
                        {{ formatEventKind(event.kind) }}
                      </Badge>
                      <span class="text-xs text-muted-foreground">
                        {{ formatTimestamp(event.timestampEpochMs) }}
                      </span>
                    </div>
                    <pre class="font-mono text-sm text-foreground">{{ event.message }}</pre>
                  </div>
                </div>
              </ScrollArea>
            </div>
          </div>

          <div
            v-else
            class="rounded-2xl border border-dashed border-border/70 bg-background/60 p-6 text-sm text-muted-foreground"
          >
            {{
              selectedDevice
                ? t("dashboard.sessions.readySelected")
                : t("dashboard.sessions.readyEmpty")
            }}
          </div>

          <template v-if="selectedDevice">
            <Separator class="bg-border/70" />

            <div class="space-y-4">
              <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                <div class="space-y-1">
                  <p class="text-sm font-medium text-foreground">
                    {{ t("dashboard.workspace.browser.title") }}
                  </p>
                  <p class="text-sm text-muted-foreground">
                    {{ t("dashboard.workspace.browser.description") }}
                  </p>
                </div>
                <div class="flex flex-wrap gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                    :disabled="!workspaceListing?.parentPath || workspaceLoading"
                    @click="navigateWorkspaceUp"
                  >
                    {{ t("dashboard.workspace.browser.up") }}
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                    :disabled="workspaceLoading"
                    @click="refreshWorkspace"
                  >
                    <RefreshCw class="size-4" />
                    {{ t("common.refresh") }}
                  </Button>
                </div>
              </div>

              <div
                v-if="!selectedDeviceSupportsWorkspace"
                class="rounded-2xl border border-dashed border-border/70 bg-background/60 p-4 text-sm text-muted-foreground"
              >
                {{ t("dashboard.workspace.browser.unsupported") }}
              </div>

              <div v-else class="grid gap-4 xl:grid-cols-[300px_minmax(0,1fr)]">
                <div class="rounded-2xl border border-border/70 bg-background/60 p-4">
                  <div class="mb-4 space-y-3">
                    <div class="space-y-1">
                      <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                        {{ t("dashboard.workspace.browser.root") }}
                      </p>
                      <p class="font-mono text-xs text-foreground">
                        {{ workspaceListing?.rootPath ?? selectedDeviceWorkingRoot }}
                      </p>
                    </div>
                    <div class="space-y-1">
                      <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                        {{ t("dashboard.workspace.browser.path") }}
                      </p>
                      <p class="font-mono text-xs text-foreground">
                        {{ workspaceListing?.path ?? selectedDeviceWorkingRoot }}
                      </p>
                    </div>
                    <p class="text-xs text-muted-foreground">
                      {{
                        t("dashboard.workspace.browser.entries", {
                          count: workspaceListing?.entries.length ?? 0
                        })
                      }}
                    </p>
                  </div>

                  <div
                    v-if="workspaceError"
                    class="mb-4 rounded-2xl border border-rose-500/30 bg-rose-500/10 p-4 text-sm text-rose-700 dark:text-rose-100"
                  >
                    {{ workspaceError }}
                  </div>

                  <div
                    v-if="workspaceLoading"
                    class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                  >
                    {{ t("dashboard.workspace.browser.loading") }}
                  </div>

                  <div
                    v-else-if="workspaceListing && !workspaceListing.entries.length"
                    class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                  >
                    {{ t("dashboard.workspace.browser.empty") }}
                  </div>

                  <ScrollArea v-else class="h-[22rem] pr-3">
                    <div class="space-y-2">
                      <button
                        v-for="entry in workspaceListing?.entries ?? []"
                        :key="entry.path"
                        type="button"
                        class="w-full rounded-2xl border p-3 text-left transition-colors"
                        :class="selectedStateClass(selectedWorkspacePath === entry.path)"
                        @click="openWorkspaceEntry(entry)"
                      >
                        <div class="flex items-start justify-between gap-3">
                          <div class="space-y-1">
                            <p class="text-sm font-medium text-foreground">{{ entry.name }}</p>
                            <p class="font-mono text-xs text-muted-foreground">
                              {{ entry.path }}
                            </p>
                          </div>
                          <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                            {{ t(`dashboard.workspace.browser.kind.${entry.kind}`) }}
                          </Badge>
                        </div>
                        <p
                          v-if="entry.kind === 'file' && entry.sizeBytes !== null"
                          class="mt-2 text-xs text-muted-foreground"
                        >
                          {{ t("dashboard.workspace.browser.size", { size: entry.sizeBytes }) }}
                        </p>
                      </button>
                    </div>
                  </ScrollArea>
                </div>

                <div class="rounded-2xl border border-border/70 bg-background/60 p-4">
                  <div class="mb-4 space-y-2">
                    <p class="text-sm font-medium text-foreground">
                      {{ t("dashboard.workspace.browser.previewTitle") }}
                    </p>
                    <p class="font-mono text-xs text-muted-foreground">
                      {{ workspacePreview?.path ?? selectedWorkspacePath ?? t("common.waiting") }}
                    </p>
                  </div>

                  <div
                    v-if="workspacePreviewLoading"
                    class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                  >
                    {{ t("dashboard.workspace.browser.previewLoading") }}
                  </div>

                  <div
                    v-else-if="workspacePreview?.kind === 'binary'"
                    class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                  >
                    {{ t("dashboard.workspace.browser.binaryNotice") }}
                  </div>

                  <div
                    v-else-if="workspacePreview?.kind === 'text'"
                    class="space-y-3"
                  >
                    <div class="grid gap-3 md:grid-cols-3">
                      <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.workspace.browser.root") }}
                        </p>
                        <p class="mt-2 font-mono text-xs text-foreground">
                          {{ workspacePreview.rootPath }}
                        </p>
                      </div>
                      <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.workspace.browser.sizeLabel") }}
                        </p>
                        <p class="mt-2 text-sm font-medium text-foreground">
                          {{ t("dashboard.workspace.browser.size", { size: workspacePreview.sizeBytes }) }}
                        </p>
                      </div>
                      <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.workspace.browser.lines") }}
                        </p>
                        <p class="mt-2 text-sm font-medium text-foreground">
                          {{ workspacePreview.totalLines ?? 0 }}
                        </p>
                      </div>
                    </div>

                    <div
                      v-if="workspacePreview.truncated"
                      class="rounded-2xl border border-amber-500/30 bg-amber-500/10 p-4 text-sm text-amber-700 dark:text-amber-100"
                    >
                      {{ t("dashboard.workspace.browser.truncated") }}
                    </div>

                    <pre class="rounded-2xl border border-border/70 bg-background/70 p-4 font-mono text-sm text-foreground">
{{ workspacePreview.content }}
                    </pre>
                  </div>

                  <div
                    v-else
                    class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                  >
                    {{ t("dashboard.workspace.browser.previewEmpty") }}
                  </div>
                </div>
              </div>
            </div>

            <template v-if="showGitInspect">
              <Separator class="bg-border/70" />

              <div class="space-y-4">
                <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
                  <div class="space-y-1">
                    <p class="text-sm font-medium text-foreground">
                      {{ t("dashboard.workspace.git.title") }}
                    </p>
                    <p class="text-sm text-muted-foreground">
                      {{ t("dashboard.workspace.git.description") }}
                    </p>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                    :disabled="gitLoading"
                    @click="refreshGitInspect"
                  >
                    <RefreshCw class="size-4" />
                    {{ t("common.refresh") }}
                  </Button>
                </div>

                <div
                  v-if="!selectedDeviceSupportsGitInspect"
                  class="rounded-2xl border border-dashed border-border/70 bg-background/60 p-4 text-sm text-muted-foreground"
                >
                  {{ t("dashboard.workspace.git.unsupported") }}
                </div>

                <div v-else class="rounded-2xl border border-border/70 bg-background/60 p-4">
                  <div
                    v-if="gitError"
                    class="mb-4 rounded-2xl border border-rose-500/30 bg-rose-500/10 p-4 text-sm text-rose-700 dark:text-rose-100"
                  >
                    {{ gitError }}
                  </div>

                  <div
                    v-if="gitLoading"
                    class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                  >
                    {{ t("dashboard.workspace.git.loading") }}
                  </div>

                  <div
                    v-else-if="!gitInspect"
                    class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                  >
                    {{ t("dashboard.workspace.git.empty") }}
                  </div>

                  <div v-else class="space-y-4">
                    <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                      <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.workspace.git.workspaceRoot") }}
                        </p>
                        <p class="mt-2 font-mono text-xs text-foreground">
                          {{ gitInspect.workspaceRoot }}
                        </p>
                      </div>
                      <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.workspace.git.repoRoot") }}
                        </p>
                        <p class="mt-2 font-mono text-xs text-foreground">
                          {{ gitInspect.repoRoot ?? t("common.pending") }}
                        </p>
                        <p class="mt-2 text-xs text-muted-foreground">
                          {{ t("dashboard.workspace.git.scopePath") }}:
                          {{ gitInspect.scopePath ?? t("common.pending") }}
                        </p>
                      </div>
                      <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.workspace.git.branch") }}
                        </p>
                        <p class="mt-2 text-sm font-medium text-foreground">
                          {{ gitInspect.branchName ?? t("common.pending") }}
                        </p>
                      </div>
                      <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.workspace.git.upstream") }}
                        </p>
                        <p class="mt-2 text-sm font-medium text-foreground">
                          {{ gitInspect.upstreamBranch ?? t("dashboard.workspace.git.noUpstream") }}
                        </p>
                        <p class="mt-2 text-xs text-muted-foreground">
                          {{ t("dashboard.workspace.git.drift") }}:
                          {{ formatGitDrift(gitInspect.aheadCount, gitInspect.behindCount) }}
                        </p>
                      </div>
                    </div>

                    <div
                      v-if="gitInspect.state !== 'ready'"
                      class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                    >
                      {{ t(`dashboard.workspace.git.state.${gitInspect.state}`) }}
                    </div>

                    <template v-else>
                      <div class="flex flex-wrap gap-2">
                        <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                          {{ t("dashboard.workspace.git.stats.changedFiles", { count: gitInspect.diffStats.changedFiles }) }}
                        </Badge>
                        <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                          {{ t("dashboard.workspace.git.stats.stagedFiles", { count: gitInspect.diffStats.stagedFiles }) }}
                        </Badge>
                        <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                          {{ t("dashboard.workspace.git.stats.unstagedFiles", { count: gitInspect.diffStats.unstagedFiles }) }}
                        </Badge>
                        <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                          {{ t("dashboard.workspace.git.stats.untrackedFiles", { count: gitInspect.diffStats.untrackedFiles }) }}
                        </Badge>
                        <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                          {{ t("dashboard.workspace.git.stats.conflictedFiles", { count: gitInspect.diffStats.conflictedFiles }) }}
                        </Badge>
                        <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                          {{ t("dashboard.workspace.git.stats.stagedLines", {
                            additions: gitInspect.diffStats.stagedAdditions,
                            deletions: gitInspect.diffStats.stagedDeletions
                          }) }}
                        </Badge>
                        <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                          {{ t("dashboard.workspace.git.stats.unstagedLines", {
                            additions: gitInspect.diffStats.unstagedAdditions,
                            deletions: gitInspect.diffStats.unstagedDeletions
                          }) }}
                        </Badge>
                      </div>

                      <div
                        v-if="!gitInspect.changedFiles.length"
                        class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                      >
                        {{ t("dashboard.workspace.git.clean") }}
                      </div>

                      <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(0,0.95fr)]">
                        <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                          <div class="mb-4 flex items-center gap-2">
                            <GitBranch class="size-4 text-emerald-300" />
                            <p class="text-sm font-medium text-foreground">
                              {{ t("dashboard.workspace.git.changedFilesTitle") }}
                            </p>
                          </div>

                          <ScrollArea class="h-[18rem] pr-3">
                            <div class="space-y-3">
                              <div
                                v-for="file in gitInspect.changedFiles"
                                :key="file.repoPath"
                                class="rounded-2xl border border-border/70 bg-background/70 p-4"
                              >
                                <div class="flex items-start justify-between gap-3">
                                  <div class="space-y-1">
                                    <p class="text-sm font-medium text-foreground">
                                      {{ file.repoPath }}
                                    </p>
                                    <p class="font-mono text-xs text-muted-foreground">
                                      {{ file.path }}
                                    </p>
                                  </div>
                                  <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                                    {{ formatGitFileStatus(file.status) }}
                                  </Badge>
                                </div>

                                <div class="mt-3 flex items-center justify-end">
                                  <Button
                                    v-if="canPreviewGitChangedFile(file)"
                                    variant="outline"
                                    size="sm"
                                    class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                                    @click="openGitChangedFile(file)"
                                  >
                                    {{ t("dashboard.workspace.git.preview") }}
                                  </Button>
                                  <p
                                    v-else-if="file.status === 'deleted'"
                                    class="text-xs text-muted-foreground"
                                  >
                                    {{ t("dashboard.workspace.git.deletedPreviewUnavailable") }}
                                  </p>
                                </div>
                              </div>
                            </div>
                          </ScrollArea>
                        </div>

                        <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                          <div class="mb-4 flex items-center gap-2">
                            <Activity class="size-4 text-sky-300" />
                            <p class="text-sm font-medium text-foreground">
                              {{ t("dashboard.workspace.git.recentCommitsTitle") }}
                            </p>
                          </div>

                          <div
                            v-if="!gitInspect.hasCommits"
                            class="rounded-2xl border border-dashed border-border/70 bg-background/70 p-4 text-sm text-muted-foreground"
                          >
                            {{ t("dashboard.workspace.git.noCommits") }}
                          </div>

                          <ScrollArea v-else class="h-[18rem] pr-3">
                            <div class="space-y-3">
                              <div
                                v-for="commit in gitInspect.recentCommits"
                                :key="commit.id"
                                class="rounded-2xl border border-border/70 bg-background/70 p-4"
                              >
                                <div class="flex items-start justify-between gap-3">
                                  <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                                    {{ commit.shortId }}
                                  </Badge>
                                  <span class="text-xs text-muted-foreground">
                                    {{ formatTimestamp(commit.committedAtEpochMs) }}
                                  </span>
                                </div>
                                <p class="mt-3 text-sm font-medium text-foreground">
                                  {{ commit.summary || commit.shortId }}
                                </p>
                                <p class="mt-2 text-xs text-muted-foreground">
                                  {{ commit.authorName }}
                                </p>
                              </div>
                            </div>
                          </ScrollArea>
                        </div>
                      </div>
                    </template>
                  </div>
                </div>
              </div>
            </template>

            <template v-if="showPreviewTools">
              <Separator class="bg-border/70" />

              <div class="rounded-2xl border border-border/70 bg-background/60 p-4">
                <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
                  <div class="space-y-1">
                    <p class="text-sm font-medium text-foreground">
                      {{ t("dashboard.preview.launchTitle") }}
                    </p>
                    <p class="text-sm text-muted-foreground">
                      {{ t("dashboard.preview.launchDescription") }}
                    </p>
                  </div>

                  <div class="grid gap-3 md:grid-cols-[1.2fr_1fr_auto] lg:min-w-[38rem]">
                    <Input
                      v-model="portForwardDraft.targetHost"
                      :placeholder="t('dashboard.placeholders.targetHost')"
                    />
                    <Input
                      v-model="portForwardDraft.targetPort"
                      inputmode="numeric"
                      :placeholder="t('dashboard.placeholders.targetPort')"
                    />
                    <Button :disabled="!canCreatePortForward" @click="store.createPortForward">
                      {{ t("dashboard.preview.open") }}
                    </Button>
                  </div>
                </div>
              </div>
            </template>
          </template>
        </CardContent>
      </Card>
    </section>

    <section v-if="showShellTools || showPreviewTools" class="space-y-4">
      <div class="space-y-2">
        <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
          {{ t("dashboard.advanced.badge") }}
        </Badge>
        <div class="space-y-1">
          <h2 class="text-2xl font-semibold tracking-tight text-foreground">
            {{ t("dashboard.advanced.title") }}
          </h2>
          <p class="text-sm text-muted-foreground">{{ t("dashboard.advanced.description") }}</p>
        </div>
      </div>

      <div class="grid gap-4 2xl:grid-cols-2">
        <Card
          v-if="showShellTools"
          class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl"
        >
          <CardHeader class="space-y-4">
            <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
              <div class="space-y-1">
                <CardTitle class="flex items-center gap-2 text-foreground">
                  <TerminalSquare class="size-4 text-violet-300" />
                  {{ t("dashboard.terminal.title") }}
                </CardTitle>
                <CardDescription>
                  {{
                    t("dashboard.terminal.visibleSummary", {
                      visible: visibleShellSessions.length,
                      total: shellSessions.length,
                      state: formatConnectionState(shellSocketState)
                    })
                  }}
                </CardDescription>
              </div>
              <Button
                variant="outline"
                size="sm"
                class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                @click="store.refreshShellSessionsFromPoll"
              >
                <RefreshCw class="size-4" />
                {{ t("common.refresh") }}
              </Button>
            </div>

            <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-[1fr_1fr_1.2fr_auto_auto]">
              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("common.scope") }}
                </span>
                <select v-model="shellScope" :class="nativeSelectClass">
                  <option v-for="option in TASK_SCOPE_OPTIONS" :key="option" :value="option">
                    {{ formatScopeOption(option) }}
                  </option>
                </select>
              </label>

              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("common.status") }}
                </span>
                <select v-model="shellStatusFilter" :class="nativeSelectClass">
                  <option v-for="option in SHELL_STATUS_OPTIONS" :key="option" :value="option">
                    {{ formatShellStatusOption(option) }}
                  </option>
                </select>
              </label>

              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("dashboard.fields.terminalCwd") }}
                </span>
                <Input
                  v-model="shellDraft.cwd"
                  :placeholder="t('dashboard.placeholders.terminalCwd')"
                />
              </label>

              <Button class="self-end" :disabled="!canOpenShell" @click="store.createShellSession">
                {{ t("dashboard.terminal.open") }}
              </Button>

              <Button
                variant="outline"
                class="self-end border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                :disabled="!canCloseShell"
                @click="store.closeSelectedShellSession"
              >
                {{ t("common.close") }}
              </Button>
            </div>

            <p v-if="selectedDevice && !selectedDeviceSupportsShell" class="text-sm text-muted-foreground">
              {{ t("dashboard.terminal.noCapability") }}
            </p>
          </CardHeader>

          <CardContent class="pt-0">
            <div class="grid gap-4 xl:grid-cols-[260px_minmax(0,1fr)]">
              <div class="space-y-3">
                <p v-if="!visibleShellSessions.length" class="text-sm text-muted-foreground">
                  {{ t("dashboard.terminal.empty") }}
                </p>

                <ScrollArea class="h-[28rem] pr-3">
                  <div class="space-y-3">
                    <button
                      v-for="session in visibleShellSessions"
                      :key="session.id"
                      type="button"
                      class="w-full rounded-2xl border p-4 text-left transition-colors"
                      :class="selectedStateClass(selectedShellSession?.id === session.id)"
                      @click="store.selectShellSession(session.id)"
                    >
                      <div class="flex items-start justify-between gap-3">
                        <p class="font-medium text-foreground">{{ session.deviceId }}</p>
                        <Badge variant="outline" :class="shellStatusClass(session.status)">
                          {{ formatShellStatus(session.status, session.closeRequested) }}
                        </Badge>
                      </div>
                      <p class="mt-3 font-mono text-xs text-muted-foreground">
                        {{ session.cwd ?? t("common.useAgentWorkingRoot") }}
                      </p>
                      <p class="mt-2 text-xs text-muted-foreground">
                        {{ formatTimestamp(session.startedAtEpochMs ?? session.createdAtEpochMs) }}
                      </p>
                    </button>
                  </div>
                </ScrollArea>
              </div>

              <div class="rounded-2xl border border-border/70 bg-background/60">
                <div
                  v-if="selectedShellSession"
                  class="flex flex-col gap-2 border-b border-border/70 px-4 py-4 sm:flex-row sm:items-start sm:justify-between"
                >
                  <div>
                    <p class="font-medium text-foreground">
                      {{ t("dashboard.terminal.detailTitle", { id: selectedShellSession.id }) }}
                    </p>
                    <p class="text-sm text-muted-foreground">
                      {{
                        t("dashboard.terminal.detailSummary", {
                          status: formatShellStatus(
                            selectedShellSession.status,
                            selectedShellSession.closeRequested
                          ),
                          time: formatTimestamp(
                            selectedShellSession.startedAtEpochMs ??
                              selectedShellSession.createdAtEpochMs
                          )
                        })
                      }}
                    </p>
                  </div>
                  <p class="font-mono text-xs text-muted-foreground">
                    {{ selectedShellSession.cwd ?? selectedDeviceWorkingRoot }}
                  </p>
                </div>

                <ScrollArea class="h-[22rem] px-4 py-4">
                  <div v-if="selectedShellSessionDetail" class="space-y-3">
                    <div
                      v-if="!shellTimeline.length"
                      class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                    >
                      {{ t("dashboard.terminal.waiting") }}
                    </div>

                    <div
                      v-for="entry in shellTimeline"
                      :key="entry.key"
                      class="rounded-2xl border p-4"
                      :class="terminalStreamClass(entry.stream)"
                    >
                      <div class="mb-3 flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                        <Badge variant="outline" class="border-current/30 bg-transparent text-current">
                          {{ formatStreamLabel(entry.stream) }}
                        </Badge>
                        <span class="text-xs text-muted-foreground">
                          {{ formatTimestamp(entry.timestampEpochMs) }}
                        </span>
                      </div>
                      <pre class="font-mono text-sm text-foreground">{{ entry.data }}</pre>
                    </div>
                  </div>

                  <div
                    v-else
                    class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                  >
                    {{ t("dashboard.terminal.select") }}
                  </div>
                </ScrollArea>

                <div class="border-t border-border/70 p-4">
                  <div class="space-y-3">
                    <Textarea
                      v-model="shellDraft.input"
                      class="min-h-28"
                      :placeholder="t('dashboard.placeholders.terminalInput')"
                      :disabled="!selectedShellSession || selectedShellSession.closeRequested"
                    />
                    <div class="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
                      <p class="text-sm text-muted-foreground">{{ t("dashboard.terminal.note") }}</p>
                      <Button :disabled="!canSendShellInput" @click="store.submitShellInput">
                        {{ t("common.sendCommand") }}
                      </Button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        <Card
          v-if="showPreviewTools"
          class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl"
        >
          <CardHeader class="space-y-4">
            <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
              <div class="space-y-1">
                <CardTitle class="flex items-center gap-2 text-foreground">
                  <Eye class="size-4 text-cyan-300" />
                  {{ t("dashboard.preview.title") }}
                </CardTitle>
                <CardDescription>
                  {{
                    t("dashboard.preview.visibleSummary", {
                      visible: visiblePortForwards.length,
                      total: portForwards.length
                    })
                  }}
                </CardDescription>
                <p class="text-sm text-muted-foreground">
                  {{ t("dashboard.preview.description") }}
                </p>
              </div>
              <Button
                variant="outline"
                size="sm"
                class="border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                @click="store.refreshPortForwardsFromPoll"
              >
                <RefreshCw class="size-4" />
                {{ t("common.refresh") }}
              </Button>
            </div>

            <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-[1fr_1fr_auto]">
              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("common.scope") }}
                </span>
                <select v-model="portForwardScope" :class="nativeSelectClass">
                  <option v-for="option in TASK_SCOPE_OPTIONS" :key="option" :value="option">
                    {{ formatScopeOption(option) }}
                  </option>
                </select>
              </label>

              <label class="grid gap-2">
                <span class="text-xs font-medium uppercase tracking-[0.16em] text-muted-foreground">
                  {{ t("common.status") }}
                </span>
                <select v-model="portForwardStatusFilter" :class="nativeSelectClass">
                  <option v-for="option in PORT_FORWARD_STATUS_OPTIONS" :key="option" :value="option">
                    {{ formatPortForwardStatusOption(option) }}
                  </option>
                </select>
              </label>

              <Button
                variant="outline"
                class="self-end border-border/70 bg-background/55 text-foreground hover:bg-accent/70"
                :disabled="!canClosePortForward"
                @click="store.closeSelectedPortForward"
              >
                {{ t("common.close") }}
              </Button>
            </div>

            <p class="text-sm text-muted-foreground">
              {{ t("dashboard.preview.serviceHint") }}
            </p>
          </CardHeader>

          <CardContent class="pt-0">
            <div class="grid gap-4 xl:grid-cols-[260px_minmax(0,1fr)]">
              <div class="space-y-3">
                <p v-if="!visiblePortForwards.length" class="text-sm text-muted-foreground">
                  {{ t("dashboard.preview.empty") }}
                </p>

                <ScrollArea class="h-[28rem] pr-3">
                  <div class="space-y-3">
                    <button
                      v-for="forward in visiblePortForwards"
                      :key="forward.id"
                      type="button"
                      class="w-full rounded-2xl border p-4 text-left transition-colors"
                      :class="selectedStateClass(selectedPortForward?.id === forward.id)"
                      @click="store.selectPortForward(forward.id)"
                    >
                      <div class="flex items-start justify-between gap-3">
                        <p class="font-medium text-foreground">{{ forward.deviceId }}</p>
                        <Badge variant="outline" :class="portForwardStatusClass(forward.status)">
                          {{ formatPortForwardStatus(forward.status) }}
                        </Badge>
                      </div>
                      <p class="mt-3 font-mono text-xs text-foreground">
                        {{ buildPreviewUrl(forward) }}
                      </p>
                      <p class="mt-1 text-xs text-muted-foreground">
                        {{
                          t("dashboard.preview.serviceTarget", {
                            host: forward.targetHost,
                            port: forward.targetPort
                          })
                        }}
                      </p>
                      <p class="mt-2 text-xs text-muted-foreground">
                        {{ formatTimestamp(forward.startedAtEpochMs ?? forward.createdAtEpochMs) }}
                      </p>
                    </button>
                  </div>
                </ScrollArea>
              </div>

              <div class="rounded-2xl border border-border/70 bg-background/60 p-4">
                <div v-if="selectedPortForward" class="space-y-4">
                  <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                    <div>
                      <p class="font-medium text-foreground">
                        {{ t("dashboard.preview.detailTitle", { id: selectedPortForward.id }) }}
                      </p>
                      <p class="text-sm text-muted-foreground">
                        {{ formatPortForwardStatus(selectedPortForward.status) }}
                      </p>
                    </div>
                    <p class="font-mono text-xs text-muted-foreground">{{ selectedPortForward.deviceId }}</p>
                  </div>

                  <div class="rounded-2xl border border-cyan-500/25 bg-cyan-500/10 p-4">
                    <p class="text-xs uppercase tracking-[0.16em] text-cyan-700 dark:text-cyan-100">
                      {{ t("dashboard.preview.previewUrl") }}
                    </p>
                    <p class="mt-2 font-mono text-sm text-foreground">
                      {{ selectedPreviewUrl }}
                    </p>
                    <p class="mt-3 text-sm text-muted-foreground">
                      {{
                        selectedPreviewIsReady
                          ? t("dashboard.preview.ready")
                          : t("dashboard.preview.waiting")
                      }}
                    </p>
                    <div class="mt-4 flex flex-wrap gap-2">
                      <a
                        :href="selectedPreviewUrl"
                        target="_blank"
                        rel="noreferrer"
                        class="inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs transition hover:bg-primary/90"
                        :class="!selectedPreviewIsReady ? 'pointer-events-none opacity-50' : ''"
                        :aria-disabled="!selectedPreviewIsReady"
                        :tabindex="selectedPreviewIsReady ? 0 : -1"
                      >
                        {{ t("dashboard.preview.openLink") }}
                      </a>
                    </div>
                  </div>

                  <div
                    v-if="selectedPreviewLoopbackWarning"
                    class="rounded-2xl border border-amber-500/30 bg-amber-500/10 p-4 text-sm text-amber-700 dark:text-amber-100"
                  >
                    {{ t("dashboard.preview.mobileWarning") }}
                  </div>

                  <div class="grid gap-3 md:grid-cols-3">
                    <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                      <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                        {{ t("common.created") }}
                      </p>
                      <p class="mt-2 text-xs text-foreground">
                        {{ formatTimestamp(selectedPortForward.createdAtEpochMs) }}
                      </p>
                    </div>
                    <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                      <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                        {{ t("common.started") }}
                      </p>
                      <p class="mt-2 text-xs text-foreground">
                        {{ formatTimestamp(selectedPortForward.startedAtEpochMs) }}
                      </p>
                    </div>
                    <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                      <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                        {{ t("common.finished") }}
                      </p>
                      <p class="mt-2 text-xs text-foreground">
                        {{ formatTimestamp(selectedPortForward.finishedAtEpochMs) }}
                      </p>
                    </div>
                  </div>

                  <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
                    <div class="space-y-1">
                      <p class="text-sm font-medium text-foreground">
                        {{ t("dashboard.preview.advancedTitle") }}
                      </p>
                      <p class="text-sm text-muted-foreground">
                        {{ t("dashboard.preview.advancedDescription") }}
                      </p>
                    </div>

                    <div class="mt-4 grid gap-3 md:grid-cols-4">
                      <div class="rounded-2xl border border-border/70 bg-background/70 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("common.protocol") }}
                        </p>
                        <p class="mt-2 text-sm font-medium text-foreground">
                          {{ formatPortForwardProtocol(selectedPortForward.protocol) }}
                        </p>
                      </div>
                      <div class="rounded-2xl border border-border/70 bg-background/70 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.preview.relayEndpoint") }}
                        </p>
                        <p class="mt-2 font-mono text-xs text-foreground">
                          {{ formatEndpoint(selectedPortForward.relayHost, selectedPortForward.relayPort) }}
                        </p>
                      </div>
                      <div class="rounded-2xl border border-border/70 bg-background/70 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.preview.targetEndpoint") }}
                        </p>
                        <p class="mt-2 font-mono text-xs text-foreground">
                          {{ formatEndpoint(selectedPortForward.targetHost, selectedPortForward.targetPort) }}
                        </p>
                      </div>
                      <div class="rounded-2xl border border-border/70 bg-background/70 p-4">
                        <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                          {{ t("dashboard.preview.transportLabel") }}
                        </p>
                        <p class="mt-2 text-sm font-medium text-foreground">
                          {{ formatPortForwardTransport(selectedPortForward.transport) }}
                        </p>
                      </div>
                    </div>
                  </div>

                  <div
                    v-if="selectedPortForward.error"
                    class="rounded-2xl border border-rose-500/30 bg-rose-500/10 p-4 text-sm text-rose-700 dark:text-rose-100"
                  >
                    {{ selectedPortForward.error }}
                  </div>
                </div>

                <div
                  v-else
                  class="rounded-2xl border border-dashed border-border/70 bg-background/55 p-4 text-sm text-muted-foreground"
                >
                  {{ t("dashboard.preview.select") }}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </section>
  </main>
</template>
