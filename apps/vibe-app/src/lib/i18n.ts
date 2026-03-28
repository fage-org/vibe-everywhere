import { createI18n } from "vue-i18n"
import en from "@/locales/en"
import zhCN from "@/locales/zh-CN"

const LOCALE_STORAGE_KEY = "vibe.everywhere.locale"
const SUPPORTED_LOCALES = ["zh-CN", "en"] as const

export type AppLocale = (typeof SUPPORTED_LOCALES)[number]

const messages = {
  en,
  "zh-CN": zhCN
}

function normalizeLocale(value?: string | null): AppLocale {
  if (!value) {
    return "en"
  }

  const normalized = value.toLowerCase()
  if (normalized.startsWith("zh")) {
    return "zh-CN"
  }

  return "en"
}

function readStoredLocale() {
  if (typeof window === "undefined") {
    return null
  }

  return window.localStorage.getItem(LOCALE_STORAGE_KEY)
}

export function resolveInitialLocale(): AppLocale {
  const stored = readStoredLocale()
  if (stored) {
    return normalizeLocale(stored)
  }

  if (typeof window === "undefined") {
    return "en"
  }

  return normalizeLocale(window.navigator.languages?.[0] ?? window.navigator.language)
}

export const i18n = createI18n({
  legacy: false,
  locale: "en",
  fallbackLocale: "en",
  messages
})

export function setAppLocale(value: string): AppLocale {
  const locale = normalizeLocale(value)
  i18n.global.locale.value = locale

  if (typeof window !== "undefined") {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, locale)
  }

  if (typeof document !== "undefined") {
    document.documentElement.lang = locale
  }

  return locale
}

export function initializeLocale() {
  return setAppLocale(resolveInitialLocale())
}

export function getSupportedLocales() {
  return [...SUPPORTED_LOCALES]
}
