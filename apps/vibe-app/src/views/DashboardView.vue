<script setup lang="ts">
import { computed } from "vue"
import { RouterLink, RouterView, useRoute } from "vue-router"
import { useI18n } from "vue-i18n"
import { Activity } from "lucide-vue-next"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent } from "@/components/ui/card"
import { cn } from "@/lib/utils"
import { provideDashboardController } from "@/views/dashboard/controller"
import {
  dashboardSections,
  type DashboardRouteName
} from "@/views/dashboard/navigation"

const { t } = useI18n()
const route = useRoute()
const dashboard = provideDashboardController()

const {
  appName,
  eventState,
  formatConnectionState,
  health,
  localizedErrorMessage,
  portForwards,
  selectedDevice,
  shellSessions,
  tasks
} = dashboard

const activeSection = computed(
  () =>
    dashboardSections.find((item) => item.routeName === route.name) ?? dashboardSections[0]
)
const isSessionRoute = computed(() => activeSection.value.routeName === "dashboard-sessions")
const mobileNavStyle = computed(() => ({
  gridTemplateColumns: `repeat(${dashboardSections.length}, minmax(0, 1fr))`
}))
const selectedDeviceLabel = computed(
  () => selectedDevice.value?.name ?? t("dashboard.shell.noDeviceSelected")
)
const quickStats = computed(() => [
  {
    key: "online",
    label: t("dashboard.stats.onlineDevices"),
    value: health.value?.onlineDeviceCount ?? 0
  },
  {
    key: "sessions",
    label: t("dashboard.stats.aiSessions"),
    value: tasks.value.length
  },
  {
    key: "advanced",
    label: t("dashboard.stats.advancedTools"),
    value: shellSessions.value.length + portForwards.value.length
  }
])

function isRouteActive(routeName: DashboardRouteName) {
  return route.name === routeName
}
</script>

