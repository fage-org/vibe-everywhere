import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui";
import type { SessionWorkspaceFile } from "../../session-files";
import type { DesktopSession } from "../../desktop-client";

type SessionFilesRouteProps = {
  session: DesktopSession | null;
  files: SessionWorkspaceFile[];
  branch: string | null;
  totalStaged: number;
  totalUnstaged: number;
  loading?: boolean;
  error?: string | null;
  onSelectFile: (relativePath: string) => void;
  onNavigateToSession: () => void;
};

export function SessionFilesRoute({
  session,
  files,
  branch,
  totalStaged,
  totalUnstaged,
  loading,
  error,
  onSelectFile,
  onNavigateToSession,
}: SessionFilesRouteProps) {
  if (loading) {
    return (
      <div style={{ maxWidth: "760px", margin: "0 auto" }}>
        <Card variant="default">
          <CardContent>
            <div style={{ textAlign: "center", padding: "var(--space-8)" }}>
              Loading session files...
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

  return (
    <div style={{ maxWidth: "760px", margin: "0 auto" }}>
      <Card variant="default">
        <CardHeader>
          <CardTitle>Session Files</CardTitle>
          <CardDescription>
            {session.metadata?.path ?? "Workspace files"}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "var(--space-3)",
              marginBottom: "var(--space-4)",
            }}
          >
            <StatCard label="Branch" value={branch ?? "Detached"} />
            <StatCard label="Staged" value={String(totalStaged)} />
            <StatCard label="Unstaged" value={String(totalUnstaged)} />
          </div>

          {files.length === 0 ? (
            <div
              style={{
                textAlign: "center",
                padding: "var(--space-8)",
                color: "var(--text-secondary)",
              }}
            >
              No files in workspace. The workspace is clean or not under git
              control.
            </div>
          ) : (
            <div style={{ display: "grid", gap: "var(--space-2)" }}>
              {files.map((file) => (
                <button
                  key={`${file.relativePath}-${file.isStaged ? "staged" : "unstaged"}`}
                  type="button"
                  onClick={() => onSelectFile(file.relativePath)}
                  style={{
                    display: "grid",
                    gridTemplateColumns: "1fr auto",
                    alignItems: "center",
                    padding: "var(--space-3)",
                    borderRadius: "var(--radius-md)",
                    border: "1px solid var(--border-default)",
                    background: "transparent",
                    cursor: "pointer",
                    textAlign: "left",
                  }}
                >
                  <div>
                    <div style={{ fontWeight: 500 }}>{file.fileName}</div>
                    <div
                      style={{
                        fontSize: "var(--text-sm)",
                        color: "var(--text-secondary)",
                      }}
                    >
                      {file.relativePath}
                    </div>
                  </div>
                  <span
                    style={{
                      padding: "var(--space-1) var(--space-2)",
                      borderRadius: "var(--radius-full)",
                      backgroundColor:
                        file.status === "added"
                          ? "var(--color-success-subtle)"
                          : file.status === "deleted"
                            ? "var(--color-error-subtle)"
                            : file.status === "modified"
                              ? "var(--color-warning-subtle)"
                              : "var(--color-surface-elevated)",
                      fontSize: "var(--text-xs)",
                    }}
                  >
                    {file.status}
                  </span>
                </button>
              ))}
            </div>
          )}

          <div style={{ marginTop: "var(--space-4)" }}>
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
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        padding: "var(--space-3)",
        backgroundColor: "var(--color-surface-elevated)",
        borderRadius: "var(--radius-md)",
      }}
    >
      <div
        style={{
          fontSize: "var(--text-xs)",
          color: "var(--text-secondary)",
          marginBottom: "var(--space-1)",
        }}
      >
        {label}
      </div>
      <div style={{ fontWeight: 500 }}>{value}</div>
    </div>
  );
}
