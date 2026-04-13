import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Body, Title3, Caption1 } from "../../components/ui/Typography";
import { Card, CardContent } from "../../components/ui";
import { tokens } from "../../design-system/tokens";
import { useDesktopState } from "../../useDesktopState";
import { approveTerminalConnection } from "../../desktop-client";

/**
 * TerminalConnectRoute - Terminal connection approval page
 *
 * Handles terminal connection requests from CLI:
 * - Receives public key from URL parameter
 * - Shows confirmation dialog with security info
 * - Authorizes or rejects the connection
 */
export function TerminalConnectRoute({ publicKey }: { publicKey: string }) {
  const { t } = useTranslation("ui");
  const { status, credentials } = useDesktopState();

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleApprove = useCallback(async () => {
    if (!credentials || status !== "ready") {
      setError(t("terminal.notAuthenticated"));
      return;
    }

    setIsLoading(true);
    setError(null);
    try {
      const serverUrl = new URL(window.location.href).origin;
      await approveTerminalConnection(serverUrl, credentials, publicKey);
      setSuccess(true);
    } catch (err) {
      console.error("Failed to approve terminal connection:", err);
      setError(err instanceof Error ? err.message : t("terminal.connectionFailed"));
    } finally {
      setIsLoading(false);
    }
  }, [credentials, publicKey, status, t]);

  const handleReject = () => {
    window.history.back();
  };

  // Validate public key format
  const isValidPublicKey = publicKey && publicKey.length > 0;

  if (!isValidPublicKey) {
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          height: "100%",
          backgroundColor: "var(--bg-primary)",
          overflow: "auto",
        }}
      >
        <div
          style={{
            padding: tokens.spacing[6],
            display: "flex",
            flexDirection: "column",
            gap: tokens.spacing[4],
          }}
        >
          <Card variant="default" padding="lg" style={{ borderColor: "var(--color-error)" }}>
            <CardContent>
              <div style={{ textAlign: "center" }}>
                <div
                  style={{
                    fontSize: "48px",
                    marginBottom: tokens.spacing[4],
                  }}
                >
                  ⚠️
                </div>
                <Title3 style={{ color: "var(--color-error)", marginBottom: tokens.spacing[2] }}>
                  {t("terminal.invalidLink")}
                </Title3>
                <Body color="secondary">
                  {t("terminal.invalidLinkDescription")}
                </Body>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  if (success) {
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          height: "100%",
          backgroundColor: "var(--bg-primary)",
          overflow: "auto",
        }}
      >
        <div
          style={{
            padding: tokens.spacing[6],
            display: "flex",
            flexDirection: "column",
            gap: tokens.spacing[4],
          }}
        >
          <Card variant="default" padding="lg" style={{ borderColor: "var(--color-success)" }}>
            <CardContent>
              <div style={{ textAlign: "center" }}>
                <div
                  style={{
                    fontSize: "48px",
                    marginBottom: tokens.spacing[4],
                  }}
                >
                  ✅
                </div>
                <Title3 style={{ color: "var(--color-success)", marginBottom: tokens.spacing[2] }}>
                  {t("terminal.connected")}
                </Title3>
                <Body color="secondary">
                  {t("terminal.connectedDescription")}
                </Body>
                <button
                  onClick={handleReject}
                  style={{
                    marginTop: tokens.spacing[4],
                    padding: `${tokens.spacing[3]} ${tokens.spacing[6]}`,
                    backgroundColor: "var(--color-primary)",
                    border: "none",
                    borderRadius: tokens.radii.md,
                    color: "white",
                    cursor: "pointer",
                    fontSize: tokens.typography.fontSize.base,
                    fontWeight: 600,
                  }}
                >
                  {t("terminal.done")}
                </button>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

  const maskedKey = publicKey.length > 12 ? `${publicKey.substring(0, 12)}...` : publicKey;

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "var(--bg-primary)",
        overflow: "auto",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: `${tokens.spacing[6]} ${tokens.spacing[6]} ${tokens.spacing[4]}`,
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        <Title3>{t("terminal.connectTerminal")}</Title3>
      </div>

      {/* Content */}
      <div
        style={{
          flex: 1,
          padding: tokens.spacing[6],
          display: "flex",
          flexDirection: "column",
          gap: tokens.spacing[4],
        }}
      >
        {/* Connection icon */}
        <Card variant="default" padding="lg">
          <CardContent>
            <div style={{ textAlign: "center" }}>
              <div
                style={{
                  fontSize: "48px",
                  marginBottom: tokens.spacing[4],
                }}
              >
                💻
              </div>
              <Title3 style={{ marginBottom: tokens.spacing[2] }}>
                {t("terminal.terminalRequestTitle")}
              </Title3>
              <Body color="secondary">
                {t("terminal.terminalRequestDescription")}
              </Body>
            </div>
          </CardContent>
        </Card>

        {/* Connection details */}
        <Card variant="default" padding="md">
          <CardContent>
            <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
              <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
                <span style={{ fontSize: "20px" }}>🔑</span>
                <div>
                  <Caption1 color="secondary">{t("terminal.publicKey")}</Caption1>
                  <Body
                    style={{
                      fontFamily: tokens.typography.fontFamily.mono,
                      fontSize: tokens.typography.fontSize.sm,
                    }}
                  >
                    {maskedKey}
                  </Body>
                </div>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
                <span style={{ fontSize: "20px" }}>🔒</span>
                <div>
                  <Caption1 color="secondary">{t("terminal.encryption")}</Caption1>
                  <Body style={{ color: "var(--color-success)" }}>
                    {t("terminal.endToEndEncrypted")}
                  </Body>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Error */}
        {error && (
          <Card variant="default" padding="md" style={{ borderColor: "var(--color-error)" }}>
            <Body style={{ color: "var(--color-error)" }}>{error}</Body>
          </Card>
        )}

        {/* Action buttons */}
        <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
          <button
            onClick={handleApprove}
            disabled={isLoading || status !== "ready"}
            style={{
              padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
              backgroundColor: isLoading || status !== "ready" ? "var(--surface-tertiary)" : "var(--color-primary)",
              border: "none",
              borderRadius: tokens.radii.md,
              color: isLoading || status !== "ready" ? "var(--text-tertiary)" : "white",
              cursor: isLoading || status !== "ready" ? "not-allowed" : "pointer",
              fontSize: tokens.typography.fontSize.base,
              fontWeight: 600,
            }}
          >
            {isLoading ? t("terminal.connecting") : t("terminal.acceptConnection")}
          </button>
          <button
            onClick={handleReject}
            disabled={isLoading}
            style={{
              padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
              backgroundColor: "var(--surface-secondary)",
              border: "1px solid var(--border-primary)",
              borderRadius: tokens.radii.md,
              color: "var(--text-primary)",
              cursor: isLoading ? "not-allowed" : "pointer",
              fontSize: tokens.typography.fontSize.base,
            }}
          >
            {t("terminal.reject")}
          </button>
        </div>

        {/* Security notice */}
        <Card variant="default" padding="md">
          <CardContent>
            <div style={{ display: "flex", alignItems: "flex-start", gap: tokens.spacing[3] }}>
              <span style={{ fontSize: "20px" }}>🛡️</span>
              <div>
                <Caption1 style={{ fontWeight: 600 }}>{t("terminal.securityNote")}</Caption1>
                <Caption1 color="secondary" style={{ display: "block", marginTop: tokens.spacing[1] }}>
                  {t("terminal.securityNoteDescription")}
                </Caption1>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export default TerminalConnectRoute;
