import { createRouter, createWebHashHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";
import DashboardView from "./views/DashboardView.vue";

const DEFAULT_DOCUMENT_TITLE = "Vibe Everywhere";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "dashboard",
    component: DashboardView,
    meta: {
      title: DEFAULT_DOCUMENT_TITLE
    }
  }
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 };
  }
});

router.afterEach((to) => {
  const title =
    typeof to.meta.title === "string" && to.meta.title.trim()
      ? to.meta.title
      : DEFAULT_DOCUMENT_TITLE;
  document.title = title;
});

export default router;
