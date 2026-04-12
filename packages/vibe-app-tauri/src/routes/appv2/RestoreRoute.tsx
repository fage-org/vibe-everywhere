import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui";
import { Button } from "../../components/ui";

type RestoreRouteProps = {
  qrSvg: string | null;
  error: string | null;
  isLoading: boolean;
  onRefreshQr: () => void;
  onManualRestore: () => void;
};

export function RestoreRoute({
  qrSvg,
  error,
  isLoading,
  onRefreshQr,
  onManualRestore,
}: RestoreRouteProps) {
  return (
    <div style={{ maxWidth: "760px", margin: "0 auto" }}>
      <Card variant="default">
        <CardHeader>
          <CardTitle>Restore Account</CardTitle>
          <CardDescription>
            Link this device to your existing Vibe account
          </CardDescription>
        </CardHeader>
        <CardContent>
          {error && (
            <div
              style={{
                padding: "var(--space-3)",
                marginBottom: "var(--space-4)",
                backgroundColor: "var(--color-error-subtle)",
                borderRadius: "var(--radius-md)",
                color: "var(--color-error)",
              }}
            >
              {error}
            </div>
          )}

          {isLoading ? (
            <div
              style={{
                padding: "var(--space-8)",
                textAlign: "center",
                color: "var(--text-secondary)",
              }}
            >
              Preparing QR code...
            </div>
          ) : qrSvg ? (
            <div
              style={{
                padding: "var(--space-4)",
                backgroundColor: "var(--color-surface-elevated)",
                borderRadius: "var(--radius-lg)",
                marginBottom: "var(--space-4)",
              }}
            >
              <div
                style={{ width: "100%", aspectRatio: "1" }}
                dangerouslySetInnerHTML={{ __html: qrSvg }}
              />
            </div>
          ) : (
            <div
              style={{
                padding: "var(--space-8)",
                textAlign: "center",
                color: "var(--text-secondary)",
              }}
            >
              QR code unavailable
            </div>
          )}

          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "var(--space-3)",
            }}
          >
            <Button variant="secondary" onClick={onRefreshQr}>
              Refresh QR Code
            </Button>
            <Button variant="secondary" onClick={onManualRestore}>
              Restore with Secret Key
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
