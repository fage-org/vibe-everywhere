import { forwardRef, type TextareaHTMLAttributes } from "react";
import { tokens } from "../../design-system/tokens";

export type TextAreaSize = "sm" | "md" | "lg";

export interface TextAreaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  /** Size of the textarea */
  size?: TextAreaSize;
  /** Label text */
  label?: string;
  /** Helper text displayed below textarea */
  helperText?: string;
  /** Error message */
  error?: string;
  /** Minimum number of rows */
  minRows?: number;
  /** Maximum number of rows */
  maxRows?: number;
  /** Enable auto-resize */
  autoResize?: boolean;
  /** Full width textarea */
  fullWidth?: boolean;
}

/**
 * TextArea - Multi-line text input
 *
 * Matches Happy's composer input styling:
 * - Consistent padding and border radius
 * - Support for auto-resize
 * - Clear focus states
 */
export const TextArea = forwardRef<HTMLTextAreaElement, TextAreaProps>(
  (
    {
      size = "md",
      label,
      helperText,
      error,
      minRows = 3,
      maxRows,
      autoResize = false,
      fullWidth = false,
      onChange,
      style,
      rows,
      ...props
    },
    ref,
  ) => {
    const sizeStyles: Record<TextAreaSize, React.CSSProperties> = {
      sm: {
        padding: tokens.spacing[2],
        fontSize: tokens.typography.fontSize.sm,
      },
      md: {
        padding: tokens.spacing[3],
        fontSize: tokens.typography.fontSize.base,
      },
      lg: {
        padding: tokens.spacing[4],
        fontSize: tokens.typography.fontSize.lg,
      },
    };

    const textareaStyles: React.CSSProperties = {
      width: fullWidth ? "100%" : undefined,
      minHeight: `${minRows * 1.5}em`,
      maxHeight: maxRows ? `${maxRows * 1.5}em` : undefined,
      backgroundColor: "var(--surface-secondary)",
      border: `1px solid ${error ? "var(--color-danger)" : "var(--border-primary)"}`,
      borderRadius: tokens.components.input.radius,
      color: "var(--text-primary)",
      fontFamily: "inherit",
      lineHeight: tokens.typography.lineHeight.relaxed,
      resize: autoResize ? "none" : "vertical",
      outline: "none",
      transition: `border-color ${tokens.animation.duration.fast} ${tokens.animation.easing.default}`,
      ...sizeStyles[size],
      ...style,
    };

    const containerStyles: React.CSSProperties = {
      display: "flex",
      flexDirection: "column",
      gap: tokens.spacing[1.5],
      width: fullWidth ? "100%" : undefined,
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

    const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      if (autoResize) {
        const target = e.target;
        target.style.height = "auto";
        target.style.height = `${target.scrollHeight}px`;
      }
      onChange?.(e);
    };

    const handleFocus = (e: React.FocusEvent<HTMLTextAreaElement>) => {
      e.currentTarget.style.borderColor = error
        ? "var(--color-danger)"
        : "var(--color-primary)";
      props.onFocus?.(e);
    };

    const handleBlur = (e: React.FocusEvent<HTMLTextAreaElement>) => {
      e.currentTarget.style.borderColor = error
        ? "var(--color-danger)"
        : "var(--border-primary)";
      props.onBlur?.(e);
    };

    return (
      <label style={containerStyles}>
        {label && <span style={labelStyles}>{label}</span>}
        <textarea
          ref={ref}
          rows={rows ?? minRows}
          style={textareaStyles}
          onChange={handleChange}
          onFocus={handleFocus}
          onBlur={handleBlur}
          {...props}
        />
        {(helperText || error) && (
          <span style={helperTextStyles}>{error || helperText}</span>
        )}
      </label>
    );
  },
);

TextArea.displayName = "TextArea";
