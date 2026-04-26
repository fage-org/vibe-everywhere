<template>
  <div v-if="modelValue" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click="close">
    <div class="bg-bg-primary rounded-xl border border-border shadow-lg w-full max-w-lg mx-4" @click.stop>
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-border">
        <h2 class="text-lg font-semibold">{{ $t('sessions.createTitle') }}</h2>
        <button @click="close" class="text-text-muted hover:text-text-primary transition-colors">
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="px-6 py-4 space-y-5">
        <!-- Step 1: Select Host -->
        <div>
          <label class="block text-sm font-medium mb-2">{{ $t('sessions.selectHost') }}</label>
          <select
            v-model="selectedHostId"
            class="w-full px-3 py-2 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent"
          >
            <option value="">{{ $t('common.select') || 'Select...' }}</option>
            <option v-for="host in hostStore.hosts" :key="host.host_id" :value="host.host_id">
              {{ host.host_name }} ({{ host.platform }})
            </option>
          </select>
        </div>

        <!-- Step 2: Select/Create Workspace -->
        <div v-if="selectedHostId">
          <label class="block text-sm font-medium mb-2">{{ $t('sessions.selectWorkspace') }}</label>
          <div class="space-y-2">
            <select
              v-model="selectedWorkspaceId"
              class="w-full px-3 py-2 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent"
            >
              <option value="">{{ $t('common.select') || 'Select...' }}</option>
              <option value="new">+ {{ $t('hosts.newWorkspace') || 'New Workspace' }}</option>
              <option v-for="ws in workspaceStore.workspaces" :key="ws.workspace_id" :value="ws.workspace_id">
                {{ ws.display_name }}
              </option>
            </select>

            <!-- New Workspace Path -->
            <div v-if="selectedWorkspaceId === 'new'">
              <input
                v-model="newWorkspacePath"
                type="text"
                placeholder="/path/to/workspace"
                class="w-full px-3 py-2 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent"
              />
            </div>
          </div>
        </div>

        <!-- Step 3: Select Agent -->
        <div v-if="canShowAgent">
          <label class="block text-sm font-medium mb-2">{{ $t('sessions.selectAgent') }}</label>
          <div class="flex gap-2">
            <button
              v-for="agent in agents"
              :key="agent.value"
              @click="selectedAgent = agent.value"
              :class="[
                'px-4 py-2 rounded-lg text-sm border transition-colors',
                selectedAgent === agent.value
                  ? 'border-accent bg-accent-bg text-accent'
                  : 'border-border hover:bg-bg-hover'
              ]"
            >
              {{ agent.label }}
            </button>
          </div>
        </div>

        <!-- Step 4: Initial Message -->
        <div v-if="canShowMessage">
          <label class="block text-sm font-medium mb-2">{{ $t('sessions.taskMessage') }}</label>
          <textarea
            v-model="initialMessage"
            :placeholder="$t('sessions.taskMessagePlaceholder')"
            rows="3"
            class="w-full px-3 py-2 bg-bg-secondary border border-border rounded-lg text-sm focus:outline-none focus:border-accent resize-none"
          />
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-border">
        <button
          @click="close"
          class="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
        >
          {{ $t('common.cancel') }}
        </button>
        <button
          @click="createSession"
          :disabled="!canSubmit || isCreating"
          class="px-4 py-2 bg-accent hover:bg-accent/90 text-white text-sm rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          <svg v-if="isCreating" class="animate-spin w-4 h-4" viewBox="0 0 24 24" fill="none">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          {{ isCreating ? $t('common.creating') || 'Creating...' : $t('sessions.createButton') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSessionStore } from '@/stores/sessions'
import { useHostStore } from '@/stores/hosts'
import { useWorkspaceStore } from '@/stores/workspaces'

const props = defineProps<{
  modelValue: boolean
  preselectedHostId?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const router = useRouter()
useI18n() // i18n available via template $t
const sessionStore = useSessionStore()
const hostStore = useHostStore()
const workspaceStore = useWorkspaceStore()

// State
const selectedHostId = ref('')
const selectedWorkspaceId = ref('')
const newWorkspacePath = ref('')
const selectedAgent = ref('claude_code')
const initialMessage = ref('')
const isCreating = ref(false)

// Constants
const agents = [
  { value: 'claude_code', label: 'Claude Code' },
  { value: 'acp', label: 'ACP Agent' },
]

// Computed
const canShowAgent = computed(() => {
  if (selectedWorkspaceId.value === 'new') {
    return newWorkspacePath.value.trim() !== ''
  }
  return selectedWorkspaceId.value !== ''
})

const canShowMessage = computed(() => canShowAgent.value)

const canSubmit = computed(() => {
  if (!selectedHostId.value) return false
  if (selectedWorkspaceId.value === 'new' && !newWorkspacePath.value.trim()) return false
  if (selectedWorkspaceId.value === '') return false
  if (!selectedAgent.value) return false
  return true
})

// Methods
function close() {
  emit('update:modelValue', false)
  resetForm()
}

function resetForm() {
  selectedHostId.value = props.preselectedHostId || ''
  selectedWorkspaceId.value = ''
  newWorkspacePath.value = ''
  selectedAgent.value = 'claude_code'
  initialMessage.value = ''
}

async function createSession() {
  if (!canSubmit.value) return

  isCreating.value = true

  let workspaceId = selectedWorkspaceId.value

  // Create new workspace if needed
  if (workspaceId === 'new') {
    const path = newWorkspacePath.value.trim()
    const displayName = path.split('/').pop() || path
    const newWs = await workspaceStore.createWorkspace({
      host_id: selectedHostId.value,
      path: path,
      display_name: displayName,
    })
    if (newWs) {
      workspaceId = newWs.workspace_id
    } else {
      isCreating.value = false
      return
    }
  }

  // Create session
  const sessionId = await sessionStore.createSession({
    host_id: selectedHostId.value,
    workspace_id: workspaceId,
    title: initialMessage.value.trim().slice(0, 50) || 'New Session',
    initial_message: initialMessage.value.trim(),
  })

  isCreating.value = false

  if (sessionId) {
    close()
    router.push(`/sessions/${sessionId}`)
  }
}

// Watch for host selection to load workspaces
watch(selectedHostId, async (hostId) => {
  if (hostId) {
    await workspaceStore.fetchWorkspaces(hostId)
  } else {
    workspaceStore.workspaces = []
  }
  selectedWorkspaceId.value = ''
})
</script>
