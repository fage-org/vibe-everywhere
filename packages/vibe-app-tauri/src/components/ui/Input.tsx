import { forwardRef, type InputHTMLAttributes, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";

export type InputSize = "sm" | "md" | "lg";

export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, "size"> {
  /** Size of the input */
  size?: InputSize;
  /** Label text */
  label?: string;
  /** Helper text displayed below input */
  helperText?: string;
  /** Error message */
  error?: string;
  /** Leading icon or element */
  leadingElement?: ReactNode;
  /** Trailing icon or element */
  trailingElement?: ReactNode;
  /** Full width input */
  fullWidth?: boolean;
}

/**
 * Input - Text input field
 *
 * Matches Happy's form input styling:
 * - Consistent height and padding
 * - Clear focus states with border color change
 * - Support for labels, helper text, and errors
 * - Optional leading/trailing elements
 */
export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      size = "md",
      label,
      helperText,
      error,
      leadingElement,
      trailingElement,
      fullWidth = false,
      style,
      ...props
    },
    ref,
  ) => {
    const sizeStyles: Record<InputSize, React.CSSProperties> = {
      sm: {
        height: tokens.components.input.height.sm,
        fontSize: tokens.typography.fontSize.sm,
      },
      md: {
        height: tokens.components.input.height.md,
        fontSize: tokens.typography.fontSize.base,
      },
      lg: {
        height: tokens.components.input.height.lg,
        fontSize: tokens.typography.fontSize.lg,
      },
    };

    const inputStyles: React.CSSProperties = {
      flex: 1,
      minWidth: 0,
      height: "100%",
      padding: "0 12px",
      backgroundColor: "transparent",
      border: "none",
      outline: "none",
      color: "var(--text-primary)",
      fontSize: "inherit",
      fontFamily: "inherit",
    };

    const wrapperStyles: React.CSSProperties = {
      display: "flex",
      alignItems: "center",
      gap: tokens.spacing[2],
      width: fullWidth ? "100%" : undefined,
      backgroundColor: "var(--surface-secondary)",
      border: `1px solid ${error ? "var(--color-danger)" : "var(--border-primary)"}`,
      borderRadius: tokens.components.input.radius,
      transition: `border-color ${tokens.animation.duration.fast} ${tokens.animation.easing.default}`,
      ...sizeStyles[size],
    };

    const containerStyles: React.CSSProperties = {
      display: "flex",
      flexDirection: "column",
      gap: tokens.spacing[1.5],
      width: fullWidth ? "100%" : undefined,
      ...style,
    };

    const labelStyles: React.CSSProperties = {
      fontSize: tokens.typography.fontSize.sm,
      fontWeight: tokens.typography.fontWeight.medium,
      color: "var(--text-secondary)",
    };

    const helperTextStyles: React.CSSProperties = {
      fontSize: tokens.typography.fontSize.xs,
      color: error ? "var(--color-danger)" : "var(--text-tertiary)",
    };

    return (
      <label style={containerStyles}>
        {label && <span style={labelStyles}>{label}</span>}
        <div
          style={wrapperStyles}
          onFocus={(e) => {
            e.currentTarget.style.borderColor = error
              ? "var(--color-danger)"
              : "var(--color-primary)";
          }}
          onBlur={(e) => {
            e.currentTarget.style.borderColor = error
              ? "var(--color-danger)"
              : "var(--border-primary)";
          }}
        >
          {leadingElement && (
            <span style={{ color: "var(--text-tertiary)", display: "flex" }}>
              {leadingElement}
            </span>
          )}
          <input ref={ref} style={inputStyles} {...props} />
          {trailingElement && (
            <span style={{ color: "var(--text-tertiary)", display: "flex" }}>
              {trailingElement}
            </span>
          )}
        </div>
        {(helperText || error) && (
          <span style={helperTextStyles}>{error || helperText}</span>
        )}
      </label>
    );
  },
);

Input.displayName = "Input";
