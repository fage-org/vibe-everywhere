<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { fetchAuditEvents } from "@/lib/api";
import { getSupportedLocales, setAppLocale } from "@/lib/i18n";
import { buildSettingsPolicyRows } from "@/lib/policy";
import { getSupportedThemeModes, setThemeMode, useTheme } from "@/lib/theme";
import { useAppStore } from "@/stores/app";
import { formatDateTime } from "@/lib/format";
import type { AuditRecord, ExecutionProtocol, ProviderKind, TaskExecutionMode } from "@/types";

const store = useAppStore();
const { themeMode } = useTheme();
const { t } = useI18n();
const locales = getSupportedLocales();
const themeModes = getSupportedThemeModes();
const currentLocale = computed(() => document.documentElement.lang || "en");
const globalAuditEvents = ref<AuditRecord[]>([]);
const auditFilter = ref<"all" | "task" | "shell_preview">("all");
const providerCards = computed(() => {
  const summaries = new Map<
    ProviderKind,
    { kind: ProviderKind; availableCount: number; onlineCount: number; protocol: ExecutionProtocol }
  >();

  for (const device of store.devices) {
    for (const provider of device.providers) {
      const existing = summaries.get(provider.kind) ?? {
        kind: provider.kind,
        availableCount: 0,
        onlineCount: 0,
        protocol: provider.executionProtocol
      };

      if (provider.available) {
        existing.availableCount += 1;
        if (device.online) {
          existing.onlineCount += 1;
        }
      }

      existing.protocol = provider.executionProtocol;
      summaries.set(provider.kind, existing);
    }
  }

  return [...summaries.values()];
});
const policyRows = computed(
  () =>
    buildSettingsPolicyRows(store.devices.flatMap((device) => device.providers)).map((row) => ({
      ...row,
      summary: policySummary(row.provider, row.protocol, row.mode)
    }))
);
const auditFacts = computed(() => [
  t("settings.audit.facts.projectLogs"),
  t("settings.audit.facts.taskLifecycle"),
  t("settings.audit.facts.shellPreview"),
  t("settings.audit.facts.secondarySurface")
]);
const filteredAuditEvents = computed(() =>
  globalAuditEvents.value.filter((record) => {
    if (auditFilter.value === "all") {
      return true;
    }
    if (auditFilter.value === "task") {
      return record.resourceKind === "task" || record.resourceKind === "conversation";
    }
    return record.resourceKind === "shell_session" || record.resourceKind === "preview";
  })
);
const executionModeOptions: TaskExecutionMode[] = [
  "read_only",
  "workspace_write",
  "workspace_write_and_test"
];

async function save() {
  await store.saveRelaySettings();
}

async function refreshGlobalAudit() {
  if (!store.relayBaseUrl.trim()) {
    globalAuditEvents.value = [];
    return;
  }

  try {
    globalAuditEvents.value = await fetchAuditEvents(
      store.relayBaseUrl,
      store.relayAccessToken,
      { limit: 50 }
    );
  } catch {
    globalAuditEvents.value = [];
  }
}

function providerLabel(kind: ProviderKind) {
  if (kind === "claude_code") {
    return "Claude";
  }
  if (kind === "open_code") {
    return "OpenCode";
  }
  return "Codex";
}

function protocolLabel(protocol: ExecutionProtocol) {
  return protocol === "acp" ? "ACP" : "CLI";
}

