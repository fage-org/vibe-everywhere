import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui";
import type { DesktopSession } from "../../desktop-client";

type SessionInfoRouteProps = {
  session: DesktopSession | null;
  loading?: boolean;
  error?: string;
  onNavigateToSession: () => void;
  onNavigateToFiles: () => void;
};

export function SessionInfoRoute({
  session,
  loading,
  error,
  onNavigateToSession,
  onNavigateToFiles,
}: SessionInfoRouteProps) {
  if (loading) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardContent>
            <div style={{ textAlign: "center", padding: "var(--space-8)" }}>
              Loading session info...
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardHeader>
            <CardTitle>Error</CardTitle>
          </CardHeader>
          <CardContent>
            <div style={{ color: "var(--color-error)" }}>{error}</div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!session) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardHeader>
            <CardTitle>Session Not Found</CardTitle>
            <CardDescription>
              The requested session could not be found
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  const metadata = session.metadata;

  return (
    <div style={{ maxWidth: "760px", margin: "0 auto" }}>
      <Card variant="default">
        <CardHeader>
          <CardTitle>Session Info</CardTitle>
          <CardDescription>
            {metadata?.path ?? "Session details"}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div
            style={{
              display: "grid",
              gap: "var(--space-3)",
              marginBottom: "var(--space-4)",
            }}
          >
            <InfoRow label="Session ID" value={session.id} />
            <InfoRow label="Workspace" value={metadata?.path ?? "Unavailable"} />
            <InfoRow label="Host" value={metadata?.host ?? "Unavailable"} />
            <InfoRow
              label="Model"
              value={metadata?.currentModelCode ?? "Unavailable"}
            />
            <InfoRow
              label="Created"
              value={
                session.createdAt
                  ? new Date(session.createdAt).toLocaleString()
                  : "Unavailable"
              }
            />
            <InfoRow
              label="Updated"
              value={
                session.updatedAt
                  ? new Date(session.updatedAt).toLocaleString()
                  : "Unavailable"
              }
            />
          </div>

          <div
            style={{
              display: "flex",
              gap: "var(--space-3)",
            }}
          >
            <button
              type="button"
              onClick={onNavigateToSession}
              style={{
                padding: "var(--space-2) var(--space-4)",
                borderRadius: "var(--radius-md)",
                border: "1px solid var(--border-default)",
                background: "transparent",
                cursor: "pointer",
              }}
            >
              Back to Session
            </button>
            <button
              type="button"
              onClick={onNavigateToFiles}
              style={{
                padding: "var(--space-2) var(--space-4)",
                borderRadius: "var(--radius-md)",
                border: "1px solid var(--border-default)",
                background: "transparent",
                cursor: "pointer",
              }}
            >
              View Files
            </button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "120px 1fr",
        gap: "var(--space-2)",
      }}
    >
      <dt style={{ color: "var(--text-secondary)", fontWeight: 500 }}>
        {label}
      </dt>
      <dd style={{ fontFamily: "var(--font-mono)", fontSize: "var(--text-sm)" }}>
        {value}
      </dd>
    </div>
  );
}
