/**
 * Local Settings Context
 *
 * Provides global access to local settings throughout the component tree.
 * Components can use `useLocalSettingsContext()` to access settings without prop drilling.
 *
 * Usage:
 * ```tsx
 * // At the app root
 * <LocalSettingsProvider>
 *   <App />
 * </LocalSettingsProvider>
 *
 * // In any component
 * const { settings } = useLocalSettingsContext();
 * ```
 */

import { createContext, useContext, type ReactNode } from "react";
import {
  useLocalSettings,
  type LocalSettings,
  localSettingsDefaults,
} from "./local-settings";

interface LocalSettingsContextValue {
  settings: LocalSettings;
  updateSettings: (updates: Partial<LocalSettings>) => void;
  resetSettings: () => void;
}

const LocalSettingsContext = createContext<LocalSettingsContextValue>({
  settings: localSettingsDefaults,
  updateSettings: () => {
    console.warn("LocalSettingsContext: updateSettings called outside of provider");
  },
  resetSettings: () => {
    console.warn("LocalSettingsContext: resetSettings called outside of provider");
  },
});

LocalSettingsContext.displayName = "LocalSettingsContext";

export function LocalSettingsProvider({ children }: { children: ReactNode }) {
  const { settings, updateSettings, resetSettings } = useLocalSettings();

  return (
    <LocalSettingsContext.Provider
      value={{ settings, updateSettings, resetSettings }}
    >
      {children}
    </LocalSettingsContext.Provider>
  );
}

/**
 * Hook to access local settings from context
 */
export function useLocalSettingsContext(): LocalSettingsContextValue {
  const context = useContext(LocalSettingsContext);
  if (!context) {
    // Return defaults if used outside provider (e.g., in tests)
    return {
      settings: localSettingsDefaults,
      updateSettings: () => {},
      resetSettings: () => {},
    };
  }
  return context;
}

/**
 * Hook to get render options from local settings
 *
 * This is a convenience hook for components that need rendering-related settings.
 */
export function useRichRenderOptions(): {
  showLineNumbersInDiffs: boolean;
  showLineNumbersInToolViews: boolean;
  wrapLinesInDiffs: boolean;
} {
  const { settings } = useLocalSettingsContext();
  return {
    showLineNumbersInDiffs: settings.showLineNumbers,
    showLineNumbersInToolViews: settings.showLineNumbersInToolViews,
    wrapLinesInDiffs: settings.wrapLinesInDiffs,
  };
}

export { LocalSettingsContext };
