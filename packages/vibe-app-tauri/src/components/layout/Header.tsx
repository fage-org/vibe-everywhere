import { type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import { Eyebrow, Title2, Title3 } from "../ui/Typography";

interface HeaderProps {
  /** Eyebrow text (small label above title) */
  eyebrow?: string;
  /** Main title */
  title: string;
  /** Subtitle or description */
  subtitle?: string;
  /** Size variant */
  size?: "large" | "default" | "compact";
  /** Leading element (icon, back button, etc.) */
  leading?: ReactNode;
  /** Trailing actions */
  actions?: ReactNode;
  /** Border visibility */
  border?: boolean;
}

/**
 * Header - Route header with eyebrow, title, and actions
 *
 * Matches Happy's Header component:
 * - Eyebrow label above title
 * - Title with optional subtitle
 * - Leading and trailing action areas
 * - Consistent padding and height
 */
export function Header({
  eyebrow,
  title,
  subtitle,
  size = "default",
  leading,
  actions,
  border = true,
}: HeaderProps) {
  const sizeStyles = {
    large: {
      padding: `${tokens.spacing[6]} ${tokens.spacing[6]}`,
      gap: tokens.spacing[2],
    },
    default: {
      padding: `${tokens.spacing[4]} ${tokens.spacing[6]}`,
      gap: tokens.spacing[1.5],
    },
    compact: {
      padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
      gap: tokens.spacing[1],
    },
  };

  const TitleComponent = size === "large" ? Title2 : Title3;

  return (
    <div
      style={{
        display: "flex",
        alignItems: "flex-start",
        justifyContent: "space-between",
        backgroundColor: "var(--bg-primary)",
        borderBottom: border ? "1px solid var(--border-primary)" : undefined,
        ...sizeStyles[size],
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: tokens.spacing[3],
          minWidth: 0,
          flex: 1,
        }}
      >
        {leading && (
          <div style={{ flexShrink: 0, display: "flex", alignItems: "center" }}>
            {leading}
          </div>
        )}

        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: sizeStyles[size].gap,
            minWidth: 0,
          }}
        >
          {eyebrow && <Eyebrow>{eyebrow}</Eyebrow>}

          <TitleComponent truncate>{title}</TitleComponent>

          {subtitle && (
            <p
              style={{
                margin: 0,
                fontSize: tokens.typography.fontSize.base,
                color: "var(--text-tertiary)",
                lineHeight: tokens.typography.lineHeight.normal,
              }}
            >
              {subtitle}
            </p>
          )}
        </div>
      </div>

      {actions && (
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: tokens.spacing[2],
            flexShrink: 0,
          }}
        >
          {actions}
        </div>
      )}
    </div>
  );
}

interface HeaderActionProps {
  children: ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  active?: boolean;
}

/**
 * HeaderAction - Individual action button for header
 */
export function HeaderAction({ children, onClick, disabled, active }: HeaderActionProps) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        width: "36px",
        height: "36px",
        borderRadius: tokens.radii.md,
        backgroundColor: active ? "var(--surface-secondary)" : "transparent",
        border: "none",
        color: active ? "var(--text-primary)" : "var(--text-secondary)",
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
        transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
      }}
      onMouseEnter={(e) => {
        if (!disabled && !active) {
          e.currentTarget.style.backgroundColor = "var(--surface-hover)";
          e.currentTarget.style.color = "var(--text-primary)";
        }
      }}
      onMouseLeave={(e) => {
        if (!active) {
          e.currentTarget.style.backgroundColor = "transparent";
          e.currentTarget.style.color = "var(--text-secondary)";
        }
      }}
    >
      {children}
    </button>
  );
}
