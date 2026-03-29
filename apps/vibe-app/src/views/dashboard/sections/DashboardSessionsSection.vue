<script setup lang="ts">
import { computed, ref } from "vue"
import { storeToRefs } from "pinia"
import { useI18n } from "vue-i18n"
import {
  Archive,
  Bot,
  FolderCode,
  GitBranch,
  LoaderCircle,
  MessageSquarePlus,
  RefreshCw,
  Send,
  Server,
  Sparkles,
  UserRound,
  WandSparkles
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
import { Textarea } from "@/components/ui/textarea"
import type { ConversationInputOption, TaskDetailResponse, TaskEvent } from "@/types"
import { useDashboardController } from "@/views/dashboard/controller"

type TranscriptTurn = {
  detail: TaskDetailResponse
  assistantText: string
  toolEvents: TaskEvent[]
  systemEvents: TaskEvent[]
  traceEvents: TaskEvent[]
}

type InspectorTab = "status" | "git" | "workspace" | "trace"

const CUSTOM_INPUT_SENTINEL = "__custom_input__"

const { t } = useI18n()
const dashboard = useDashboardController()
const store = dashboard.store
const inspectorTab = ref<InspectorTab>("status")
const contextExpanded = ref(false)
const {
  canPreviewGitChangedFile,
  formatConnectionState,
  formatExecutionProtocol,
  formatEventKind,
  formatGitDrift,
  formatGitFileStatus,
  formatProviderKind,
  formatTaskStatus,
  formatTimestamp,
  gitError,
  gitInspect,
  gitLoading,
  nativeSelectClass,
  navigateWorkspaceUp,
  openGitChangedFile,
  openWorkspaceEntry,
  refreshGitInspect,
  refreshWorkspace,
  relayPlaceholder,
  selectedDeviceWorkingRoot,
  selectedStateClass,
  sessionEventCounts,
  sessionLaunchState,
  taskStatusClass,
  workspaceError,
  workspaceListing,
  workspaceLoading,
  workspacePreview,
  workspacePreviewLoading
} = dashboard

const {
  conversations,
  devices,
  draft,
  eventState,
  pendingConversationInputDraft,
  relayAccessTokenInput,
  relayBaseUrl,
  relayInput,
  selectedConversationId,
  selectedDevice
} = storeToRefs(store)

const availableProviders = computed(() => store.availableProviders)
const selectedConversation = computed(() => store.selectedConversation)
const pendingInputRequest = computed(() => store.conversationPendingInput)
const currentConversationTask = computed(() => store.conversationCurrentTask)
const selectedDraftProvider = computed(() =>
  availableProviders.value.find((provider) => provider.kind === draft.value.provider) ?? null
)
const setupReady = computed(() => sessionLaunchState.value === "ready")
const setupRequired = computed(() => {
  if (!relayBaseUrl.value) {
    return true
  }
  if (!devices.value.length) {
    return true
  }
  if (selectedConversation.value) {
    return false
  }

  return !setupReady.value
})
const showContextPanel = computed(() => setupRequired.value || contextExpanded.value)
const canSendPrompt = computed(() => {
  if (!draft.value.prompt.trim() || !relayBaseUrl.value) {
    return false
  }

  if (selectedConversation.value) {
    const device = devices.value.find((item) => item.id === selectedConversation.value?.deviceId)
    return Boolean(device?.online)
  }

  return setupReady.value && Boolean(draft.value.provider)
})
const transcriptTurns = computed<TranscriptTurn[]>(() =>
  (selectedConversation.value ? store.selectedConversationDetail?.tasks ?? [] : []).map((detail) => {
    const toolEvents = detail.events.filter(
      (event) => event.kind === "tool_call" || event.kind === "tool_output"
    )
    const systemEvents = detail.events.filter(
      (event) =>
        event.kind === "system" ||
        event.kind === "status" ||
        event.kind === "provider_stderr"
    )
    const traceEvents = [...toolEvents, ...systemEvents].sort((left, right) => {
      if (left.timestampEpochMs === right.timestampEpochMs) {
        return left.seq - right.seq
      }

      return left.timestampEpochMs - right.timestampEpochMs
    })

    return {
      detail,
      assistantText: buildAssistantText(detail),
      toolEvents,
      systemEvents,
      traceEvents
    }
  })
)
const traceTurns = computed<TranscriptTurn[]>(() =>
  transcriptTurns.value
    .filter((turn) => turn.traceEvents.length)
    .sort((left, right) => {
      const leftTimestamp =
        left.traceEvents[left.traceEvents.length - 1]?.timestampEpochMs ??
        left.detail.task.createdAtEpochMs
      const rightTimestamp =
        right.traceEvents[right.traceEvents.length - 1]?.timestampEpochMs ??
        right.detail.task.createdAtEpochMs

      return rightTimestamp - leftTimestamp
    })
)
const activeTraceEventCount = computed(() =>
  traceTurns.value.reduce((total, turn) => total + turn.traceEvents.length, 0)
)
const selectedPendingOption = computed(() =>
  pendingInputRequest.value?.options.find(
    (option) => option.id === pendingConversationInputDraft.value.optionId
  )
)
const showPendingTextInput = computed(() => {
  if (!pendingInputRequest.value) {
    return false
  }
  if (!pendingInputRequest.value.options.length && pendingInputRequest.value.allowCustomInput) {
    return true
  }
  if (pendingConversationInputDraft.value.optionId === CUSTOM_INPUT_SENTINEL) {
    return true
  }

  return Boolean(selectedPendingOption.value?.requiresTextInput)
})
const canSubmitPendingText = computed(
  () => showPendingTextInput.value && Boolean(pendingConversationInputDraft.value.text.trim())
)
const currentProviderLabel = computed(() => {
  if (selectedConversation.value) {
    return formatProviderKind(selectedConversation.value.provider)
  }
  if (draft.value.provider) {
    return formatProviderKind(draft.value.provider)
  }

  return t("common.pending")
})
const currentProtocolLabel = computed(() => {
  if (selectedConversation.value) {
    return formatExecutionProtocol(selectedConversation.value.executionProtocol)
  }
  if (selectedDraftProvider.value) {
    return formatExecutionProtocol(selectedDraftProvider.value.executionProtocol)
  }

  return t("common.pending")
})
const currentWorkingDirectory = computed(() => {
  if (selectedConversation.value?.cwd) {
    return selectedConversation.value.cwd
  }
  if (draft.value.cwd.trim()) {
    return draft.value.cwd.trim()
  }

  return selectedDeviceWorkingRoot.value
})
const currentModelLabel = computed(() => {
  if (selectedConversation.value?.model) {
    return selectedConversation.value.model
  }
  if (draft.value.model.trim()) {
    return draft.value.model.trim()
  }

  return t("common.defaultModel")
})
const currentThreadStatus = computed(() => {
  if (currentConversationTask.value) {
    return formatTaskStatus(currentConversationTask.value.status)
  }

  return t(`dashboard.sessions.launchState.${sessionLaunchState.value}`)
})
const sessionSummaryCards = computed(() => [
  {
    key: "relay",
    label: t("dashboard.shell.connectionState"),
    value: formatConnectionState(eventState.value),
    meta: relayBaseUrl.value || relayPlaceholder.value
  },
  {
    key: "device",
    label: t("dashboard.shell.selectedDevice"),
    value:
      selectedConversation.value?.deviceId
        ? deviceName(selectedConversation.value.deviceId)
        : (selectedDevice.value?.name ?? t("dashboard.shell.noDeviceSelected")),
    meta: currentWorkingDirectory.value
  },
  {
    key: "provider",
    label: t("common.provider"),
    value: currentProviderLabel.value,
    meta: currentProtocolLabel.value
  },
  {
    key: "threads",
    label: t("dashboard.stats.aiSessions"),
    value: String(conversations.value.length),
    meta: selectedConversation.value
      ? formatTimestamp(selectedConversation.value.updatedAtEpochMs)
      : t("dashboard.chat.historySummary")
  }
])
const contextTitle = computed(() =>
  setupRequired.value ? t("dashboard.chat.setupTitle") : t("dashboard.chat.threadControl")
)
const contextSummary = computed(() =>
  setupRequired.value ? t("dashboard.chat.setupSummary") : t("dashboard.chat.threadSummary")
)
const activeThreadTitle = computed(
  () => selectedConversation.value?.title ?? t("dashboard.chat.composeTitle")
)
const activeThreadSummary = computed(() => {
  if (selectedConversation.value) {
    return t("dashboard.chat.composeSummary", {
      device: deviceName(selectedConversation.value.deviceId),
      provider: formatProviderKind(selectedConversation.value.provider)
    })
  }

  return t("dashboard.chat.composeEmptySummary")
})
const inspectorMeta = computed(() => {
  switch (inspectorTab.value) {
    case "git":
      return {
        title: t("dashboard.workspace.git.title"),
        description: t("dashboard.chat.gitSummary")
      }
    case "workspace":
      return {
        title: t("dashboard.workspace.browser.title"),
        description: t("dashboard.chat.workspaceSummary")
      }
    case "trace":
      return {
        title: t("dashboard.chat.panels.trace"),
        description: t("dashboard.chat.traceSummary")
      }
    default:
      return {
        title: t("dashboard.chat.inspectorTitle"),
        description: t("dashboard.chat.inspectorSummary")
      }
  }
})
const overviewCards = computed(() => [
  {
    key: "status",
    label: t("common.status"),
    value: currentThreadStatus.value
  },
  {
    key: "latest",
    label: t("dashboard.chat.latestTurn"),
    value: currentConversationTask.value
      ? formatTimestamp(currentConversationTask.value.createdAtEpochMs)
      : t("common.pending")
  },
  {
    key: "turns",
    label: t("dashboard.stats.aiSessions"),
    value: String(transcriptTurns.value.length)
  },
  {
    key: "trace",
    label: t("dashboard.chat.toolEvents"),
    value: String(activeTraceEventCount.value)
  }
])

function buildAssistantText(detail: TaskDetailResponse) {
  const assistant = detail.events
    .filter((event) => event.kind === "assistant_delta")
    .map((event) => event.message)
    .join("")
    .trim()
  if (assistant) {
    return assistant
  }

  return detail.events
    .filter((event) => event.kind === "provider_stdout")
    .map((event) => event.message)
    .join("\n")
    .trim()
}

function deviceName(deviceId: string) {
  return devices.value.find((device) => device.id === deviceId)?.name ?? deviceId
}

function latestConversationTask(conversationId: string) {
  return store.tasks.find((task) => task.conversationId === conversationId) ?? null
}

function selectDevice(deviceId: string) {
  void store.selectDevice(deviceId)
}

function handleDeviceChange(event: Event) {
  const deviceId = (event.target as HTMLSelectElement).value
  if (!deviceId) {
    return
  }
  selectDevice(deviceId)
}

function selectConversation(conversationId: string) {
  inspectorTab.value = "status"
  void store.selectConversation(conversationId)
}

function submitPrompt() {
  void store.sendConversationPrompt()
}

function startNewConversation() {
  inspectorTab.value = "status"
  store.startNewConversationDraft()
}

function archiveConversation() {
  void store.archiveSelectedConversation()
}

function chooseProvider(provider: string) {
  draft.value.provider = provider as typeof draft.value.provider
}

function choosePendingOption(option: ConversationInputOption) {
  pendingConversationInputDraft.value.optionId = option.id
  if (!option.requiresTextInput) {
    void store.respondToConversationInput({ optionId: option.id })
  }
}

function choosePendingCustomInput() {
  pendingConversationInputDraft.value.optionId = CUSTOM_INPUT_SENTINEL
}

function submitPendingInput() {
  const optionId = pendingConversationInputDraft.value.optionId
  void store.respondToConversationInput({
    optionId: optionId && optionId !== CUSTOM_INPUT_SENTINEL ? optionId : undefined,
    text: pendingConversationInputDraft.value.text.trim() || undefined
  })
}

function previewGitChangedFile(path: string) {
  const file = gitInspect.value?.changedFiles.find((item) => item.path === path)
  if (!file) {
    return
  }
  inspectorTab.value = "workspace"
  void openGitChangedFile(file)
}

function refreshSessionSurface() {
  void Promise.all([store.reloadAll(), refreshGitInspect(), refreshWorkspace()])
}

function toggleContextPanel() {
  contextExpanded.value = !contextExpanded.value
}

function setInspectorPanel(tab: InspectorTab) {
  inspectorTab.value = tab
}

function assistantFallbackText(turn: TranscriptTurn) {
  switch (turn.detail.task.status) {
    case "waiting_input":
      return t("dashboard.chat.waitingInput")
    case "failed":
      return t("dashboard.chat.traceOnlyFailed")
    case "succeeded":
      return t("dashboard.chat.traceOnlyCompleted")
    case "canceled":
      return t("dashboard.chat.traceOnlyCanceled")
    default:
      return t("dashboard.chat.generating")
  }
}

function traceBadgeLabel(turn: TranscriptTurn) {
  const parts: string[] = []

  if (turn.toolEvents.length) {
    parts.push(`${t("dashboard.chat.toolEvents")} ${turn.toolEvents.length}`)
  }
  if (turn.systemEvents.length) {
    parts.push(`${t("dashboard.chat.systemEvents")} ${turn.systemEvents.length}`)
  }

  return parts.join(" · ")
}

function traceCalloutClass(turn: TranscriptTurn) {
  if (
    turn.detail.task.status === "failed" ||
    turn.detail.task.status === "waiting_input" ||
    turn.systemEvents.some((event) => event.kind === "provider_stderr")
  ) {
    return "border-amber-500/30 bg-amber-500/8 text-foreground"
  }

  return "border-border/70 bg-background/45 text-muted-foreground"
}
</script>

<template>
  <section class="space-y-4">
    <div class="space-y-4">
      <Card class="border-border/70 bg-card/85 shadow-xl backdrop-blur-xl">
        <CardContent class="space-y-5 p-4 md:p-5">
          <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
            <div class="space-y-3">
              <Badge
                variant="outline"
                class="border-amber-500/30 bg-amber-500/12 text-amber-700 dark:text-amber-100"
              >
                <WandSparkles class="size-3.5" />
                {{ t("dashboard.chat.liveBadge") }}
              </Badge>

              <div class="space-y-2">
                <h2 class="text-2xl font-semibold tracking-tight text-foreground md:text-3xl">
                  {{ contextTitle }}
                </h2>
                <p class="max-w-3xl text-sm leading-6 text-muted-foreground md:text-base">
                  {{ contextSummary }}
                </p>
                <p
                  v-if="relayBaseUrl"
                  class="font-mono text-xs leading-5 text-muted-foreground break-all"
                >
                  {{ t("dashboard.chat.relayHintLabel") }} · {{ relayBaseUrl }}
                </p>
              </div>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button type="button" variant="outline" size="sm" @click="toggleContextPanel">
                <Server class="size-4" />
                {{
                  showContextPanel
                    ? t("dashboard.chat.hideContextControls")
                    : t("dashboard.chat.contextControls")
                }}
              </Button>
              <Button type="button" variant="outline" size="sm" @click="refreshSessionSurface">
                <RefreshCw class="size-4" />
                {{ t("common.refresh") }}
              </Button>
              <Button type="button" variant="outline" size="sm" @click="startNewConversation">
                <MessageSquarePlus class="size-4" />
                {{ t("dashboard.chat.newConversation") }}
              </Button>
              <Button
                v-if="selectedConversation"
                type="button"
                variant="outline"
                size="sm"
                @click="archiveConversation"
              >
                <Archive class="size-4" />
                {{ t("dashboard.chat.archive") }}
              </Button>
            </div>
          </div>

          <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
            <div
              v-for="card in sessionSummaryCards"
              :key="card.key"
              class="rounded-3xl border border-border/70 bg-background/55 p-4"
            >
              <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ card.label }}
              </p>
              <p class="mt-2 text-sm font-semibold text-foreground md:text-base">
                {{ card.value }}
              </p>
              <p class="mt-2 truncate text-xs text-muted-foreground" :title="card.meta">
                {{ card.meta }}
              </p>
            </div>
          </div>

          <div class="space-y-3">
            <div class="flex items-center justify-between gap-3">
              <div class="space-y-1">
                <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("dashboard.chat.threadSwitcherTitle") }}
                </p>
                <p class="text-sm text-muted-foreground">
                  {{ t("dashboard.chat.historySummary") }}
                </p>
              </div>
              <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
                {{ conversations.length }}
              </Badge>
            </div>

            <ScrollArea class="w-full">
              <div class="flex gap-2 pb-1">
                <button
                  type="button"
                  class="min-w-[220px] rounded-3xl border border-dashed border-border/70 bg-background/50 px-4 py-3 text-left transition hover:bg-accent/40"
                  :class="!selectedConversationId ? 'border-primary/50 bg-primary/10' : ''"
                  @click="startNewConversation"
                >
                  <p class="text-sm font-medium text-foreground">
                    {{ t("dashboard.chat.startBlank") }}
                  </p>
                  <p class="mt-1 text-xs leading-5 text-muted-foreground">
                    {{ t("dashboard.chat.startBlankSummary") }}
                  </p>
                </button>

                <button
                  v-for="conversation in conversations"
                  :key="conversation.id"
                  type="button"
                  class="min-w-[220px] max-w-[260px] rounded-3xl border px-4 py-3 text-left transition"
                  :class="selectedStateClass(conversation.id === selectedConversationId)"
                  @click="selectConversation(conversation.id)"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <p class="truncate text-sm font-medium text-foreground">
                        {{ conversation.title }}
                      </p>
                      <p class="mt-1 truncate text-xs text-muted-foreground">
                        {{ deviceName(conversation.deviceId) }} ·
                        {{ formatProviderKind(conversation.provider) }}
                      </p>
                    </div>
                    <Badge
                      variant="outline"
                      class="shrink-0"
                      :class="taskStatusClass(latestConversationTask(conversation.id)?.status ?? 'pending')"
                    >
                      {{
                        formatTaskStatus(
                          latestConversationTask(conversation.id)?.status ?? "pending"
                        )
                      }}
                    </Badge>
                  </div>
                  <p class="mt-3 text-xs text-muted-foreground">
                    {{ formatTimestamp(conversation.updatedAtEpochMs) }}
                  </p>
                </button>
              </div>
            </ScrollArea>
          </div>
        </CardContent>
      </Card>

      <Card
        v-if="showContextPanel"
        class="border-amber-500/20 bg-card/85 shadow-xl backdrop-blur-xl"
      >
        <CardHeader class="space-y-2">
          <CardTitle class="text-base text-foreground">
            {{ t("dashboard.chat.contextControls") }}
          </CardTitle>
          <CardDescription>{{ contextSummary }}</CardDescription>
        </CardHeader>
        <CardContent class="grid gap-4 pt-0 xl:grid-cols-[minmax(0,1.2fr)_minmax(0,0.8fr)]">
          <div class="grid gap-4">
            <div class="space-y-2">
              <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.relayBaseUrl") }}
              </label>
              <div class="flex flex-col gap-2 md:flex-row">
                <Input
                  v-model="relayInput"
                  :placeholder="relayPlaceholder"
                  class="border-border/70 bg-background/75"
                />
                <Button type="button" class="md:min-w-28" @click="store.applyRelayBaseUrl">
                  {{ t("common.connect") }}
                </Button>
              </div>
            </div>

            <div class="space-y-2">
              <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.fields.accessToken") }}
              </label>
              <Input
                v-model="relayAccessTokenInput"
                type="password"
                :placeholder="t('common.optionalAccessToken')"
                class="border-border/70 bg-background/75"
              />
            </div>

            <div class="grid gap-4 md:grid-cols-2">
              <div class="space-y-2">
                <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("dashboard.shell.selectedDevice") }}
                </label>
                <select
                  :value="selectedDevice?.id ?? ''"
                  :class="nativeSelectClass"
                  @change="handleDeviceChange"
                >
                  <option value="" disabled>
                    {{ t("dashboard.shell.noDeviceSelected") }}
                  </option>
                  <option v-for="device in devices" :key="device.id" :value="device.id">
                    {{ device.name }}
                  </option>
                </select>
              </div>

              <div class="space-y-2">
                <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("common.model") }}
                </label>
                <Input
                  v-model="draft.model"
                  :placeholder="t('common.defaultModel')"
                  class="border-border/70 bg-background/75"
                />
              </div>
            </div>

            <div class="space-y-2">
              <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("common.provider") }}
              </label>
              <div class="flex flex-wrap gap-2">
                <Button
                  v-for="provider in availableProviders"
                  :key="provider.kind"
                  type="button"
                  variant="outline"
                  size="sm"
                  :class="
                    draft.provider === provider.kind
                      ? 'border-sky-500/40 bg-sky-500/15 text-sky-900 hover:bg-sky-500/20 dark:text-white'
                      : 'border-border/70 bg-background/55 text-foreground hover:bg-accent/70'
                  "
                  @click="chooseProvider(provider.kind)"
                >
                  {{ formatProviderKind(provider.kind) }}
                </Button>
              </div>
            </div>
          </div>

          <div class="grid gap-4">
            <div class="space-y-2">
              <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("common.title") }}
              </label>
              <Input
                v-model="draft.title"
                :placeholder="t('dashboard.chat.autoTitleHint')"
                class="border-border/70 bg-background/75"
              />
            </div>

            <div class="space-y-2">
              <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.chat.cwd") }}
              </label>
              <Input
                v-model="draft.cwd"
                :placeholder="t('common.useAgentWorkingRoot')"
                class="border-border/70 bg-background/75"
              />
            </div>

            <div class="rounded-3xl border border-border/70 bg-background/55 p-4">
              <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("common.status") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">
                {{ t(`dashboard.sessions.launchState.${sessionLaunchState}`) }}
              </p>
            </div>

            <div class="rounded-3xl border border-border/70 bg-background/55 p-4">
              <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.chat.relayHintLabel") }}
              </p>
              <p class="mt-2 font-mono text-xs leading-5 text-foreground break-all">
                {{ relayBaseUrl || relayPlaceholder }}
              </p>
            </div>
          </div>

          <ScrollArea class="h-[420px]">
            <div class="space-y-2 p-3">
              <button
                type="button"
                class="w-full rounded-2xl border border-dashed border-border/70 bg-background/45 px-4 py-3 text-left transition hover:bg-accent/40"
                :class="
                  !selectedConversationId
                    ? 'border-sky-500/40 bg-sky-500/10'
                    : 'border-dashed border-border/70'
                "
                @click="startNewConversation"
              >
                <p class="text-sm font-medium text-foreground">
                  {{ t("dashboard.chat.startBlank") }}
                </p>
                <p class="mt-1 text-xs leading-5 text-muted-foreground">
                  {{ t("dashboard.chat.startBlankSummary") }}
                </p>
              </button>

              <button
                v-for="conversation in conversations"
                :key="conversation.id"
                type="button"
                class="w-full rounded-3xl border px-4 py-3 text-left transition"
                :class="selectedStateClass(conversation.id === selectedConversationId)"
                @click="selectConversation(conversation.id)"
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0 space-y-2">
                    <p class="truncate text-sm font-medium text-foreground">
                      {{ conversation.title }}
                    </p>
                    <div class="flex flex-wrap gap-2 text-xs text-muted-foreground">
                      <span>{{ deviceName(conversation.deviceId) }}</span>
                      <span>·</span>
                      <span>{{ formatProviderKind(conversation.provider) }}</span>
                    </div>
                  </div>
                  <Badge
                    variant="outline"
                    class="shrink-0"
                    :class="taskStatusClass(latestConversationTask(conversation.id)?.status ?? 'pending')"
                  >
                    {{
                      formatTaskStatus(
                        latestConversationTask(conversation.id)?.status ?? "pending"
                      )
                    }}
                  </Badge>
                </div>
                <p class="mt-3 text-xs text-muted-foreground">
                  {{ formatTimestamp(conversation.updatedAtEpochMs) }}
                </p>
              </button>
            </div>
          </ScrollArea>
        </CardContent>
      </Card>
    </div>

    <Card class="border-border/70 bg-card/85 shadow-2xl backdrop-blur-xl">
      <CardContent class="flex h-full min-h-[780px] flex-col p-0">
        <header class="border-b border-border/70 px-5 py-4">
          <div class="flex flex-col gap-4 xl:flex-row xl:items-start xl:justify-between">
            <div class="space-y-2">
              <Badge
                variant="outline"
                class="border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-100"
              >
                {{ selectedConversation ? t("dashboard.chat.activeThread") : t("dashboard.chat.newThread") }}
              </Badge>
              <div class="space-y-1">
                <h2 class="text-2xl font-semibold tracking-tight text-foreground">
                  {{ selectedConversation?.title ?? t("dashboard.chat.composeTitle") }}
                </h2>
                <p class="text-sm leading-6 text-muted-foreground">
                  {{
                    selectedConversation
                      ? t("dashboard.chat.composeSummary", {
                          device: deviceName(selectedConversation.deviceId),
                          provider: formatProviderKind(selectedConversation.provider)
                        })
                      : t("dashboard.chat.composeEmptySummary")
                  }}
                </p>
              </div>
            </div>

            <div class="flex flex-wrap items-center gap-2">
              <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
                <Server class="size-3.5" />
                {{ selectedConversation ? deviceName(selectedConversation.deviceId) : (selectedDevice?.name ?? t("dashboard.shell.noDeviceSelected")) }}
              </Badge>
              <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
                <Sparkles class="size-3.5" />
                {{
                  selectedConversation
                    ? formatProviderKind(selectedConversation.provider)
                    : (draft.provider ? formatProviderKind(draft.provider) : t("common.provider"))
                }}
              </Badge>
              <Badge
                v-if="currentConversationTask"
                variant="outline"
                :class="taskStatusClass(currentConversationTask.status)"
              >
                {{ formatTaskStatus(currentConversationTask.status) }}
              </Badge>
              <Button
                v-if="selectedConversation"
                type="button"
                size="sm"
                variant="outline"
                @click="archiveConversation"
              >
                <Archive class="size-4" />
                {{ t("dashboard.chat.archive") }}
              </Button>
            </div>
          </div>
        </header>

        <ScrollArea class="min-h-0 flex-1">
          <div class="space-y-8 px-5 py-6">
            <template v-if="transcriptTurns.length">
              <article
                v-for="turn in transcriptTurns"
                :key="turn.detail.task.id"
                class="space-y-4"
              >
                <div class="flex justify-end">
                  <div class="max-w-[88%] rounded-[28px] bg-slate-900 px-5 py-4 text-sm leading-7 text-slate-50 shadow-lg dark:bg-slate-50 dark:text-slate-900">
                    <div class="mb-3 flex items-center justify-between gap-3">
                      <div class="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-slate-300 dark:text-slate-500">
                        <UserRound class="size-3.5" />
                        {{ t("dashboard.chat.userTurn") }}
                      </div>
                      <span class="text-xs text-slate-300/80 dark:text-slate-500">
                        {{ formatTimestamp(turn.detail.task.createdAtEpochMs) }}
                      </span>
                    </div>
                    <p class="whitespace-pre-wrap">{{ turn.detail.task.prompt }}</p>
                  </div>
                </div>

                <div class="flex gap-3">
                  <div class="mt-1 flex size-10 shrink-0 items-center justify-center rounded-2xl bg-amber-500/12 text-amber-700 dark:text-amber-100">
                    <Bot class="size-5" />
                  </div>
                  <div class="min-w-0 flex-1 space-y-3">
                    <div class="rounded-[28px] border border-border/70 bg-background/70 px-5 py-4 shadow-sm">
                      <div class="mb-3 flex flex-wrap items-center gap-2">
                        <Badge
                          variant="outline"
                          :class="taskStatusClass(turn.detail.task.status)"
                        >
                          {{ formatTaskStatus(turn.detail.task.status) }}
                        </Badge>
                        <Badge
                          variant="outline"
                          class="border-border/70 bg-background/70 text-foreground"
                        >
                          {{ formatExecutionProtocol(turn.detail.task.executionProtocol) }}
                        </Badge>
                      </div>

                      <p
                        v-if="turn.assistantText"
                        class="whitespace-pre-wrap text-sm leading-7 text-foreground"
                      >
                        {{ turn.assistantText }}
                      </p>
                      <p
                        v-else
                        class="text-sm leading-7 text-muted-foreground"
                      >
                        {{ assistantFallbackText(turn) }}
                      </p>
                    </div>

                    <button
                      v-if="turn.traceEvents.length"
                      type="button"
                      class="w-full rounded-2xl border px-4 py-3 text-left transition hover:bg-accent/35"
                      :class="traceCalloutClass(turn)"
                      @click="setInspectorPanel('trace')"
                    >
                      <div class="flex items-start justify-between gap-3">
                        <div class="min-w-0 space-y-1">
                          <p class="text-sm font-medium text-foreground">
                            {{ t("dashboard.chat.traceEntryTitle") }}
                          </p>
                          <p class="text-xs leading-5 text-muted-foreground">
                            {{ traceBadgeLabel(turn) }}
                          </p>
                        </div>
                        <Badge
                          variant="outline"
                          class="shrink-0 border-border/70 bg-background/60 text-foreground"
                        >
                          {{ t("dashboard.chat.panels.trace") }}
                        </Badge>
                      </div>
                    </button>
                  </div>
                </div>
              </article>
            </template>

            <div
              v-else
              class="rounded-[32px] border border-dashed border-border/70 bg-gradient-to-br from-amber-500/8 via-background to-sky-500/8 px-6 py-10 text-center"
            >
              <div class="mx-auto max-w-xl space-y-3">
                <Badge
                  variant="outline"
                  class="border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-100"
                >
                  {{ t("dashboard.chat.emptyBadge") }}
                </Badge>
                <h3 class="text-2xl font-semibold tracking-tight text-foreground">
                  {{ t("dashboard.chat.emptyTitle") }}
                </h3>
                <p class="text-sm leading-7 text-muted-foreground">
                  {{ t("dashboard.chat.emptySummary") }}
                </p>
              </div>
            </div>
          </div>
        </ScrollArea>

        <div
          v-if="pendingInputRequest"
          class="border-t border-border/70 bg-amber-500/6 px-5 py-4"
        >
          <div class="space-y-4 rounded-[28px] border border-amber-500/25 bg-background/85 p-5 shadow-sm">
            <div class="space-y-1">
              <Badge
                variant="outline"
                class="border-amber-500/30 bg-amber-500/12 text-amber-700 dark:text-amber-100"
              >
                {{ t("dashboard.chat.inputRequestBadge") }}
              </Badge>
              <h3 class="text-lg font-semibold text-foreground">
                {{ pendingInputRequest.prompt }}
              </h3>
              <p class="text-sm text-muted-foreground">
                {{ t("dashboard.chat.inputRequestSummary") }}
              </p>
            </div>

            <div v-if="pendingInputRequest.options.length" class="flex flex-wrap gap-2">
              <button
                v-for="option in pendingInputRequest.options"
                :key="option.id"
                type="button"
                class="rounded-2xl border px-4 py-3 text-left transition"
                :class="
                  pendingConversationInputDraft.optionId === option.id
                    ? 'border-sky-500/40 bg-sky-500/12 text-sky-900 dark:text-sky-100'
                    : 'border-border/70 bg-background/60 text-foreground hover:bg-accent/40'
                "
                @click="choosePendingOption(option)"
              >
                <p class="text-sm font-medium">{{ option.label }}</p>
                <p v-if="option.description" class="mt-1 text-xs leading-5 text-muted-foreground">
                  {{ option.description }}
                </p>
              </button>

              <button
                v-if="pendingInputRequest.allowCustomInput"
                type="button"
                class="rounded-2xl border px-4 py-3 text-left transition"
                :class="
                  pendingConversationInputDraft.optionId === CUSTOM_INPUT_SENTINEL
                    ? 'border-sky-500/40 bg-sky-500/12 text-sky-900 dark:text-sky-100'
                    : 'border-border/70 bg-background/60 text-foreground hover:bg-accent/40'
                "
                @click="choosePendingCustomInput"
              >
                <p class="text-sm font-medium">{{ t("dashboard.chat.customOption") }}</p>
                <p class="mt-1 text-xs leading-5 text-muted-foreground">
                  {{ t("dashboard.chat.customOptionSummary") }}
                </p>
              </button>
            </div>

            <div v-if="showPendingTextInput" class="space-y-3">
              <Textarea
                v-model="pendingConversationInputDraft.text"
                :placeholder="pendingInputRequest.customInputPlaceholder ?? t('dashboard.chat.customInputPlaceholder')"
                class="min-h-[120px] border-border/70 bg-background/75"
              />
              <div class="flex justify-end">
                <Button type="button" :disabled="!canSubmitPendingText" @click="submitPendingInput">
                  <Send class="size-4" />
                  {{ t("dashboard.chat.submitChoice") }}
                </Button>
              </div>
            </div>
          </div>
        </div>

        <footer class="border-t border-border/70 bg-card/90 px-5 py-4">
          <form class="space-y-3" @submit.prevent="submitPrompt">
            <div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_220px]">
              <Textarea
                v-model="draft.prompt"
                :placeholder="
                  selectedConversation
                    ? t('dashboard.chat.replyPlaceholder')
                    : t('dashboard.chat.startPlaceholder')
                "
                class="min-h-[144px] border-border/70 bg-background/75 text-base leading-7"
              />
              <div class="space-y-3">
                <div class="space-y-2">
                  <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                    {{ t("common.model") }}
                  </label>
                  <Input v-model="draft.model" :placeholder="t('common.defaultModel')" />
                </div>
                <div v-if="!selectedConversation" class="space-y-2">
                  <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                    {{ t("common.title") }}
                  </label>
                  <Input v-model="draft.title" :placeholder="t('dashboard.chat.autoTitleHint')" />
                </div>
                <Button type="submit" class="w-full" :disabled="!canSendPrompt">
                  <Send class="size-4" />
                  {{
                    selectedConversation
                      ? t("dashboard.chat.sendReply")
                      : t("dashboard.chat.startConversation")
                  }}
                </Button>
              </div>
            </div>
          </form>
        </footer>
      </CardContent>
    </Card>

    <div class="space-y-4">
      <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
        <CardHeader class="space-y-2">
          <div class="flex flex-wrap gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              :class="
                inspectorTab === 'status'
                  ? 'border-sky-500/40 bg-sky-500/12 text-sky-900 dark:text-sky-100'
                  : 'border-border/70 bg-background/55 text-foreground'
              "
              @click="setInspectorPanel('status')"
            >
              {{ t("dashboard.chat.panels.status") }}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              :class="
                inspectorTab === 'git'
                  ? 'border-sky-500/40 bg-sky-500/12 text-sky-900 dark:text-sky-100'
                  : 'border-border/70 bg-background/55 text-foreground'
              "
              @click="setInspectorPanel('git')"
            >
              {{ t("dashboard.chat.panels.git") }}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              :class="
                inspectorTab === 'workspace'
                  ? 'border-sky-500/40 bg-sky-500/12 text-sky-900 dark:text-sky-100'
                  : 'border-border/70 bg-background/55 text-foreground'
              "
              @click="setInspectorPanel('workspace')"
            >
              {{ t("dashboard.chat.panels.files") }}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              :class="
                inspectorTab === 'trace'
                  ? 'border-sky-500/40 bg-sky-500/12 text-sky-900 dark:text-sky-100'
                  : 'border-border/70 bg-background/55 text-foreground'
              "
              @click="setInspectorPanel('trace')"
            >
              {{ t("dashboard.chat.panels.trace") }}
              <Badge variant="outline" class="ml-2 border-border/70 bg-background/60 text-foreground">
                {{ activeTraceEventCount }}
              </Badge>
            </Button>
          </div>
          <div class="space-y-1">
            <CardTitle class="text-base">{{ inspectorMeta.title }}</CardTitle>
            <CardDescription>{{ inspectorMeta.description }}</CardDescription>
          </div>
        </CardHeader>
        <CardContent v-if="inspectorTab === 'status'" class="space-y-3">
          <div class="flex flex-wrap gap-2">
            <Badge
              v-if="currentConversationTask"
              variant="outline"
              :class="taskStatusClass(currentConversationTask.status)"
            >
              {{ formatTaskStatus(currentConversationTask.status) }}
            </Badge>
            <Badge
              v-if="currentConversationTask"
              variant="outline"
              class="border-border/70 bg-background/60 text-foreground"
            >
              {{ formatExecutionProtocol(currentConversationTask.executionProtocol) }}
            </Badge>
            <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
              {{ t("dashboard.stats.aiSessions") }} · {{ transcriptTurns.length }}
            </Badge>
          </div>

          <div class="grid gap-3 sm:grid-cols-2 2xl:grid-cols-1">
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.chat.latestTurn") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">
                {{
                  currentConversationTask
                    ? formatTimestamp(currentConversationTask.createdAtEpochMs)
                    : t("common.pending")
                }}
              </p>
            </div>
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.chat.toolEvents") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">
                {{ sessionEventCounts.tool }}
              </p>
            </div>
          </div>

          <Button
            type="button"
            variant="outline"
            size="sm"
            class="w-full"
            @click="refreshSessionSurface"
          >
            <RefreshCw class="size-4" />
            {{ t("common.refresh") }}
          </Button>
        </CardContent>
      </Card>

      <Card
        v-if="inspectorTab === 'git'"
        class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl"
      >
        <CardHeader class="space-y-1">
          <CardTitle class="flex items-center gap-2 text-base">
            <GitBranch class="size-4" />
            {{ t("dashboard.workspace.git.title") }}
          </CardTitle>
          <CardDescription>{{ t("dashboard.chat.gitSummary") }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-3">
          <div v-if="gitLoading" class="flex items-center gap-2 text-sm text-muted-foreground">
            <LoaderCircle class="size-4 animate-spin" />
            {{ t("common.waiting") }}
          </div>
          <p v-else-if="gitError" class="text-sm text-rose-600 dark:text-rose-200">
            {{ gitError }}
          </p>
          <template v-else-if="gitInspect?.state === 'ready'">
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-sm font-medium text-foreground">
                {{ gitInspect.branchName ?? "HEAD" }}
              </p>
              <p class="mt-1 text-xs text-muted-foreground">
                {{
                  formatGitDrift(
                    gitInspect.aheadCount,
                    gitInspect.behindCount
                  )
                }}
              </p>
            </div>

            <div class="space-y-2">
              <button
                v-for="file in gitInspect.changedFiles.slice(0, 8)"
                :key="file.path"
                type="button"
                class="w-full rounded-2xl border border-border/70 bg-background/55 px-4 py-3 text-left transition hover:bg-accent/35"
                @click="previewGitChangedFile(file.path)"
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <p class="truncate text-sm font-medium text-foreground">{{ file.path }}</p>
                    <p class="mt-1 text-xs text-muted-foreground">
                      {{ formatGitFileStatus(file.status) }}
                    </p>
                  </div>
                  <Badge
                    variant="outline"
                    class="shrink-0"
                    :class="
                      canPreviewGitChangedFile(file)
                        ? 'border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-100'
                        : 'border-border/70 bg-background/55 text-muted-foreground'
                    "
                  >
                    {{ canPreviewGitChangedFile(file) ? t("common.refresh") : t("common.pending") }}
                  </Badge>
                </div>
              </button>
            </div>
          </template>
          <p v-else class="text-sm text-muted-foreground">
            {{ t("dashboard.chat.gitEmpty") }}
          </p>
        </CardContent>
      </Card>

      <Card
        v-if="inspectorTab === 'workspace'"
        class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl"
      >
        <CardHeader class="space-y-1">
          <CardTitle class="flex items-center gap-2 text-base">
            <FolderCode class="size-4" />
            {{ t("dashboard.workspace.browser.title") }}
          </CardTitle>
          <CardDescription>{{ t("dashboard.chat.workspaceSummary") }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-3">
          <div v-if="workspaceLoading" class="flex items-center gap-2 text-sm text-muted-foreground">
            <LoaderCircle class="size-4 animate-spin" />
            {{ t("common.waiting") }}
          </div>
          <p
            v-else-if="workspaceError"
            class="text-sm text-rose-600 dark:text-rose-200"
          >
            {{ workspaceError }}
          </p>
          <template v-else-if="workspaceListing">
            <div class="space-y-2">
              <button
                v-for="entry in workspaceListing.entries.slice(0, 8)"
                :key="entry.path"
                type="button"
                class="w-full rounded-2xl border border-border/70 bg-background/55 px-4 py-3 text-left transition hover:bg-accent/35"
                @click="openWorkspaceEntry(entry)"
              >
                <div class="flex items-center justify-between gap-3">
                  <div class="min-w-0">
                    <p class="truncate text-sm font-medium text-foreground">
                      {{ entry.name }}
                    </p>
                    <p class="mt-1 text-xs text-muted-foreground">
                      {{ entry.path }}
                    </p>
                  </div>
                  <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
                    {{ entry.kind }}
                  </Badge>
                </div>
              </button>
            </div>

            <div
              v-if="workspacePreview?.content"
              class="rounded-2xl border border-border/70 bg-slate-950 px-4 py-3 text-xs leading-6 text-slate-100 dark:bg-slate-900"
            >
              <p class="mb-2 text-slate-400">{{ workspacePreview.path }}</p>
              <pre class="whitespace-pre-wrap">{{ workspacePreview.content }}</pre>
            </div>
          </template>
          <p v-else class="text-sm text-muted-foreground">
            {{ t("dashboard.chat.workspaceEmpty") }}
          </p>
        </CardContent>
      </Card>

      <Card
        v-if="inspectorTab === 'trace'"
        class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl"
      >
        <CardHeader class="space-y-1">
          <CardTitle class="text-base">{{ t("dashboard.chat.panels.trace") }}</CardTitle>
          <CardDescription>{{ t("dashboard.chat.traceSummary") }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-3">
          <Button type="button" variant="outline" size="sm" class="w-full" @click="refreshSessionSurface">
            <RefreshCw class="size-4" />
            {{ t("common.refresh") }}
          </Button>

          <div v-if="traceTurns.length" class="space-y-3">
            <div
              v-for="turn in traceTurns"
              :key="`${turn.detail.task.id}-trace`"
              class="rounded-2xl border border-border/70 bg-background/55 p-4"
            >
              <div class="flex flex-wrap items-center justify-between gap-3">
                <div class="space-y-1">
                  <p class="text-sm font-medium text-foreground">
                    {{ formatTimestamp(turn.detail.task.createdAtEpochMs) }}
                  </p>
                  <p class="text-xs leading-5 text-muted-foreground whitespace-pre-wrap">
                    {{ turn.detail.task.prompt }}
                  </p>
                </div>
                <Badge variant="outline" :class="taskStatusClass(turn.detail.task.status)">
                  {{ formatTaskStatus(turn.detail.task.status) }}
                </Badge>
              </div>

              <div class="mt-4 space-y-2">
                <div
                  v-for="event in turn.traceEvents"
                  :key="`${turn.detail.task.id}-${event.seq}`"
                  class="rounded-2xl border border-border/60 bg-background/70 px-4 py-3"
                >
                  <div class="flex flex-wrap items-center justify-between gap-2">
                    <span class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                      {{ formatEventKind(event.kind) }}
                    </span>
                    <span class="text-xs text-muted-foreground">
                      {{ formatTimestamp(event.timestampEpochMs) }}
                    </span>
                  </div>
                  <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-foreground">
                    {{ event.message }}
                  </p>
                </div>
              </div>
            </div>
          </div>
          <p v-else class="text-sm text-muted-foreground">
            {{ t("dashboard.chat.traceEmpty") }}
          </p>
        </CardContent>
      </Card>
    </div>
  </section>
</template>
