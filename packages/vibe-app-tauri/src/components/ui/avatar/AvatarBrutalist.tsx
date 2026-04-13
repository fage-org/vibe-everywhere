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
 * Generate brutalist-style shape parameters based on ID
 */
function generateBrutalistShape(
  id: string
): { color: string; pattern: number; rotation: number } {
  const hash = hashCode(id);

  const colors = [
    "#ff6b6b",
    "#4ecdc4",
    "#45b7d1",
    "#96ceb4",
    "#ffeaa7",
    "#dfe6e9",
    "#fd79a8",
    "#a29bfe",
    "#6c5ce7",
    "#00b894",
  ];

  return {
    color: colors[hash % colors.length],
    pattern: hash % 4,
    rotation: (hash % 8) * 45,
  };
}

export interface AvatarBrutalistProps {
  /** Unique identifier for consistent shape generation */
  id: string;
  /** Square avatar instead of circle */
  square?: boolean;
  /** Size in pixels */
  size?: number;
  /** Monochrome mode */
  monochrome?: boolean;
}

/**
 * AvatarBrutalist - Brutalist-style avatar component
 *
 * Generates bold, geometric shapes with high contrast colors.
 * Uses a hash function to ensure consistency for the same ID.
 */
export const AvatarBrutalist = memo(function AvatarBrutalist(
  props: AvatarBrutalistProps
) {
  const { id, square, size = 48, monochrome } = props;

  const shape = generateBrutalistShape(id);

  const containerStyles: CSSProperties = {
    width: size,
    height: size,
    borderRadius: square ? 0 : size / 2,
    backgroundColor: monochrome ? "var(--surface-tertiary)" : shape.color,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    overflow: "hidden",
    position: "relative",
    flexShrink: 0,
  };

  const innerSize = size * 0.6;

  // Pattern styles based on hash
  const patternStyles: CSSProperties[] = [
    // Pattern 0: Circle
    {
      width: innerSize,
      height: innerSize,
      borderRadius: "50%",
      backgroundColor: monochrome
        ? "var(--text-quaternary)"
        : "rgba(0, 0, 0, 0.2)",
    },
    // Pattern 1: Square
    {
      width: innerSize,
      height: innerSize,
      backgroundColor: monochrome
        ? "var(--text-quaternary)"
        : "rgba(0, 0, 0, 0.2)",
    },
    // Pattern 2: Triangle
    {
      width: 0,
      height: 0,
      borderLeft: `${innerSize / 2}px solid transparent`,
      borderRight: `${innerSize / 2}px solid transparent`,
      borderBottom: `${innerSize}px solid ${
        monochrome ? "var(--text-quaternary)" : "rgba(0, 0, 0, 0.2)"
      }`,
      backgroundColor: "transparent",
    },
    // Pattern 3: Diamond
    {
      width: innerSize * 0.7,
      height: innerSize * 0.7,
      backgroundColor: monochrome
        ? "var(--text-quaternary)"
        : "rgba(0, 0, 0, 0.2)",
      transform: `rotate(${shape.rotation}deg)`,
    },
  ];

  return (
    <div style={containerStyles}>
      <div
        style={{
          ...patternStyles[shape.pattern],
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      />
    </div>
  );
});
