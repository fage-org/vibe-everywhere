import { useState, useEffect, useMemo, useRef, useCallback } from "react";
import { useTranslation } from "react-i18next";
import type { DesktopArtifact } from "../../desktop-client";
import { tokens } from "../../design-system/tokens";
import { Card, CardContent, Button, Input, Badge, ShimmerCard } from "../../components/ui";
import { Title2, Body, Subheadline, Eyebrow, Caption1 } from "../../components/ui/Typography";
import { useDesktopState } from "../../useDesktopState";
import { navigateToPath } from "../../router";

export interface ArtifactEditRouteProps {
  /** Artifact ID */
  artifactId: string;
  /** Optional artifact override (for testing) */
  artifact?: DesktopArtifact | null;
  /** Loading state override */
  loading?: boolean;
  /** Error state override */
  error?: string | null;
  /** Callback when navigating back */
  onBack?: () => void;
  /** Callback after successful save */
  onSave?: (artifact: DesktopArtifact) => void;
}

/**
 * ArtifactEditRoute - Artifact editor page
 *
 * Features:
 * - Text editor for content
 * - Title editing
 * - Save/publish actions
 * - Unsaved changes warning
 */
export function ArtifactEditRoute({
  artifactId,
  artifact: artifactOverride,
  loading: loadingOverride,
  error: errorOverride,
  onBack,
  onSave,
}: ArtifactEditRouteProps) {
  const { t } = useTranslation("routes");
  const { artifacts, loadArtifact, updateArtifact, globalError } = useDesktopState();

  const [localArtifact, setLocalArtifact] = useState<DesktopArtifact | null>(artifactOverride ?? null);
  const [isLoading, setIsLoading] = useState(loadingOverride ?? !artifactOverride);
  const [error, setError] = useState<string | null>(errorOverride ?? globalError);
  const [isSaving, setIsSaving] = useState(false);

  // Form state
  const [title, setTitle] = useState("");
  const [body, setBody] = useState("");
  const [originalTitle, setOriginalTitle] = useState("");
  const [originalBody, setOriginalBody] = useState("");

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Track unsaved changes
  const hasChanges = title !== originalTitle || body !== originalBody;

  // Load artifact if not provided
  useEffect(() => {
    if (artifactOverride) {
      setLocalArtifact(artifactOverride);
      setTitle(artifactOverride.title ?? "");
      setBody(artifactOverride.body ?? "");
      setOriginalTitle(artifactOverride.title ?? "");
      setOriginalBody(artifactOverride.body ?? "");
      setIsLoading(false);
      return;
    }

    // Try to find in existing artifacts
    const existing = artifacts.find((a) => a.id === artifactId);
    if (existing) {
      setLocalArtifact(existing);
      setTitle(existing.title ?? "");
      setBody(existing.body ?? "");
      setOriginalTitle(existing.title ?? "");
      setOriginalBody(existing.body ?? "");
      setIsLoading(false);
      return;
    }

    // Load from server
    setIsLoading(true);
    loadArtifact(artifactId)
      .then((loaded) => {
        setLocalArtifact(loaded);
        setTitle(loaded?.title ?? "");
        setBody(loaded?.body ?? "");
        setOriginalTitle(loaded?.title ?? "");
        setOriginalBody(loaded?.body ?? "");
        setError(null);
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : "Failed to load artifact");
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [artifactId, artifactOverride, artifacts, loadArtifact]);

  const handleSave = useCallback(async () => {
    if (!localArtifact || isSaving) return;

    setIsSaving(true);
    try {
      const updated = await updateArtifact(localArtifact.id, {
        title: title || null,
        body: body || null,
      });
      setOriginalTitle(title);
      setOriginalBody(body);
      setLocalArtifact(updated);

      if (onSave) {
        onSave(updated);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save artifact");
    } finally {
      setIsSaving(false);
    }
  }, [localArtifact, title, body, isSaving, updateArtifact, onSave]);

  const handleCancel = () => {
    if (onBack) {
      onBack();
    } else {
      navigateToPath(`/(app)/artifacts/${artifactId}`);
    }
  };

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "s") {
        e.preventDefault();
        if (hasChanges && !isSaving) {
          handleSave();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [hasChanges, isSaving, handleSave]);

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
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[2] }}>
              <Eyebrow>{t("artifactEdit.header.eyebrow")}</Eyebrow>
              {hasChanges && (
                <Badge variant="warning" size="sm">{t("artifactEdit.badges.unsaved")}</Badge>
              )}
            </div>
            <Title2>{t("artifactEdit.title")}</Title2>
          </div>

          <div style={{ display: "flex", gap: tokens.spacing[2] }}>
            <Button variant="ghost" size="sm" onClick={handleCancel}>
              {t("artifactEdit.actions.cancel")}
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSave}
              disabled={!hasChanges || isSaving}
              loading={isSaving}
            >
              {t("artifactEdit.actions.save")}
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
          <Card variant="default" padding="md" style={{ borderColor: "var(--color-error)" }}>
            <Body style={{ color: "var(--color-error)" }}>{error}</Body>
          </Card>
        )}

        {isLoading && !error && (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: tokens.spacing[4],
            }}
          >
            <ShimmerCard style={{ height: "60px" }} />
            <ShimmerCard style={{ height: "400px" }} />
          </div>
        )}

        {!isLoading && !error && localArtifact && (
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
              <Caption1 color="tertiary">{t("artifactEdit.fields.title")}</Caption1>
              <Input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder={t("artifactEdit.fields.titlePlaceholder")}
                style={{
                  marginTop: tokens.spacing[2],
                  fontSize: tokens.typography.fontSize.lg,
                  fontWeight: tokens.typography.fontWeight.medium,
                }}
              />
            </div>

            {/* Content Editor */}
            <div style={{ display: "flex", flexDirection: "column", flex: 1 }}>
              <Caption1 color="tertiary">{t("artifactEdit.fields.content")}</Caption1>
              <Card
                variant="default"
                padding="none"
                style={{ marginTop: tokens.spacing[2], flex: 1 }}
              >
                <textarea
                  ref={textareaRef}
                  value={body}
                  onChange={(e) => setBody(e.target.value)}
                  placeholder={t("artifactEdit.fields.contentPlaceholder")}
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
              <Caption1 color="tertiary" style={{ marginTop: tokens.spacing[2] }}>
                {t("artifactEdit.hints.shortcut")}
              </Caption1>
            </div>

            {/* Info Section */}
            <Card variant="default" padding="md">
              <CardContent>
                <div
                  style={{
                    display: "grid",
                    gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))",
                    gap: tokens.spacing[4],
                  }}
                >
                  <div>
                    <Caption1 color="tertiary">{t("artifactEdit.metadata.id")}</Caption1>
                    <Body
                      style={{
                        marginTop: tokens.spacing[1],
                        fontFamily: tokens.typography.fontFamily.mono,
                        fontSize: tokens.typography.fontSize.xs,
                      }}
                    >
                      {localArtifact.id.slice(0, 8)}...
                    </Body>
                  </div>
                  <div>
                    <Caption1 color="tertiary">{t("artifactEdit.metadata.version")}</Caption1>
                    <Body style={{ marginTop: tokens.spacing[1] }}>{localArtifact.seq}</Body>
                  </div>
                  <div>
                    <Caption1 color="tertiary">{t("artifactEdit.metadata.status")}</Caption1>
                    <div style={{ marginTop: tokens.spacing[1] }}>
                      {localArtifact.isDecrypted ? (
                        <Badge variant="success">{t("artifactEdit.status.decrypted")}</Badge>
                      ) : (
                        <Badge variant="warning">{t("artifacts.badges.locked")}</Badge>
                      )}
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </div>
    </div>
  );
}