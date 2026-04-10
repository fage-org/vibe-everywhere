import { createContext, useContext, useEffect, useState, type ReactNode } from "react";

export type ColorScheme = "light" | "dark" | "system";

interface ThemeContextValue {
  /** Current color scheme */
  colorScheme: ColorScheme;
  /** Currently applied theme (resolved from system preference if needed) */
  theme: "light" | "dark";
  /** Set the color scheme */
  setColorScheme: (scheme: ColorScheme) => void;
  /** Toggle between light and dark */
  toggleTheme: () => void;
  /** True if the theme is currently dark */
  isDark: boolean;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

interface ThemeProviderProps {
  children: ReactNode;
  /** Default color scheme */
  defaultScheme?: ColorScheme;
  /** Storage key for persisting preference */
  storageKey?: string;
}

/**
 * ThemeProvider - Manages theme state and CSS variable injection
 *
 * Matches Happy's theme behavior:
 * - Supports light, dark, and system modes
 * - Persists preference to localStorage
 * - Respects system preference when in "system" mode
 * - Applies data-theme attribute to document element
 */
export function ThemeProvider({
  children,
  defaultScheme = "system",
  storageKey = "vibe-theme",
}: ThemeProviderProps) {
  const [colorScheme, setColorSchemeState] = useState<ColorScheme>(() => {
    if (typeof window === "undefined") return defaultScheme;
    try {
      const stored = localStorage.getItem(storageKey) as ColorScheme | null;
      return stored ?? defaultScheme;
    } catch {
      return defaultScheme;
    }
  });

  const [systemTheme, setSystemTheme] = useState<"light" | "dark">(() => {
    if (typeof window === "undefined") return "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  });

  // Resolve the actual theme to apply
  const theme = colorScheme === "system" ? systemTheme : colorScheme;
  const isDark = theme === "dark";

  // Persist preference to localStorage
  useEffect(() => {
    try {
      localStorage.setItem(storageKey, colorScheme);
    } catch {
      // Ignore storage errors
    }
  }, [colorScheme, storageKey]);

  // Listen for system theme changes
  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = (e: MediaQueryListEvent) => {
      setSystemTheme(e.matches ? "dark" : "light");
    };

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  }, []);

  // Apply theme to document
  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme);
    document.documentElement.style.colorScheme = theme;
  }, [theme]);

  const setColorScheme = (scheme: ColorScheme) => {
    setColorSchemeState(scheme);
  };

  const toggleTheme = () => {
    if (colorScheme === "system") {
      // When in system mode, toggle to the opposite of current system theme
      setColorSchemeState(systemTheme === "dark" ? "light" : "dark");
    } else {
      // Toggle between light and dark
      setColorSchemeState(colorScheme === "dark" ? "light" : "dark");
    }
  };

  const value: ThemeContextValue = {
    colorScheme,
    theme,
    setColorScheme,
    toggleTheme,
    isDark,
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

/**
 * Hook to access theme context
 * @throws Error if used outside of ThemeProvider
 */
export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error("useTheme must be used within a ThemeProvider");
  }
  return context;
}

/**
 * Hook that returns just the current theme (light/dark)
 * Safe to use without ThemeProvider (defaults to dark)
 */
export function useCurrentTheme(): "light" | "dark" {
  const context = useContext(ThemeContext);
  return context?.theme ?? "dark";
}

/**
 * Hook that returns whether the current theme is dark
 * Safe to use without ThemeProvider (defaults to true)
 */
export function useIsDark(): boolean {
  const context = useContext(ThemeContext);
  return context?.isDark ?? true;
}
