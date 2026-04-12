import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui";
import type { SessionWorkspaceFileContent } from "../../session-files";

type SessionFileRouteProps = {
  sessionId: string;
  filePath: string | null;
  file: SessionWorkspaceFileContent | null;
  loading?: boolean;
  error?: string | null;
  onNavigateToFiles: () => void;
  onNavigateToSession: () => void;
};

export function SessionFileRoute({
  sessionId,
  filePath,
  file,
  loading,
  error,
  onNavigateToFiles,
  onNavigateToSession,
}: SessionFileRouteProps) {
  if (loading) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardContent>
            <div style={{ textAlign: "center", padding: "var(--space-8)" }}>
              Loading file...
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
            <div style={{ marginTop: "var(--space-4)" }}>
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
                Back to Files
              </button>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!filePath) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardHeader>
            <CardTitle>No File Selected</CardTitle>
            <CardDescription>
              Please select a file from the session files list
            </CardDescription>
          </CardHeader>
          <CardContent>
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
              Browse Files
            </button>
          </CardContent>
        </Card>
      </div>
    );
  }

  if (!file) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardHeader>
            <CardTitle>File Not Found</CardTitle>
            <CardDescription>{filePath}</CardDescription>
          </CardHeader>
          <CardContent>
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
              Back to Files
            </button>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div style={{ maxWidth: "100%", margin: "0 auto" }}>
      <Card variant="default">
        <CardHeader>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "flex-start",
            }}
          >
            <div>
              <CardTitle>{file.relativePath}</CardTitle>
              <CardDescription>
                {file.language ?? "Unknown"} •{" "}
                {file.isBinary ? "Binary" : "Text"}
              </CardDescription>
            </div>
            <div style={{ display: "flex", gap: "var(--space-2)" }}>
              <button
                type="button"
                onClick={onNavigateToFiles}
                style={{
                  padding: "var(--space-2) var(--space-3)",
                  borderRadius: "var(--radius-md)",
                  border: "1px solid var(--border-default)",
                  background: "transparent",
                  cursor: "pointer",
                  fontSize: "var(--text-sm)",
                }}
              >
                Files
              </button>
              <button
                type="button"
                onClick={onNavigateToSession}
                style={{
                  padding: "var(--space-2) var(--space-3)",
                  borderRadius: "var(--radius-md)",
                  border: "1px solid var(--border-default)",
                  background: "transparent",
                  cursor: "pointer",
                  fontSize: "var(--text-sm)",
                }}
              >
                Session
              </button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          {file.isBinary ? (
            <div
              style={{
                padding: "var(--space-8)",
                textAlign: "center",
                color: "var(--text-secondary)",
              }}
            >
              Binary file - cannot display content
            </div>
          ) : (
            <div
              style={{
                backgroundColor: "var(--color-surface-elevated)",
                borderRadius: "var(--radius-lg)",
                overflow: "hidden",
              }}
            >
              {file.diff ? (
                <pre
                  style={{
                    padding: "var(--space-4)",
                    margin: 0,
                    overflow: "auto",
                    maxHeight: "60vh",
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-sm)",
                    lineHeight: 1.6,
                  }}
                >
                  {file.diff}
                </pre>
              ) : (
                <pre
                  style={{
                    padding: "var(--space-4)",
                    margin: 0,
                    overflow: "auto",
                    maxHeight: "60vh",
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-sm)",
                    lineHeight: 1.6,
                    whiteSpace: "pre-wrap",
                  }}
                >
                  {file.content}
                </pre>
              )}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
