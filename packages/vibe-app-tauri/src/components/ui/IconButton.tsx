import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";

export type IconButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "success";
export type IconButtonSize = "sm" | "md" | "lg";

export interface IconButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, "children"> {
  /** Visual style variant */
  variant?: IconButtonVariant;
  /** Size of the button */
  size?: IconButtonSize;
  /** Loading state */
  loading?: boolean;
  /** Icon content */
  children: ReactNode;
  /** Accessible label */
  "aria-label": string;
}

/**
 * IconButton - Icon-only button for toolbars and compact actions
 *
 * A button designed for icon-only content with proper sizing and accessibility.
 */
export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(
  (
    {
      variant = "ghost",
      size = "md",
      loading = false,
      children,
      disabled,
      style,
      ...props
    },
    ref,
  ) => {
    const isDisabled = disabled || loading;

    const variantStyles: Record<IconButtonVariant, React.CSSProperties> = {
      primary: {
        backgroundColor: "var(--color-primary)",
        color: "#ffffff",
        border: "none",
      },
      secondary: {
        backgroundColor: "var(--surface-secondary)",
        color: "var(--text-primary)",
        border: "1px solid var(--border-primary)",
      },
      ghost: {
        backgroundColor: "transparent",
        color: "var(--text-primary)",
        border: "none",
      },
      danger: {
        backgroundColor: "var(--color-danger)",
        color: "#ffffff",
        border: "none",
      },
      success: {
        backgroundColor: "var(--color-success)",
        color: "#ffffff",
        border: "none",
      },
    };

    const sizeStyles: Record<IconButtonSize, React.CSSProperties> = {
      sm: {
        width: tokens.components.button.height.sm,
        height: tokens.components.button.height.sm,
        fontSize: tokens.typography.fontSize.sm,
      },
      md: {
        width: tokens.components.button.height.md,
        height: tokens.components.button.height.md,
        fontSize: tokens.typography.fontSize.base,
      },
      lg: {
        width: tokens.components.button.height.lg,
        height: tokens.components.button.height.lg,
        fontSize: tokens.typography.fontSize.lg,
      },
    };

    const baseStyles: React.CSSProperties = {
      display: "inline-flex",
      alignItems: "center",
      justifyContent: "center",
      borderRadius: tokens.components.button.radius,
      cursor: isDisabled ? "not-allowed" : "pointer",
      opacity: isDisabled ? 0.5 : 1,
      transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
      whiteSpace: "nowrap",
      userSelect: "none",
      padding: 0,
      ...variantStyles[variant],
      ...sizeStyles[size],
      ...style,
    };

    return (
      <button
        ref={ref}
        disabled={isDisabled}
        style={baseStyles}
        {...props}
      >
        {loading ? (
          <span
            style={{
              display: "inline-block",
              width: "1em",
              height: "1em",
              border: "2px solid currentColor",
              borderRightColor: "transparent",
              borderRadius: "50%",
              animation: "spin 1s linear infinite",
            }}
          />
        ) : (
          children
        )}
      </button>
    );
  },
);

IconButton.displayName = "IconButton";

export default IconButton;