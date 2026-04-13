import { type ReactNode, type CSSProperties } from "react";
import { tokens } from "../../design-system/tokens";

export interface ShimmerViewProps {
  children: ReactNode;
  shimmerColors?: readonly [string, string, ...string[]];
  shimmerWidthPercent?: number;
  duration?: number;
  style?: CSSProperties;
}

/**
 * ShimmerView - Skeleton loading animation component
 *
 * Features:
 * - Displays children outline with shimmer animation
 * - Customizable colors and animation speed
 * - Uses CSS animation for performance
 *
 * Usage:
 * ```tsx
 * <ShimmerView>
 *   <div style={{ width: 200, height: 20, borderRadius: 4 }} />
 * </ShimmerView>
 * ```
 */
export function ShimmerView({
  children,
  shimmerColors = [
    "var(--surface-tertiary)",
    "var(--surface-secondary)",
    "var(--surface-primary)",
    "var(--surface-secondary)",
    "var(--surface-tertiary)",
  ],
  shimmerWidthPercent = 80,
  duration = 1500,
  style,
}: ShimmerViewProps) {
  // Create gradient string from colors
  const gradientStops = shimmerColors
    .map((color, index) => {
      const percent = (index / (shimmerColors.length - 1)) * 100;
      return `${color} ${percent}%`;
    })
    .join(", ");

  const shimmerWidth = shimmerWidthPercent;

  const containerStyles: CSSProperties = {
    position: "relative",
    overflow: "hidden",
    ...style,
  };

  const backgroundStyles: CSSProperties = {
    backgroundColor: shimmerColors[0],
  };

  const shimmerStyles: CSSProperties = {
    position: "absolute",
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundImage: `linear-gradient(90deg, ${gradientStops})`,
    backgroundSize: `${shimmerWidth}% 100%`,
    backgroundRepeat: "no-repeat",
    animation: `shimmer-slide ${duration}ms linear infinite`,
  };

  // Hidden children to establish size
  const hiddenStyles: CSSProperties = {
    visibility: "hidden",
    pointerEvents: "none",
  };

  // Mask container to show children shape
  const maskStyles: CSSProperties = {
    position: "absolute",
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    overflow: "hidden",
  };

  return (
    <div style={containerStyles}>
      {/* Hidden children to establish size */}
      <div style={hiddenStyles}>{children}</div>

      {/* Shimmer overlay */}
      <div style={maskStyles}>
        {/* Base background */}
        <div style={{ ...backgroundStyles, width: "100%", height: "100%" }} />

        {/* Animated shimmer */}
        <div style={shimmerStyles} />
      </div>
    </div>
  );
}

// CSS keyframes animation (inject via style tag or global CSS)
// This component relies on the following CSS being present:
//
// @keyframes shimmer-slide {
//   0% { backgroundPosition: -100% 0; }
//   100% { backgroundPosition: 200% 0; }
// }

/**
 * ShimmerText - Shimmer loading for text placeholder
 */
export interface ShimmerTextProps {
  width?: number | string;
  height?: number | string;
  borderRadius?: number | string;
  style?: CSSProperties;
}

export function ShimmerText({
  width = "100%",
  height = 20,
  borderRadius = 4,
  style,
}: ShimmerTextProps) {
  return (
    <ShimmerView style={style}>
      <div
        style={{
          width,
          height,
          borderRadius,
          backgroundColor: "var(--surface-tertiary)",
        }}
      />
    </ShimmerView>
  );
}

/**
 * ShimmerAvatar - Shimmer loading for avatar placeholder
 */
export interface ShimmerAvatarProps {
  size?: number;
  style?: CSSProperties;
}

export function ShimmerAvatar({ size = 48, style }: ShimmerAvatarProps) {
  return (
    <ShimmerView style={style}>
      <div
        style={{
          width: size,
          height: size,
          borderRadius: size / 2,
          backgroundColor: "var(--surface-tertiary)",
        }}
      />
    </ShimmerView>
  );
}

/**
 * ShimmerCard - Shimmer loading for card placeholder
 */
export interface ShimmerCardProps {
  width?: number | string;
  height?: number | string;
  style?: CSSProperties;
}

export function ShimmerCard({
  width = "100%",
  height = 100,
  style,
}: ShimmerCardProps) {
  return (
    <ShimmerView style={{ ...style, borderRadius: tokens.radii.lg }}>
      <div
        style={{
          width,
          height,
          borderRadius: tokens.radii.lg,
          backgroundColor: "var(--surface-tertiary)",
        }}
      />
    </ShimmerView>
  );
}