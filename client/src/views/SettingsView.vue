<template>
  <div class="h-full overflow-y-auto p-6">
    <div class="max-w-2xl mx-auto">
      <h1 class="text-2xl font-semibold mb-6">{{ $t('settings.title') }}</h1>

      <!-- Server Connection -->
      <section class="bg-bg-secondary rounded-lg border border-border p-5 mb-6">
        <h2 class="text-lg font-medium mb-4">{{ $t('settings.serverConnection') }}</h2>

        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div>
              <p class="text-sm font-medium">{{ $t('settings.serverUrl') }}</p>
              <p class="text-sm text-text-muted">{{ authStore.serverUrl }}</p>
            </div>
            <span class="px-2 py-1 bg-success-bg text-success text-xs rounded-full">
              {{ $t('settings.connected') }}
            </span>
          </div>

          <button
            @click="logout"
            class="px-4 py-2 bg-danger-bg text-danger rounded-lg text-sm hover:bg-danger/10 transition-colors"
          >
            {{ $t('common.logout') || 'Logout' }}
          </button>
        </div>
      </section>

      <!-- Current Device -->
      <section class="bg-bg-secondary rounded-lg border border-border p-5 mb-6">
        <h2 class="text-lg font-medium mb-4">{{ $t('settings.currentDevice') }}</h2>

        <div class="space-y-3 text-sm">
          <div class="flex justify-between">
            <span class="text-text-muted">{{ $t('settings.deviceName') }}</span>
            <span>{{ authStore.deviceName }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-text-muted">{{ $t('settings.deviceId') }}</span>
            <span class="font-mono">{{ authStore.deviceId?.slice(0, 16) }}...</span>
          </div>
        </div>
      </section>

      <!-- Server Info -->
      <section v-if="settingsStore.serverInfo" class="bg-bg-secondary rounded-lg border border-border p-5 mb-6">
        <h2 class="text-lg font-medium mb-4">{{ $t('settings.about') }}</h2>
        <div class="space-y-3 text-sm">
          <div class="flex justify-between">
            <span class="text-text-muted">{{ $t('settings.version') }}</span>
            <span>{{ settingsStore.serverInfo.version }}</span>
          </div>
          <div class="flex justify-between">
            <span class="text-text-muted">{{ $t('settings.build') }}</span>
            <span class="font-mono">{{ settingsStore.serverInfo.build }}</span>
          </div>
        </div>
      </section>

      <!-- Language -->
      <section class="bg-bg-secondary rounded-lg border border-border p-5 mb-6">
        <h2 class="text-lg font-medium mb-4">{{ $t('settings.language') }}</h2>

        <div class="flex gap-2">
          <button
            v-for="lang in languages"
            :key="lang.value"
            @click="setLanguage(lang.value)"
            :class="[
              'px-4 py-2 rounded-lg text-sm border transition-colors',
              settingsStore.language === lang.value
                ? 'border-accent bg-accent-bg text-accent'
                : 'border-border hover:bg-bg-hover'
            ]"
          >
            {{ lang.label }}
          </button>
        </div>
      </section>

      <!-- Notification Preferences -->
      <section class="bg-bg-secondary rounded-lg border border-border p-5">
        <h2 class="text-lg font-medium mb-4">{{ $t('settings.notificationPrefs') }}</h2>

        <div class="space-y-4">
          <div
            v-for="pref in notificationPrefs"
            :key="pref.key"
            class="flex items-center justify-between"
          >
            <div>
              <p class="text-sm font-medium">{{ pref.label }}</p>
              <p class="text-xs text-text-muted">{{ pref.description }}</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                v-model="settingsStore.notificationPrefs[pref.key]"
                type="checkbox"
                class="sr-only peer"
                @change="saveNotificationPrefs"
              />
              <div class="w-11 h-6 bg-bg-tertiary peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-accent"></div>
            </label>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'

const router = useRouter()
const { t, locale } = useI18n()
const authStore = useAuthStore()
const settingsStore = useSettingsStore()

const languages = [
  { value: 'zh-CN', label: '中文' },
  { value: 'en-US', label: 'English' },
]

const notificationPrefs = computed(() => [
  {
    key: 'permission_request_enabled',
    label: t('settings.permissionNotif'),
    description: t('settings.permissionNotifDesc'),
  },
  {
    key: 'task_completed_enabled',
    label: t('settings.taskCompleteNotif'),
    description: t('settings.taskCompleteNotifDesc'),
  },
  {
    key: 'task_failed_enabled',
    label: t('settings.taskFailedNotif'),
    description: t('settings.taskFailedNotifDesc'),
  },
  {
    key: 'session_error_enabled',
    label: t('settings.sessionErrorNotif'),
    description: t('settings.sessionErrorNotifDesc'),
  },
])

function setLanguage(lang: string) {
  settingsStore.setLanguage(lang)
  locale.value = lang
}

async function saveNotificationPrefs() {
  await settingsStore.saveNotificationPrefs(settingsStore.notificationPrefs)
}

async function logout() {
  await authStore.logout()
  router.push('/setup')
}

// Initialize
settingsStore.loadSettings().then(() => {
  locale.value = settingsStore.language
  settingsStore.fetchServerInfo()
})
</script>
