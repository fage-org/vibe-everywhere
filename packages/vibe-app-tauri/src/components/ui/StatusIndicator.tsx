import { type CSSProperties } from "react";
import { tokens } from "../../design-system/tokens";

export type StatusType = "online" | "offline" | "processing" | "error" | "warning" | "success";
export type StatusSize = "sm" | "md" | "lg";

export interface StatusIndicatorProps {
  status: StatusType;
  size?: StatusSize;
  label?: string;
  showPulse?: boolean;
  style?: CSSProperties;
}

/**
 * StatusIndicator - Status indicator component
 *
 * Features:
 * - Multiple status variants (online, offline, processing, error, warning, success)
 * - Size variants (sm, md, lg)
 * - Optional label text
 * - Pulse animation for processing state
 */
export function StatusIndicator({
  status,
  size = "md",
  label,
  showPulse = true,
  style,
}: StatusIndicatorProps) {
  const sizeMap: Record<StatusSize, number> = {
    sm: 8,
    md: 12,
    lg: 16,
  };

  const colorMap: Record<StatusType, string> = {
    online: "var(--color-success)",
    offline: "var(--text-quaternary)",
    processing: "var(--color-info)",
    error: "var(--color-danger)",
    warning: "var(--color-warning)",
    success: "var(--color-success)",
  };

  const dotSize = sizeMap[size];
  const color = colorMap[status];

  const dotStyles: CSSProperties = {
    width: dotSize,
    height: dotSize,
    borderRadius: "50%",
    backgroundColor: color,
    flexShrink: 0,
  };

  // Add pulse animation for processing state
  const pulseStyles: CSSProperties =
    status === "processing" && showPulse
      ? {
          ...dotStyles,
          animation: "status-pulse 1.5s ease-in-out infinite",
        }
      : dotStyles;

  const containerStyles: CSSProperties = {
    display: "inline-flex",
    alignItems: "center",
    gap: tokens.spacing[2],
    ...style,
  };

  const labelStyles: CSSProperties = {
    fontSize: tokens.typography.fontSize[size === "lg" ? "base" : size === "sm" ? "xs" : "sm"],
    color: "var(--text-secondary)",
    fontWeight: tokens.typography.fontWeight.medium,
  };

  return (
    <span style={containerStyles}>
      <span style={pulseStyles} />
      {label && <span style={labelStyles}>{label}</span>}
    </span>
  );
}

/**
 * getStatusLabel - Get default label for status type
 */
export function getStatusLabel(status: StatusType): string {
  const labels: Record<StatusType, string> = {
    online: "Online",
    offline: "Offline",
    processing: "Processing",
    error: "Error",
    warning: "Warning",
    success: "Success",
  };
  return labels[status];
}

// CSS keyframes for pulse animation (inject via global CSS):
//
// @keyframes status-pulse {
//   0%, 100% { opacity: 1; transform: scale(1); }
//   50% { opacity: 0.5; transform: scale(1.2); }
// }

/**
 * StatusBadge - Status indicator with badge styling
 */
export interface StatusBadgeProps {
  status: StatusType;
  size?: StatusSize;
  style?: CSSProperties;
}

export function StatusBadge({ status, size = "md", style }: StatusBadgeProps) {
  const sizeStyles: Record<StatusSize, CSSProperties> = {
    sm: {
      padding: `${tokens.spacing[0.5]} ${tokens.spacing[2]}`,
      fontSize: tokens.typography.fontSize.xs,
    },
    md: {
      padding: `${tokens.spacing[1]} ${tokens.spacing[3]}`,
      fontSize: tokens.typography.fontSize.sm,
    },
    lg: {
      padding: `${tokens.spacing[1.5]} ${tokens.spacing[4]}`,
      fontSize: tokens.typography.fontSize.base,
    },
  };

  const colorMap: Record<StatusType, { bg: string; text: string }> = {
    online: {
      bg: "rgba(52, 199, 89, 0.15)",
      text: "var(--color-success)",
    },
    offline: {
      bg: "var(--surface-tertiary)",
      text: "var(--text-tertiary)",
    },
    processing: {
      bg: "rgba(90, 200, 250, 0.15)",
      text: "var(--color-info)",
    },
    error: {
      bg: "rgba(255, 59, 48, 0.15)",
      text: "var(--color-danger)",
    },
    warning: {
      bg: "rgba(255, 149, 0, 0.15)",
      text: "var(--color-warning)",
    },
    success: {
      bg: "rgba(52, 199, 89, 0.15)",
      text: "var(--color-success)",
    },
  };

  const { bg, text } = colorMap[status];

  const badgeStyles: CSSProperties = {
    display: "inline-flex",
    alignItems: "center",
    gap: tokens.spacing[1],
    borderRadius: tokens.radii.full,
    backgroundColor: bg,
    color: text,
    fontWeight: tokens.typography.fontWeight.medium,
    ...sizeStyles[size],
    ...style,
  };

  return (
    <span style={badgeStyles}>
      <StatusIndicator status={status} size="sm" showPulse={status === "processing"} />
      {getStatusLabel(status)}
    </span>
  );
}