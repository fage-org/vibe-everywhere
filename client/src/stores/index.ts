import { createPinia } from 'pinia'

export const pinia = createPinia()

// Export all stores
export { useAuthStore } from './auth'
export { useSettingsStore } from './settings'
export { useHostStore } from './hosts'
export { useWorkspaceStore } from './workspaces'
export { useSessionStore } from './sessions'
export { useArchiveStore } from './archives'
export { usePermissionStore } from './permissions'
export { useEventStore } from './events'
export { useToastStore } from './toast'

export default pinia
