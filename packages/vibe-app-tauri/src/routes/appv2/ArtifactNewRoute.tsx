import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { tokens } from "../../design-system/tokens";
import { Card, CardContent, Button, Input, Badge } from "../../components/ui";
import { Title2, Body, Subheadline, Eyebrow, Caption1 } from "../../components/ui/Typography";
import { useDesktopState } from "../../useDesktopState";
import { navigateToPath } from "../../router";

export interface ArtifactNewRouteProps {
  /** Callback when navigating back */
  onBack?: () => void;
  /** Callback after successful creation */
  onCreate?: (artifactId: string) => void;
  /** Optional initial session ID to associate */
  sessionId?: string;
}

/**
 * ArtifactNewRoute - Create new artifact page
 *
 * Features:
 * - Title input
 * - Content editor
 * - Draft/public toggle
 * - Session association
 */
export function ArtifactNewRoute({
  onBack,
  onCreate,
  sessionId,
}: ArtifactNewRouteProps) {
  const { t } = useTranslation("routes");
  const { createArtifact } = useDesktopState();

  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [draft, setDraft] = useState(true);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = useCallback(async () => {
    if (isCreating) return;

    setIsCreating(true);
    setError(null);

    try {
      const artifact = await createArtifact({
        title: title.trim() || null,
        body: body || null,
        sessions: sessionId ? [sessionId] : undefined,
        draft,
      });

      if (onCreate) {
        onCreate(artifact.id);
      } else {
        navigateToPath(`/(app)/artifacts/${artifact.id}`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create artifact");
    } finally {
      setIsCreating(false);
    }
  }, [title, body, draft, sessionId, isCreating, createArtifact, onCreate]);

  const handleCancel = () => {
    if (onBack) {
      onBack();
    } else {
      navigateToPath(`/(app)/artifacts/index`);
    }
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "var(--bg-primary)",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: `${tokens.spacing[6]} ${tokens.spacing[6]} ${tokens.spacing[4]}`,
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "flex-start",
            justifyContent: "space-between",
            gap: tokens.spacing[4],
          }}
        >
          <div>
            <Eyebrow>{t("artifactNew.header.eyebrow")}</Eyebrow>
            <Title2>{t("artifactNew.title")}</Title2>
            <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
              {t("artifactNew.description")}
            </Body>
          </div>

          <div style={{ display: "flex", gap: tokens.spacing[2] }}>
            <Button variant="ghost" size="sm" onClick={handleCancel}>
              {t("artifactNew.actions.cancel")}
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleCreate}
              disabled={isCreating}
              loading={isCreating}
            >
              {t("artifactNew.actions.create")}
            </Button>
          </div>
        </div>
      </div>

      {/* Content */}
      <div
        style={{
          flex: 1,
          overflow: "auto",
          padding: tokens.spacing[6],
        }}
      >
        {error && (
          <Card variant="default" padding="md" style={{ marginBottom: tokens.spacing[4], borderColor: "var(--color-error)" }}>
            <Body style={{ color: "var(--color-error)" }}>{error}</Body>
          </Card>
        )}

        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: tokens.spacing[6],
            maxWidth: "900px",
          }}
        >
          {/* Title Input */}
          <div>
            <Caption1 color="tertiary">{t("artifactNew.fields.title")}</Caption1>
            <Input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder={t("artifactNew.fields.titlePlaceholder")}
              style={{
                marginTop: tokens.spacing[2],
                fontSize: tokens.typography.fontSize.lg,
                fontWeight: tokens.typography.fontWeight.medium,
              }}
            />
          </div>

          {/* Content Editor */}
          <div style={{ display: "flex", flexDirection: "column", flex: 1 }}>
            <Caption1 color="tertiary">{t("artifactNew.fields.content")}</Caption1>
            <Card
              variant="default"
              padding="none"
              style={{ marginTop: tokens.spacing[2], flex: 1 }}
            >
              <textarea
                value={body}
                onChange={(e) => setBody(e.target.value)}
                placeholder={t("artifactNew.fields.contentPlaceholder")}
                style={{
                  width: "100%",
                  minHeight: "400px",
                  padding: tokens.spacing[4],
                  backgroundColor: "transparent",
                  border: "none",
                  color: "var(--text-primary)",
                  fontFamily: tokens.typography.fontFamily.mono,
                  fontSize: tokens.typography.fontSize.sm,
                  lineHeight: 1.6,
                  resize: "vertical",
                  outline: "none",
                }}
              />
            </Card>
          </div>

          {/* Options */}
          <Card variant="default" padding="md">
            <CardContent>
              <Eyebrow>{t("artifactNew.options.title")}</Eyebrow>

              {/* Draft Toggle */}
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  padding: `${tokens.spacing[4]} 0`,
                  borderBottom: "1px solid var(--border-primary)",
                }}
              >
                <div>
                  <Subheadline>{t("artifactNew.options.draft")}</Subheadline>
                  <Body color="tertiary" style={{ marginTop: tokens.spacing[1] }}>
                    {t("artifactNew.options.draftDescription")}
                  </Body>
                </div>
                <button
                  onClick={() => setDraft(!draft)}
                  style={{
                    position: "relative",
                    width: "48px",
                    height: "28px",
                    borderRadius: tokens.radii.full,
                    backgroundColor: draft ? "var(--color-primary)" : "var(--surface-tertiary)",
                    border: "none",
                    cursor: "pointer",
                    transition: `background-color ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
                  }}
                >
                  <span
                    style={{
                      position: "absolute",
                      top: "2px",
                      left: draft ? "22px" : "2px",
                      width: "24px",
                      height: "24px",
                      borderRadius: tokens.radii.full,
                      backgroundColor: "#ffffff",
                      boxShadow: tokens.shadows.sm,
                      transition: `left ${tokens.animation.duration.fast} ${tokens.animation.easing.ios}`,
                    }}
                  />
                </button>
              </div>

              {/* Status Badge */}
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                  padding: `${tokens.spacing[4]} 0`,
                }}
              >
                <div>
                  <Subheadline>{t("artifactNew.options.status")}</Subheadline>
                  <Body color="tertiary" style={{ marginTop: tokens.spacing[1] }}>
                    {t("artifactNew.options.statusDescription")}
                  </Body>
                </div>
                <Badge variant={draft ? "warning" : "success"}>
                  {draft
                    ? t("artifactNew.status.draft")
                    : t("artifactNew.status.published")}
                </Badge>
              </div>

              {/* Session Association */}
              {sessionId && (
                <div
                  style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: `${tokens.spacing[4]} 0`,
                    borderTop: "1px solid var(--border-primary)",
                  }}
                >
                  <div>
                    <Subheadline>{t("artifactNew.options.session")}</Subheadline>
                    <Body color="tertiary" style={{ marginTop: tokens.spacing[1] }}>
                      {t("artifactNew.options.sessionDescription")}
                    </Body>
                  </div>
                  <Badge variant="secondary" size="sm">
                    {sessionId.slice(0, 8)}...
                  </Badge>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Help Text */}
          <Card variant="default" padding="md">
            <CardContent>
              <Eyebrow>{t("artifactNew.help.title")}</Eyebrow>
              <Body color="secondary" style={{ marginTop: tokens.spacing[3] }}>
                {t("artifactNew.help.content")}
              </Body>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}