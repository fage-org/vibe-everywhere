import { memo, type CSSProperties } from "react";
import { AvatarGradient } from "./AvatarGradient";
import { AvatarBrutalist } from "./AvatarBrutalist";
import { useLocalSettingsContext } from "../../../LocalSettingsContext";

export type AvatarStyle = "gradient" | "pixelated" | "brutalist";

export interface AvatarProps {
  /** Unique identifier for avatar generation */
  id: string;
  /** Title attribute (unused but kept for API compatibility) */
  title?: boolean;
  /** Square avatar instead of circle */
  square?: boolean;
  /** Size in pixels */
  size?: number;
  /** Monochrome mode */
  monochrome?: boolean;
  /** AI provider flavor for icon overlay */
  flavor?: "claude" | "codex" | "gemini" | "openclaw" | null;
  /** Custom image URL */
  imageUrl?: string | null;
  /** Thumbhash for placeholder */
  thumbhash?: string | null;
}

/**
 * Avatar - Main avatar component with multiple styles
 *
 * Features:
 * - Multiple style variants (gradient, pixelated, brutalist)
 * - Consistent generation based on ID
 * - AI provider icon overlay
 * - Custom image support
 * - Settings integration for style preference
 */
export const Avatar = memo(function Avatar(props: AvatarProps) {
  const {
    id,
    square,
    size = 48,
    monochrome,
    flavor,
    imageUrl,
    thumbhash,
  } = props;

  const { settings } = useLocalSettingsContext();
  const avatarStyle = settings.avatarStyle || "gradient";
  const showFlavorIcons = settings.showFlavorIcons ?? true;

  // Render custom image if provided
  if (imageUrl) {
    const imageElement = (
      <img
        src={imageUrl}
        alt=""
        style={{
          width: size,
          height: size,
          borderRadius: square ? 0 : size / 2,
          objectFit: "cover",
        }}
      />
    );

    // Add flavor icon overlay if enabled
    if (showFlavorIcons && flavor) {
      return (
        <AvatarWithFlavor
          size={size}
          flavor={flavor}
          square={square}
        >
          {imageElement}
        </AvatarWithFlavor>
      );
    }

    return imageElement;
  }

  // Determine which avatar variant to render
  let AvatarComponent: React.ComponentType<{
    id: string;
    square?: boolean;
    size?: number;
    monochrome?: boolean;
  }>;

  if (avatarStyle === "brutalist") {
    AvatarComponent = AvatarBrutalist;
  } else {
    // Default to gradient (pixelated can be added later)
    AvatarComponent = AvatarGradient;
  }

  // Only wrap in container if showing flavor icons and flavor was provided
  if (showFlavorIcons && flavor !== null && flavor !== undefined) {
    return (
      <AvatarWithFlavor
        size={size}
        flavor={flavor}
        square={square}
      >
        <AvatarComponent
          id={id}
          square={square}
          size={size}
          monochrome={monochrome}
        />
      </AvatarWithFlavor>
    );
  }

  return (
    <AvatarComponent
      id={id}
      square={square}
      size={size}
      monochrome={monochrome}
    />
  );
});

// =============================================================================
// Flavor Icon Overlay
// =============================================================================

interface AvatarWithFlavorProps {
  size: number;
  flavor: "claude" | "codex" | "gemini" | "openclaw";
  square?: boolean;
  children: React.ReactNode;
}

function AvatarWithFlavor({
  size,
  flavor,
  square,
  children,
}: AvatarWithFlavorProps) {
  const circleSize = Math.round(size * 0.35);

  // Flavor icon colors (simple CSS backgrounds)
  const flavorColors: Record<string, string> = {
    claude: "var(--color-primary)",
    codex: "#10a37f",
    gemini: "#4285f4",
    openclaw: "#ff6b35",
  };

  const iconSize = Math.round(size * 0.2);

  const containerStyles: CSSProperties = {
    position: "relative",
    width: size,
    height: size,
    flexShrink: 0,
  };

  const badgeStyles: CSSProperties = {
    position: "absolute",
    bottom: -2,
    right: -2,
    width: circleSize,
    height: circleSize,
    borderRadius: "50%",
    backgroundColor: "var(--surface-primary)",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    boxShadow: "0 1px 3px rgba(0, 0, 0, 0.2)",
  };

  const iconStyles: CSSProperties = {
    width: iconSize,
    height: iconSize,
    borderRadius: "50%",
    backgroundColor: flavorColors[flavor] || flavorColors.claude,
  };

  return (
    <div style={containerStyles}>
      {children}
      <div style={badgeStyles}>
        <div style={iconStyles} />
      </div>
    </div>
  );
}

export default Avatar;
