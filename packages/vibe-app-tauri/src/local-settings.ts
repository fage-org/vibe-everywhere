/**
 * Local Settings - Client-side only preferences
 *
 * These settings are stored locally on the device and are NOT synced to the server.
 * They represent device-specific preferences that make sense to be different on each device.
 *
 * For server-synced settings, see the account settings API.
 */

import { z } from "zod";

//
// Schema Definition
//

export const LocalSettingsSchema = z.object({
  // Theme preferences
  themePreference: z
    .enum(["light", "dark", "adaptive"])
    .describe("Theme preference: light, dark, or adaptive (follows system)"),

  // Display preferences
  viewInline: z.boolean().describe("Whether to view inline tool calls"),
  expandTodos: z.boolean().describe("Whether to expand todo lists"),
  showLineNumbers: z.boolean().describe("Whether to show line numbers in diffs"),
  showLineNumbersInToolViews: z
    .boolean()
    .describe("Whether to show line numbers in tool view diffs"),
  wrapLinesInDiffs: z
    .boolean()
    .describe("Whether to wrap long lines in diff views"),
  alwaysShowContextSize: z
    .boolean()
    .describe("Always show context size in agent input"),

  // Avatar preferences
  avatarStyle: z
    .enum(["pixelated", "gradient", "brutalist"])
    .describe("Avatar display style"),
  showFlavorIcons: z
    .boolean()
    .describe("Whether to show AI provider icons in avatars"),

  // Session list preferences
  compactSessionView: z
    .boolean()
    .describe("Whether to use compact view for active sessions"),
  hideInactiveSessions: z
    .boolean()
    .describe("Hide inactive sessions in the main list"),

  // Input preferences
  agentInputEnterToSend: z
    .boolean()
    .describe("Whether pressing Enter submits/sends in the agent input (web)"),

  // Developer preferences
  debugMode: z.boolean().describe("Enable debug logging"),
  devModeEnabled: z.boolean().describe("Enable developer menu in settings"),

  // Language preference
  preferredLanguage: z
    .string()
    .nullable()
    .describe("Preferred UI language (null for auto-detect from device locale)"),
});

const LocalSettingsSchemaPartial = LocalSettingsSchema.passthrough().partial();

export type LocalSettings = z.infer<typeof LocalSettingsSchema>;

//
// Default Values
//

export const localSettingsDefaults: LocalSettings = {
  themePreference: "adaptive",
  viewInline: false,
  expandTodos: true,
  showLineNumbers: true,
  showLineNumbersInToolViews: false,
  wrapLinesInDiffs: false,
  alwaysShowContextSize: false,
  avatarStyle: "brutalist",
  showFlavorIcons: false,
  compactSessionView: false,
  hideInactiveSessions: false,
  agentInputEnterToSend: true,
  debugMode: false,
  devModeEnabled: false,
  preferredLanguage: null,
};

Object.freeze(localSettingsDefaults);

//
// Storage Key
//

const LOCAL_SETTINGS_KEY = "vibe-local-settings";

//
// Parsing
//

export function localSettingsParse(settings: unknown): LocalSettings {
  if (!settings || typeof settings !== "object") {
    return { ...localSettingsDefaults };
  }

  const parsed = LocalSettingsSchemaPartial.safeParse(settings);
  if (!parsed.success) {
    return { ...localSettingsDefaults };
  }

  return { ...localSettingsDefaults, ...parsed.data };
}

//
// Applying Changes
//

export function applyLocalSettings(
  settings: LocalSettings,
  delta: Partial<LocalSettings>
): LocalSettings {
  return { ...localSettingsDefaults, ...settings, ...delta };
}

//
// Storage Operations
//

/**
 * Load local settings from localStorage
 */
export function loadLocalSettings(): LocalSettings {
  try {
    const stored = localStorage.getItem(LOCAL_SETTINGS_KEY);
    if (!stored) {
      return { ...localSettingsDefaults };
    }
    const parsed = JSON.parse(stored);
    return localSettingsParse(parsed);
  } catch {
    return { ...localSettingsDefaults };
  }
}

/**
 * Save local settings to localStorage
 */
export function saveLocalSettings(settings: LocalSettings): void {
  try {
    localStorage.setItem(LOCAL_SETTINGS_KEY, JSON.stringify(settings));
  } catch (error) {
    console.error("Failed to save local settings:", error);
  }
}

/**
 * Update a subset of local settings
 */
export function updateLocalSettings(
  current: LocalSettings,
  updates: Partial<LocalSettings>
): LocalSettings {
  const updated = applyLocalSettings(current, updates);
  saveLocalSettings(updated);
  return updated;
}

//
// React Hook
//

import { useState, useEffect, useCallback } from "react";

/**
 * React hook for managing local settings
 */
export function useLocalSettings(): {
  settings: LocalSettings;
  updateSettings: (updates: Partial<LocalSettings>) => void;
  resetSettings: () => void;
} {
  const [settings, setSettings] = useState<LocalSettings>(() =>
    loadLocalSettings()
  );

  // Sync with localStorage changes from other tabs/windows
  useEffect(() => {
    // Skip if window is not available (e.g., in test environments)
    if (typeof window === "undefined" || typeof window.addEventListener !== "function") {
      return;
    }

    const handleStorageChange = (event: StorageEvent) => {
      if (event.key === LOCAL_SETTINGS_KEY && event.newValue) {
        try {
          const parsed = JSON.parse(event.newValue);
          setSettings(localSettingsParse(parsed));
        } catch {
          // Ignore parse errors
        }
      }
    };

    window.addEventListener("storage", handleStorageChange);
    return () => window.removeEventListener("storage", handleStorageChange);
  }, []);

  const updateSettings = useCallback((updates: Partial<LocalSettings>) => {
    setSettings((current) => {
      const updated = applyLocalSettings(current, updates);
      saveLocalSettings(updated);
      return updated;
    });
  }, []);

  const resetSettings = useCallback(() => {
    setSettings({ ...localSettingsDefaults });
    saveLocalSettings(localSettingsDefaults);
  }, []);

  return { settings, updateSettings, resetSettings };
}

export default {
  LocalSettingsSchema,
  localSettingsDefaults,
  localSettingsParse,
  applyLocalSettings,
  loadLocalSettings,
  saveLocalSettings,
  updateLocalSettings,
  useLocalSettings,
};
