import { forwardRef, type HTMLAttributes, type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";

// =============================================================================
// Typography Components
// =============================================================================

interface TypographyProps extends HTMLAttributes<HTMLElement> {
  children: ReactNode;
  /** Color variant */
  color?: "primary" | "secondary" | "tertiary" | "quaternary";
  /** Whether to truncate with ellipsis */
  truncate?: boolean;
  /** Number of lines to clamp (requires webkit) */
  lineClamp?: number;
}

const colorMap = {
  primary: "var(--text-primary)",
  secondary: "var(--text-secondary)",
  tertiary: "var(--text-tertiary)",
  quaternary: "var(--text-quaternary)",
};

/**
 * LargeTitle - Largest heading style
 * Use for: Main page titles, hero text
 */
export const LargeTitle = forwardRef<HTMLHeadingElement, TypographyProps>(
  ({ children, color = "primary", truncate, lineClamp, style, ...props }, ref) => (
    <h1
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.textStyle.largeTitle.fontSize,
        fontWeight: tokens.typography.textStyle.largeTitle.fontWeight,
        lineHeight: tokens.typography.textStyle.largeTitle.lineHeight,
        letterSpacing: tokens.typography.textStyle.largeTitle.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </h1>
  ),
);

LargeTitle.displayName = "LargeTitle";

/**
 * Title1 - Primary title style
 * Use for: Section titles, modal headers
 */
export const Title1 = forwardRef<HTMLHeadingElement, TypographyProps>(
  ({ children, color = "primary", truncate, lineClamp, style, ...props }, ref) => (
    <h1
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.textStyle.title1.fontSize,
        fontWeight: tokens.typography.textStyle.title1.fontWeight,
        lineHeight: tokens.typography.textStyle.title1.lineHeight,
        letterSpacing: tokens.typography.textStyle.title1.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </h1>
  ),
);

Title1.displayName = "Title1";

/**
 * Title2 - Secondary title style
 * Use for: Card titles, subsection headers
 */
export const Title2 = forwardRef<HTMLHeadingElement, TypographyProps>(
  ({ children, color = "primary", truncate, lineClamp, style, ...props }, ref) => (
    <h2
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.textStyle.title2.fontSize,
        fontWeight: tokens.typography.textStyle.title2.fontWeight,
        lineHeight: tokens.typography.textStyle.title2.lineHeight,
        letterSpacing: tokens.typography.textStyle.title2.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </h2>
  ),
);

Title2.displayName = "Title2";

/**
 * Title3 - Tertiary title style
 * Use for: List item titles, small headers
 */
export const Title3 = forwardRef<HTMLHeadingElement, TypographyProps>(
  ({ children, color = "primary", truncate, lineClamp, style, ...props }, ref) => (
    <h3
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.textStyle.title3.fontSize,
        fontWeight: tokens.typography.textStyle.title3.fontWeight,
        lineHeight: tokens.typography.textStyle.title3.lineHeight,
        letterSpacing: tokens.typography.textStyle.title3.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </h3>
  ),
);

Title3.displayName = "Title3";

/**
 * Headline - Emphasized text style
 * Use for: Navigation labels, button text, emphasized content
 */
export const Headline = forwardRef<HTMLParagraphElement, TypographyProps>(
  ({ children, color = "primary", truncate, lineClamp, style, ...props }, ref) => (
    <p
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.textStyle.headline.fontSize,
        fontWeight: tokens.typography.textStyle.headline.fontWeight,
        lineHeight: tokens.typography.textStyle.headline.lineHeight,
        letterSpacing: tokens.typography.textStyle.headline.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </p>
  ),
);

Headline.displayName = "Headline";

/**
 * Body - Primary body text
 * Use for: Main content, descriptions
 */
export const Body = forwardRef<HTMLParagraphElement, TypographyProps & { bold?: boolean }>(
  ({ children, color = "primary", bold = false, truncate, lineClamp, style, ...props }, ref) => (
    <p
      ref={ref}
      style={{
        margin: 0,
        fontSize: bold
          ? tokens.typography.textStyle.bodyBold.fontSize
          : tokens.typography.textStyle.body.fontSize,
        fontWeight: bold
          ? tokens.typography.textStyle.bodyBold.fontWeight
          : tokens.typography.textStyle.body.fontWeight,
        lineHeight: tokens.typography.textStyle.body.lineHeight,
        letterSpacing: tokens.typography.textStyle.body.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </p>
  ),
);

Body.displayName = "Body";

/**
 * Callout - Secondary body text
 * Use for: Supporting information, captions
 */
export const Callout = forwardRef<HTMLParagraphElement, TypographyProps & { bold?: boolean }>(
  ({ children, color = "secondary", bold = false, truncate, lineClamp, style, ...props }, ref) => (
    <p
      ref={ref}
      style={{
        margin: 0,
        fontSize: bold
          ? tokens.typography.textStyle.calloutBold.fontSize
          : tokens.typography.textStyle.callout.fontSize,
        fontWeight: bold
          ? tokens.typography.textStyle.calloutBold.fontWeight
          : tokens.typography.textStyle.callout.fontWeight,
        lineHeight: tokens.typography.textStyle.callout.lineHeight,
        letterSpacing: tokens.typography.textStyle.callout.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </p>
  ),
);

Callout.displayName = "Callout";

/**
 * Subheadline - Tertiary text style
 * Use for: Labels, metadata, timestamps
 */
export const Subheadline = forwardRef<HTMLParagraphElement, TypographyProps & { bold?: boolean }>(
  ({ children, color = "tertiary", bold = false, truncate, lineClamp, style, ...props }, ref) => (
    <p
      ref={ref}
      style={{
        margin: 0,
        fontSize: bold
          ? tokens.typography.textStyle.subheadlineBold.fontSize
          : tokens.typography.textStyle.subheadline.fontSize,
        fontWeight: bold
          ? tokens.typography.textStyle.subheadlineBold.fontWeight
          : tokens.typography.textStyle.subheadline.fontWeight,
        lineHeight: tokens.typography.textStyle.subheadline.lineHeight,
        letterSpacing: tokens.typography.textStyle.subheadline.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </p>
  ),
);

Subheadline.displayName = "Subheadline";

/**
 * Footnote - Small text style
 * Use for: Fine print, legal text, helper text
 */
export const Footnote = forwardRef<HTMLParagraphElement, TypographyProps & { bold?: boolean }>(
  ({ children, color = "tertiary", bold = false, truncate, lineClamp, style, ...props }, ref) => (
    <p
      ref={ref}
      style={{
        margin: 0,
        fontSize: bold
          ? tokens.typography.textStyle.footnoteBold.fontSize
          : tokens.typography.textStyle.footnote.fontSize,
        fontWeight: bold
          ? tokens.typography.textStyle.footnoteBold.fontWeight
          : tokens.typography.textStyle.footnote.fontWeight,
        lineHeight: tokens.typography.textStyle.footnote.lineHeight,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        display: lineClamp ? "-webkit-box" : undefined,
        WebkitLineClamp: lineClamp,
        WebkitBoxOrient: lineClamp ? "vertical" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </p>
  ),
);

Footnote.displayName = "Footnote";

/**
 * Caption1 - Extra small text
 * Use for: Timestamps, metadata, badges
 */
export const Caption1 = forwardRef<HTMLSpanElement, TypographyProps>(
  ({ children, color = "quaternary", truncate, style, ...props }, ref) => (
    <span
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.textStyle.caption1.fontSize,
        fontWeight: tokens.typography.textStyle.caption1.fontWeight,
        lineHeight: tokens.typography.textStyle.caption1.lineHeight,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </span>
  ),
);

Caption1.displayName = "Caption1";

/**
 * Caption2 - Smallest text
 * Use for: Very fine details, inline metadata
 */
export const Caption2 = forwardRef<HTMLSpanElement, TypographyProps>(
  ({ children, color = "quaternary", truncate, style, ...props }, ref) => (
    <span
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.textStyle.caption2.fontSize,
        fontWeight: tokens.typography.textStyle.caption2.fontWeight,
        lineHeight: tokens.typography.textStyle.caption2.lineHeight,
        letterSpacing: tokens.typography.textStyle.caption2.letterSpacing,
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </span>
  ),
);

Caption2.displayName = "Caption2";

/**
 * Eyebrow - Small uppercase label
 * Use for: Section labels, category tags
 */
export const Eyebrow = forwardRef<HTMLSpanElement, TypographyProps>(
  ({ children, color = "tertiary", truncate, style, ...props }, ref) => (
    <span
      ref={ref}
      style={{
        margin: 0,
        fontSize: tokens.typography.fontSize.xs,
        fontWeight: tokens.typography.fontWeight.semibold,
        lineHeight: tokens.typography.lineHeight.normal,
        letterSpacing: "0.05em",
        textTransform: "uppercase",
        color: colorMap[color],
        overflow: truncate ? "hidden" : undefined,
        textOverflow: truncate ? "ellipsis" : undefined,
        whiteSpace: truncate ? "nowrap" : undefined,
        ...style,
      }}
      {...props}
    >
      {children}
    </span>
  ),
);

Eyebrow.displayName = "Eyebrow";
