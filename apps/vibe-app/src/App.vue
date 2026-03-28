<script setup lang="ts">
import { watch } from "vue"
import { storeToRefs } from "pinia"
import { syncThemeWithAppConfig } from "@/lib/theme"
import { useControlStore } from "@/stores/control"

const store = useControlStore()
const { appConfig } = storeToRefs(store)

watch(
  appConfig,
  (value) => {
    syncThemeWithAppConfig(value)
  },
  { immediate: true }
)
</script>

<template>
  <div class="min-h-screen">
    <router-view />
  </div>
</template>
