/**
 * Breakpoint System for vibe-app-tauri
 *
 * Provides responsive breakpoint values and utilities for media queries.
 * Aligned with common responsive design patterns.
 */

export const breakpoints = {
  /** Small devices (phones, 320px and up) */
  sm: 320,
  /** Medium devices (tablets portrait, 640px and up) */
  md: 640,
  /** Large devices (tablets landscape, small laptops, 768px and up) */
  lg: 768,
  /** Extra large devices (desktops, 1024px and up) */
  xl: 1024,
  /** 2XL devices (large desktops, 1280px and up) */
  "2xl": 1280,
  /** 3XL devices (extra large desktops, 1536px and up) */
  "3xl": 1536,
} as const;

export type BreakpointKey = keyof typeof breakpoints;

/**
 * Media query strings for each breakpoint
 */
export const mediaQueries: Record<BreakpointKey, string> = {
  sm: `(min-width: ${breakpoints.sm}px)`,
  md: `(min-width: ${breakpoints.md}px)`,
  lg: `(min-width: ${breakpoints.lg}px)`,
  xl: `(min-width: ${breakpoints.xl}px)`,
  "2xl": `(min-width: ${breakpoints["2xl"]}px)`,
  "3xl": `(min-width: ${breakpoints["3xl"]}px)`,
};

/**
 * Get the current active breakpoint based on window width
 */
export function getActiveBreakpoint(width: number): BreakpointKey {
  if (width >= breakpoints["3xl"]) return "3xl";
  if (width >= breakpoints["2xl"]) return "2xl";
  if (width >= breakpoints.xl) return "xl";
  if (width >= breakpoints.lg) return "lg";
  if (width >= breakpoints.md) return "md";
  return "sm";
}

/**
 * Check if current width matches a breakpoint
 */
export function isBreakpoint(width: number, breakpoint: BreakpointKey): boolean {
  return width >= breakpoints[breakpoint];
}

/**
 * Get responsive value based on breakpoint
 */
export function getResponsiveValue<T>(
  width: number,
  values: Partial<Record<BreakpointKey, T>> & { base: T }
): T {
  const activeBreakpoint = getActiveBreakpoint(width);
  const breakpointOrder: BreakpointKey[] = [
    "3xl",
    "2xl",
    "xl",
    "lg",
    "md",
    "sm",
  ];

  const activeIndex = breakpointOrder.indexOf(activeBreakpoint);

  // Find the first matching value from current breakpoint down
  for (let i = activeIndex; i < breakpointOrder.length; i++) {
    const bp = breakpointOrder[i];
    if (values[bp] !== undefined) {
      return values[bp]!;
    }
  }

  return values.base;
}

/**
 * Container max-widths for each breakpoint
 */
export const containerWidths: Record<BreakpointKey, string> = {
  sm: "100%",
  md: "640px",
  lg: "768px",
  xl: "1024px",
  "2xl": "1280px",
  "3xl": "1536px",
};

/**
 * Sidebar widths for each breakpoint
 */
export const sidebarWidths: Record<BreakpointKey, string> = {
  sm: "100%", // Full screen on mobile
  md: "280px",
  lg: "280px",
  xl: "320px",
  "2xl": "320px",
  "3xl": "360px",
};

/**
 * Layout configuration for responsive behavior
 */
export const layoutConfig = {
  /** Show sidebar as overlay below this width */
  sidebarOverlayBreakpoint: "md" as BreakpointKey,
  /** Collapse sidebar to icons below this width */
  sidebarCollapseBreakpoint: "lg" as BreakpointKey,
  /** Show mobile navigation below this width */
  mobileNavBreakpoint: "md" as BreakpointKey,
  /** Use compact layout below this width */
  compactBreakpoint: "sm" as BreakpointKey,
} as const;

export type LayoutConfig = typeof layoutConfig;
