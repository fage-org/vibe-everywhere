import { type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { tokens } from "../../design-system/tokens";
import { Card, CardContent, CardDescription, CardHeader, CardTitle, Button } from "../ui";
import { Title1, Title2, Body, Subheadline, Eyebrow } from "../ui/Typography";
import type { Session } from "../surfaces";

export interface HomeSurfaceProps {
  /** Recent sessions */
  recentSessions?: Session[];
  /** Quick actions */
  quickActions?: QuickAction[];
  /** Stats to display */
  stats?: StatItem[];
  /** Callback when session is selected */
  onSessionSelect?: (session: Session) => void;
  /** Callback for new session */
  onNewSession?: () => void;
  /** Callback to view all sessions */
  onViewAllSessions?: () => void;
  /** Hero content */
  hero?: ReactNode;
}

export interface QuickAction {
  id: string;
  label: string;
  description?: string;
  icon?: ReactNode;
  onClick: () => void;
}

export interface StatItem {
  label: string;
  value: string | number;
  change?: string;
  trend?: "up" | "down" | "neutral";
}

/**
 * HomeSurface - Home/Index route surface
 *
 * Matches Happy's home view:
 * - Entry hero with branding
 * - Recent sessions overview
 * - Quick actions grid
 * - Stats/overview cards
 */
export function HomeSurface({
  recentSessions = [],
  quickActions = [],
  stats = [],
  onSessionSelect,
  onNewSession,
  onViewAllSessions,
  hero,
}: HomeSurfaceProps) {
  const { t } = useTranslation(['routes', 'common']);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        gap: tokens.spacing[8],
        padding: tokens.spacing[6],
        maxWidth: "1200px",
        margin: "0 auto",
      }}
    >
      {/* Hero Section */}
      {hero || (
        <div
          style={{
            textAlign: "center",
            padding: `${tokens.spacing[8]} 0`,
          }}
        >
          <Title1 style={{ marginBottom: tokens.spacing[3] }}>
            {t('routes:home.title')}
          </Title1>
          <Body color="secondary" style={{ maxWidth: "600px", margin: "0 auto" }}>
            {t('routes:home.description')}
          </Body>
          <div style={{ marginTop: tokens.spacing[6] }}>
            <Button variant="primary" size="lg" onClick={onNewSession}>
              {t('routes:home.actions.newSession')}
            </Button>
          </div>
        </div>
      )}

      {/* Stats Row */}
      {stats.length > 0 && (
        <div
          style={{
            display: "grid",
            gridTemplateColumns: `repeat(${Math.min(stats.length, 4)}, 1fr)`,
            gap: tokens.spacing[4],
          }}
        >
          {stats.map((stat, index) => (
            <StatCard key={index} stat={stat} />
          ))}
        </div>
      )}

      {/* Quick Actions */}
      {quickActions.length > 0 && (
        <section>
          <Eyebrow style={{ marginBottom: tokens.spacing[4] }}>{t('routes:home.sections.quickActions')}</Eyebrow>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))",
              gap: tokens.spacing[4],
            }}
          >
            {quickActions.map((action) => (
              <QuickActionCard key={action.id} action={action} />
            ))}
          </div>
        </section>
      )}

      {/* Recent Sessions */}
      {recentSessions.length > 0 && (
        <section>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              marginBottom: tokens.spacing[4],
            }}
          >
            <Eyebrow>{t('routes:home.sections.recentSessions')}</Eyebrow>
            <Button variant="ghost" size="sm" onClick={onViewAllSessions}>
              {t('routes:home.actions.viewAll')}
            </Button>
          </div>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))",
              gap: tokens.spacing[4],
            }}
          >
            {recentSessions.slice(0, 6).map((session) => (
              <SessionCard
                key={session.id}
                session={session}
                onClick={() => onSessionSelect?.(session)}
              />
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

interface StatCardProps {
  stat: StatItem;
}

function StatCard({ stat }: StatCardProps) {
  const trendColor =
    stat.trend === "up"
      ? "var(--color-success)"
      : stat.trend === "down"
        ? "var(--color-danger)"
        : "var(--text-tertiary)";

  const trendIcon = stat.trend === "up" ? "↑" : stat.trend === "down" ? "↓" : "→";

  return (
    <Card variant="elevated">
      <CardContent>
        <Subheadline color="tertiary">{stat.label}</Subheadline>
        <Title2 style={{ marginTop: tokens.spacing[2] }}>{stat.value}</Title2>
        {stat.change && (
          <Subheadline style={{ color: trendColor, marginTop: tokens.spacing[1] }}>
            {trendIcon} {stat.change}
          </Subheadline>
        )}
      </CardContent>
    </Card>
  );
}

interface QuickActionCardProps {
  action: QuickAction;
}

function QuickActionCard({ action }: QuickActionCardProps) {
  return (
    <Card
      variant="interactive"
      onClick={action.onClick}
      style={{ cursor: "pointer" }}
    >
      <CardHeader>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: tokens.spacing[3],
          }}
        >
          {action.icon && (
            <span
              style={{
                fontSize: "1.5rem",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: "40px",
                height: "40px",
                borderRadius: tokens.radii.lg,
                backgroundColor: "var(--surface-secondary)",
              }}
            >
              {action.icon}
            </span>
          )}
          <div>
            <CardTitle>{action.label}</CardTitle>
            {action.description && (
              <CardDescription>{action.description}</CardDescription>
            )}
          </div>
        </div>
      </CardHeader>
    </Card>
  );
}

interface SessionCardProps {
  session: Session;
  onClick?: () => void;
}

function SessionCard({ session, onClick }: SessionCardProps) {
  const { t } = useTranslation(['common']);

  return (
    <Card variant="interactive" onClick={onClick}>
      <CardHeader>
        <CardTitle truncate>{session.title}</CardTitle>
        {session.subtitle && <CardDescription truncate>{session.subtitle}</CardDescription>}
      </CardHeader>
      <CardContent>
        {session.lastMessage && (
          <Subheadline color="tertiary" truncate style={{ marginBottom: tokens.spacing[3] }}>
            {session.lastMessage}
          </Subheadline>
        )}
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <Body color="tertiary" style={{ fontSize: tokens.typography.fontSize.sm }}>
            {formatTimeAgo(session.lastActivityAt, t)}
          </Body>
          {session.unreadCount && session.unreadCount > 0 && (
            <span
              style={{
                display: "inline-flex",
                alignItems: "center",
                justifyContent: "center",
                minWidth: "20px",
                height: "20px",
                padding: "0 6px",
                backgroundColor: "var(--color-primary)",
                color: "#ffffff",
                borderRadius: tokens.radii.full,
                fontSize: tokens.typography.fontSize.xs,
                fontWeight: tokens.typography.fontWeight.medium,
              }}
            >
              {session.unreadCount}
            </span>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function formatTimeAgo(date: Date, t: (key: string, options?: Record<string, unknown>) => string): string {
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
