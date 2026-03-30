<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatDateTime } from "@/lib/format";
import { resolvePolicyRuntimeContext } from "@/lib/policy";
import { useAppStore } from "@/stores/app";
import type {
  ConversationDetailResponse,
  ExecutionProtocol,
  ProviderKind,
  TaskDetailResponse,
  TaskEvent,
  TaskExecutionMode,
  TaskStatus
} from "@/types";

const props = defineProps<{
  detail: ConversationDetailResponse | null;
  conversations: { id: string; title: string; updatedAtEpochMs: number }[];
  activeConversationId: string | null;
  projectProviders?: ProviderKind[];
  projectTitle?: string | null;
}>();

const emit = defineEmits<{
  selectConversation: [conversationId: string];
  sendPrompt: [prompt: string, model?: string, executionMode?: TaskExecutionMode];
  respondInput: [optionId?: string, text?: string];
  cancelTask: [taskId: string];
  openTab: [tab: "changes" | "logs"];
}>();

const { t } = useI18n();
const store = useAppStore();
const prompt = ref("");
const model = ref("");
const executionMode = ref<TaskExecutionMode>(store.defaultExecutionMode);
const customReply = ref("");
const pendingConfirmation = ref(false);

watch(
  () => store.defaultExecutionMode,
  (value) => {
    executionMode.value = value;
  }
);

const executionModeOptions: { value: TaskExecutionMode; labelKey: string; summaryKey: string }[] = [
  {
    value: "read_only",
    labelKey: "conversation.executionMode.readOnly",
    summaryKey: "conversation.executionMode.readOnlySummary"
  },
  {
    value: "workspace_write",
    labelKey: "conversation.executionMode.workspaceWrite",
    summaryKey: "conversation.executionMode.workspaceWriteSummary"
  },
  {
    value: "workspace_write_and_test",
    labelKey: "conversation.executionMode.workspaceWriteAndTest",
    summaryKey: "conversation.executionMode.workspaceWriteAndTestSummary"
  }
];

const selectedExecutionMode = computed(
  () => executionModeOptions.find((option) => option.value === executionMode.value) ?? executionModeOptions[1]
);
const composerPolicyContext = computed(() =>
  resolvePolicyRuntimeContext(props.detail, props.projectProviders)
);

const composerPolicySummary = computed(() =>
  buildPolicySummary(
    composerPolicyContext.value.provider,
    composerPolicyContext.value.executionProtocol,
    executionMode.value
  )
);

const turnCards = computed(() =>
  (props.detail?.tasks ?? []).map((taskDetail) => {
    const assistantText = taskDetail.events
      .filter((event) => event.kind === "assistant_delta")
      .map((event) => event.message)
      .join("")
      .trim();
    const nonAssistantEvents = taskDetail.events.filter((event) => event.kind !== "assistant_delta");

    return {
      id: taskDetail.task.id,
      task: taskDetail.task,
      prompt: taskDetail.task.prompt,
      summary: buildTurnSummary(taskDetail, assistantText),
      policySummary: buildPolicySummary(
        taskDetail.task.provider,
        taskDetail.task.executionProtocol,
        taskDetail.task.executionMode
      ),
      eventSummary: buildEventSummary(nonAssistantEvents),
      eventLines: nonAssistantEvents.slice(-6),
      rawLines: taskDetail.events
    };
  })
);

function submitPrompt() {
  if (!prompt.value.trim()) {
    return;
  }

  if (requiresSensitiveConfirmation(prompt.value, executionMode.value) && !pendingConfirmation.value) {
    pendingConfirmation.value = true;
    return;
  }

  emit(
    "sendPrompt",
    prompt.value.trim(),
    model.value.trim() || undefined,
    executionMode.value
  );
  prompt.value = "";
  model.value = "";
  pendingConfirmation.value = false;
}

function submitCustomReply() {
  if (!customReply.value.trim()) {
    return;
  }

  emit("respondInput", undefined, customReply.value.trim());
  customReply.value = "";
}

function requiresSensitiveConfirmation(promptText: string, mode: TaskExecutionMode) {
  if (!store.sensitiveConfirmationEnabled) {
    return false;
  }
  if (mode === "read_only") {
    return false;
  }

  return /(rm\s+-rf|git\s+reset\s+--hard|drop\s+table|truncate\s+table|force\s+push|delete\s+.*file|remove\s+.*directory)/i.test(
    promptText
  );
}

function buildTurnSummary(taskDetail: TaskDetailResponse, assistantText: string) {
  if (assistantText) {
    return assistantText;
  }
  if (taskDetail.task.error) {
    return taskDetail.task.error;
  }

  return t(`conversation.statusSummary.${taskDetail.task.status}`);
}

