/**
 * Design Tokens for vibe-app-tauri
 * Aligned with Happy's design system (happy.ai)
 *
 * This file contains the single source of truth for all visual design values.
 * All components should reference these tokens rather than hardcoding values.
 */

// =============================================================================
// Color Tokens
// =============================================================================

export const colors = {
  // Base colors
  black: "#000000",
  white: "#ffffff",

  // Gray scale (iOS-style grays)
  gray: {
    50: "#f2f2f7",
    100: "#e5e5ea",
    200: "#d1d1d6",
    300: "#c7c7cc",
    400: "#aeaeb2",
    500: "#8e8e93",
    600: "#636366",
    700: "#48484a",
    800: "#3a3a3c",
    900: "#2c2c2e",
    950: "#1c1c1e",
  },

  // Semantic colors (iOS system colors)
  red: {
    light: "#ff3b30",
    dark: "#ff453a",
  },
  orange: {
    light: "#ff9500",
    dark: "#ff9f0a",
  },
  yellow: {
    light: "#ffcc00",
    dark: "#ffd60a",
  },
  green: {
    light: "#34c759",
    dark: "#30d158",
  },
  teal: {
    light: "#5ac8fa",
    dark: "#64d2ff",
  },
  blue: {
    light: "#007aff",
    dark: "#0a84ff",
  },
  indigo: {
    light: "#5856d6",
    dark: "#5e5ce6",
  },
  purple: {
    light: "#af52de",
    dark: "#bf5af2",
  },
  pink: {
    light: "#ff2d55",
    dark: "#ff375f",
  },
  brown: {
    light: "#a2845e",
    dark: "#ac8e68",
  },

  // Semantic mappings
  primary: {
    light: "#007aff",
    dark: "#0a84ff",
  },
  success: {
    light: "#34c759",
    dark: "#30d158",
  },
  warning: {
    light: "#ff9500",
    dark: "#ff9f0a",
  },
  danger: {
    light: "#ff3b30",
    dark: "#ff453a",
  },
  info: {
    light: "#5ac8fa",
    dark: "#64d2ff",
  },
} as const;

// =============================================================================
// Theme Tokens (Backgrounds, Surfaces, Text)
// =============================================================================

export const theme = {
  dark: {
    // Backgrounds
    background: {
      primary: "#000000",
      secondary: "#1c1c1e",
      tertiary: "#2c2c2e",
      elevated: "#1c1c1e",
    },
    // Surfaces (cards, panels)
    surface: {
      primary: "#1c1c1e",
      secondary: "#2c2c2e",
      tertiary: "#3a3a3c",
      elevated: "#2c2c2e",
    },
    // Text
    text: {
      primary: "#ffffff",
      secondary: "#ebebf5",
      tertiary: "#8e8e93",
      quaternary: "#636366",
      placeholder: "#8e8e93",
      disabled: "#636366",
    },
    // Borders
    border: {
      primary: "#38383a",
      secondary: "#48484a",
      tertiary: "#636366",
    },
    // Separators
    separator: {
      primary: "#38383a",
      secondary: "#2c2c2e",
    },
    // Overlays
    overlay: {
      thin: "rgba(0, 0, 0, 0.25)",
      thick: "rgba(0, 0, 0, 0.5)",
      ultraThick: "rgba(0, 0, 0, 0.75)",
    },
  },
  light: {
    // Backgrounds
    background: {
      primary: "#f2f2f7",
      secondary: "#ffffff",
      tertiary: "#f2f2f7",
      elevated: "#ffffff",
    },
    // Surfaces
    surface: {
      primary: "#ffffff",
      secondary: "#f2f2f7",
      tertiary: "#e5e5ea",
      elevated: "#ffffff",
    },
    // Text
    text: {
      primary: "#000000",
      secondary: "#3c3c43",
      tertiary: "#8e8e93",
      quaternary: "#c7c7cc",
      placeholder: "#8e8e93",
      disabled: "#c7c7cc",
    },
    // Borders
    border: {
      primary: "#c7c7cc",
      secondary: "#d1d1d6",
      tertiary: "#e5e5ea",
    },
    // Separators
    separator: {
      primary: "#c7c7cc",
      secondary: "#e5e5ea",
    },
    // Overlays
    overlay: {
      thin: "rgba(0, 0, 0, 0.1)",
      thick: "rgba(0, 0, 0, 0.25)",
      ultraThick: "rgba(0, 0, 0, 0.5)",
    },
  },
} as const;

// =============================================================================
// Typography Tokens
// =============================================================================

