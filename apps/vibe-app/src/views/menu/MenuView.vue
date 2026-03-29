<script setup lang="ts">
import { useRouter } from "vue-router";
import { Layers3, Settings2, Sparkles } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";

const { t } = useI18n();
const router = useRouter();
</script>

<template>
  <section class="space-y-5">
    <header class="rounded-[30px] border border-border/70 bg-card/85 px-5 py-5 shadow-xl backdrop-blur-xl md:px-7">
      <Badge
        variant="outline"
        class="border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-100"
      >
        {{ t("menuPage.badge") }}
      </Badge>
      <div class="mt-3 space-y-2">
        <h1 class="text-3xl font-semibold tracking-tight text-foreground">
          {{ t("menuPage.title") }}
        </h1>
        <p class="max-w-2xl text-sm leading-7 text-muted-foreground">
          {{ t("menuPage.summary") }}
        </p>
      </div>
    </header>

    <div class="grid gap-4 md:grid-cols-2">
      <button
        type="button"
        class="text-left"
        @click="router.push({ name: 'menu-settings-server' })"
      >
        <Card class="h-full border-border/70 bg-card/82 shadow-xl backdrop-blur-xl transition hover:bg-card/92">
          <CardContent class="space-y-3 p-5">
            <div class="flex size-12 items-center justify-center rounded-2xl bg-sky-500/12 text-sky-700 dark:text-sky-100">
              <Settings2 class="size-5" />
            </div>
            <div class="space-y-1">
              <p class="text-lg font-semibold text-foreground">
                {{ t("menuPage.serverSettingsTitle") }}
              </p>
              <p class="text-sm leading-6 text-muted-foreground">
                {{ t("menuPage.serverSettingsSummary") }}
              </p>
            </div>
          </CardContent>
        </Card>
      </button>

      <Card class="border-border/70 bg-card/82 shadow-xl backdrop-blur-xl">
        <CardContent class="space-y-3 p-5">
          <div class="flex size-12 items-center justify-center rounded-2xl bg-amber-500/12 text-amber-700 dark:text-amber-100">
            <Layers3 class="size-5" />
          </div>
          <div class="space-y-1">
            <p class="text-lg font-semibold text-foreground">
              {{ t("menuPage.secondaryToolsTitle") }}
            </p>
            <p class="text-sm leading-6 text-muted-foreground">
              {{ t("menuPage.secondaryToolsSummary") }}
            </p>
          </div>
          <Badge variant="outline" class="border-border/70 bg-background/60 text-foreground">
            <Sparkles class="mr-1 size-3.5" />
            {{ t("menuPage.secondaryToolsBadge") }}
          </Badge>
        </CardContent>
      </Card>
    </div>
  </section>
</template>
