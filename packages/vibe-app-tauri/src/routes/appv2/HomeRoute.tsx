import { HomeSurface, type QuickAction, type StatItem } from "../../components/routes";
import type { Session } from "../../components/surfaces";

type HomeRouteProps = {
  sessions: Session[];
  quickActions: QuickAction[];
  stats: StatItem[];
  onSessionSelect: (session: Session) => void;
  onNewSession: () => void;
  onViewAllSessions: () => void;
};

export function HomeRoute({
  sessions,
  quickActions,
  stats,
  onSessionSelect,
  onNewSession,
  onViewAllSessions,
}: HomeRouteProps) {
  return (
    <HomeSurface
      recentSessions={sessions}
      onSessionSelect={onSessionSelect}
      onNewSession={onNewSession}
      onViewAllSessions={onViewAllSessions}
      quickActions={quickActions}
      stats={stats}
    />
  );
}
