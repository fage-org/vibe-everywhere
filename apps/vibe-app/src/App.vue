<script setup lang="ts">
import { watch } from "vue"
import { syncThemeWithAppConfig } from "@/lib/theme"
import { useAppStore } from "@/stores/app"

const store = useAppStore()

watch(
  () => store.appConfig,
  (value) => {
    syncThemeWithAppConfig(value)
  },
  { immediate: true }
)
</script>

<template>
  <div class="min-h-screen h-full">
    <router-view />
  </div>
</template>
