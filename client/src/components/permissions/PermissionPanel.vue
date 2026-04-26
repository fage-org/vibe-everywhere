<template>
  <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click="close">
    <div class="bg-bg-primary rounded-xl border border-border shadow-lg w-full max-w-lg mx-4" @click.stop>
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-border">
        <h2 class="text-lg font-semibold">{{ $t('sessionDetail.pendingRequests') }}</h2>
        <button @click="close" class="text-text-muted hover:text-text-primary transition-colors">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <!-- Permissions List -->
      <div class="px-6 py-4 space-y-4 max-h-[60vh] overflow-y-auto">
        <div
          v-for="permission in permissions"
          :key="permission.permission_id"
          class="border border-border rounded-lg p-4"
        >
          <div class="flex items-start justify-between mb-3">
            <div>
              <div class="flex items-center gap-2 mb-1">
                <span
                  :class="[
                    'text-[10px] px-2 py-0.5 rounded-full',
                    riskClass(permission.risk_type)
                  ]"
                >
                  {{ riskText(permission.risk_type) }}
                </span>
                <span class="text-xs text-text-muted">
                  {{ formatTime(permission.created_at) }}
                </span>
              </div>
              <p class="font-medium">{{ permission.summary }}</p>
            </div>
          </div>

          <p v-if="permission.target" class="text-sm text-text-secondary mb-3">
            {{ permission.target }}
          </p>

          <!-- Action Buttons -->
          <div class="flex gap-2">
            <button
              @click="deny(permission.permission_id)"
              class="flex-1 px-4 py-2 border border-border rounded-lg text-sm hover:bg-bg-secondary transition-colors"
            >
              {{ $t('permissions.deny') }}
            </button>
            <button
              @click="allow(permission.permission_id)"
              class="flex-1 px-4 py-2 bg-accent hover:bg-accent/90 text-white rounded-lg text-sm transition-colors"
            >
              {{ $t('permissions.allow') }}
            </button>
          </div>

          <label class="flex items-center gap-2 mt-3 text-sm cursor-pointer">
            <input
              v-model="rememberSession[permission.permission_id]"
              type="checkbox"
              class="rounded border-border"
            />
            {{ $t('permissions.remember') }}
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePermissionStore } from '@/stores/permissions'
import type { PermissionRequest, RiskType } from '@/types'
import type { PermissionDecision } from '@/types/generated/PermissionDecision'

const props = defineProps<{
  modelValue: boolean
  permissions: PermissionRequest[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const { t } = useI18n()
const permissionStore = usePermissionStore()

const rememberSession = reactive<Record<string, boolean>>({})

function close() {
  emit('update:modelValue', false)
}

function riskText(risk: RiskType): string {
  switch (risk) {
    case 'write_fs': return t('permissions.fileWrite')
    case 'exec_cmd': return t('permissions.execCmd')
    case 'network': return t('permissions.network')
    default: return risk
  }
}

function riskClass(risk: RiskType): string {
  switch (risk) {
    case 'write_fs': return 'bg-warning-bg text-warning'
    case 'exec_cmd': return 'bg-danger-bg text-danger'
    case 'network': return 'bg-accent-bg text-accent'
    default: return 'bg-bg-tertiary text-text-muted'
  }
}

function formatTime(time: string): string {
  return new Date(time).toLocaleString()
}

async function allow(permissionId: string) {
  const remember = rememberSession[permissionId] || false
  await permissionStore.respondToPermission(permissionId, 'approve_once' as PermissionDecision, remember)
  if (props.permissions.length === 1) {
    close()
  }
}

async function deny(permissionId: string) {
  await permissionStore.respondToPermission(permissionId, 'deny_once' as PermissionDecision, false)
  if (props.permissions.length === 1) {
    close()
  }
}
</script>
