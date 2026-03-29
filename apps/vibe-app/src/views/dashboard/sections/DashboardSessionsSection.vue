<script setup lang="ts">
import { computed } from "vue"
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
}

const CUSTOM_INPUT_SENTINEL = "__custom_input__"

const { t } = useI18n()
const dashboard = useDashboardController()
const store = dashboard.store
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
  openGitChangedFile,
  openWorkspaceEntry,
  refreshGitInspect,
  refreshWorkspace,
  relayPlaceholder,
  selectedStateClass,
  sessionEventCounts,
  sessionLaunchState,
  taskStatusClass,
  workspaceError,
  workspaceListing,
  workspaceLoading,
  workspacePreview
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
  selectedConversationDetail,
  selectedConversationId,
  selectedDevice
} = storeToRefs(store)

const availableProviders = computed(() => store.availableProviders)
const selectedConversation = computed(() => store.selectedConversation)
const pendingInputRequest = computed(() => store.conversationPendingInput)
const currentConversationTask = computed(() => store.conversationCurrentTask)
const setupReady = computed(() => sessionLaunchState.value === "ready")
const showSetupPanel = computed(() => {
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
  (selectedConversationDetail.value?.tasks ?? []).map((detail) => ({
    detail,
    assistantText: buildAssistantText(detail),
    toolEvents: detail.events.filter(
      (event) => event.kind === "tool_call" || event.kind === "tool_output"
    ),
    systemEvents: detail.events.filter(
      (event) =>
        event.kind === "system" ||
        event.kind === "status" ||
        event.kind === "provider_stderr"
    )
  }))
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
  void store.selectConversation(conversationId)
}

function submitPrompt() {
  void store.sendConversationPrompt()
}

function startNewConversation() {
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
  void openGitChangedFile(file)
}

function refreshInspector() {
  void Promise.all([refreshGitInspect(), refreshWorkspace()])
}
</script>

<template>
  <section class="grid gap-4 2xl:grid-cols-[260px_minmax(0,1fr)_320px]">
    <div class="space-y-4">
      <Card class="border-border/70 bg-card/85 shadow-xl backdrop-blur-xl">
        <CardContent class="space-y-4 p-4">
          <div class="flex items-start justify-between gap-3">
            <div class="space-y-1">
              <Badge
                variant="outline"
                class="border-amber-500/30 bg-amber-500/12 text-amber-700 dark:text-amber-100"
              >
                <WandSparkles class="size-3.5" />
                {{ t("dashboard.chat.liveBadge") }}
              </Badge>
              <h2 class="text-lg font-semibold tracking-tight text-foreground">
                {{
                  showSetupPanel
                    ? t("dashboard.chat.setupTitle")
                    : t("dashboard.chat.threadControl")
                }}
              </h2>
              <p class="text-sm leading-6 text-muted-foreground">
                {{
                  showSetupPanel
                    ? t("dashboard.chat.setupSummary")
                    : t("dashboard.chat.threadSummary")
                }}
              </p>
            </div>

            <Button type="button" variant="outline" size="sm" @click="startNewConversation">
              <MessageSquarePlus class="size-4" />
              {{ t("dashboard.chat.newConversation") }}
            </Button>
          </div>

          <div class="grid gap-3 sm:grid-cols-3 2xl:grid-cols-1">
            <div class="rounded-2xl border border-border/70 bg-background/55 p-3">
              <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.shell.connectionState") }}
              </p>
              <div class="mt-2 flex items-center gap-2 text-sm font-medium text-foreground">
                <Sparkles class="size-4 text-amber-500" />
                {{ formatConnectionState(eventState) }}
              </div>
            </div>
            <div class="rounded-2xl border border-border/70 bg-background/55 p-3">
              <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.shell.selectedDevice") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">
                {{ selectedDevice?.name ?? t("dashboard.shell.noDeviceSelected") }}
              </p>
            </div>
            <div class="rounded-2xl border border-border/70 bg-background/55 p-3">
              <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.stats.aiSessions") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">
                {{ conversations.length }}
              </p>
            </div>
          </div>

          <div v-if="showSetupPanel" class="space-y-3 rounded-3xl border border-amber-500/20 bg-amber-500/8 p-4">
            <div class="space-y-2">
              <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.relayBaseUrl") }}
              </label>
              <div class="flex flex-col gap-2">
                <Input
                  v-model="relayInput"
                  :placeholder="relayPlaceholder"
                  class="border-border/70 bg-background/75"
                />
                <Button type="button" @click="store.applyRelayBaseUrl">
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

            <div class="grid gap-3 sm:grid-cols-2">
              <div class="space-y-2">
                <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("common.title") }}
                </label>
                <Input v-model="draft.title" :placeholder="t('dashboard.chat.autoTitleHint')" />
              </div>
              <div class="space-y-2">
                <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("common.model") }}
                </label>
                <Input v-model="draft.model" :placeholder="t('common.defaultModel')" />
              </div>
            </div>

            <div class="space-y-2">
              <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("dashboard.chat.cwd") }}
              </label>
              <Input v-model="draft.cwd" :placeholder="t('common.useAgentWorkingRoot')" />
            </div>
          </div>

          <details v-else class="rounded-2xl border border-border/70 bg-background/55 p-4">
            <summary class="cursor-pointer text-sm font-medium text-foreground">
              {{ t("dashboard.chat.switchRelay") }}
            </summary>
            <div class="mt-3 space-y-3">
              <Input
                v-model="relayInput"
                :placeholder="relayPlaceholder"
                class="border-border/70 bg-background/75"
              />
              <Input
                v-model="relayAccessTokenInput"
                type="password"
                :placeholder="t('common.optionalAccessToken')"
                class="border-border/70 bg-background/75"
              />
              <Button type="button" size="sm" @click="store.applyRelayBaseUrl">
                {{ t("dashboard.chat.reconnect") }}
              </Button>
            </div>
          </details>
        </CardContent>
      </Card>

      <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
        <CardHeader class="space-y-1">
          <CardTitle class="text-base">{{ t("dashboard.chat.historyTitle") }}</CardTitle>
          <CardDescription>{{ t("dashboard.chat.historySummary") }}</CardDescription>
        </CardHeader>
        <CardContent class="p-0">
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
                        {{
                          turn.detail.task.status === "waiting_input"
                            ? t("dashboard.chat.waitingInput")
                            : t("dashboard.chat.generating")
                        }}
                      </p>
                    </div>

                    <div v-if="turn.toolEvents.length" class="grid gap-2">
                      <div
                        v-for="toolEvent in turn.toolEvents"
                        :key="`${turn.detail.task.id}-${toolEvent.seq}`"
                        class="rounded-2xl border border-border/70 bg-background/60 px-4 py-3"
                      >
                        <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
                          {{ formatEventKind(toolEvent.kind) }}
                        </p>
                        <p class="mt-2 whitespace-pre-wrap text-sm leading-6 text-foreground">
                          {{ toolEvent.message }}
                        </p>
                      </div>
                    </div>

                    <div v-if="turn.systemEvents.length" class="space-y-2">
                      <div
                        v-for="systemEvent in turn.systemEvents"
                        :key="`${turn.detail.task.id}-${systemEvent.seq}`"
                        class="rounded-2xl border border-border/60 bg-background/45 px-4 py-3 text-sm text-muted-foreground"
                      >
                        <div class="flex flex-wrap items-center justify-between gap-2">
                          <span>{{ formatEventKind(systemEvent.kind) }}</span>
                          <span class="text-xs">{{ formatTimestamp(systemEvent.timestampEpochMs) }}</span>
                        </div>
                        <p class="mt-2 whitespace-pre-wrap leading-6">
                          {{ systemEvent.message }}
                        </p>
                      </div>
                    </div>
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
        <CardHeader class="space-y-1">
          <CardTitle class="text-base">{{ t("dashboard.chat.inspectorTitle") }}</CardTitle>
          <CardDescription>{{ t("dashboard.chat.inspectorSummary") }}</CardDescription>
        </CardHeader>
        <CardContent class="space-y-3">
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

          <Button type="button" variant="outline" size="sm" class="w-full" @click="refreshInspector">
            <RefreshCw class="size-4" />
            {{ t("common.refresh") }}
          </Button>
        </CardContent>
      </Card>

      <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
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

      <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
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
    </div>
  </section>
</template>
