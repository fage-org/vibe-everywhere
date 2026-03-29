<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { ArrowLeft, Palette, Server, Languages } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import { useRouter } from "vue-router";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { useDashboardController } from "@/views/dashboard/controller";

const { t } = useI18n();
const router = useRouter();
const dashboard = useDashboardController();
const store = dashboard.store;
const { relayAccessTokenInput, relayBaseUrl, relayInput } = storeToRefs(store);

const currentRelay = computed(
  () => relayBaseUrl.value || t("settingsPage.notConnected"),
);
const localeOptions = computed(() => dashboard.localeOptions.value);
const themeOptions = computed(() => dashboard.themeOptions.value);
const currentLocale = computed(() => dashboard.locale.value);
const currentThemeMode = computed(() => dashboard.themeMode.value);
</script>

<template>
  <section class="space-y-5">
    <header class="rounded-[30px] border border-border/70 bg-card/85 px-5 py-5 shadow-xl backdrop-blur-xl md:px-7">
      <div class="flex items-start gap-3">
        <Button type="button" variant="outline" size="icon" @click="router.push({ name: 'menu' })">
          <ArrowLeft class="size-4" />
        </Button>
        <div class="space-y-3">
          <Badge
            variant="outline"
            class="border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-100"
          >
            {{ t("settingsPage.badge") }}
          </Badge>
          <div class="space-y-2">
            <h1 class="text-3xl font-semibold tracking-tight text-foreground">
              {{ t("settingsPage.title") }}
            </h1>
            <p class="max-w-2xl text-sm leading-7 text-muted-foreground">
              {{ t("settingsPage.summary") }}
            </p>
          </div>
        </div>
      </div>
    </header>

    <div class="grid gap-4 xl:grid-cols-[minmax(0,1.2fr)_minmax(340px,0.8fr)]">
      <Card class="border-border/70 bg-card/82 shadow-xl backdrop-blur-xl">
        <CardContent class="space-y-5 p-5">
          <div class="flex items-center gap-3">
            <div class="flex size-11 items-center justify-center rounded-2xl bg-sky-500/12 text-sky-700 dark:text-sky-100">
              <Server class="size-5" />
            </div>
            <div>
              <p class="text-lg font-semibold text-foreground">
                {{ t("settingsPage.serverTitle") }}
              </p>
              <p class="text-sm text-muted-foreground">
                {{ t("settingsPage.serverSummary") }}
              </p>
            </div>
          </div>

          <div class="space-y-2">
            <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("dashboard.relayBaseUrl") }}
            </label>
            <Input
              :model-value="relayInput"
              :placeholder="dashboard.relayPlaceholder"
              class="border-border/70 bg-background/75"
              @update:model-value="store.relayInput = String($event)"
            />
          </div>

          <div class="space-y-2">
            <label class="text-[11px] font-medium uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("dashboard.fields.accessToken") }}
            </label>
            <Input
              :model-value="relayAccessTokenInput"
              type="password"
              :placeholder="t('common.optionalAccessToken')"
              class="border-border/70 bg-background/75"
              @update:model-value="store.relayAccessTokenInput = String($event)"
            />
          </div>

          <div class="flex flex-wrap gap-3">
            <Button type="button" @click="store.applyRelayBaseUrl">
              {{ t("settingsPage.saveAndConnect") }}
            </Button>
            <Button type="button" variant="outline" @click="store.reloadAll">
              {{ t("common.refresh") }}
            </Button>
          </div>
        </CardContent>
      </Card>

      <div class="space-y-4">
        <Card class="border-border/70 bg-card/82 shadow-xl backdrop-blur-xl">
          <CardContent class="space-y-4 p-5">
            <div class="flex items-center gap-3">
              <div class="flex size-11 items-center justify-center rounded-2xl bg-emerald-500/12 text-emerald-700 dark:text-emerald-100">
                <Languages class="size-5" />
              </div>
              <p class="text-lg font-semibold text-foreground">
                {{ t("settingsPage.localeTitle") }}
              </p>
            </div>
            <div class="flex flex-wrap gap-2">
              <Button
                v-for="option in localeOptions"
                :key="option.value"
                type="button"
                variant="outline"
                :class="currentLocale === option.value ? 'border-primary/50 bg-primary/12 text-primary' : ''"
                @click="dashboard.switchLocale(option.value)"
              >
                {{ option.label }}
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card class="border-border/70 bg-card/82 shadow-xl backdrop-blur-xl">
          <CardContent class="space-y-4 p-5">
            <div class="flex items-center gap-3">
              <div class="flex size-11 items-center justify-center rounded-2xl bg-amber-500/12 text-amber-700 dark:text-amber-100">
                <Palette class="size-5" />
              </div>
              <p class="text-lg font-semibold text-foreground">
                {{ t("settingsPage.themeTitle") }}
              </p>
            </div>
            <div class="flex flex-wrap gap-2">
              <Button
                v-for="option in themeOptions"
                :key="option.value"
                type="button"
                variant="outline"
                :class="currentThemeMode === option.value ? 'border-primary/50 bg-primary/12 text-primary' : ''"
                @click="dashboard.switchTheme(option.value)"
              >
                {{ option.label }}
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card class="border-border/70 bg-card/82 shadow-xl backdrop-blur-xl">
          <CardContent class="space-y-2 p-5">
            <p class="text-[11px] uppercase tracking-[0.18em] text-muted-foreground">
              {{ t("settingsPage.currentServer") }}
            </p>
            <p class="break-all text-sm font-medium text-foreground">
              {{ currentRelay }}
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  </section>
</template>
