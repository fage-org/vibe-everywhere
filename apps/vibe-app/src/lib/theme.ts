import { computed, readonly, ref } from "vue"

const THEME_STORAGE_KEY = "vibe.everywhere.theme"
const SUPPORTED_THEME_MODES = ["system", "light", "dark"] as const

export type ThemeMode = (typeof SUPPORTED_THEME_MODES)[number]
export type ResolvedTheme = Exclude<ThemeMode, "system">

const themeMode = ref<ThemeMode>("system")
const resolvedTheme = ref<ResolvedTheme>("light")

let mediaQuery: MediaQueryList | null = null
let mediaQueryBound = false

function normalizeThemeMode(value?: string | null): ThemeMode {
  if (value === "light" || value === "dark" || value === "system") {
    return value
  }

  return "system"
}

function readStoredTheme() {
  if (typeof window === "undefined") {
    return null
  }

  return window.localStorage.getItem(THEME_STORAGE_KEY)
}

function hasStoredTheme() {
  return Boolean(readStoredTheme())
}

function resolveSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return "light"
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"
}

function extractConfiguredTheme(config?: unknown): ThemeMode | null {
  if (!config || typeof config !== "object") {
    return null
  }

  const record = config as Record<string, unknown>
  const directValue =
    typeof record.defaultThemeMode === "string"
      ? record.defaultThemeMode
      : typeof record.themeMode === "string"
        ? record.themeMode
        : null
  if (directValue) {
    return normalizeThemeMode(directValue)
  }

  if (!record.theme || typeof record.theme !== "object") {
    return null
  }

  const nested = record.theme as Record<string, unknown>
  const nestedValue =
    typeof nested.defaultMode === "string"
      ? nested.defaultMode
      : typeof nested.mode === "string"
        ? nested.mode
        : null

  return nestedValue ? normalizeThemeMode(nestedValue) : null
}

function applyResolvedTheme(nextTheme: ResolvedTheme) {
  resolvedTheme.value = nextTheme

  if (typeof document === "undefined") {
    return
  }

  const root = document.documentElement
  root.classList.toggle("dark", nextTheme === "dark")
  root.classList.toggle("light", nextTheme === "light")
  root.dataset.theme = nextTheme
  root.dataset.themeMode = themeMode.value
  root.style.colorScheme = nextTheme
}

function applyThemeMode(nextThemeMode: ThemeMode) {
  themeMode.value = nextThemeMode
  applyResolvedTheme(nextThemeMode === "system" ? resolveSystemTheme() : nextThemeMode)
}

function bindSystemThemeListener() {
  if (mediaQueryBound || typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return
  }

  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
  const handleSystemThemeChange = () => {
    if (themeMode.value !== "system") {
      return
    }

    applyResolvedTheme(mediaQuery?.matches ? "dark" : "light")
  }

  if (typeof mediaQuery.addEventListener === "function") {
    mediaQuery.addEventListener("change", handleSystemThemeChange)
  } else if (typeof mediaQuery.addListener === "function") {
    mediaQuery.addListener(handleSystemThemeChange)
  }

  mediaQueryBound = true
}

export function initializeTheme(config?: unknown) {
  bindSystemThemeListener()

  const storedTheme = readStoredTheme()
  const nextThemeMode = storedTheme
    ? normalizeThemeMode(storedTheme)
    : extractConfiguredTheme(config) ?? "system"
  applyThemeMode(nextThemeMode)

  return nextThemeMode
}

export function setThemeMode(value: string) {
  const nextThemeMode = normalizeThemeMode(value)
  applyThemeMode(nextThemeMode)

  if (typeof window !== "undefined") {
    window.localStorage.setItem(THEME_STORAGE_KEY, nextThemeMode)
  }

  return nextThemeMode
}

export function syncThemeWithAppConfig(config?: unknown) {
  if (hasStoredTheme()) {
    applyThemeMode(themeMode.value)
    return themeMode.value
  }

  const configuredTheme = extractConfiguredTheme(config)
  if (!configuredTheme) {
    return themeMode.value
  }

  applyThemeMode(configuredTheme)
  return configuredTheme
}

export function getSupportedThemeModes() {
  return [...SUPPORTED_THEME_MODES]
}

export function useTheme() {
  return {
    themeMode: readonly(themeMode),
    resolvedTheme: computed(() => resolvedTheme.value)
  }
}
