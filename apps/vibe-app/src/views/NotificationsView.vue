<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatDateTime, formatRelativeTime } from "@/lib/format";
import { describeNotificationPreference } from "@/lib/policy";
import { buildProjectRouteParam } from "@/lib/projects";
import { useAppStore } from "@/stores/app";

const store = useAppStore();
const router = useRouter();
const { t } = useI18n();
const filter = ref<"all" | "unread" | "failed" | "waiting_input" | "completed">("all");
const preferenceProjects = computed(() => store.projectSummaries);
const preferenceProjectRows = computed(() =>
  preferenceProjects.value.map((project) => {
    const overridePreference = store.notificationPreferenceOverrideForProject(
      project.deviceId,
      project.cwd
    );
    return {
      project,
      preference: describeNotificationPreference(
        store.defaultNotificationPreference,
        overridePreference
      )
    };
  })
);
const visibleNotifications = computed(() =>
  store.notificationItems.filter((item) => {
    if (filter.value === "all") {
      return true;
    }
    if (filter.value === "unread") {
      return !item.seen;
    }
    return item.kind === filter.value;
  })
);
const unreadNotifications = computed(() => visibleNotifications.value.filter((item) => !item.seen));
const recentNotifications = computed(() => visibleNotifications.value.filter((item) => item.seen));

function openNotification(
  taskId: string,
  deviceId: string,
  cwd: string | null,
  conversationId: string | null,
  tab: "conversation" | "changes" | "logs"
) {
  store.markNotificationSeen(taskId);
  store.markProjectVisited(deviceId, cwd);
  void router.push({
    name: "project-workspace",
    params: {
      deviceId,
      projectPath: buildProjectRouteParam(cwd)
    },
    query: {
      ...(conversationId ? { conversationId } : {}),
      tab
    }
  });
}

function itemTone(kind: "waiting_input" | "failed" | "completed") {
  if (kind === "failed") {
    return "danger" as const;
  }
  if (kind === "waiting_input") {
    return "warning" as const;
  }
  return "success" as const;
}
</script>

