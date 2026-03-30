<script setup lang="ts">
import { computed, ref } from "vue";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatDateTime } from "@/lib/format";
import type { AuditRecord, ConversationDetailResponse, TaskEvent } from "@/types";

const props = defineProps<{
  detail: ConversationDetailResponse | null;
  auditEvents: AuditRecord[];
}>();

const { t } = useI18n();
const filter = ref<"all" | "errors" | "tools" | "provider">("all");

function auditOutcomeTone(record: AuditRecord): "success" | "warning" | "danger" {
  if (record.outcome === "failed") {
    return "danger";
  }
  if (record.outcome === "rejected") {
    return "warning";
  }
  return "success";
}

const errorSummaries = computed(() =>
  (props.detail?.tasks ?? [])
    .filter(
      (task) =>
        task.task.status === "failed" ||
        task.events.some((event) => event.kind === "provider_stderr")
    )
    .map((task) => ({
      id: task.task.id,
      title: task.task.title || task.task.prompt,
      summary:
        task.task.error ||
        task.events.find((event) => event.kind === "provider_stderr")?.message ||
        t("logs.errorFallback"),
      createdAtEpochMs: task.task.createdAtEpochMs
    }))
);

const auditSummaries = computed(() =>
  props.auditEvents.map((record) => ({
    ...record,
    actionLabel: t(`logs.audit.actions.${record.action}`),
    outcomeTone: auditOutcomeTone(record)
  }))
);

function visibleEvents(events: TaskEvent[]) {
  if (filter.value === "errors") {
    return events.filter((event) => event.kind === "provider_stderr");
  }
  if (filter.value === "tools") {
    return events.filter((event) => event.kind === "tool_call" || event.kind === "tool_output");
  }
  if (filter.value === "provider") {
    return events.filter(
      (event) => event.kind === "provider_stdout" || event.kind === "provider_stderr"
    );
  }
  return events;
}
</script>

<template>
  <div class="space-y-5">
    <section
      v-if="auditSummaries.length"
      class="rounded-[1.5rem] border border-sky-500/20 bg-sky-500/8 p-4"
    >
      <div class="flex items-center gap-2">
        <StatusBadge>{{ t("logs.audit.title") }}</StatusBadge>
        <p class="text-sm text-foreground">{{ t("logs.audit.summary") }}</p>
      </div>
      <div class="mt-4 space-y-3">
        <article
          v-for="item in auditSummaries"
          :key="item.id"
          class="rounded-2xl border border-sky-500/20 bg-background/80 px-4 py-3"
        >
          <div class="flex items-center justify-between gap-3">
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold text-foreground">{{ item.actionLabel }}</p>
              <p class="mt-1 truncate text-xs text-muted-foreground">
                {{ item.resourceKind }} · {{ item.resourceId }}
              </p>
            </div>
            <StatusBadge :tone="item.outcomeTone">
              {{ t(`logs.audit.outcomes.${item.outcome}`) }}
            </StatusBadge>
          </div>
          <p v-if="item.message" class="mt-2 whitespace-pre-wrap text-sm text-foreground">
            {{ item.message }}
          </p>
          <p class="mt-2 text-xs text-muted-foreground">
            {{ formatDateTime(item.timestampEpochMs) }}
          </p>
        </article>
      </div>
    </section>

    <section
      v-if="errorSummaries.length"
      class="rounded-[1.5rem] border border-rose-500/20 bg-rose-500/8 p-4"
    >
      <div class="flex items-center gap-2">
        <StatusBadge tone="danger">{{ t("logs.errorSummaryTitle") }}</StatusBadge>
        <p class="text-sm text-foreground">{{ t("logs.errorSummaryBody") }}</p>
      </div>
      <div class="mt-4 space-y-3">
        <article
          v-for="item in errorSummaries"
          :key="item.id"
          class="rounded-2xl border border-rose-500/20 bg-background/80 px-4 py-3"
        >
          <div class="flex items-center justify-between gap-3">
            <p class="text-sm font-semibold text-foreground">{{ item.title }}</p>
            <span class="text-xs text-muted-foreground">{{ formatDateTime(item.createdAtEpochMs) }}</span>
          </div>
          <p class="mt-2 whitespace-pre-wrap text-sm text-rose-700 dark:text-rose-300">{{ item.summary }}</p>
        </article>
      </div>
    </section>

    <div
      class="rounded-[1.5rem] border border-white/55 bg-white/80 p-4 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
    >
      <div class="flex flex-wrap gap-2">
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'all' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'all'"
        >
          {{ t("logs.filters.all") }}
        </button>
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'errors' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'errors'"
        >
          {{ t("logs.filters.errors") }}
        </button>
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'tools' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'tools'"
        >
          {{ t("logs.filters.tools") }}
        </button>
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'provider' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'provider'"
        >
          {{ t("logs.filters.provider") }}
        </button>
      </div>

      <div class="mt-5 space-y-3">
        <article
          v-for="task in detail?.tasks ?? []"
          :key="task.task.id"
          class="rounded-2xl border border-border bg-background/75 p-4"
        >
          <div class="flex items-center justify-between gap-3">
            <div>
              <p class="text-sm font-semibold">{{ task.task.title || task.task.prompt }}</p>
              <p class="text-xs text-muted-foreground">{{ formatDateTime(task.task.createdAtEpochMs) }}</p>
            </div>
            <StatusBadge
              :tone="
                task.task.status === 'failed'
                  ? 'danger'
                  : task.task.status === 'waiting_input'
                    ? 'warning'
                    : task.task.status === 'running'
                      ? 'default'
                      : 'success'
              "
            >
              {{ task.task.status }}
            </StatusBadge>
          </div>
          <div class="mt-4 space-y-2">
            <div
              v-for="event in visibleEvents(task.events)"
              :key="`${task.task.id}-${event.seq}`"
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
            <p
              v-if="!visibleEvents(task.events).length"
              class="rounded-xl border border-dashed border-border bg-background px-3 py-3 text-xs text-muted-foreground"
            >
              {{ t("logs.noFilteredEvents") }}
            </p>
          </div>
        </article>
        <p v-if="!(detail?.tasks.length)" class="text-sm text-muted-foreground">{{ t("logs.empty") }}</p>
      </div>
    </div>
  </div>
</template>
