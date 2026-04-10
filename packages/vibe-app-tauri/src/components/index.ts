// Main component exports for vibe-app-tauri
// Aligned with Happy's design system (happy.ai)

// Design System
export * from "../design-system/tokens";

// Providers
export { ThemeProvider, useTheme, useCurrentTheme, useIsDark } from "./providers/ThemeProvider";
export type { ColorScheme, ThemeContextValue } from "./providers/ThemeProvider";

// UI Primitives
export * from "./ui";

// Layout
export * from "./layout";

// Surfaces
export * from "./surfaces";

// Renderers
export * from "./renderers";

// Routes
export * from "./routes";
