import { useState } from "react";
import { useTranslation } from "react-i18next";
import { tokens } from "../../design-system/tokens";
import { Card, CardContent, Button, Badge } from "../ui";
import { Header } from "../layout";
import { Body, Subheadline, Caption1 } from "../ui/Typography";

export type NotificationType = "info" | "success" | "warning" | "error" | "system";

export interface Notification {
  /** Unique notification ID */
  id: string;
  /** Notification title */
  title: string;
  /** Notification message */
  message: string;
  /** Notification type */
  type: NotificationType;
  /** Whether notification is read */
  read: boolean;
  /** Creation timestamp */
  createdAt: Date;
  /** Optional action */
  action?: {
    label: string;
    onClick: () => void;
  };
  /** Optional icon */
  icon?: React.ReactNode;
}

export interface InboxSurfaceProps {
  /** Notifications list */
  notifications: Notification[];
  /** Callback when notification is clicked */
  onNotificationClick?: (notification: Notification) => void;
  /** Callback when notification is marked as read */
  onMarkAsRead?: (notificationId: string) => void;
  /** Callback when notification is dismissed */
  onDismiss?: (notificationId: string) => void;
  /** Callback to mark all as read */
  onMarkAllAsRead?: () => void;
  /** Loading state */
  loading?: boolean;
  /** Filter state */
  filter?: "all" | "unread";
  /** Callback when filter changes */
  onFilterChange?: (filter: "all" | "unread") => void;
}

/**
 * InboxSurface - Inbox/Notifications route surface
 *
 * Matches Happy's InboxView:
 * - Notification list with read/unread states
 * - Type-based styling and icons
 * - Swipe/dismiss actions
 * - Mark all as read
 * - Filter by read status
 */
