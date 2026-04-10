import { forwardRef, type HTMLAttributes, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";

export type BadgeVariant =
  | "default"
  | "primary"
  | "secondary"
  | "success"
  | "warning"
  | "danger"
  | "info"
  | "outline"
  | "ghost";

export type BadgeSize = "sm" | "md";

export interface BadgeProps extends HTMLAttributes<HTMLSpanElement> {
  /** Visual style variant */
  variant?: BadgeVariant;
  /** Size of the badge */
  size?: BadgeSize;
  /** Badge content */
  children: ReactNode;
}

/**
 * Badge - Status indicators, labels, and tags
 *
 * Matches Happy's pill styling:
 * - Consistent border radius (pill shape)
 * - Multiple color variants for different statuses
 * - Compact padding for dense UIs
 */
export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(
  ({ variant = "default", size = "md", children, style, ...props }, ref) => {
    const variantStyles: Record<BadgeVariant, React.CSSProperties> = {
      default: {
        backgroundColor: "var(--surface-secondary)",
        color: "var(--text-secondary)",
        border: "1px solid var(--border-primary)",
      },
      primary: {
        backgroundColor: "var(--color-primary)",
        color: "#ffffff",
        border: "none",
      },
      secondary: {
        backgroundColor: "var(--surface-tertiary)",
        color: "var(--text-primary)",
        border: "none",
      },
      success: {
        backgroundColor: "var(--color-success)",
        color: "#000000",
        border: "none",
      },
      warning: {
        backgroundColor: "var(--color-warning)",
        color: "#000000",
        border: "none",
      },
      danger: {
        backgroundColor: "var(--color-danger)",
        color: "#ffffff",
        border: "none",
      },
      info: {
        backgroundColor: "var(--color-info)",
        color: "#000000",
        border: "none",
      },
      outline: {
        backgroundColor: "transparent",
        color: "var(--text-secondary)",
        border: "1px solid var(--border-primary)",
      },
      ghost: {
        backgroundColor: "transparent",
        color: "var(--text-tertiary)",
        border: "none",
      },
    };

    const sizeStyles: Record<BadgeSize, React.CSSProperties> = {
      sm: {
        height: "18px",
        padding: "0 6px",
        fontSize: tokens.typography.fontSize.xs,
        fontWeight: tokens.typography.fontWeight.medium,
      },
      md: {
        height: tokens.components.badge.height,
        padding: tokens.components.badge.padding,
        fontSize: tokens.typography.fontSize.sm,
        fontWeight: tokens.typography.fontWeight.medium,
      },
    };

    const baseStyles: React.CSSProperties = {
      display: "inline-flex",
      alignItems: "center",
      justifyContent: "center",
      borderRadius: tokens.components.badge.radius,
      whiteSpace: "nowrap",
      userSelect: "none",
      ...variantStyles[variant],
      ...sizeStyles[size],
      ...style,
    };

    return (
      <span ref={ref} style={baseStyles} {...props}>
        {children}
      </span>
    );
  },
);

Badge.displayName = "Badge";
