import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger" | "success";
export type ButtonSize = "sm" | "md" | "lg";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /** Visual style variant */
  variant?: ButtonVariant;
  /** Size of the button */
  size?: ButtonSize;
  /** Loading state */
  loading?: boolean;
  /** Icon to display before the text */
  leadingIcon?: ReactNode;
  /** Icon to display after the text */
  trailingIcon?: ReactNode;
  /** Full width button */
  fullWidth?: boolean;
  /** Children content */
  children: ReactNode;
}

/**
 * Button - Primary interaction element
 *
 * Matches Happy's button treatment:
 * - Consistent padding and border radius
 * - Clear hover and active states
 * - Focus ring for accessibility
 * - Support for icons and loading state
 */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  (
    {
      variant = "primary",
      size = "md",
      loading = false,
      leadingIcon,
      trailingIcon,
      fullWidth = false,
      children,
      disabled,
      style,
      ...props
    },
    ref,
  ) => {
    const isDisabled = disabled || loading;

    const variantStyles: Record<ButtonVariant, React.CSSProperties> = {
      primary: {
        backgroundColor: "var(--color-primary)",
        color: "#ffffff",
        border: "none",
      },
      secondary: {
        backgroundColor: "var(--surface-secondary)",
        color: "var(--color-primary)",
        border: "1px solid var(--border-primary)",
      },
      ghost: {
        backgroundColor: "transparent",
        color: "var(--color-primary)",
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

    const sizeStyles: Record<ButtonSize, React.CSSProperties> = {
      sm: {
        height: tokens.components.button.height.sm,
        padding: tokens.components.button.padding.sm,
        fontSize: tokens.typography.fontSize.sm,
        fontWeight: tokens.typography.fontWeight.medium,
      },
      md: {
        height: tokens.components.button.height.md,
        padding: tokens.components.button.padding.md,
        fontSize: tokens.typography.fontSize.base,
        fontWeight: tokens.typography.fontWeight.medium,
      },
      lg: {
        height: tokens.components.button.height.lg,
        padding: tokens.components.button.padding.lg,
        fontSize: tokens.typography.fontSize.lg,
        fontWeight: tokens.typography.fontWeight.semibold,
      },
    };

    const baseStyles: React.CSSProperties = {
      display: "inline-flex",
      alignItems: "center",
      justifyContent: "center",
      gap: "6px",
      borderRadius: tokens.components.button.radius,
      cursor: isDisabled ? "not-allowed" : "pointer",
      opacity: isDisabled ? 0.5 : 1,
      transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
      whiteSpace: "nowrap",
      userSelect: "none",
      width: fullWidth ? "100%" : undefined,
      ...variantStyles[variant],
      ...sizeStyles[size],
      ...style,
    };

    return (
      <button ref={ref} disabled={isDisabled} style={baseStyles} {...props}>
        {loading && (
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
        )}
        {!loading && leadingIcon}
        <span>{children}</span>
        {!loading && trailingIcon}
      </button>
    );
  },
);

Button.displayName = "Button";

// Add spin animation
if (typeof document !== "undefined") {
  const style = document.createElement("style");
  style.textContent = `
    @keyframes spin {
      to { transform: rotate(360deg); }
    }
  `;
  document.head.appendChild(style);
}
