import { memo, type CSSProperties } from "react";

/**
 * Generate a consistent hash code from a string
 */
function hashCode(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash;
  }
  return Math.abs(hash);
}

/**
 * Generate a gradient background based on ID
 * Creates consistent, unique gradients for each ID
 */
function generateGradient(id: string): { from: string; to: string } {
  const hash = hashCode(id);

  // Color palettes for gradient generation
  const colors = [
    ["#667eea", "#764ba2"],
    ["#f093fb", "#f5576c"],
    ["#4facfe", "#00f2fe"],
    ["#43e97b", "#38f9d7"],
    ["#fa709a", "#fee140"],
    ["#a8edea", "#fed6e3"],
    ["#ff9a9e", "#fecfef"],
    ["#ffecd2", "#fcb69f"],
    ["#a1c4fd", "#c2e9fb"],
    ["#d299c2", "#fef9d7"],
    ["#89f7fe", "#66a6ff"],
    ["#cd9cf2", "#f6f3ff"],
    ["#fddb92", "#d1fdff"],
    ["#96fbc4", "#f9f586"],
    ["#e0c3fc", "#8ec5fc"],
    ["#f5f7fa", "#c3cfe2"],
    ["#667eea", "#764ba2"],
    ["#ff0844", "#ffb199"],
    ["#b721ff", "#21d4fd"],
    ["#6a11cb", "#2575fc"],
  ];

  return {
    from: colors[hash % colors.length][0],
    to: colors[hash % colors.length][1],
  };
}

export interface AvatarGradientProps {
  /** Unique identifier for consistent gradient generation */
  id: string;
  /** Square avatar instead of circle */
  square?: boolean;
  /** Size in pixels */
  size?: number;
  /** Monochrome mode */
  monochrome?: boolean;
}

/**
 * AvatarGradient - Gradient-based avatar component
 *
 * Generates consistent, unique gradient backgrounds based on the provided ID.
 * Uses a hash function to ensure the same ID always produces the same gradient.
 */
export const AvatarGradient = memo(function AvatarGradient(
  props: AvatarGradientProps
) {
  const { id, square, size = 48, monochrome } = props;

  const gradient = generateGradient(id);

  const containerStyles: CSSProperties = {
    width: size,
    height: size,
    borderRadius: square ? 0 : size / 2,
    background: monochrome
      ? "var(--text-tertiary)"
      : `linear-gradient(135deg, ${gradient.from}, ${gradient.to})`,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    overflow: "hidden",
    flexShrink: 0,
  };

  return <div style={containerStyles} />;
});
