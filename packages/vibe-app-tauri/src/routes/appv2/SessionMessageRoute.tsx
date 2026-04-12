import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui";
import type { Message } from "../../components/surfaces";

type SessionMessageRouteProps = {
  sessionId: string;
  messageId: string;
  message: Message | null;
  loading?: boolean;
  error?: string;
  onNavigateToSession: () => void;
  onNavigateToFiles: () => void;
};

export function SessionMessageRoute({
  sessionId,
  messageId,
  message,
  loading,
  error,
  onNavigateToSession,
  onNavigateToFiles,
}: SessionMessageRouteProps) {
  if (loading) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardContent>
            <div style={{ textAlign: "center", padding: "var(--space-8)" }}>
              Loading message...
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

  if (!message) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardHeader>
            <CardTitle>Message Not Found</CardTitle>
            <CardDescription>
              Message {messageId} in session {sessionId} could not be found
            </CardDescription>
          </CardHeader>
        </Card>
      </div>
    );
  }

  return (
    <div style={{ maxWidth: "760px", margin: "0 auto" }}>
      <Card variant="default">
        <CardHeader>
          <CardTitle>Message Detail</CardTitle>
          <CardDescription>
            Deep-linked message from session
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div
            style={{
              display: "flex",
              gap: "var(--space-2)",
              marginBottom: "var(--space-4)",
            }}
          >
            <span
              style={{
                padding: "var(--space-1) var(--space-2)",
                borderRadius: "var(--radius-full)",
                backgroundColor: "var(--color-accent-subtle)",
                fontSize: "var(--text-xs)",
              }}
            >
              {message.role}
            </span>
          </div>

          <div
            style={{
              padding: "var(--space-4)",
              backgroundColor: "var(--color-surface-elevated)",
              borderRadius: "var(--radius-lg)",
              marginBottom: "var(--space-4)",
              whiteSpace: "pre-wrap",
              fontFamily: "var(--font-mono)",
              fontSize: "var(--text-sm)",
            }}
          >
            {message.content}
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
              Open Files
            </button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