export function InboxSurface({
  notifications,
  onNotificationClick,
  onMarkAsRead,
  onDismiss,
  onMarkAllAsRead,
  loading,
  filter = "all",
  onFilterChange,
}: InboxSurfaceProps) {
  const { t } = useTranslation(['routes', 'common']);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const filteredNotifications =
    filter === "unread" ? notifications.filter((n) => !n.read) : notifications;

  const unreadCount = notifications.filter((n) => !n.read).length;

  const typeConfig: Record<NotificationType, { color: string; icon: string }> = {
    info: { color: "var(--color-primary)", icon: "ℹ️" },
    success: { color: "var(--color-success)", icon: "✓" },
    warning: { color: "var(--color-warning)", icon: "⚠️" },
    error: { color: "var(--color-danger)", icon: "✗" },
    system: { color: "var(--text-tertiary)", icon: "⚙️" },
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "var(--bg-primary)",
      }}
    >
      {/* Header */}
      <Header
        eyebrow={t('routes:inbox.header.eyebrow')}
        title={t('routes:inbox.title')}
        subtitle={unreadCount > 0 ? `${unreadCount} ${t('routes:inbox.unreadCount')}` : undefined}
        size="compact"
        actions={
          unreadCount > 0 && onMarkAllAsRead ? (
            <Button variant="ghost" size="sm" onClick={onMarkAllAsRead}>
              {t('routes:inbox.actions.markAllRead')}
            </Button>
          ) : undefined
        }
      />

      {/* Filter Tabs */}
      <div
        style={{
          display: "flex",
          gap: tokens.spacing[1],
          padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        <FilterTab
          label={t('routes:inbox.filters.all')}
          count={notifications.length}
          active={filter === "all"}
          onClick={() => onFilterChange?.("all")}
        />
        <FilterTab
          label={t('routes:inbox.filters.unread')}
          count={unreadCount}
          active={filter === "unread"}
          onClick={() => onFilterChange?.("unread")}
        />
      </div>

      {/* Notifications List */}
      <div
        style={{
          flex: 1,
          overflow: "auto",
          padding: tokens.spacing[4],
        }}
      >
        {loading ? (
          <InboxSkeleton />
        ) : filteredNotifications.length === 0 ? (
          <EmptyInbox />
        ) : (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: tokens.spacing[3],
              maxWidth: "800px",
            }}
          >
            {filteredNotifications.map((notification) => {
              const config = typeConfig[notification.type];
              const isSelected = selectedId === notification.id;

              return (
                <Card
                  key={notification.id}
                  variant={notification.read ? "default" : "elevated"}
                  onClick={() => {
                    setSelectedId(notification.id);
                    onNotificationClick?.(notification);
                    if (!notification.read) {
                      onMarkAsRead?.(notification.id);
                    }
                  }}
                  style={{
                    cursor: "pointer",
                    borderLeft: notification.read
                      ? undefined
                      : `3px solid ${config.color}`,
                  }}
                >
                  <CardContent>
                    <div
                      style={{
                        display: "flex",
                        gap: tokens.spacing[3],
                        alignItems: "flex-start",
                      }}
                    >
                      {/* Icon */}
                      <div
                        style={{
                          width: "40px",
                          height: "40px",
                          borderRadius: tokens.radii.lg,
                          backgroundColor: "var(--surface-secondary)",
                          display: "flex",
                          alignItems: "center",
                          justifyContent: "center",
                          fontSize: "1.25rem",
                          flexShrink: 0,
                        }}
                      >
                        {notification.icon || config.icon}
                      </div>

                      {/* Content */}
                      <div style={{ flex: 1, minWidth: 0 }}>
                        <div
                          style={{
                            display: "flex",
                            alignItems: "center",
                            justifyContent: "space-between",
                            gap: tokens.spacing[2],
                          }}
                        >
                          <Subheadline
                            truncate
                            style={{
                              fontWeight: notification.read
                                ? tokens.typography.fontWeight.regular
                                : tokens.typography.fontWeight.semibold,
                            }}
                          >
                            {notification.title}
                          </Subheadline>
                          <Caption1 color="quaternary">
                            {formatTime(notification.createdAt, t)}
                          </Caption1>
                        </div>

                        <Body
                          color="secondary"
                          style={{
                            marginTop: tokens.spacing[1],
                            fontSize: tokens.typography.fontSize.sm,
                          }}
                        >
                          {notification.message}
                        </Body>

                        {/* Action */}
                        {notification.action && (
                          <div style={{ marginTop: tokens.spacing[3] }}>
                            <Button
                              variant="secondary"
                              size="sm"
                              onClick={(e) => {
                                e.stopPropagation();
                                notification.action?.onClick();
                              }}
                            >
                              {notification.action.label}
                            </Button>
                          </div>
                        )}
                      </div>

                      {/* Dismiss */}
                      {onDismiss && (
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            onDismiss(notification.id);
                          }}
                          style={{
                            background: "none",
                            border: "none",
                            color: "var(--text-tertiary)",
                            cursor: "pointer",
                            padding: tokens.spacing[1],
                            flexShrink: 0,
                          }}
                        >
                          ✕
                        </button>
                      )}
                    </div>
                  </CardContent>
                </Card>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

interface FilterTabProps {
  label: string;
  count: number;
  active: boolean;
  onClick: () => void;
}

function FilterTab({ label, count, active, onClick }: FilterTabProps) {
  return (
    <button
      onClick={onClick}
      style={{
        display: "flex",
        alignItems: "center",
        gap: tokens.spacing[2],
        padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
        backgroundColor: active ? "var(--surface-secondary)" : "transparent",
        border: "none",
        borderRadius: tokens.radii.md,
        color: active ? "var(--text-primary)" : "var(--text-secondary)",
        fontSize: tokens.typography.fontSize.sm,
        fontWeight: active
          ? tokens.typography.fontWeight.semibold
          : tokens.typography.fontWeight.regular,
        cursor: "pointer",
        transition: `all ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
      }}
    >
      {label}
      {count > 0 && (
        <Badge variant={active ? "primary" : "default"} size="sm">
          {count}
        </Badge>
      )}
    </button>
  );
}

function EmptyInbox() {
  const { t } = useTranslation(['routes']);

  return (
    <div
      style={
        {
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          padding: `${tokens.spacing[12]} 0`,
          color: "var(--text-tertiary)",
        }
      }
    >
      <div style={{ fontSize: "4rem", marginBottom: tokens.spacing[4] }}>📭</div>
      <Body>{t('routes:inbox.empty.title')}</Body>
      <Body color="tertiary" style={{ marginTop: tokens.spacing[2] }}>
        {t('routes:inbox.empty.subtitle')}
      </Body>
    </div>
  );
}

function InboxSkeleton() {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
      {[1, 2, 3].map((i) => (
        <Card key={i}>
          <CardContent>
            <div style={{ display: "flex", gap: tokens.spacing[3], alignItems: "center" }}>
              <div
                style={{
                  width: "40px",
                  height: "40px",
                  borderRadius: tokens.radii.lg,
                  backgroundColor: "var(--surface-secondary)",
                  animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
                }}
              />
              <div style={{ flex: 1 }}>
                <div
                  style={{
                    height: "16px",
                    width: "40%",
                    borderRadius: tokens.radii.sm,
                    backgroundColor: "var(--surface-secondary)",
                    animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
                  }}
                />
                <div
                  style={{
                    height: "12px",
                    width: "60%",
                    borderRadius: tokens.radii.sm,
                    backgroundColor: "var(--surface-secondary)",
                    marginTop: tokens.spacing[2],
                    animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite",
                  }}
                />
              </div>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function formatTime(date: Date, t: (key: string, options?: Record<string, unknown>) => string): string {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return t('common:time.justNow');
  if (minutes < 60) return t('common:time.minutesAgo', { count: minutes });
  if (hours < 24) return t('common:time.hoursAgo', { count: hours });
  if (days < 7) return t('common:time.daysAgo', { count: days });
  return date.toLocaleDateString();
}