export const typography = {
  // Font families
  fontFamily: {
    sans: '"SF Pro Display", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    mono: '"SF Mono", "SFMono-Regular", ui-monospace, monospace',
    display: '"SF Pro Display", -apple-system, BlinkMacSystemFont, sans-serif',
  },

  // Font sizes (in rem)
  fontSize: {
    xs: "0.75rem",      // 12px
    sm: "0.8125rem",    // 13px
    base: "0.9375rem",  // 15px (iOS body)
    lg: "1.0625rem",    // 17px (iOS body large)
    xl: "1.25rem",      // 20px
    "2xl": "1.5rem",    // 24px
    "3xl": "1.75rem",   // 28px
    "4xl": "2rem",      // 32px
    "5xl": "2.5rem",    // 40px
  },

  // Font weights
  fontWeight: {
    regular: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },

  // Line heights
  lineHeight: {
    tight: 1.2,
    snug: 1.3,
    normal: 1.4,
    relaxed: 1.5,
    loose: 1.6,
  },

  // Letter spacing
  letterSpacing: {
    tighter: "-0.05em",
    tight: "-0.025em",
    normal: "0",
    wide: "0.025em",
  },

  // Text styles (combinations)
  textStyle: {
    // Large titles
    largeTitle: {
      fontSize: "2.125rem",  // 34px
      fontWeight: 700,
      lineHeight: 1.2,
      letterSpacing: "-0.02em",
    },
    // Titles
    title1: {
      fontSize: "1.75rem",   // 28px
      fontWeight: 700,
      lineHeight: 1.25,
      letterSpacing: "-0.02em",
    },
    title2: {
      fontSize: "1.375rem",  // 22px
      fontWeight: 700,
      lineHeight: 1.3,
      letterSpacing: "-0.01em",
    },
    title3: {
      fontSize: "1.25rem",   // 20px
      fontWeight: 600,
      lineHeight: 1.3,
      letterSpacing: "-0.01em",
    },
    // Headlines
    headline: {
      fontSize: "1.0625rem", // 17px
      fontWeight: 600,
      lineHeight: 1.3,
      letterSpacing: "-0.01em",
    },
    // Body
    body: {
      fontSize: "1.0625rem", // 17px
      fontWeight: 400,
      lineHeight: 1.4,
      letterSpacing: "-0.01em",
    },
    bodyBold: {
      fontSize: "1.0625rem", // 17px
      fontWeight: 600,
      lineHeight: 1.4,
      letterSpacing: "-0.01em",
    },
    // Callout
    callout: {
      fontSize: "1rem",      // 16px
      fontWeight: 400,
      lineHeight: 1.4,
      letterSpacing: "-0.01em",
    },
    calloutBold: {
      fontSize: "1rem",      // 16px
      fontWeight: 600,
      lineHeight: 1.4,
      letterSpacing: "-0.01em",
    },
    // Subheadline
    subheadline: {
      fontSize: "0.9375rem", // 15px
      fontWeight: 400,
      lineHeight: 1.4,
      letterSpacing: "-0.01em",
    },
    subheadlineBold: {
      fontSize: "0.9375rem", // 15px
      fontWeight: 600,
      lineHeight: 1.4,
      letterSpacing: "-0.01em",
    },
    // Footnote
    footnote: {
      fontSize: "0.8125rem", // 13px
      fontWeight: 400,
      lineHeight: 1.4,
      letterSpacing: "0",
    },
    footnoteBold: {
      fontSize: "0.8125rem", // 13px
      fontWeight: 600,
      lineHeight: 1.4,
      letterSpacing: "0",
    },
    // Caption
    caption1: {
      fontSize: "0.75rem",   // 12px
      fontWeight: 400,
      lineHeight: 1.3,
      letterSpacing: "0",
    },
    caption2: {
      fontSize: "0.6875rem", // 11px
      fontWeight: 400,
      lineHeight: 1.3,
      letterSpacing: "0.01em",
    },
  },
} as const;

// =============================================================================
// Spacing Tokens (4px base grid)
// =============================================================================

export const spacing = {
  0: "0",
  0.5: "0.125rem",  // 2px
  1: "0.25rem",     // 4px
  1.5: "0.375rem",  // 6px
  2: "0.5rem",      // 8px
  2.5: "0.625rem",  // 10px
  3: "0.75rem",     // 12px
  3.5: "0.875rem",  // 14px
  4: "1rem",        // 16px
  5: "1.25rem",     // 20px
  6: "1.5rem",      // 24px
  7: "1.75rem",     // 28px
  8: "2rem",        // 32px
  9: "2.25rem",     // 36px
  10: "2.5rem",     // 40px
  11: "2.75rem",    // 44px
  12: "3rem",       // 48px
  14: "3.5rem",     // 56px
  16: "4rem",       // 64px
  20: "5rem",       // 80px
  24: "6rem",       // 96px
} as const;

