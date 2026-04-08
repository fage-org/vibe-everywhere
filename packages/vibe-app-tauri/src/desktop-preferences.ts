export type DesktopThemePreference = "adaptive" | "light" | "dark";
export type DesktopDensityPreference = "compact" | "desktop" | "comfortable";
export type DesktopAvatarStyle = "gradient" | "pixelated" | "brutalist";

export type DesktopAppearanceSettings = {
  themePreference: DesktopThemePreference;
  density: DesktopDensityPreference;
  compactSessionView: boolean;
  inlineToolCalls: boolean;
  expandTodoLists: boolean;
  showLineNumbersInDiffs: boolean;
  showLineNumbersInToolViews: boolean;
  wrapLinesInDiffs: boolean;
  alwaysShowContextSize: boolean;
  showFlavorIcons: boolean;
  avatarStyle: DesktopAvatarStyle;
};

export type DesktopVoiceSettings = {
  assistantLanguage: string | null;
  customAgentId: string | null;
  bypassToken: boolean;
};

export type DesktopLanguageSettings = {
  appLanguage: string;
};

const APPEARANCE_SETTINGS_KEY = "vibe-app-tauri.appearance-settings";
const VOICE_SETTINGS_KEY = "vibe-app-tauri.voice-settings";
const LANGUAGE_SETTINGS_KEY = "vibe-app-tauri.language-settings";

export const defaultAppearanceSettings: DesktopAppearanceSettings = {
  themePreference: "adaptive",
  density: "desktop",
  compactSessionView: false,
  inlineToolCalls: false,
  expandTodoLists: false,
  showLineNumbersInDiffs: false,
  showLineNumbersInToolViews: false,
  wrapLinesInDiffs: false,
  alwaysShowContextSize: false,
  showFlavorIcons: false,
  avatarStyle: "gradient",
};

export const defaultVoiceSettings: DesktopVoiceSettings = {
  assistantLanguage: null,
  customAgentId: null,
  bypassToken: false,
};

export const defaultLanguageSettings: DesktopLanguageSettings = {
  appLanguage: "en",
};

function getStorage(): Storage | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function readStoredJson<T extends object>(key: string): Partial<T> | null {
  const raw = getStorage()?.getItem(key);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw) as unknown;
    return parsed && typeof parsed === "object" ? (parsed as Partial<T>) : null;
  } catch {
    return null;
  }
}

function writeStoredJson<T extends object>(key: string, value: T): void {
  getStorage()?.setItem(key, JSON.stringify(value));
}

function coerceBoolean(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

function coerceString<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return typeof value === "string" && allowed.includes(value as T) ? (value as T) : fallback;
}

export function loadAppearanceSettings(): DesktopAppearanceSettings {
  const stored = readStoredJson<DesktopAppearanceSettings>(APPEARANCE_SETTINGS_KEY);
  if (!stored) {
    return { ...defaultAppearanceSettings };
  }

  return {
    themePreference: coerceString(
      stored.themePreference,
      ["adaptive", "light", "dark"] as const,
      defaultAppearanceSettings.themePreference,
    ),
    density: coerceString(
      stored.density,
      ["compact", "desktop", "comfortable"] as const,
      defaultAppearanceSettings.density,
    ),
    compactSessionView: coerceBoolean(
      stored.compactSessionView,
      defaultAppearanceSettings.compactSessionView,
    ),
    inlineToolCalls: coerceBoolean(
      stored.inlineToolCalls,
      defaultAppearanceSettings.inlineToolCalls,
    ),
    expandTodoLists: coerceBoolean(
      stored.expandTodoLists,
      defaultAppearanceSettings.expandTodoLists,
    ),
    showLineNumbersInDiffs: coerceBoolean(
      stored.showLineNumbersInDiffs,
      defaultAppearanceSettings.showLineNumbersInDiffs,
    ),
    showLineNumbersInToolViews: coerceBoolean(
      stored.showLineNumbersInToolViews,
      defaultAppearanceSettings.showLineNumbersInToolViews,
    ),
    wrapLinesInDiffs: coerceBoolean(
      stored.wrapLinesInDiffs,
      defaultAppearanceSettings.wrapLinesInDiffs,
    ),
    alwaysShowContextSize: coerceBoolean(
      stored.alwaysShowContextSize,
      defaultAppearanceSettings.alwaysShowContextSize,
    ),
    showFlavorIcons: coerceBoolean(
      stored.showFlavorIcons,
      defaultAppearanceSettings.showFlavorIcons,
    ),
    avatarStyle: coerceString(
      stored.avatarStyle,
      ["gradient", "pixelated", "brutalist"] as const,
      defaultAppearanceSettings.avatarStyle,
    ),
  };
}

export function saveAppearanceSettings(settings: DesktopAppearanceSettings): void {
  writeStoredJson(APPEARANCE_SETTINGS_KEY, settings);
}

export function loadVoiceSettings(): DesktopVoiceSettings {
  const stored = readStoredJson<DesktopVoiceSettings>(VOICE_SETTINGS_KEY);
  if (!stored) {
    return { ...defaultVoiceSettings };
  }

  return {
    assistantLanguage:
      stored.assistantLanguage === null || typeof stored.assistantLanguage === "string"
        ? stored.assistantLanguage
        : defaultVoiceSettings.assistantLanguage,
    customAgentId:
      stored.customAgentId === null || typeof stored.customAgentId === "string"
        ? stored.customAgentId
        : defaultVoiceSettings.customAgentId,
    bypassToken: coerceBoolean(stored.bypassToken, defaultVoiceSettings.bypassToken),
  };
}

export function saveVoiceSettings(settings: DesktopVoiceSettings): void {
  writeStoredJson(VOICE_SETTINGS_KEY, settings);
}

export function loadLanguageSettings(): DesktopLanguageSettings {
  const stored = readStoredJson<DesktopLanguageSettings>(LANGUAGE_SETTINGS_KEY);
  if (!stored) {
    return { ...defaultLanguageSettings };
  }

  return {
    appLanguage:
      typeof stored.appLanguage === "string" && stored.appLanguage.trim()
        ? stored.appLanguage
        : defaultLanguageSettings.appLanguage,
  };
}

export function saveLanguageSettings(settings: DesktopLanguageSettings): void {
  writeStoredJson(LANGUAGE_SETTINGS_KEY, settings);
}

export function resolveDesktopThemePreference(
  preference: DesktopThemePreference,
  systemPrefersDark?: boolean,
): "light" | "dark" {
  if (preference === "light" || preference === "dark") {
    return preference;
  }

  return systemPrefersDark ? "dark" : "light";
}
