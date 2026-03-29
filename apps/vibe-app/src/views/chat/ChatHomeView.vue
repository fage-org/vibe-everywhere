<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
import { storeToRefs } from "pinia";
import { HardDrive, MessageSquarePlus, RefreshCw, Settings2 } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { useDashboardController } from "@/views/dashboard/controller";
import { collectProjectsForDevice, encodeProjectKey } from "@/views/chat/projects";

const { t } = useI18n();
const router = useRouter();
const dashboard = useDashboardController();
const store = dashboard.store;
const { devices, eventState } = storeToRefs(store);
const onlineDeviceCount = computed(() => dashboard.health.value?.onlineDeviceCount ?? 0);

const deviceGroups = computed(() =>
  devices.value.map((device) => ({
    device,
    projects: collectProjectsForDevice(store.visibleConversations, device.id),
  })),
);

function openProject(deviceId: string, cwd: string | null) {
  void router.push({
    name: "chat-project",
    params: {
      deviceId,
      projectKey: encodeProjectKey(cwd),
    },
  });
}
</script>

<template>
  <section class="space-y-5">
    <header
      class="rounded-[30px] border border-border/70 bg-card/85 px-5 py-5 shadow-xl backdrop-blur-xl md:px-7"
    >
      <div class="flex flex-col gap-4 md:flex-row md:items-start md:justify-between">
        <div class="space-y-3">
          <Badge
            variant="outline"
            class="w-fit border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-100"
          >
            {{ t("chatHome.badge") }}
          </Badge>
          <div class="space-y-2">
            <h1 class="text-3xl font-semibold tracking-tight text-foreground md:text-4xl">
              {{ t("chatHome.title") }}
            </h1>
            <p class="max-w-3xl text-sm leading-7 text-muted-foreground md:text-base">
              {{ t("chatHome.summary") }}
            </p>
          </div>
        </div>

        <div class="grid gap-3 sm:grid-cols-3 md:min-w-[420px]">
          <div class="rounded-3xl border border-border/70 bg-background/60 p-4">
            <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("chatHome.connectionState") }}
            </p>
            <p class="mt-2 text-sm font-medium text-foreground">
              {{ dashboard.formatConnectionState(eventState) }}
            </p>
          </div>
          <div class="rounded-3xl border border-border/70 bg-background/60 p-4">
            <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("chatHome.onlineDevices") }}
            </p>
            <p class="mt-2 text-2xl font-semibold text-foreground">
              {{ onlineDeviceCount }}
            </p>
          </div>
          <div class="rounded-3xl border border-border/70 bg-background/60 p-4">
            <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("chatHome.totalProjects") }}
            </p>
            <p class="mt-2 text-2xl font-semibold text-foreground">
              {{ deviceGroups.reduce((count, item) => count + item.projects.length, 0) }}
            </p>
          </div>
        </div>
      </div>

      <div class="mt-5 flex flex-wrap gap-3">
        <Button type="button" variant="outline" @click="store.reloadAll">
          <RefreshCw class="size-4" />
          {{ t("common.refresh") }}
        </Button>
        <Button type="button" variant="outline" @click="router.push({ name: 'menu-settings-server' })">
          <Settings2 class="size-4" />
          {{ t("chatHome.serverSettings") }}
        </Button>
      </div>
    </header>

    <div class="grid gap-4 xl:grid-cols-2">
      <Card
        v-for="group in deviceGroups"
        :key="group.device.id"
        class="border-border/70 bg-card/82 shadow-xl backdrop-blur-xl"
      >
        <CardContent class="space-y-4 p-5">
          <div class="flex items-start justify-between gap-4">
            <div class="space-y-1">
              <p class="text-xl font-semibold text-foreground">
                {{ group.device.name }}
              </p>
              <p class="text-sm text-muted-foreground">
                {{ group.device.platform }} · {{ group.device.metadata.arch }}
              </p>
              <p class="font-mono text-xs text-muted-foreground">
                {{ group.device.metadata.workingRoot ?? t("common.useAgentWorkingRoot") }}
              </p>
            </div>
            <Badge
              variant="outline"
              :class="
                group.device.online
                  ? dashboard.statusBadgeClass('online')
                  : dashboard.statusBadgeClass('offline')
              "
            >
              {{
                group.device.online
                  ? t("common.online")
                  : t("common.offline")
              }}
            </Badge>
          </div>

          <div class="flex items-center justify-between">
            <div class="text-sm text-muted-foreground">
              {{ t("chatHome.deviceProjectCount", { count: group.projects.length }) }}
            </div>
            <Button
              type="button"
              size="sm"
              variant="outline"
              @click="openProject(group.device.id, null)"
            >
              <MessageSquarePlus class="size-4" />
              {{ t("chatHome.defaultWorkspaceEntry") }}
            </Button>
          </div>

          <div v-if="group.projects.length" class="space-y-3">
            <button
              v-for="project in group.projects"
              :key="project.key"
              type="button"
              class="w-full rounded-[26px] border border-border/70 bg-background/55 p-4 text-left transition hover:bg-accent/35"
              @click="openProject(group.device.id, project.cwd)"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 space-y-1">
                  <div class="flex items-center gap-2">
                    <HardDrive class="size-4 text-sky-500" />
                    <p class="truncate text-base font-medium text-foreground">
                      {{ project.name }}
                    </p>
                  </div>
                  <p class="truncate font-mono text-xs text-muted-foreground">
                    {{ project.cwd ?? t("chatHome.defaultWorkspacePath") }}
                  </p>
                </div>
                <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
                  {{ t("chatHome.topicCount", { count: project.topicCount }) }}
                </Badge>
              </div>

              <div class="mt-4 space-y-1">
                <p class="truncate text-sm font-medium text-foreground">
                  {{ project.latestConversationTitle }}
                </p>
                <p class="text-xs text-muted-foreground">
                  {{ dashboard.formatProviderKind(project.latestProvider) }}
                  ·
                  {{ dashboard.formatTimestamp(project.latestUpdatedAtEpochMs) }}
                </p>
              </div>
            </button>
          </div>

          <div
            v-else
            class="rounded-[26px] border border-dashed border-border/70 bg-background/40 px-5 py-8 text-center"
          >
            <p class="text-sm text-muted-foreground">
              {{ t("chatHome.emptyProjects") }}
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  </section>
</template>