<template>
  <section class="space-y-5">
    <div class="rounded-[1.8rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
      <StatusBadge>{{ t("notifications.badge") }}</StatusBadge>
      <h2 class="mt-3 text-xl font-semibold">{{ t("notifications.title") }}</h2>
      <p class="mt-2 max-w-2xl text-sm text-muted-foreground">
        {{ t("notifications.summary") }}
      </p>
      <div class="mt-4 flex flex-wrap gap-2">
        <StatusBadge tone="muted">{{ t("notifications.unreadCount", { count: store.unreadNotificationCount }) }}</StatusBadge>
        <StatusBadge tone="muted">{{ t("notifications.visibleCount", { count: visibleNotifications.length }) }}</StatusBadge>
      </div>
    </div>

    <section
      class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
    >
      <div class="flex items-center justify-between gap-3">
        <div>
          <h3 class="text-lg font-semibold">{{ t("notifications.preferencesTitle") }}</h3>
          <p class="mt-2 text-sm text-muted-foreground">{{ t("notifications.preferencesSummary") }}</p>
        </div>
        <StatusBadge tone="muted">{{ preferenceProjects.length }}</StatusBadge>
      </div>

      <div class="mt-4 rounded-2xl border border-border bg-background/75 px-4 py-4">
        <p class="text-sm font-semibold text-foreground">{{ t("notifications.defaultPreferenceTitle") }}</p>
        <p class="mt-1 text-sm text-muted-foreground">{{ t("notifications.defaultPreferenceSummary") }}</p>
        <div class="mt-3 flex flex-wrap gap-2">
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

      <div v-if="preferenceProjectRows.length" class="mt-4 space-y-3">
        <article
          v-for="row in preferenceProjectRows"
          :key="row.project.key"
          class="rounded-2xl border border-border bg-background/75 px-4 py-4"
        >
          <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
            <div>
              <p class="text-sm font-semibold text-foreground">{{ row.project.title }}</p>
              <p class="mt-1 text-xs text-muted-foreground">{{ row.project.deviceName }} · {{ row.project.pathLabel }}</p>
              <p class="mt-1 text-xs text-muted-foreground">
                {{
                  row.preference.inherited
                    ? t("notifications.preferenceInherited", {
                        value: t(
                          row.preference.effectivePreference === "all"
                            ? "notifications.preferenceAll"
                            : "notifications.preferenceImportant"
                        )
                      })
                    : t("notifications.preferenceOverride", {
                        value: t(
                          row.preference.effectivePreference === "all"
                            ? "notifications.preferenceAll"
                            : "notifications.preferenceImportant"
                        )
                      })
                }}
              </p>
            </div>
            <div class="flex flex-wrap gap-2">
              <button
                class="rounded-full border px-4 py-2 text-sm"
                :class="
                  store.notificationPreferenceForProject(row.project.deviceId, row.project.cwd) === 'important'
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-background'
                "
                @click="store.setNotificationPreference(row.project.deviceId, row.project.cwd, 'important')"
              >
                {{ t("notifications.preferenceImportant") }}
              </button>
              <button
                class="rounded-full border px-4 py-2 text-sm"
                :class="
                  store.notificationPreferenceForProject(row.project.deviceId, row.project.cwd) === 'all'
                    ? 'border-primary bg-primary text-primary-foreground'
                    : 'border-border bg-background'
                "
                @click="store.setNotificationPreference(row.project.deviceId, row.project.cwd, 'all')"
              >
                {{ t("notifications.preferenceAll") }}
              </button>
              <button
                class="rounded-full border border-border bg-background px-4 py-2 text-sm"
                @click="store.clearNotificationPreference(row.project.deviceId, row.project.cwd)"
              >
                {{ t("notifications.preferenceReset") }}
              </button>
            </div>
          </div>
        </article>
      </div>
      <p v-else class="mt-4 text-sm text-muted-foreground">{{ t("notifications.preferencesEmpty") }}</p>
    </section>

    <section class="rounded-[1.6rem] border border-white/55 bg-white/80 p-5 backdrop-blur dark:border-white/10 dark:bg-slate-950/55">
      <div class="flex flex-wrap gap-2">
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'all' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'all'"
        >
          {{ t("common.all") }}
        </button>
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'unread' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'unread'"
        >
          {{ t("notifications.unread") }}
        </button>
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'failed' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'failed'"
        >
          {{ t("projectCard.failed") }}
        </button>
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'waiting_input' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'waiting_input'"
        >
          {{ t("projectCard.waiting") }}
        </button>
        <button
          class="rounded-full border px-4 py-2 text-sm"
          :class="filter === 'completed' ? 'border-primary bg-primary text-primary-foreground' : 'border-border bg-background/70'"
          @click="filter = 'completed'"
        >
          {{ t("notifications.completed") }}
        </button>
      </div>
    </section>

    <div
      v-if="!visibleNotifications.length"
      class="rounded-[1.6rem] border border-dashed border-border bg-background/65 p-6 text-sm text-muted-foreground"
    >
      {{ t("notifications.empty") }}
    </div>

    <section v-if="unreadNotifications.length" class="space-y-3">
      <div class="flex items-center gap-2">
        <StatusBadge>{{ t("notifications.unread") }}</StatusBadge>
        <p class="text-sm text-muted-foreground">{{ t("notifications.unreadSummary") }}</p>
      </div>
      <button
        v-for="item in unreadNotifications"
        :key="item.id"
        class="block w-full rounded-[1.5rem] border border-white/55 bg-white/80 p-4 text-left backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
        @click="openNotification(item.taskId, item.deviceId, item.cwd, item.conversationId, item.targetTab)"
      >
        <div class="flex items-start justify-between gap-4">
          <div class="space-y-2">
            <div class="flex items-center gap-2">
              <StatusBadge :tone="itemTone(item.kind)">
                {{
                  item.kind === "failed"
                    ? t("projectCard.failed")
                    : item.kind === "waiting_input"
                      ? t("projectCard.waiting")
                      : t("notifications.completed")
                }}
              </StatusBadge>
              <StatusBadge tone="muted">{{ t("notifications.newBadge") }}</StatusBadge>
              <span class="text-xs text-muted-foreground">{{ formatRelativeTime(item.timestampEpochMs) }}</span>
            </div>
            <p class="text-sm font-semibold text-foreground">{{ item.title }}</p>
            <p class="text-sm text-muted-foreground">{{ item.summary }}</p>
            <div class="flex flex-wrap gap-2 pt-1">
              <button
                class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                @click.stop="openNotification(item.taskId, item.deviceId, item.cwd, item.conversationId, 'conversation')"
              >
                {{ t("notifications.actions.conversation") }}
              </button>
              <button
                class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                @click.stop="openNotification(item.taskId, item.deviceId, item.cwd, item.conversationId, 'changes')"
              >
                {{ t("notifications.actions.changes") }}
              </button>
              <button
                class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                @click.stop="openNotification(item.taskId, item.deviceId, item.cwd, item.conversationId, 'logs')"
              >
                {{ t("notifications.actions.logs") }}
              </button>
            </div>
          </div>
          <div class="text-right text-xs text-muted-foreground">
            <p>{{ formatDateTime(item.timestampEpochMs) }}</p>
            <p class="mt-2 font-medium text-primary">{{ t(`notifications.actions.${item.targetTab}`) }}</p>
          </div>
        </div>
      </button>
    </section>

    <section v-if="recentNotifications.length" class="space-y-3">
      <div class="flex items-center gap-2">
        <StatusBadge tone="muted">{{ t("notifications.recent") }}</StatusBadge>
        <p class="text-sm text-muted-foreground">{{ t("notifications.recentSummary") }}</p>
      </div>
      <button
        v-for="item in recentNotifications"
        :key="item.id"
        class="block w-full rounded-[1.5rem] border border-white/55 bg-white/80 p-4 text-left backdrop-blur dark:border-white/10 dark:bg-slate-950/55"
        @click="openNotification(item.taskId, item.deviceId, item.cwd, item.conversationId, item.targetTab)"
      >
        <div class="flex items-start justify-between gap-4">
          <div class="space-y-2">
            <div class="flex items-center gap-2">
              <StatusBadge :tone="itemTone(item.kind)">
                {{
                  item.kind === "failed"
                    ? t("projectCard.failed")
                    : item.kind === "waiting_input"
                      ? t("projectCard.waiting")
                      : t("notifications.completed")
                }}
              </StatusBadge>
              <span class="text-xs text-muted-foreground">{{ formatRelativeTime(item.timestampEpochMs) }}</span>
            </div>
            <p class="text-sm font-semibold text-foreground">{{ item.title }}</p>
            <p class="text-sm text-muted-foreground">{{ item.summary }}</p>
            <div class="flex flex-wrap gap-2 pt-1">
              <button
                class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                @click.stop="openNotification(item.taskId, item.deviceId, item.cwd, item.conversationId, 'conversation')"
              >
                {{ t("notifications.actions.conversation") }}
              </button>
              <button
                class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                @click.stop="openNotification(item.taskId, item.deviceId, item.cwd, item.conversationId, 'changes')"
              >
                {{ t("notifications.actions.changes") }}
              </button>
              <button
                class="rounded-full border border-border bg-background px-3 py-1 text-xs font-medium text-foreground"
                @click.stop="openNotification(item.taskId, item.deviceId, item.cwd, item.conversationId, 'logs')"
              >
                {{ t("notifications.actions.logs") }}
              </button>
            </div>
          </div>
          <div class="text-right text-xs text-muted-foreground">
            <p>{{ formatDateTime(item.timestampEpochMs) }}</p>
            <p class="mt-2 font-medium text-primary">{{ t(`notifications.actions.${item.targetTab}`) }}</p>
          </div>
        </div>
      </button>
    </section>
  </section>
</template>
