import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import type { DesktopArtifact } from "../../desktop-client";
import { tokens } from "../../design-system/tokens";
import { Card, CardContent, Button, Badge, ShimmerCard } from "../../components/ui";
import { Title2, Body, Subheadline, Eyebrow, Caption1 } from "../../components/ui/Typography";
import { useDesktopState } from "../../useDesktopState";
import { navigateToPath } from "../../router";

export interface ArtifactDetailRouteProps {
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
  /** Callback when editing artifact */
  onEdit?: (artifactId: string) => void;
  /** Callback when deleting artifact */
  onDelete?: (artifactId: string) => void;
}

/**
 * ArtifactDetailRoute - Artifact detail and view page
 *
 * Displays:
 * - Artifact metadata (title, created, updated)
 * - Content viewer with syntax highlighting
 * - Edit and delete actions
 * - Session associations
 */
export function ArtifactDetailRoute({
  artifactId,
  artifact: artifactOverride,
  loading: loadingOverride,
  error: errorOverride,
  onBack,
  onEdit,
  onDelete,
}: ArtifactDetailRouteProps) {
  const { t } = useTranslation("routes");
  const { artifacts, loadArtifact, deleteArtifact, globalError } = useDesktopState();

  const [localArtifact, setLocalArtifact] = useState<DesktopArtifact | null>(artifactOverride ?? null);
  const [isLoading, setIsLoading] = useState(loadingOverride ?? !artifactOverride);
  const [error, setError] = useState<string | null>(errorOverride ?? globalError);
  const [isDeleting, setIsDeleting] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  // Load artifact if not provided
  useEffect(() => {
    if (artifactOverride) {
      setLocalArtifact(artifactOverride);
      setIsLoading(false);
      return;
    }

    // Try to find in existing artifacts
    const existing = artifacts.find((a) => a.id === artifactId);
    if (existing) {
      setLocalArtifact(existing);
      setIsLoading(false);
      return;
    }

    // Load from server
    setIsLoading(true);
    loadArtifact(artifactId)
      .then((loaded) => {
        setLocalArtifact(loaded);
        setError(null);
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : "Failed to load artifact");
      })
      .finally(() => {
        setIsLoading(false);
      });
  }, [artifactId, artifactOverride, artifacts, loadArtifact]);

  const handleEdit = () => {
    if (onEdit) {
      onEdit(artifactId);
    } else {
      navigateToPath(`/(app)/artifacts/edit/${artifactId}`);
    }
  };

  const handleDelete = async () => {
    if (isDeleting) return;

    setIsDeleting(true);
    try {
      await deleteArtifact(artifactId);
      setShowDeleteConfirm(false);
      if (onBack) {
        onBack();
      } else {
        navigateToPath(`/(app)/artifacts/index`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete artifact");
    } finally {
      setIsDeleting(false);
    }
  };

  const formattedCreatedAt = useMemo(() => {
    if (!localArtifact) return "";
    const date = new Date(localArtifact.createdAt);
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }, [localArtifact]);

  const formattedUpdatedAt = useMemo(() => {
    if (!localArtifact) return "";
    const date = new Date(localArtifact.updatedAt);
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "long",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }, [localArtifact]);

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
              <Eyebrow>{t("artifactDetail.header.eyebrow")}</Eyebrow>
              {localArtifact?.draft && (
                <Badge variant="warning" size="sm">{t("artifacts.badges.draft")}</Badge>
              )}
            </div>
            <Title2
              style={{
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap",
              }}
            >
              {localArtifact?.title ?? t("artifacts.untitled")}
            </Title2>
          </div>

          <div style={{ display: "flex", gap: tokens.spacing[2] }}>
            <Button variant="ghost" size="sm" onClick={onBack ? onBack : () => navigateToPath(`/(app)/artifacts/index`)}>
              {t("artifactDetail.actions.back")}
            </Button>
            {localArtifact?.isDecrypted && (
              <Button variant="secondary" size="sm" onClick={handleEdit}>
                {t("artifactDetail.actions.edit")}
              </Button>
            )}
            <Button
              variant="danger"
              size="sm"
              onClick={() => setShowDeleteConfirm(true)}
              disabled={!localArtifact || isDeleting}
            >
              {t("artifactDetail.actions.delete")}
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
            <ShimmerCard style={{ height: "100px" }} />
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
            {/* Metadata */}
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
                    <Caption1 color="tertiary">{t("artifactDetail.metadata.createdAt")}</Caption1>
                    <Body style={{ marginTop: tokens.spacing[1] }}>{formattedCreatedAt}</Body>
                  </div>
                  <div>
                    <Caption1 color="tertiary">{t("artifactDetail.metadata.updatedAt")}</Caption1>
                    <Body style={{ marginTop: tokens.spacing[1] }}>{formattedUpdatedAt}</Body>
                  </div>
                  <div>
                    <Caption1 color="tertiary">{t("artifactDetail.metadata.version")}</Caption1>
                    <Body style={{ marginTop: tokens.spacing[1] }}>{localArtifact.seq}</Body>
                  </div>
                  {!localArtifact.isDecrypted && (
                    <div>
                      <Caption1 color="tertiary">{t("artifactDetail.metadata.status")}</Caption1>
                      <div style={{ marginTop: tokens.spacing[1] }}>
                        <Badge variant="warning">{t("artifacts.badges.locked")}</Badge>
                      </div>
                    </div>
                  )}
                </div>

                {localArtifact.sessions && localArtifact.sessions.length > 0 && (
                  <div style={{ marginTop: tokens.spacing[4] }}>
                    <Caption1 color="tertiary">{t("artifactDetail.metadata.sessions")}</Caption1>
                    <div
                      style={{
                        display: "flex",
                        flexWrap: "wrap",
                        gap: tokens.spacing[2],
                        marginTop: tokens.spacing[2],
                      }}
                    >
                      {localArtifact.sessions.map((sessionId) => (
                        <Badge key={sessionId} variant="secondary" size="sm">
                          {sessionId.slice(0, 8)}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Content Viewer */}
            <Card variant="default" padding="none">
              <CardContent>
                <div
                  style={{
                    padding: tokens.spacing[4],
                    borderBottom: "1px solid var(--border-primary)",
                  }}
                >
                  <Eyebrow>{t("artifactDetail.content.title")}</Eyebrow>
                </div>
                <div
                  style={{
                    padding: tokens.spacing[4],
                    backgroundColor: "var(--surface-secondary)",
                    maxHeight: "500px",
                    overflow: "auto",
                  }}
                >
                  {localArtifact.isDecrypted ? (
                    localArtifact.body ? (
                      <pre
                        style={{
                          margin: 0,
                          fontFamily: tokens.typography.fontFamily.mono,
                          fontSize: tokens.typography.fontSize.sm,
                          lineHeight: 1.6,
                          whiteSpace: "pre-wrap",
                          wordBreak: "break-word",
                          color: "var(--text-primary)",
                        }}
                      >
                        {localArtifact.body}
                      </pre>
                    ) : (
                      <Body color="tertiary">{t("artifactDetail.content.empty")}</Body>
                    )
                  ) : (
                    <Body color="tertiary">{t("artifactDetail.content.encrypted")}</Body>
                  )}
                </div>
              </CardContent>
            </Card>
          </div>
        )}
      </div>

      {/* Delete Confirmation Modal */}
      {showDeleteConfirm && (
        <div
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: "rgba(0, 0, 0, 0.5)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
        >
          <Card variant="default" padding="lg" style={{ maxWidth: "400px", width: "100%" }}>
            <Title2>{t("artifactDetail.delete.title")}</Title2>
            <Body color="secondary" style={{ marginTop: tokens.spacing[3] }}>
              {t("artifactDetail.delete.description")}
            </Body>
            <div
              style={{
                display: "flex",
                gap: tokens.spacing[3],
                marginTop: tokens.spacing[6],
                justifyContent: "flex-end",
              }}
            >
              <Button
                variant="ghost"
                onClick={() => setShowDeleteConfirm(false)}
                disabled={isDeleting}
              >
                {t("artifactDetail.delete.cancel")}
              </Button>
              <Button
                variant="danger"
                onClick={handleDelete}
                loading={isDeleting}
              >
                {t("artifactDetail.delete.confirm")}
              </Button>
            </div>
          </Card>
        </div>
      )}
    </div>
  );
}