function buildEventSummary(events: TaskEvent[]) {
  const toolCalls = events.filter((event) => event.kind === "tool_call").length;
  const toolOutputs = events.filter((event) => event.kind === "tool_output").length;
  const stderr = events.filter((event) => event.kind === "provider_stderr").length;
  const status = events.filter((event) => event.kind === "status").length;
  const parts = [
    status ? t("conversation.eventSummary.status", { count: status }) : null,
    toolCalls ? t("conversation.eventSummary.toolCalls", { count: toolCalls }) : null,
    toolOutputs ? t("conversation.eventSummary.toolOutputs", { count: toolOutputs }) : null,
    stderr ? t("conversation.eventSummary.errors", { count: stderr }) : null
  ].filter((value): value is string => Boolean(value));

  return parts.length ? parts.join(" · ") : t("conversation.eventSummary.idle");
}

function buildPolicySummary(
  provider: ProviderKind,
  executionProtocol: ExecutionProtocol,
  mode: TaskExecutionMode
) {
  if (executionProtocol === "acp") {
    if (mode === "read_only") {
      return t("conversation.policySummary.acp.readOnly");
    }
    if (mode === "workspace_write") {
      return t("conversation.policySummary.acp.workspaceWrite");
    }
    return t("conversation.policySummary.acp.workspaceWriteAndTest");
  }

  if (provider === "codex") {
    if (mode === "read_only") {
      return t("conversation.policySummary.codex.readOnly");
    }
    if (mode === "workspace_write") {
      return t("conversation.policySummary.codex.workspaceWrite");
    }
    return t("conversation.policySummary.codex.workspaceWriteAndTest");
  }

  if (provider === "claude_code") {
    if (mode === "read_only") {
      return t("conversation.policySummary.claude.readOnly");
    }
    if (mode === "workspace_write") {
      return t("conversation.policySummary.claude.workspaceWrite");
    }
    return t("conversation.policySummary.claude.workspaceWriteAndTest");
  }

  if (mode === "read_only") {
    return t("conversation.policySummary.generic.readOnly");
  }
  if (mode === "workspace_write") {
    return t("conversation.policySummary.generic.workspaceWrite");
  }
  return t("conversation.policySummary.generic.workspaceWriteAndTest");
}

function taskTone(status: TaskStatus) {
  if (status === "failed" || status === "canceled") {
    return "danger" as const;
  }
  if (status === "waiting_input" || status === "cancel_requested") {
    return "warning" as const;
  }
  if (status === "running" || status === "assigned" || status === "pending") {
    return "default" as const;
  }
  return "success" as const;
}

function canCancelTask(status: TaskStatus, cancelRequested: boolean) {
  if (cancelRequested) {
    return false;
  }

  return status === "pending" || status === "assigned" || status === "running";
}

function canRetryTask(status: TaskStatus) {
  return status === "failed" || status === "canceled";
}

function canOpenChanges(status: TaskStatus) {
  return status === "succeeded";
}

function canOpenLogs(status: TaskStatus) {
  return status === "failed" || status === "succeeded" || status === "canceled";
}

function buildRetryPrompt(turn: (typeof turnCards.value)[number]) {
  return [
    "Retry the previous request in this project.",
    `Original request: ${turn.prompt}`,
    "If the previous attempt failed, explain the failure briefly, then continue with the fix."
  ].join("\n\n");
}

function buildExplainPrompt(turn: (typeof turnCards.value)[number]) {
  return [
    "Explain the result of the previous task in this project.",
    `Original request: ${turn.prompt}`,
    "Summarize what changed, what commands or tools were used, and any remaining risk."
  ].join("\n\n");
}
</script>