// =============================================================================
// Border Radius Tokens
// =============================================================================

export const radii = {
  none: "0",
  xs: "4px",
  sm: "6px",
  md: "8px",
  lg: "10px",
  xl: "12px",
  "2xl": "16px",
  "3xl": "20px",
  "4xl": "24px",
  full: "9999px",
} as const;

// =============================================================================
// Shadow Tokens
// =============================================================================

export const shadows = {
  none: "none",
  xs: "0 1px 2px rgba(0, 0, 0, 0.05)",
  sm: "0 1px 3px rgba(0, 0, 0, 0.1), 0 1px 2px rgba(0, 0, 0, 0.06)",
  md: "0 4px 6px rgba(0, 0, 0, 0.1), 0 2px 4px rgba(0, 0, 0, 0.06)",
  lg: "0 10px 15px rgba(0, 0, 0, 0.1), 0 4px 6px rgba(0, 0, 0, 0.05)",
  xl: "0 20px 25px rgba(0, 0, 0, 0.1), 0 10px 10px rgba(0, 0, 0, 0.04)",
  "2xl": "0 25px 50px rgba(0, 0, 0, 0.25)",
  inner: "inset 0 2px 4px rgba(0, 0, 0, 0.06)",
  // iOS-style shadows
  ios: {
    small: "0 2px 8px rgba(0, 0, 0, 0.12)",
    medium: "0 4px 16px rgba(0, 0, 0, 0.16)",
    large: "0 8px 32px rgba(0, 0, 0, 0.2)",
  },
} as const;

// =============================================================================
// Animation Tokens
// =============================================================================

export const animation = {
  duration: {
    instant: "0ms",
    fast: "100ms",
    normal: "200ms",
    slow: "300ms",
    slower: "400ms",
  },
  easing: {
    default: "cubic-bezier(0.4, 0, 0.2, 1)",
    in: "cubic-bezier(0.4, 0, 1, 1)",
    out: "cubic-bezier(0, 0, 0.2, 1)",
    inOut: "cubic-bezier(0.4, 0, 0.2, 1)",
    bounce: "cubic-bezier(0.68, -0.55, 0.265, 1.55)",
    // iOS-style easing
    ios: "cubic-bezier(0.32, 0.72, 0, 1)",
  },
} as const;

// =============================================================================
// Z-Index Tokens
// =============================================================================

export const zIndex = {
  hide: -1,
  base: 0,
  docked: 10,
  dropdown: 1000,
  sticky: 1100,
  banner: 1200,
  overlay: 1300,
  modal: 1400,
  popover: 1500,
  skipLink: 1600,
  toast: 1700,
  tooltip: 1800,
} as const;

// =============================================================================
// Layout Tokens
// =============================================================================

export const layout = {
  // Container widths
  container: {
    sm: "640px",
    md: "768px",
    lg: "1024px",
    xl: "1280px",
    "2xl": "1536px",
  },
  // Sidebar widths
  sidebar: {
    collapsed: "64px",
    narrow: "200px",
    default: "260px",
    wide: "320px",
  },
  // Header heights
  header: {
    compact: "44px",
    default: "52px",
    comfortable: "64px",
  },
  // Touch targets (minimum 44px for iOS)
  touchTarget: {
    min: "44px",
    comfortable: "48px",
  },
} as const;

// =============================================================================
// Component-Specific Tokens
// =============================================================================

export const components = {
  // Button
  button: {
    height: {
      sm: "28px",
      md: "36px",
      lg: "44px",
    },
    padding: {
      sm: "0 12px",
      md: "0 16px",
      lg: "0 20px",
    },
    radius: "8px",
  },
  // Input
  input: {
    height: {
      sm: "32px",
      md: "44px",
      lg: "52px",
    },
    padding: "0 12px",
    radius: "10px",
  },
  // Card
  card: {
    radius: "12px",
    padding: "16px",
  },
  // Badge
  badge: {
    height: "20px",
    padding: "0 8px",
    radius: "10px",
  },
} as const;

// =============================================================================
// Export all tokens
// =============================================================================

export const tokens = {
  colors,
  theme,
  typography,
  spacing,
  radii,
  shadows,
  animation,
  zIndex,
  layout,
  components,
} as const;

export type Tokens = typeof tokens;
export type ColorScheme = "light" | "dark";
