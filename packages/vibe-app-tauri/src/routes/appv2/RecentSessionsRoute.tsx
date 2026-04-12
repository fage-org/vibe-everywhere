import { Header } from "../../components/layout";
import { SessionList, type Session } from "../../components/surfaces";

type RecentSessionsRouteProps = {
  title: string;
  eyebrow: string;
  sessions: Session[];
  isLoading: boolean;
  onSessionSelect: (session: Session) => void;
};

export function RecentSessionsRoute({
  title,
  eyebrow,
  sessions,
  isLoading,
  onSessionSelect,
}: RecentSessionsRouteProps) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 16, height: "100%" }}>
      <Header eyebrow={eyebrow} title={title} size="compact" />
      <SessionList sessions={sessions} onSelect={onSessionSelect} loading={isLoading} />
    </div>
  );
}