function policySummary(
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

onMounted(async () => {
  await refreshGlobalAudit();
});

watch(
  () => store.lastRefreshEpochMs,
  async () => {
    await refreshGlobalAudit();
  }
);
</script>

<template>
  <section class="space-y-5">
    <div class="rounded-[1.8rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
      <StatusBadge>{{ t("settings.badge") }}</StatusBadge>
      <h2 class="mt-3 text-xl font-semibold">{{ t("settings.title") }}</h2>
      <p class="mt-2 max-w-2xl text-sm text-muted-foreground">
        {{ t("settings.summary") }}
      </p>
    </div>

    <section class="grid gap-5 xl:grid-cols-[1.1fr_0.9fr]">
      <article class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <h3 class="text-lg font-semibold">{{ t("settings.serverTitle") }}</h3>
        <p class="mt-2 text-sm text-muted-foreground">
          {{ t("settings.serverSummary") }}
        </p>
        <div class="mt-5 space-y-4">
          <label class="block space-y-2 text-sm">
            <span class="font-medium">{{ t("settings.relayUrl") }}</span>
            <input
              v-model="store.relayBaseUrlInput"
              type="url"
              class="w-full rounded-2xl border border-border bg-background px-4 py-3"
              placeholder="https://relay.example.com"
            />
          </label>
          <label class="block space-y-2 text-sm">
            <span class="font-medium">{{ t("settings.accessToken") }}</span>
            <input
              v-model="store.relayAccessTokenInput"
              type="password"
              class="w-full rounded-2xl border border-border bg-background px-4 py-3"
              :placeholder="t('settings.accessTokenPlaceholder')"
            />
          </label>
          <button class="rounded-full bg-primary px-4 py-2.5 text-sm font-medium text-primary-foreground" @click="save">
            {{ t("settings.save") }}
          </button>
          <p class="text-xs text-muted-foreground">
            {{ t("settings.currentServer", { value: store.relayBaseUrl || t("settings.notConfigured") }) }}
          </p>
        </div>
      </article>

      <div class="space-y-5">
        <article class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
          <h3 class="text-lg font-semibold">{{ t("settings.language") }}</h3>
          <div class="mt-4 flex flex-wrap gap-2">
            <button
              v-for="locale in locales"
              :key="locale"
              class="rounded-full border px-4 py-2 text-sm"
              :class="
                currentLocale === locale
                  ? 'border-primary bg-primary text-primary-foreground'
                  : 'border-border'
              "
              @click="setAppLocale(locale)"
            >
              {{ locale }}
            </button>
          </div>
        </article>

        <article class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
          <h3 class="text-lg font-semibold">{{ t("settings.theme") }}</h3>
          <div class="mt-4 flex flex-wrap gap-2">
            <button
              v-for="mode in themeModes"
              :key="mode"
              class="rounded-full border px-4 py-2 text-sm capitalize"
              :class="
                themeMode === mode
                  ? 'border-primary bg-primary text-primary-foreground'
                  : 'border-border'
              "
              @click="setThemeMode(mode)"
            >
              {{ mode }}
            </button>
          </div>
        </article>
      </div>
    </section>

    <section class="grid gap-5 xl:grid-cols-[0.92fr_1.08fr]">
      <article class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <div class="flex items-center gap-2">
          <StatusBadge>{{ t("settings.policy.badge") }}</StatusBadge>
          <h3 class="text-lg font-semibold">{{ t("settings.policy.title") }}</h3>
        </div>
        <p class="mt-2 text-sm text-muted-foreground">
          {{ t("settings.policy.summary") }}
        </p>
        <div class="mt-5 space-y-3">
          <article
            v-for="provider in providerCards"
            :key="provider.kind"
            class="rounded-2xl border border-border bg-background/75 px-4 py-3"
          >
            <div class="flex items-center justify-between gap-3">
              <div>
                <p class="text-sm font-semibold text-foreground">{{ providerLabel(provider.kind) }}</p>
                <p class="mt-1 text-xs text-muted-foreground">{{ protocolLabel(provider.protocol) }}</p>
              </div>
              <StatusBadge :tone="provider.onlineCount > 0 ? 'success' : 'muted'">
                {{ t("settings.policy.providerAvailability", { online: provider.onlineCount, available: provider.availableCount }) }}
              </StatusBadge>
            </div>
          </article>
          <p
            v-if="!providerCards.length"
            class="rounded-2xl border border-dashed border-border bg-background/75 px-4 py-3 text-sm text-muted-foreground"
          >
            {{ t("settings.policy.empty") }}
          </p>
        </div>
      </article>

      <article class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <div class="flex items-center gap-2">
          <StatusBadge>{{ t("settings.audit.badge") }}</StatusBadge>
          <h3 class="text-lg font-semibold">{{ t("settings.audit.title") }}</h3>
        </div>
        <p class="mt-2 text-sm text-muted-foreground">
          {{ t("settings.audit.summary") }}
        </p>
        <div class="mt-5 space-y-3">
          <article
            v-for="row in policyRows"
            :key="row.id"
            class="rounded-2xl border border-border bg-background/75 px-4 py-3"
          >
            <div class="flex flex-wrap items-center gap-2">
              <StatusBadge tone="muted">{{ providerLabel(row.provider) }}</StatusBadge>
              <StatusBadge tone="muted">{{ protocolLabel(row.protocol) }}</StatusBadge>
              <StatusBadge tone="muted">{{ t(`conversation.executionModeMeta.${row.mode}`) }}</StatusBadge>
            </div>
            <p class="mt-2 text-sm text-foreground">{{ row.summary }}</p>
          </article>
        </div>
        <div class="mt-5 space-y-2 rounded-2xl border border-dashed border-border bg-background/70 px-4 py-4">
          <p class="text-xs font-semibold uppercase tracking-[0.18em] text-muted-foreground">
            {{ t("settings.audit.coverageTitle") }}
          </p>
          <p
            v-for="fact in auditFacts"
            :key="fact"
            class="text-sm text-muted-foreground"
          >
            {{ fact }}
          </p>
        </div>
      </article>
    </section>

    <section class="grid gap-5 xl:grid-cols-[0.9fr_1.1fr]">
      <article class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <div class="flex items-center gap-2">
          <StatusBadge>{{ t("settings.policy.manageBadge") }}</StatusBadge>
          <h3 class="text-lg font-semibold">{{ t("settings.policy.manageTitle") }}</h3>
        </div>
        <p class="mt-2 text-sm text-muted-foreground">
          {{ t("settings.policy.manageSummary") }}
        </p>

        <div class="mt-5 space-y-5">
          <div class="space-y-2">
            <p class="text-sm font-semibold text-foreground">{{ t("settings.policy.defaultExecutionMode") }}</p>
            <div class="flex flex-wrap gap-2">
              <button
                v-for="mode in executionModeOptions"
                :key="mode"
                class="rounded-full border px-4 py-2 text-sm"
                :class="
                  store.defaultExecutionMode === mode
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-background'
                "
                @click="store.setDefaultExecutionMode(mode)"
              >
                {{ t(`conversation.executionModeMeta.${mode}`) }}
              </button>
            </div>
          </div>

          <div class="space-y-2">
            <p class="text-sm font-semibold text-foreground">{{ t("settings.policy.defaultNotifications") }}</p>
            <div class="flex flex-wrap gap-2">
              <button
                class="rounded-full border px-4 py-2 text-sm"
                :class="
                  store.defaultNotificationPreference === 'important'
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-background'
                "
                @click="store.setDefaultNotificationPreference('important')"
              >
                {{ t("notifications.preferenceImportant") }}
              </button>
              <button
                class="rounded-full border px-4 py-2 text-sm"
                :class="
                  store.defaultNotificationPreference === 'all'
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-background'
                "
                @click="store.setDefaultNotificationPreference('all')"
              >
                {{ t("notifications.preferenceAll") }}
              </button>
            </div>
          </div>

          <div class="space-y-2">
            <p class="text-sm font-semibold text-foreground">{{ t("settings.policy.sensitiveConfirm") }}</p>
            <div class="flex flex-wrap gap-2">
              <button
                class="rounded-full border px-4 py-2 text-sm"
                :class="
                  store.sensitiveConfirmationEnabled
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-background'
                "
                @click="store.setSensitiveConfirmationEnabled(true)"
              >
                {{ t("settings.policy.confirmEnabled") }}
              </button>
              <button
                class="rounded-full border px-4 py-2 text-sm"
                :class="
                  !store.sensitiveConfirmationEnabled
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-background'
                "
                @click="store.setSensitiveConfirmationEnabled(false)"
              >
                {{ t("settings.policy.confirmDisabled") }}
              </button>
            </div>
          </div>
        </div>
      </article>

      <article class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
        <div class="flex items-center gap-2">
          <StatusBadge>{{ t("settings.audit.manageBadge") }}</StatusBadge>
          <h3 class="text-lg font-semibold">{{ t("settings.audit.manageTitle") }}</h3>
        </div>
        <p class="mt-2 text-sm text-muted-foreground">
          {{ t("settings.audit.manageSummary") }}
        </p>

        <div class="mt-4 flex flex-wrap gap-2">
          <button
            class="rounded-full border px-4 py-2 text-sm"
            :class="auditFilter === 'all' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
            @click="auditFilter = 'all'"
          >
            {{ t("common.all") }}
          </button>
          <button
            class="rounded-full border px-4 py-2 text-sm"
            :class="auditFilter === 'task' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
            @click="auditFilter = 'task'"
          >
            {{ t("settings.audit.filters.task") }}
          </button>
          <button
            class="rounded-full border px-4 py-2 text-sm"
            :class="auditFilter === 'shell_preview' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
            @click="auditFilter = 'shell_preview'"
          >
            {{ t("settings.audit.filters.shellPreview") }}
          </button>
        </div>

        <div class="mt-5 space-y-3">
          <article
            v-for="record in filteredAuditEvents"
            :key="record.id"
            class="rounded-2xl border border-border bg-background/75 px-4 py-3"
          >
            <div class="flex items-center justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-semibold text-foreground">
                  {{ t(`logs.audit.actions.${record.action}`) }}
                </p>
                <p class="mt-1 truncate text-xs text-muted-foreground">
                  {{ record.resourceKind }} · {{ record.resourceId }}
                </p>
              </div>
              <StatusBadge tone="muted">
                {{ t(`logs.audit.outcomes.${record.outcome}`) }}
              </StatusBadge>
            </div>
            <p v-if="record.message" class="mt-2 whitespace-pre-wrap text-sm text-foreground">
              {{ record.message }}
            </p>
            <p class="mt-2 text-xs text-muted-foreground">{{ formatDateTime(record.timestampEpochMs) }}</p>
          </article>
          <p
            v-if="!filteredAuditEvents.length"
            class="rounded-2xl border border-dashed border-border bg-background/75 px-4 py-3 text-sm text-muted-foreground"
          >
            {{ t("settings.audit.empty") }}
          </p>
        </div>
      </article>
    </section>
  </section>
</template>
