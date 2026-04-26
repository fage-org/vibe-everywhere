<template>
  <div class="relative">
    <button
      @click="isOpen = !isOpen"
      class="p-2 text-text-muted hover:text-text-primary transition-colors"
    >
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="1" />
        <circle cx="19" cy="12" r="1" />
        <circle cx="5" cy="12" r="1" />
      </svg>
    </button>

    <div
      v-if="isOpen"
      class="absolute right-0 top-full mt-1 w-56 bg-bg-primary rounded-lg border border-border shadow-lg z-50 py-1"
    >
      <button
        @click="pauseSession"
        class="w-full px-4 py-2 text-left text-sm hover:bg-bg-secondary flex items-center gap-3"
      >
        <svg class="w-4 h-4 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="6" y="4" width="4" height="16" />
          <rect x="14" y="4" width="4" height="16" />
        </svg>
        <div>
          <div>{{ $t('sessions.actions.pause') }}</div>
          <div class="text-xs text-text-muted">{{ $t('sessions.actions.pauseDesc') }}</div>
        </div>
      </button>

      <button
        @click="interruptTask"
        class="w-full px-4 py-2 text-left text-sm hover:bg-bg-secondary flex items-center gap-3"
      >
        <svg class="w-4 h-4 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="6" y="4" width="4" height="16" />
          <rect x="14" y="4" width="4" height="16" />
        </svg>
        <div>
          <div>{{ $t('sessions.actions.interrupt') }}</div>
          <div class="text-xs text-text-muted">{{ $t('sessions.actions.interruptDesc') }}</div>
        </div>
      </button>

      <button
        @click="rerunTask"
        class="w-full px-4 py-2 text-left text-sm hover:bg-bg-secondary flex items-center gap-3"
      >
        <svg class="w-4 h-4 text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10" />
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
        </svg>
        <div>
          <div>{{ $t('sessions.actions.rerun') }}</div>
          <div class="text-xs text-text-muted">{{ $t('sessions.actions.rerunDesc') }}</div>
        </div>
      </button>

      <div class="border-t border-border my-1"></div>

      <button
        @click="terminateSession"
        class="w-full px-4 py-2 text-left text-sm hover:bg-bg-secondary flex items-center gap-3 text-danger"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
          <line x1="9" y1="9" x2="15" y2="15" />
          <line x1="15" y1="9" x2="9" y2="15" />
        </svg>
        <div>
          <div>{{ $t('sessions.actions.terminate') }}</div>
          <div class="text-xs text-danger/70">{{ $t('sessions.actions.terminateDesc') }}</div>
        </div>
      </button>

      <button
        @click="closeAndArchive"
        class="w-full px-4 py-2 text-left text-sm hover:bg-bg-secondary flex items-center gap-3 text-danger"
      >
        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="21 8 21 21 3 21 3 8" />
          <rect x="1" y="3" width="22" height="5" />
          <line x1="10" y1="12" x2="14" y2="12" />
        </svg>
        <div>
          <div>{{ $t('sessions.actions.close') }}</div>
          <div class="text-xs text-danger/70">{{ $t('sessions.actions.closeDesc') }}</div>
        </div>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSessionStore } from '@/stores/sessions'
import type { Session } from '@/types'

const props = defineProps<{
  session: Session
}>()

const router = useRouter()
const { t } = useI18n()
const sessionStore = useSessionStore()

const isOpen = ref(false)

function pauseSession() {
  sessionStore.controlSession(props.session.session_id, 'pause')
  isOpen.value = false
}

function interruptTask() {
  sessionStore.controlSession(props.session.session_id, 'interrupt')
  isOpen.value = false
}

function rerunTask() {
  sessionStore.controlSession(props.session.session_id, 'rerun')
  isOpen.value = false
}

function terminateSession() {
  if (confirm(t('sessions.actions.terminateConfirm') || 'Are you sure you want to terminate this session?')) {
    sessionStore.controlSession(props.session.session_id, 'terminate')
  }
  isOpen.value = false
}

async function closeAndArchive() {
  if (confirm(t('sessions.actions.closeConfirm'))) {
    const success = await sessionStore.closeSession(props.session.session_id)
    if (success) {
      router.push('/sessions')
    }
  }
  isOpen.value = false
}
</script>
