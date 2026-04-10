import { type ReactNode } from "react";
import { tokens } from "../../design-system/tokens";
import { Body, Caption1, Subheadline } from "../ui/Typography";

export interface Session {
  /** Unique session ID */
  id: string;
  /** Session title */
  title: string;
  /** Session subtitle/description */
  subtitle?: string;
  /** Last message preview */
  lastMessage?: string;
  /** Timestamp of last activity */
  lastActivityAt: Date;
  /** Session status */
  status?: "active" | "idle" | "completed" | "error";
  /** Whether session is unread */
  unread?: boolean;
  /** Unread count */
  unreadCount?: number;
  /** Session icon or avatar */
  icon?: ReactNode;
  /** Additional metadata */
  metadata?: {
    model?: string;
    workspace?: string;
  };
}

export interface SessionListProps {
  /** List of sessions to display */
  sessions: Session[];
  /** Currently selected session ID */
  selectedId?: string;
  /** Callback when a session is selected */
  onSelect?: (session: Session) => void;
  /** Callback when a session is archived/deleted */
  onArchive?: (session: Session) => void;
  /** Empty state content */
  emptyState?: ReactNode;
  /** Loading state */
  loading?: boolean;
}

/**
 * SessionList - List of sessions matching Happy's SessionsList
 *
 * Features:
 * - Dense item layout with title, subtitle, and metadata
 * - Status indicators and unread badges
 * - Hover and active states
 * - Timestamp formatting
 */
export function SessionList({
  sessions,
  selectedId,
  onSelect,
  onArchive,
  emptyState,
  loading,
}: SessionListProps) {
  if (loading) {
    return <SessionListSkeleton />;
  }

  if (sessions.length === 0 && emptyState) {
    return <div style={{ padding: tokens.spacing[6] }}>{emptyState}</div>;
  }

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        overflow: "auto",
      }}
    >
      {sessions.map((session) => (
        <SessionListItem
          key={session.id}
          session={session}
          isSelected={session.id === selectedId}
          onClick={() => onSelect?.(session)}
          onArchive={() => onArchive?.(session)}
        />
      ))}
    </div>
  );
}

interface SessionListItemProps {
  session: Session;
  isSelected?: boolean;
  onClick?: () => void;
  onArchive?: () => void;
}

function SessionListItem({ session, isSelected, onClick }: SessionListItemProps) {
  const statusColor = getStatusColor(session.status);

  return (
    <button
      onClick={onClick}
      style={{
        display: "flex",
        alignItems: "flex-start",
        gap: tokens.spacing[3],
        width: "100%",
        padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
        backgroundColor: isSelected ? "var(--surface-secondary)" : "transparent",
        border: "none",
        borderBottom: "1px solid var(--border-primary)",
        cursor: "pointer",
        textAlign: "left",
        transition: `background-color ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
      }}
      onMouseEnter={(e) => {
        if (!isSelected) {
          e.currentTarget.style.backgroundColor = "var(--surface-hover)";
        }
      }}
      onMouseLeave={(e) => {
        if (!isSelected) {
          e.currentTarget.style.backgroundColor = "transparent";
        }
      }}
    >
      {/* Icon/Avatar */}
      <div
        style={{
          flexShrink: 0,
          width: "40px",
          height: "40px",
          borderRadius: tokens.radii.lg,
          backgroundColor: "var(--surface-tertiary)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "var(--text-secondary)",
        }}
      >
        {session.icon || (
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
          </svg>
        )}
      </div>

      {/* Content */}
      <div
        style={{
          flex: 1,
          minWidth: 0,
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[1],
        }}
      >
        {/* Title Row */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: tokens.spacing[2],
          }}
        >
          <Body bold truncate style={{ flex: 1 }}>
            {session.title}
          </Body>
          <Caption1 color="tertiary">{formatTimestamp(session.lastActivityAt)}</Caption1>
        </div>

        {/* Subtitle */}
        {session.subtitle && (
          <Subheadline color="secondary" truncate>
            {session.subtitle}
          </Subheadline>
        )}

        {/* Last Message Preview */}
        {session.lastMessage && (
          <Subheadline color="tertiary" truncate>
            {session.lastMessage}
          </Subheadline>
        )}

        {/* Metadata Row */}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: tokens.spacing[2],
            marginTop: tokens.spacing[0.5],
          }}
        >
          {/* Status Indicator */}
          {session.status && (
            <span
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: tokens.spacing[1],
              }}
            >
              <span
                style={{
                  width: "6px",
                  height: "6px",
                  borderRadius: "50%",
                  backgroundColor: statusColor,
                }}
              />
              <Caption1 color="tertiary" style={{ textTransform: "capitalize" }}>
                {session.status}
              </Caption1>
            </span>
          )}

          {/* Model Badge */}
          {session.metadata?.model && (
            <Caption1 color="quaternary">{session.metadata.model}</Caption1>
          )}
        </div>
      </div>

      {/* Unread Badge */}
      {(session.unread || (session.unreadCount && session.unreadCount > 0)) && (
        <div
          style={{
            flexShrink: 0,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            minWidth: "20px",
            height: "20px",
            padding: "0 6px",
            backgroundColor: "var(--color-primary)",
            borderRadius: tokens.radii.full,
            fontSize: tokens.typography.fontSize.xs,
            fontWeight: tokens.typography.fontWeight.medium,
            color: "#ffffff",
          }}
        >
          {session.unreadCount || "•"}
        </div>
      )}
    </button>
  );
}

function SessionListSkeleton() {
  return (
    <div style={{ padding: tokens.spacing[4] }}>
      {[1, 2, 3, 4, 5].map((i) => (
        <div
          key={i}
          style={{
            display: "flex",
            alignItems: "center",
            gap: tokens.spacing[3],
            padding: `${tokens.spacing[3]} 0`,
            borderBottom: "1px solid var(--border-primary)",
          }}
        >
          <div
            style={{
              width: "40px",
              height: "40px",
              borderRadius: tokens.radii.lg,
              backgroundColor: "var(--surface-secondary)",
              animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
            }}
          />
          <div style={{ flex: 1, display: "flex", flexDirection: "column", gap: tokens.spacing[2] }}>
            <div
              style={{
                height: "16px",
                width: "60%",
                borderRadius: tokens.radii.sm,
                backgroundColor: "var(--surface-secondary)",
                animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
              }}
            />
            <div
              style={{
                height: "12px",
                width: "40%",
                borderRadius: tokens.radii.sm,
                backgroundColor: "var(--surface-secondary)",
                animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
              }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}

function getStatusColor(status?: Session["status"]): string {
  switch (status) {
    case "active":
      return "var(--color-success)";
    case "idle":
      return "var(--color-warning)";
    case "error":
      return "var(--color-danger)";
    case "completed":
      return "var(--color-primary)";
    default:
      return "var(--text-quaternary)";
  }
}

function formatTimestamp(date: Date): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return "now";
  if (minutes < 60) return `${minutes}m`;
  if (hours < 24) return `${hours}h`;
  if (days < 7) return `${days}d`;
  return date.toLocaleDateString(undefined, { month: "short", day: "numeric" });
}
