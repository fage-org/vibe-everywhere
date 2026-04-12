import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../../components/ui";
import { Button, Input, TextArea } from "../../components/ui";

type ManualRestoreRouteProps = {
  onSubmit: (secret: string) => Promise<void>;
  isSubmitting: boolean;
  error: string | null;
};

export function ManualRestoreRoute({
  onSubmit,
  isSubmitting,
  error,
}: ManualRestoreRouteProps) {
  const [secret, setSecret] = useState("");

  const handleSubmit = async () => {
    if (secret.trim()) {
      await onSubmit(secret.trim());
    }
  };

  return (
    <div style={{ maxWidth: "760px", margin: "0 auto" }}>
      <Card variant="default">
        <CardHeader>
          <CardTitle>Manual Restore</CardTitle>
          <CardDescription>
            Restore your account using your secret key
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

          <div style={{ marginBottom: "var(--space-4)" }}>
            <label
              htmlFor="secret-key"
              style={{
                display: "block",
                marginBottom: "var(--space-2)",
                fontWeight: 500,
              }}
            >
              Secret Key
            </label>
            <TextArea
              id="secret-key"
              value={secret}
              onChange={(e) => setSecret(e.target.value)}
              placeholder="Enter your secret key..."
              rows={4}
              disabled={isSubmitting}
            />
          </div>

          <Button
            variant="primary"
            onClick={handleSubmit}
            disabled={!secret.trim() || isSubmitting}
          >
            {isSubmitting ? "Restoring..." : "Restore Account"}
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
