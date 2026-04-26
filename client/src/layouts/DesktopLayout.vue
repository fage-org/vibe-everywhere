<template>
  <div class="flex flex-col h-screen">
    <!-- Sidebar -->
    <aside class="flex flex-row h-full">
      <nav class="w-14 min-w-14 bg-bg-secondary border-r border-border flex flex-col items-center py-3 gap-1">
        <!-- Logo -->
        <router-link to="/sessions" class="w-8 h-8 rounded-lg bg-gradient-to-br from-accent to-purple flex items-center justify-center font-bold text-sm text-white mb-3">
          V
        </router-link>

        <!-- Nav Items -->
        <router-link
          to="/sessions"
          :class="[
            'w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors',
            $route.path.startsWith('/sessions') ? 'bg-accent-bg text-accent' : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'
          ]"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
          </svg>
        </router-link>

        <router-link
          to="/hosts"
          :class="[
            'w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors',
            $route.path.startsWith('/hosts') ? 'bg-accent-bg text-accent' : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'
          ]"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="2" y="3" width="20" height="14" rx="2"/>
            <line x1="8" y1="21" x2="16" y2="21"/>
            <line x1="12" y1="17" x2="12" y2="21"/>
          </svg>
        </router-link>

        <router-link
          to="/archives"
          :class="[
            'w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors',
            $route.path.startsWith('/archives') ? 'bg-accent-bg text-accent' : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'
          ]"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="22 12 16 12 14 15 10 15 8 12 2 12"/>
            <path d="M5.45 5.11L2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z"/>
          </svg>
        </router-link>

        <router-link
          to="/notifications"
          :class="[
            'w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors',
            $route.path.startsWith('/notifications') ? 'bg-accent-bg text-accent' : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'
          ]"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9"/>
            <path d="M13.73 21a2 2 0 0 1-3.46 0"/>
          </svg>
        </router-link>

        <div class="w-7 h-px bg-border my-2"></div>

        <div class="flex-1"></div>

        <router-link
          to="/settings"
          :class="[
            'w-10 h-10 rounded-md flex items-center justify-center cursor-pointer transition-colors',
            $route.path.startsWith('/settings') ? 'bg-accent-bg text-accent' : 'text-text-secondary hover:bg-bg-hover hover:text-text-primary'
          ]"
        >
          <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="3"/>
            <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>
          </svg>
        </router-link>
      </nav>

      <div class="flex-1 overflow-hidden">
        <router-view />
      </div>
    </aside>
  </div>
</template>

<script setup lang="ts">
// No script needed - all logic is in template with $route
</script>
