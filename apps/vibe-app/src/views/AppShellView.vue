<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import { useI18n } from "vue-i18n";
import StatusBadge from "@/components/common/StatusBadge.vue";
import { formatRelativeTime } from "@/lib/format";
import { useAppStore } from "@/stores/app";

const route = useRoute();
const store = useAppStore();
const { t } = useI18n();

const navItems = [
  { name: "home", key: "nav.home" },
  { name: "projects", key: "nav.projects" },
  { name: "notifications", key: "nav.notifications" },
  { name: "settings", key: "nav.settings" }
];

onMounted(async () => {
  if (!store.isBootstrapping && !store.lastRefreshEpochMs) {
    await store.bootstrap();
  }
});

onUnmounted(() => {
  store.stopAutoRefresh();
});
</script>

<template>
  <div class="min-h-screen">
    <div class="mx-auto flex min-h-screen max-w-[1640px] flex-col px-4 pb-24 pt-4 md:px-6 xl:grid xl:grid-cols-[260px_minmax(0,1fr)] xl:gap-6 xl:px-8 xl:pb-8">
      <aside
        class="hidden xl:sticky xl:top-4 xl:flex xl:h-[calc(100vh-2rem)] xl:flex-col xl:rounded-[1.8rem] xl:border xl:border-white/55 xl:bg-white/80 xl:p-5 xl:shadow-[0_22px_60px_-35px_rgba(15,23,42,0.55)] xl:backdrop-blur dark:xl:border-white/10 dark:xl:bg-slate-950/55"
      >
        <div class="space-y-2">
          <div class="flex flex-wrap items-center gap-2">
            <StatusBadge>{{ t("shell.badge") }}</StatusBadge>
            <StatusBadge :tone="store.errorMessage ? 'danger' : store.onlineHostCount ? 'success' : 'warning'">
              {{
                store.errorMessage
                  ? t("shell.serverError")
                  : store.onlineHostCount
                    ? t("shell.hostsOnline", { count: store.onlineHostCount })
                    : t("shell.noHostOnline")
              }}
            </StatusBadge>
          </div>
          <h1 class="text-xl font-semibold">{{ t("shell.title") }}</h1>
          <p class="text-sm text-muted-foreground">
            {{
              store.relayBaseUrl
                ? `${store.activeServerLabel} · ${store.relayBaseUrl}`
                : t("shell.emptyServer")
            }}
          </p>
        </div>

        <nav class="mt-6 space-y-2">
          <RouterLink
            v-for="item in navItems"
            :key="item.name"
            :to="{ name: item.name }"
            class="flex items-center rounded-2xl px-4 py-3 text-sm font-medium transition"
            :class="
              route.name === item.name
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-background/70 hover:text-foreground'
            "
          >
            {{ t(item.key) }}
          </RouterLink>
        </nav>

        <div class="mt-auto grid gap-2 rounded-2xl bg-background/75 p-4 text-xs text-muted-foreground">
          <span>{{ t("shell.runningTasks", { count: store.runningTaskCount }) }}</span>
          <span>{{ t("shell.attention", { count: store.attentionCount }) }}</span>
          <span>{{ t("common.refreshedAt", { value: formatRelativeTime(store.lastRefreshEpochMs) }) }}</span>
        </div>
      </aside>

      <div class="flex min-h-screen flex-col xl:min-h-0">
        <header
          class="sticky top-4 z-10 mb-5 rounded-[1.8rem] border border-white/55 bg-white/80 px-5 py-4 shadow-[0_22px_60px_-35px_rgba(15,23,42,0.55)] backdrop-blur dark:border-white/10 dark:bg-slate-950/55 xl:mb-6"
        >
          <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
            <div class="space-y-2">
              <div class="flex flex-wrap items-center gap-2 xl:hidden">
                <StatusBadge>{{ t("shell.badge") }}</StatusBadge>
                <StatusBadge :tone="store.errorMessage ? 'danger' : store.onlineHostCount ? 'success' : 'warning'">
                  {{
                    store.errorMessage
                      ? t("shell.serverError")
                      : store.onlineHostCount
                        ? t("shell.hostsOnline", { count: store.onlineHostCount })
                        : t("shell.noHostOnline")
                  }}
                </StatusBadge>
              </div>
              <h1 class="text-2xl font-semibold">{{ t("shell.title") }}</h1>
              <p class="max-w-3xl text-sm text-muted-foreground">
                {{
                  store.relayBaseUrl
                    ? `${store.activeServerLabel} · ${store.relayBaseUrl}`
                    : t("shell.emptyServer")
                }}
              </p>
            </div>
            <div class="grid gap-2 text-right text-xs text-muted-foreground xl:hidden">
              <span>{{ t("shell.runningTasks", { count: store.runningTaskCount }) }}</span>
              <span>{{ t("shell.attention", { count: store.attentionCount }) }}</span>
              <span>{{ t("common.refreshedAt", { value: formatRelativeTime(store.lastRefreshEpochMs) }) }}</span>
            </div>
          </div>
        </header>

        <main class="flex-1">
          <RouterView />
        </main>
      </div>

      <nav
        class="fixed inset-x-4 bottom-4 z-20 mx-auto flex max-w-[760px] items-center justify-between rounded-full border border-white/55 bg-white/85 px-3 py-3 shadow-[0_28px_70px_-40px_rgba(15,23,42,0.6)] backdrop-blur dark:border-white/10 dark:bg-slate-950/80 md:inset-x-6 xl:hidden"
      >
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          :to="{ name: item.name }"
          class="flex min-w-0 flex-1 justify-center rounded-full px-3 py-2 text-sm font-medium transition"
          :class="
            route.name === item.name
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:text-foreground'
          "
        >
          {{ t(item.key) }}
        </RouterLink>
      </nav>
    </div>
  </div>
</template>