<template>
  <main
    :class="
      cn(
        'mx-auto min-h-screen w-full px-3 py-4 md:px-6 xl:px-8',
        isSessionRoute ? 'max-w-[1800px]' : 'flex max-w-[1600px] gap-4'
      )
    "
  >
    <aside
      v-if="!isSessionRoute"
      class="sticky top-4 hidden h-[calc(100vh-2rem)] w-[290px] shrink-0 flex-col gap-4 lg:flex"
    >
      <Card class="border-border/70 bg-card/85 shadow-2xl backdrop-blur-xl">
        <CardContent class="space-y-5 p-5">
          <div class="space-y-3">
            <Badge
              variant="outline"
              class="border-amber-400/30 bg-amber-400/12 text-amber-700 dark:text-amber-100"
            >
              {{ t("dashboard.shell.badge") }}
            </Badge>
            <div class="space-y-2">
              <h1 class="text-2xl font-semibold tracking-tight text-foreground">
                {{ appName }}
              </h1>
              <p class="text-sm leading-6 text-muted-foreground">
                {{ t("dashboard.shell.description") }}
              </p>
            </div>
          </div>

          <div class="grid gap-3">
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                {{ t("dashboard.shell.connectionState") }}
              </p>
              <div class="mt-2 flex items-center gap-2 text-sm font-medium text-foreground">
                <Activity class="size-4 text-amber-400" />
                {{ formatConnectionState(eventState) }}
              </div>
            </div>

            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                {{ t("dashboard.shell.selectedDevice") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">
                {{ selectedDeviceLabel }}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
        <CardContent class="p-3">
          <nav class="space-y-2">
            <RouterLink
              v-for="item in dashboardSections"
              :key="item.routeName"
              :to="{ name: item.routeName }"
              :class="
                cn(
                  'flex items-start gap-3 rounded-2xl border px-4 py-3 transition-colors',
                  isRouteActive(item.routeName)
                    ? 'border-primary/50 bg-primary/12 shadow-lg shadow-primary/10'
                    : 'border-border/70 bg-background/45 hover:bg-accent/40'
                )
              "
            >
              <component
                :is="item.icon"
                class="mt-0.5 size-4 shrink-0"
                :class="isRouteActive(item.routeName) ? 'text-primary' : 'text-muted-foreground'"
              />
              <div class="min-w-0 space-y-1">
                <p class="text-sm font-medium text-foreground">
                  {{ t(item.titleKey) }}
                </p>
                <p class="text-xs leading-5 text-muted-foreground">
                  {{ t(item.descriptionKey) }}
                </p>
              </div>
            </RouterLink>
          </nav>
        </CardContent>
      </Card>

      <Card class="border-border/70 bg-card/80 shadow-xl backdrop-blur-xl">
        <CardContent class="grid gap-3 p-4">
          <div
            v-for="stat in quickStats"
            :key="stat.key"
            class="rounded-2xl border border-border/70 bg-background/55 p-4"
          >
            <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
              {{ stat.label }}
            </p>
            <p class="mt-2 text-2xl font-semibold text-foreground">
              {{ stat.value }}
            </p>
          </div>
        </CardContent>
      </Card>
    </aside>

    <section :class="cn('min-w-0 pb-24 lg:pb-6', isSessionRoute ? 'w-full' : 'flex-1')">
      <Card
        v-if="!isSessionRoute"
        class="mb-4 border-border/70 bg-card/85 shadow-xl backdrop-blur-xl lg:hidden"
      >
        <CardContent class="space-y-4 p-5">
          <div class="space-y-2">
            <Badge
              variant="outline"
              class="border-amber-400/30 bg-amber-400/12 text-amber-700 dark:text-amber-100"
            >
              {{ t("dashboard.shell.badge") }}
            </Badge>
            <div class="space-y-1">
              <h1 class="text-2xl font-semibold tracking-tight text-foreground">
                {{ appName }}
              </h1>
              <p class="text-sm leading-6 text-muted-foreground">
                {{ t("dashboard.shell.description") }}
              </p>
            </div>
          </div>

          <div class="grid gap-3 sm:grid-cols-2">
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                {{ t("dashboard.shell.connectionState") }}
              </p>
              <div class="mt-2 flex items-center gap-2 text-sm font-medium text-foreground">
                <Activity class="size-4 text-amber-400" />
                {{ formatConnectionState(eventState) }}
              </div>
            </div>
            <div class="rounded-2xl border border-border/70 bg-background/55 p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
                {{ t("dashboard.shell.selectedDevice") }}
              </p>
              <p class="mt-2 text-sm font-medium text-foreground">
                {{ selectedDeviceLabel }}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <header
        v-if="isSessionRoute"
        class="mb-4 rounded-[28px] border border-border/70 bg-card/80 px-4 py-4 shadow-xl backdrop-blur-xl"
      >
        <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div class="space-y-1">
            <p class="text-xs uppercase tracking-[0.18em] text-muted-foreground">
              {{ appName }}
            </p>
            <div class="flex flex-wrap items-center gap-2 text-sm text-muted-foreground">
              <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
                {{ formatConnectionState(eventState) }}
              </Badge>
              <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
                {{ selectedDeviceLabel }}
              </Badge>
            </div>
          </div>

          <nav class="flex flex-wrap gap-2">
            <RouterLink
              v-for="item in dashboardSections"
              :key="item.routeName"
              :to="{ name: item.routeName }"
              :class="
                cn(
                  'rounded-2xl border px-4 py-2 text-sm transition-colors',
                  isRouteActive(item.routeName)
                    ? 'border-primary/50 bg-primary/12 text-primary'
                    : 'border-border/70 bg-background/55 text-muted-foreground hover:bg-accent/40 hover:text-foreground'
                )
              "
            >
              {{ t(item.titleKey) }}
            </RouterLink>
          </nav>
        </div>
      </header>

      <header
        v-else
        class="mb-4 flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between"
      >
        <div class="space-y-2">
          <Badge variant="outline" class="border-border/70 bg-background/55 text-foreground">
            {{ t(activeSection.titleKey) }}
          </Badge>
          <div class="space-y-1">
            <h2 class="text-3xl font-semibold tracking-tight text-foreground md:text-4xl">
              {{ t(activeSection.titleKey) }}
            </h2>
            <p class="max-w-3xl text-sm leading-6 text-muted-foreground md:text-base">
              {{ t(activeSection.descriptionKey) }}
            </p>
          </div>
        </div>

        <div class="grid gap-3 sm:grid-cols-3 xl:min-w-[420px]">
          <div
            v-for="stat in quickStats"
            :key="stat.key"
            class="rounded-2xl border border-border/70 bg-card/75 p-4 shadow-lg backdrop-blur-xl"
          >
            <p class="text-xs uppercase tracking-[0.16em] text-muted-foreground">
              {{ stat.label }}
            </p>
            <p class="mt-2 text-2xl font-semibold text-foreground">
              {{ stat.value }}
            </p>
          </div>
        </div>
      </header>

      <Card
        v-if="localizedErrorMessage"
        class="mb-4 border-rose-500/25 bg-rose-500/10 text-rose-700 shadow-none dark:text-rose-100"
      >
        <CardContent class="p-4">
          <p class="text-sm">{{ localizedErrorMessage }}</p>
        </CardContent>
      </Card>

      <RouterView />
    </section>

    <nav
      class="fixed inset-x-0 bottom-0 z-30 border-t border-border/70 bg-background/92 backdrop-blur-xl lg:hidden"
    >
      <div class="mx-auto grid max-w-[680px] px-2 py-2" :style="mobileNavStyle">
        <RouterLink
          v-for="item in dashboardSections"
          :key="item.routeName"
          :to="{ name: item.routeName }"
          :class="
            cn(
              'flex min-w-0 flex-col items-center gap-1 rounded-2xl px-2 py-2 text-center transition-colors',
              isRouteActive(item.routeName)
                ? 'bg-primary/12 text-primary'
                : 'text-muted-foreground hover:bg-accent/40 hover:text-foreground'
            )
          "
        >
          <component :is="item.icon" class="size-4 shrink-0" />
          <span class="truncate text-[11px] font-medium">
            {{ t(item.titleKey) }}
          </span>
        </RouterLink>
      </div>
    </nav>
  </main>
</template>
