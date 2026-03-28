import { createApp, watchEffect } from "vue"
import { createPinia } from "pinia"
import App from "./App.vue"
import { i18n, initializeLocale } from "./lib/i18n"
import { initializeTheme } from "./lib/theme"
import router from "./router"
import "./styles.css"

async function bootstrap() {
  initializeLocale()
  initializeTheme()

  const app = createApp(App)
  app.use(createPinia())
  app.use(i18n)
  app.use(router)
  await router.isReady()

  watchEffect(() => {
    const currentRoute = router.currentRoute.value
    const titleKey =
      typeof currentRoute.meta.titleKey === "string" && currentRoute.meta.titleKey.trim()
        ? currentRoute.meta.titleKey
        : "app.title"
    document.title = i18n.global.t(titleKey)
  })

  app.mount("#root")
}

void bootstrap()
