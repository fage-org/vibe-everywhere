import { forwardRef, type HTMLAttributes, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";

export type CardVariant = "default" | "elevated" | "outlined" | "interactive";

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  /** Visual style variant */
  variant?: CardVariant;
  /** Card content */
  children: ReactNode;
  /** Padding size */
  padding?: "none" | "sm" | "md" | "lg";
}

/**
 * Card - Surface container for grouping related content
 *
 * Matches Happy's grouped surface treatment:
 * - Consistent border radius and shadows
 * - Optional hover states for interactive cards
 * - Flexible padding options
 */
export const Card = forwardRef<HTMLDivElement, CardProps>(
  (
    { variant = "default", children, padding = "md", style, ...props },
    ref,
  ) => {
    const variantStyles: Record<CardVariant, React.CSSProperties> = {
      default: {
        backgroundColor: "var(--surface-primary)",
        border: "1px solid var(--border-primary)",
        boxShadow: "none",
      },
      elevated: {
        backgroundColor: "var(--surface-elevated)",
        border: "none",
        boxShadow: tokens.shadows.ios.medium,
      },
      outlined: {
        backgroundColor: "transparent",
        border: "1px solid var(--border-primary)",
        boxShadow: "none",
      },
      interactive: {
        backgroundColor: "var(--surface-primary)",
        border: "1px solid var(--border-primary)",
        boxShadow: "none",
        cursor: "pointer",
        transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
      },
    };

    const paddingStyles: Record<string, React.CSSProperties> = {
      none: { padding: 0 },
      sm: { padding: tokens.spacing[3] },
      md: { padding: tokens.spacing[4] },
      lg: { padding: tokens.spacing[6] },
    };

    const baseStyles: React.CSSProperties = {
      borderRadius: tokens.components.card.radius,
      overflow: "hidden",
      ...variantStyles[variant],
      ...paddingStyles[padding],
      ...style,
    };

    // Add hover effect for interactive variant
    const handleMouseEnter = (e: React.MouseEvent<HTMLDivElement>) => {
      if (variant === "interactive") {
        e.currentTarget.style.backgroundColor = "var(--surface-secondary)";
        e.currentTarget.style.borderColor = "var(--border-secondary)";
      }
      props.onMouseEnter?.(e);
    };

    const handleMouseLeave = (e: React.MouseEvent<HTMLDivElement>) => {
      if (variant === "interactive") {
        e.currentTarget.style.backgroundColor = "var(--surface-primary)";
        e.currentTarget.style.borderColor = "var(--border-primary)";
      }
      props.onMouseLeave?.(e);
    };

    return (
      <div
        ref={ref}
        style={baseStyles}
        {...props}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      >
        {children}
      </div>
    );
  },
);

Card.displayName = "Card";

// =============================================================================
// Card Subcomponents
// =============================================================================

export interface CardHeaderProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export const CardHeader = forwardRef<HTMLDivElement, CardHeaderProps>(
  ({ children, style, ...props }, ref) => (
    <div
      ref={ref}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        gap: tokens.spacing[3],
        marginBottom: tokens.spacing[4],
        ...style,
      }}
      {...props}
    >
      {children}
    </div>
  ),
);

CardHeader.displayName = "CardHeader";

export interface CardTitleProps extends HTMLAttributes<HTMLHeadingElement> {
  children: ReactNode;
  /** Whether to truncate text with ellipsis */
  truncate?: boolean;
}

export const CardTitle = forwardRef<HTMLHeadingElement, CardTitleProps>(
  ({ children, truncate, style, ...props }, ref) => (
    <h3
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.fontSize.lg,
        fontWeight: tokens.typography.fontWeight.semibold,
        color: "var(--text-primary)",
        lineHeight: tokens.typography.lineHeight.snug,
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </h3>
  ),
);

CardTitle.displayName = "CardTitle";

export interface CardDescriptionProps extends HTMLAttributes<HTMLParagraphElement> {
  children: ReactNode;
  /** Whether to truncate text with ellipsis */
  truncate?: boolean;
}

export const CardDescription = forwardRef<HTMLParagraphElement, CardDescriptionProps>(
  ({ children, truncate, style, ...props }, ref) => (
    <p
      ref={ref}
      style={{
        margin: 0,
        marginTop: tokens.spacing[1],
        fontSize: tokens.typography.fontSize.sm,
        color: "var(--text-tertiary)",
        lineHeight: tokens.typography.lineHeight.normal,
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </p>
  ),
);

CardDescription.displayName = "CardDescription";

export interface CardContentProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export const CardContent = forwardRef<HTMLDivElement, CardContentProps>(
  ({ children, style, ...props }, ref) => (
    <div
      ref={ref}
      style={{
        color: "var(--text-secondary)",
        ...style,
      }}
      {...props}
    >
      {children}
    </div>
  ),
);

CardContent.displayName = "CardContent";

export interface CardFooterProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
}

export const CardFooter = forwardRef<HTMLDivElement, CardFooterProps>(
  ({ children, style, ...props }, ref) => (
    <div
      ref={ref}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "flex-end",
        gap: tokens.spacing[2],
        marginTop: tokens.spacing[4],
        paddingTop: tokens.spacing[4],
        borderTop: "1px solid var(--border-primary)",
        ...style,
      }}
      {...props}
    >
      {children}
    </div>
  ),
);

CardFooter.displayName = "CardFooter";
