import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const routes: RouteRecordRaw[] = [
  {
    path: '/setup',
    name: 'ServerSetup',
    component: () => import('@/views/ServerSetupView.vue'),
    meta: { public: true },
  },
  {
    path: '/',
    component: () => import('@/layouts/DesktopLayout.vue'),
    redirect: '/sessions',
    children: [
      {
        path: 'sessions',
        name: 'Sessions',
        component: () => import('@/views/SessionsView.vue'),
      },
      {
        path: 'sessions/:id',
        name: 'SessionDetail',
        component: () => import('@/views/SessionDetailView.vue'),
      },
      {
        path: 'hosts',
        name: 'Hosts',
        component: () => import('@/views/HostsView.vue'),
      },
      {
        path: 'hosts/:id/workspaces',
        name: 'Workspaces',
        component: () => import('@/views/WorkspacesView.vue'),
      },
      {
        path: 'archives',
        name: 'Archives',
        component: () => import('@/views/ArchivesView.vue'),
      },
      {
        path: 'archives/:id',
        name: 'ArchiveDetail',
        component: () => import('@/views/ArchiveDetailView.vue'),
      },
      {
        path: 'notifications',
        name: 'Notifications',
        component: () => import('@/views/NotificationsView.vue'),
      },
      {
        path: 'settings',
        name: 'Settings',
        component: () => import('@/views/SettingsView.vue'),
      },
    ],
  },
  {
    path: '/:pathMatch(.*)*',
    redirect: '/sessions',
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

// Auth guard - redirect to setup if not authenticated
let authInitialized = false

router.beforeEach(async (to, _from, next) => {
  // Skip auth check for public routes
  if (to.meta?.public) {
    next()
    return
  }

  const authStore = useAuthStore()

  // Initialize auth state on first navigation
  if (!authInitialized) {
    await authStore.loadCredentials()
    authInitialized = true
  }

  // Redirect to setup if not authenticated
  if (!authStore.isAuthenticated) {
    next({ name: 'ServerSetup' })
    return
  }

  next()
})

export default router
