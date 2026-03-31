<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatDateTime } from "@/lib/format";
import { useAppStore } from "@/stores/app";
import type {
  ConversationDetailResponse,
  ProviderKind,
  TaskDetailResponse,
  TaskEvent,
  TaskExecutionMode,
  TaskStatus
} from "@/types";

const props = defineProps<{
  detail: ConversationDetailResponse | null;
  projectProviders?: ProviderKind[];
  projectTitle?: string | null;
  isDraftConversation?: boolean;
  selectedModel?: string;
  modelOptions?: { label: string; value: string }[];
  canCompose?: boolean;
  emptySummary?: string;
}>();

const emit = defineEmits<{
  sendPrompt: [prompt: string, model?: string, executionMode?: TaskExecutionMode];
  respondInput: [optionId?: string, text?: string];
  cancelTask: [taskId: string];
  openTab: [tab: "changes" | "files"];
  "update:selectedModel": [value: string];
}>();

const { t } = useI18n();
const store = useAppStore();
const prompt = ref("");
const model = ref("");
const executionMode = ref<TaskExecutionMode>(store.defaultExecutionMode);
const customReply = ref("");

watch(
  () => props.selectedModel,
  (value) => {
    model.value = value ?? "";
  },
  { immediate: true }
);

watch(model, (value) => {
  emit("update:selectedModel", value);
});

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
const canCompose = computed(() => props.canCompose !== false);

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
      eventSummary: buildEventSummary(nonAssistantEvents),
      eventLines: nonAssistantEvents.slice(-6),
      rawLines: taskDetail.events
    };
  })
);

const hasTurns = computed(() => turnCards.value.length > 0);

function submitPrompt() {
  if (!canCompose.value || !prompt.value.trim()) {
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
}

function submitCustomReply() {
  if (!canCompose.value || !customReply.value.trim()) {
    return;
  }

  emit("respondInput", undefined, customReply.value.trim());
  customReply.value = "";
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
  <section class="flex min-h-[calc(100vh-9rem)] flex-col">
    <div class="flex-1 space-y-6 px-1 pb-6 pt-2">
      <article
        v-if="!hasTurns"
        class="mx-auto max-w-2xl rounded-[2rem] border border-dashed border-border bg-background/70 px-6 py-10 text-center"
      >
          <p class="text-sm font-medium text-foreground">
          {{
            isDraftConversation
              ? t("conversation.newChatTitle")
              : t("conversation.firstTurnTitle")
          }}
        </p>
        <p class="mt-3 text-sm text-muted-foreground">
          {{ emptySummary || t("conversation.firstTurnSummary", { project: projectTitle || t("workspace.title") }) }}
        </p>
      </article>

      <article
        v-for="turn in turnCards"
        :key="turn.id"
        class="mx-auto max-w-3xl space-y-4"
      >
        <div class="flex justify-end">
          <div class="max-w-[88%] rounded-[1.6rem] bg-primary px-4 py-3 text-primary-foreground shadow-sm">
            <p class="whitespace-pre-wrap text-sm leading-6">{{ turn.prompt }}</p>
            <p class="mt-2 text-[11px] text-primary-foreground/75">
              {{ formatDateTime(turn.task.createdAtEpochMs) }} ·
              {{ t(`conversation.executionModeMeta.${turn.task.executionMode}`) }}
            </p>
          </div>
        </div>

        <div class="max-w-[92%] rounded-[1.6rem] border border-border/70 bg-white/90 px-4 py-4 shadow-sm dark:bg-slate-950/50">
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
          </div>

          <p class="mt-4 whitespace-pre-wrap text-sm leading-7 text-foreground">
            {{ turn.summary }}
          </p>

          <div v-if="turn.eventLines.length" class="mt-4 space-y-2">
            <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              {{ turn.eventSummary }}
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
              <p class="mt-1 whitespace-pre-wrap leading-5">{{ event.message }}</p>
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
                <p class="mt-1 whitespace-pre-wrap leading-5">{{ event.message }}</p>
              </div>
            </div>
          </details>
        </div>
      </article>
    </div>

    <div class="sticky bottom-0 mt-auto border-t border-border/60 bg-[linear-gradient(180deg,rgba(244,244,240,0),rgba(244,244,240,0.96)_20%,rgba(244,244,240,0.98)_100%)] px-1 pb-2 pt-4 dark:bg-[linear-gradient(180deg,rgba(9,9,11,0),rgba(9,9,11,0.94)_20%,rgba(9,9,11,0.98)_100%)]">
      <div
        v-if="detail?.pendingInputRequest"
        class="mx-auto mb-4 max-w-3xl rounded-[1.4rem] border border-amber-500/30 bg-amber-500/8 p-4"
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
            :disabled="!canCompose"
            :placeholder="detail.pendingInputRequest.customInputPlaceholder || t('conversation.replyPlaceholder')"
          />
          <button class="rounded-full bg-primary px-4 py-2 text-sm font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50" :disabled="!canCompose" @click="submitCustomReply">
            {{ t("conversation.sendCustomReply") }}
          </button>
        </div>
      </div>

      <div class="mx-auto max-w-3xl rounded-[1.8rem] border border-white/60 bg-white/90 p-3 shadow-[0_24px_60px_-35px_rgba(15,23,42,0.55)] backdrop-blur dark:border-white/10 dark:bg-slate-950/75">
        <div class="grid gap-3 md:grid-cols-[160px_180px_minmax(0,1fr)_auto]">
          <div class="space-y-2 rounded-2xl bg-background/75 px-3 py-3">
            <p class="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("conversation.executionMode.title") }}
            </p>
            <select
              v-model="executionMode"
              class="w-full rounded-2xl border border-border bg-background px-3 py-2.5 text-sm"
              :disabled="!canCompose"
            >
              <option
                v-for="option in executionModeOptions"
                :key="option.value"
                :value="option.value"
              >
                {{ t(option.labelKey) }}
              </option>
            </select>
            <p class="text-[11px] text-muted-foreground">{{ t(selectedExecutionMode.summaryKey) }}</p>
          </div>
          <select
            v-if="modelOptions?.length"
            v-model="model"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :disabled="!canCompose"
          >
            <option value="">{{ t("conversation.optionalModel") }}</option>
            <option
              v-for="option in modelOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </select>
          <input
            v-else
            v-model="model"
            type="text"
            class="rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :disabled="!canCompose"
            :placeholder="t('conversation.optionalModel')"
          />
          <textarea
            v-model="prompt"
            rows="3"
            class="min-h-28 rounded-2xl border border-border bg-background px-4 py-3 text-sm"
            :disabled="!canCompose"
            :placeholder="canCompose ? t('conversation.promptPlaceholder') : t('conversation.disabledPlaceholder')"
          />
          <button class="rounded-[1.4rem] bg-primary px-5 py-3 text-sm font-medium text-primary-foreground disabled:cursor-not-allowed disabled:opacity-50" :disabled="!canCompose" @click="submitPrompt">
            {{ t("conversation.send") }}
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
