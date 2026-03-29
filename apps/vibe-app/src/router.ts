import { createRouter, createWebHashHistory } from "vue-router"
import type { RouteRecordRaw } from "vue-router"
import DashboardAdvancedSection from "./views/dashboard/sections/DashboardAdvancedSection.vue"
import DashboardDevicesSection from "./views/dashboard/sections/DashboardDevicesSection.vue"
import AppShellView from "./views/AppShellView.vue"
import ChatHomeView from "./views/chat/ChatHomeView.vue"
import ProjectChatView from "./views/chat/ProjectChatView.vue"
import MenuView from "./views/menu/MenuView.vue"
import ServerSettingsView from "./views/menu/ServerSettingsView.vue"

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    component: AppShellView,
    meta: {
      titleKey: "app.title"
    },
    children: [
      {
        path: "",
        redirect: {
          name: "chat-home"
        }
      },
      {
        path: "chat",
        name: "chat-home",
        component: ChatHomeView,
        meta: {
          titleKey: "chatHome.title"
        }
      },
      {
        path: "chat/device/:deviceId/project/:projectKey",
        name: "chat-project",
        component: ProjectChatView,
        meta: {
          titleKey: "chatProject.title"
        }
      },
      {
        path: "devices",
        name: "devices",
        component: DashboardDevicesSection,
        meta: {
          titleKey: "appShell.nav.devices"
        }
      },
      {
        path: "menu",
        name: "menu",
        component: MenuView,
        meta: {
          titleKey: "appShell.nav.menu"
        }
      },
      {
        path: "menu/settings/server",
        name: "menu-settings-server",
        component: ServerSettingsView,
        meta: {
          titleKey: "settingsPage.title"
        }
      },
      {
        path: "menu/advanced",
        name: "menu-advanced",
        component: DashboardAdvancedSection,
        meta: {
          titleKey: "dashboard.nav.advanced"
        }
      }
    ]
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 }
  }
})

export default router
