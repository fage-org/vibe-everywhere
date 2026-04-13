import { useTranslation } from "react-i18next";
import { Body, Title3, Caption1 } from "../../components/ui/Typography";
import { Card, CardContent } from "../../components/ui";
import { tokens } from "../../design-system/tokens";

/**
 * TerminalRoute - Terminal feature main page
 *
 * Displays information about terminal connection:
 * - How to connect a terminal via CLI
 * - Security information
 * - Connection status
 */
export function TerminalRoute() {
  const { t } = useTranslation("ui");

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
        <Title3>{t("terminal.title")}</Title3>
        <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
          {t("terminal.description")}
        </Body>
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
        {/* How to connect */}
        <Card variant="default" padding="lg">
          <CardContent>
            <Title3 style={{ marginBottom: tokens.spacing[4] }}>
              {t("terminal.howToConnect")}
            </Title3>

            <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[4] }}>
              {/* Step 1 */}
              <div style={{ display: "flex", gap: tokens.spacing[3] }}>
                <div
                  style={{
                    width: "32px",
                    height: "32px",
                    borderRadius: "50%",
                    backgroundColor: "var(--color-primary)",
                    color: "white",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontWeight: 600,
                    flexShrink: 0,
                  }}
                >
                  1
                </div>
                <div>
                  <Body bold>{t("terminal.step1Title")}</Body>
                  <Caption1 color="secondary">{t("terminal.step1Description")}</Caption1>
                  <Card
                    variant="default"
                    padding="sm"
                    style={{
                      marginTop: tokens.spacing[2],
                      backgroundColor: "var(--surface-secondary)",
                    }}
                  >
                    <code
                      style={{
                        fontFamily: tokens.typography.fontFamily.mono,
                        fontSize: tokens.typography.fontSize.sm,
                      }}
                    >
                      vibe auth login
                    </code>
                  </Card>
                </div>
              </div>

              {/* Step 2 */}
              <div style={{ display: "flex", gap: tokens.spacing[3] }}>
                <div
                  style={{
                    width: "32px",
                    height: "32px",
                    borderRadius: "50%",
                    backgroundColor: "var(--color-primary)",
                    color: "white",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontWeight: 600,
                    flexShrink: 0,
                  }}
                >
                  2
                </div>
                <div>
                  <Body bold>{t("terminal.step2Title")}</Body>
                  <Caption1 color="secondary">{t("terminal.step2Description")}</Caption1>
                </div>
              </div>

              {/* Step 3 */}
              <div style={{ display: "flex", gap: tokens.spacing[3] }}>
                <div
                  style={{
                    width: "32px",
                    height: "32px",
                    borderRadius: "50%",
                    backgroundColor: "var(--color-primary)",
                    color: "white",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontWeight: 600,
                    flexShrink: 0,
                  }}
                >
                  3
                </div>
                <div>
                  <Body bold>{t("terminal.step3Title")}</Body>
                  <Caption1 color="secondary">{t("terminal.step3Description")}</Caption1>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Security information */}
        <Card variant="default" padding="lg">
          <CardContent>
            <Title3 style={{ marginBottom: tokens.spacing[4] }}>
              {t("terminal.security")}
            </Title3>

            <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
              <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
                <span style={{ fontSize: "20px" }}>🔐</span>
                <div>
                  <Body bold>{t("terminal.endToEndEncryption")}</Body>
                  <Caption1 color="secondary">{t("terminal.endToEndEncryptionDescription")}</Caption1>
                </div>
              </div>

              <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
                <span style={{ fontSize: "20px" }}>🔑</span>
                <div>
                  <Body bold>{t("terminal.publicKeyAuth")}</Body>
                  <Caption1 color="secondary">{t("terminal.publicKeyAuthDescription")}</Caption1>
                </div>
              </div>

              <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
                <span style={{ fontSize: "20px" }}>✅</span>
                <div>
                  <Body bold>{t("terminal.explicitApproval")}</Body>
                  <Caption1 color="secondary">{t("terminal.explicitApprovalDescription")}</Caption1>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Info card */}
        <Card variant="default" padding="md" style={{ backgroundColor: "var(--surface-secondary)" }}>
          <CardContent>
            <div style={{ display: "flex", alignItems: "flex-start", gap: tokens.spacing[3] }}>
              <span style={{ fontSize: "20px" }}>💡</span>
              <Caption1 color="secondary">
                {t("terminal.infoCard")}
              </Caption1>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export default TerminalRoute;
