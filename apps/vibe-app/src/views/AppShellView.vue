<script setup lang="ts">
import { computed } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import { Grid2x2, Menu, Server } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import { provideDashboardController } from "@/views/dashboard/controller";
import { cn } from "@/lib/utils";

const { t } = useI18n();
const route = useRoute();

provideDashboardController();

const navItems = computed(() => [
  {
    name: "chat-home",
    icon: Grid2x2,
    label: t("appShell.nav.chat"),
    active:
      route.name === "chat-home" ||
      route.name === "chat-project",
  },
  {
    name: "devices",
    icon: Server,
    label: t("appShell.nav.devices"),
    active: route.name === "devices",
  },
  {
    name: "menu",
    icon: Menu,
    label: t("appShell.nav.menu"),
    active:
      route.name === "menu" ||
      route.name === "menu-settings-server",
  },
]);
</script>

<template>
  <div class="mx-auto min-h-screen w-full max-w-[1680px] px-3 pb-24 pt-3 md:px-5 md:pt-5">
    <RouterView />

    <nav
      class="fixed inset-x-0 bottom-0 z-40 border-t border-border/70 bg-background/92 backdrop-blur-xl"
    >
      <div class="mx-auto flex max-w-[720px] items-center justify-around gap-2 px-3 py-3">
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          :to="{ name: item.name }"
          :class="
            cn(
              'flex min-w-0 flex-1 items-center justify-center gap-2 rounded-2xl px-4 py-3 text-sm font-medium transition-colors',
              item.active
                ? 'bg-primary/12 text-primary'
                : 'text-muted-foreground hover:bg-accent/40 hover:text-foreground',
            )
          "
        >
          <component :is="item.icon" class="size-4 shrink-0" />
          <span>{{ item.label }}</span>
        </RouterLink>
      </div>
    </nav>
  </div>
</template>