<template>
  <div class="grid gap-5 xl:grid-cols-[280px_minmax(0,1fr)]">
    <aside class="space-y-3">
      <div class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <div class="flex items-center justify-between">
          <h3 class="text-sm font-semibold uppercase tracking-[0.18em] text-muted-foreground">{{ t("conversation.topics") }}</h3>
          <StatusBadge>{{ conversations.length }}</StatusBadge>
        </div>
        <div class="mt-4 space-y-2">
          <div
            v-if="!conversations.length"
            class="rounded-2xl border border-dashed border-border bg-background/65 px-4 py-5 text-sm text-muted-foreground"
          >
            <p class="font-medium text-foreground">{{ t("conversation.emptyTitle") }}</p>
            <p class="mt-2">
              {{ t("conversation.emptySummary", { project: projectTitle || t("workspace.title") }) }}
            </p>
          </div>
          <button
            v-for="conversation in conversations"
            :key="conversation.id"
            class="w-full rounded-2xl border px-3 py-3 text-left"
            :class="
              conversation.id === activeConversationId
                ? 'border-primary bg-primary/10'
                : 'border-border bg-background/65'
            "
            @click="emit('selectConversation', conversation.id)"
          >
            <p class="truncate text-sm font-medium text-foreground">{{ conversation.title }}</p>
            <p class="mt-1 text-xs text-muted-foreground">
              {{ formatDateTime(conversation.updatedAtEpochMs) }}
            </p>
          </button>
        </div>
      </div>
    </aside>

    <section class="space-y-4">
      <div
        class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
      >
        <div class="space-y-4">
          <article
            v-if="!turnCards.length"
            class="rounded-[1.35rem] border border-dashed border-border bg-background/75 p-5"
          >
            <p class="text-sm font-medium text-foreground">{{ t("conversation.firstTurnTitle") }}</p>
            <p class="mt-2 text-sm text-muted-foreground">
              {{ t("conversation.firstTurnSummary", { project: projectTitle || t("workspace.title") }) }}
            </p>
          </article>
          <article
            v-for="turn in turnCards"
            :key="turn.id"
            class="space-y-3 rounded-[1.35rem] border border-border bg-background/75 p-4"
          >
            <div class="ml-auto max-w-[85%] rounded-[1.15rem] bg-primary px-4 py-3 text-primary-foreground">
              <p class="whitespace-pre-wrap text-sm">{{ turn.prompt }}</p>
              <p class="mt-3 text-[11px] text-primary-foreground/80">
                {{ formatDateTime(turn.task.createdAtEpochMs) }} ·
                {{ t(`conversation.executionModeMeta.${turn.task.executionMode}`) }}
              </p>
            </div>

            <div class="rounded-[1.15rem] border border-border bg-white/80 p-4 dark:bg-slate-950/40">
              <div class="flex flex-wrap items-center gap-2">
                <StatusBadge :tone="taskTone(turn.task.status)">
                  {{ t(`conversation.statusLabel.${turn.task.status}`) }}
                </StatusBadge>
                <StatusBadge tone="muted">{{ turn.task.provider }}</StatusBadge>
                <StatusBadge v-if="turn.task.model" tone="muted">{{ turn.task.model }}</StatusBadge>
                <button
                  v-if="canCancelTask(turn.task.status, turn.task.cancelRequested)"
                  class="rounded-full border border-amber-500/30 bg-amber-500/10 px-3 py-1 text-xs font-medium text-amber-700 dark:text-amber-300"
                  @click="emit('cancelTask', turn.task.id)"
                >
                  {{ t("conversation.stopTask") }}
                </button>
                <button
                  v-if="canRetryTask(turn.task.status)"
                  class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                  @click="emit('sendPrompt', buildRetryPrompt(turn), turn.task.model || undefined, turn.task.executionMode)"
                >
                  {{ t("conversation.retryTask") }}
                </button>
                <button
                  v-if="turn.task.status !== 'running' && turn.task.status !== 'pending' && turn.task.status !== 'assigned'"
                  class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                  @click="emit('sendPrompt', buildExplainPrompt(turn), turn.task.model || undefined, 'read_only')"
                >
                  {{ t("conversation.explainResult") }}
                </button>
                <button
                  v-if="canOpenChanges(turn.task.status)"
                  class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                  @click="emit('openTab', 'changes')"
                >
                  {{ t("conversation.viewChanges") }}
                </button>
                <button
                  v-if="canOpenLogs(turn.task.status)"
                  class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                  @click="emit('openTab', 'logs')"
                >
                  {{ t("conversation.viewLogs") }}
                </button>
              </div>

              <p class="mt-3 whitespace-pre-wrap text-sm text-foreground">
                {{ turn.summary }}
              </p>

              <p class="mt-3 text-xs text-muted-foreground">
                {{ turn.eventSummary }}
              </p>

              <div class="mt-3 rounded-xl border border-dashed border-border bg-background/60 px-3 py-2">
                <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("conversation.policyTitle") }}
                </p>
                <p class="mt-1 text-xs text-muted-foreground">
                  {{ turn.policySummary }}
                </p>
              </div>

              <div v-if="turn.eventLines.length" class="mt-4 space-y-2">
                <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                  {{ t("conversation.eventStreamTitle") }}
                </p>
                <div
                  v-for="event in turn.eventLines"
                  :key="`${turn.id}-${event.seq}`"
                  class="rounded-xl border px-3 py-2 text-xs"
                  :class="
                    event.kind === 'provider_stderr'
                      ? 'border-rose-500/20 bg-rose-500/8 text-rose-700 dark:text-rose-300'
                      : event.kind === 'tool_call' || event.kind === 'tool_output'
                        ? 'border-sky-500/20 bg-sky-500/8'
                        : 'border-border bg-background'
                  "
                >
                  <p class="font-medium">{{ event.kind }}</p>
                  <p class="mt-1 whitespace-pre-wrap">{{ event.message }}</p>
                </div>
              </div>

              <details class="mt-4 rounded-xl border border-dashed border-border bg-background/60 px-3 py-2">
                <summary class="cursor-pointer text-xs font-medium text-muted-foreground">
                  {{ t("conversation.rawEventsTitle") }}
                </summary>
                <div class="mt-3 space-y-2">
                  <div
                    v-for="event in turn.rawLines"
                    :key="`${turn.id}-raw-${event.seq}`"
                    class="rounded-lg border border-border bg-background px-3 py-2 text-xs"
                  >
                    <p class="font-medium">{{ event.kind }}</p>
                    <p class="mt-1 whitespace-pre-wrap">{{ event.message }}</p>
                  </div>
                </div>
              </details>
            </div>
          </article>
        </div>
      </div>

      <div
        v-if="detail?.pendingInputRequest"
        class="rounded-[1.5rem] border border-amber-500/30 bg-amber-500/8 p-4"
      >
        <div class="flex items-center gap-2">
          <StatusBadge tone="warning">{{ t("conversation.waitingInput") }}</StatusBadge>
          <p class="text-sm font-medium">{{ detail.pendingInputRequest.prompt }}</p>
        </div>
        <div class="mt-4 flex flex-wrap gap-2">
          <button
            v-for="option in detail.pendingInputRequest.options"
            :key="option.id"
            class="rounded-full border border-amber-500/30 bg-background px-3 py-2 text-sm"
            @click="emit('respondInput', option.id)"
          >
            {{ option.label }}
          </button>
        </div>
        <div v-if="detail.pendingInputRequest.allowCustomInput" class="mt-4 space-y-2">
          <textarea
            v-model="customReply"
            rows="3"
            class="w-full rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :placeholder="detail.pendingInputRequest.customInputPlaceholder || t('conversation.replyPlaceholder')"
          />
          <button class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground" @click="submitCustomReply">
            {{ t("conversation.sendCustomReply") }}
          </button>
        </div>
      </div>

      <div
        class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
      >
        <div class="grid gap-3 xl:grid-cols-[220px_180px_minmax(0,1fr)_auto]">
          <div class="space-y-2 rounded-2xl bg-background/75 px-4 py-3">
            <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("conversation.executionMode.title") }}
            </p>
            <select
              v-model="executionMode"
              class="w-full rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            >
              <option
                v-for="option in executionModeOptions"
                :key="option.value"
                :value="option.value"
              >
                {{ t(option.labelKey) }}
              </option>
            </select>
            <p class="text-xs text-muted-foreground">{{ t(selectedExecutionMode.summaryKey) }}</p>
            <div class="rounded-xl border border-dashed border-border bg-background/60 px-3 py-2">
              <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
                {{ t("conversation.policyTitle") }}
              </p>
              <p class="mt-1 text-xs text-muted-foreground">{{ composerPolicySummary }}</p>
            </div>
          </div>
          <input
            v-model="model"
            type="text"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :placeholder="t('conversation.optionalModel')"
          />
          <textarea
            v-model="prompt"
            rows="3"
            class="min-h-28 rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :placeholder="t('conversation.promptPlaceholder')"
          />
          <button class="rounded-full bg-primary px-5 py-3 text-sm font-medium text-primary-foreground" @click="submitPrompt">
            {{ pendingConfirmation ? t("conversation.confirmSend") : t("conversation.send") }}
          </button>
        </div>
        <div
          v-if="pendingConfirmation"
          class="mt-4 rounded-2xl border border-amber-500/30 bg-amber-500/8 p-4 text-sm"
        >
          <div class="flex items-center gap-2">
            <StatusBadge tone="warning">{{ t("conversation.sensitiveConfirmTitle") }}</StatusBadge>
            <p class="font-medium text-foreground">{{ t("conversation.sensitiveConfirmSummary") }}</p>
          </div>
          <p class="mt-3 text-muted-foreground">{{ t("conversation.sensitiveConfirmDetail") }}</p>
          <div class="mt-4 flex flex-wrap gap-2">
            <button
              class="rounded-full bg-primary px-4 py-2 font-medium text-primary-foreground"
              @click="submitPrompt"
            >
              {{ t("conversation.confirmSend") }}
            </button>
            <button
              class="rounded-full border border-border px-4 py-2"
              @click="pendingConfirmation = false"
            >
              {{ t("conversation.cancelConfirm") }}
            </button>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>
