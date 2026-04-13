import { useState, useEffect, useCallback } from "react";
import {
  breakpoints,
  mediaQueries,
  getActiveBreakpoint,
  type BreakpointKey,
} from "../design-system/breakpoints";

/**
 * Hook for responsive media queries
 *
 * @param query - CSS media query string
 * @returns boolean indicating if the query matches
 *
 * @example
 * ```tsx
 * const isMobile = useMediaQuery("(max-width: 768px)");
 * const isLarge = useMediaQuery(mediaQueries.lg);
 * ```
 */
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() => {
    // Check if window is available (SSR compatibility)
    if (typeof window !== "undefined") {
      return window.matchMedia(query).matches;
    }
    return false;
  });

  useEffect(() => {
    if (typeof window === "undefined") return;

    const mediaQuery = window.matchMedia(query);
    setMatches(mediaQuery.matches);

    const handler = (event: MediaQueryListEvent) => {
      setMatches(event.matches);
    };

    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  }, [query]);

  return matches;
}

/**
 * Hook for checking if viewport matches a breakpoint
 *
 * @param breakpoint - Breakpoint key (sm, md, lg, xl, 2xl, 3xl)
 * @returns boolean indicating if viewport is at or above the breakpoint
 *
 * @example
 * ```tsx
 * const isDesktop = useBreakpoint("lg");
 * ```
 */
export function useBreakpoint(breakpoint: BreakpointKey): boolean {
  return useMediaQuery(mediaQueries[breakpoint]);
}

/**
 * Hook for getting the current active breakpoint
 *
 * @returns Current breakpoint key
 *
 * @example
 * ```tsx
 * const breakpoint = useActiveBreakpoint();
 * // "sm" | "md" | "lg" | "xl" | "2xl" | "3xl"
 * ```
 */
export function useActiveBreakpoint(): BreakpointKey {
  const [breakpoint, setBreakpoint] = useState<BreakpointKey>(() => {
    if (typeof window !== "undefined") {
      return getActiveBreakpoint(window.innerWidth);
    }
    return "sm";
  });

  useEffect(() => {
    if (typeof window === "undefined") return;

    const handleResize = () => {
      setBreakpoint(getActiveBreakpoint(window.innerWidth));
    };

    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  return breakpoint;
}

/**
 * Hook for responsive value selection
 *
 * @param values - Object with values for each breakpoint (base is required)
 * @returns The value for the current breakpoint
 *
 * @example
 * ```tsx
 * const columns = useResponsiveValue({
 *   base: 1,
 *   md: 2,
 *   lg: 3,
 *   xl: 4,
 * });
 * ```
 */
export function useResponsiveValue<T>(
  values: Partial<Record<BreakpointKey, T>> & { base: T }
): T {
  const breakpoint = useActiveBreakpoint();

  const breakpointOrder: BreakpointKey[] = [
    "3xl",
    "2xl",
    "xl",
    "lg",
    "md",
    "sm",
  ];

  const activeIndex = breakpointOrder.indexOf(breakpoint);

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
 * Hook for checking if viewport is mobile-sized
 *
 * @returns boolean indicating if viewport is below md breakpoint
 */
export function useIsMobile(): boolean {
  return !useBreakpoint("md");
}

/**
 * Hook for checking if viewport is tablet-sized
 *
 * @returns boolean indicating if viewport is between md and xl breakpoints
 */
export function useIsTablet(): boolean {
  const isMdOrLarger = useBreakpoint("md");
  const isXlOrLarger = useBreakpoint("xl");
  return isMdOrLarger && !isXlOrLarger;
}

/**
 * Hook for checking if viewport is desktop-sized
 *
 * @returns boolean indicating if viewport is at or above xl breakpoint
 */
export function useIsDesktop(): boolean {
  return useBreakpoint("xl");
}

/**
 * Hook for window dimensions
 *
 * @returns Object with width and height
 */
export function useWindowSize(): { width: number; height: number } {
  const [size, setSize] = useState<{ width: number; height: number }>(() => {
    if (typeof window !== "undefined") {
      return {
        width: window.innerWidth,
        height: window.innerHeight,
      };
    }
    return { width: 0, height: 0 };
  });

  useEffect(() => {
    if (typeof window === "undefined") return;

    const handleResize = () => {
      setSize({
        width: window.innerWidth,
        height: window.innerHeight,
      });
    };

    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, []);

  return size;
}

/**
 * Hook for responsive layout state
 *
 * @returns Layout state object with various flags
 */
export function useResponsiveLayout(): {
  breakpoint: BreakpointKey;
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  showMobileNav: boolean;
  showSidebarOverlay: boolean;
  width: number;
  height: number;
} {
  const breakpoint = useActiveBreakpoint();
  const isMobile = useIsMobile();
  const isTablet = useIsTablet();
  const isDesktop = useIsDesktop();
  const { width, height } = useWindowSize();

  return {
    breakpoint,
    isMobile,
    isTablet,
    isDesktop,
    showMobileNav: isMobile,
    showSidebarOverlay: width < breakpoints.md,
    width,
    height,
  };
}
