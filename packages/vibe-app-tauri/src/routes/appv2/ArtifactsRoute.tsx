import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import type { DesktopArtifact } from "../../desktop-client";
import { tokens } from "../../design-system/tokens";
import { Card, CardContent, Button, Input, Badge, ShimmerCard } from "../../components/ui";
import { Title2, Body, Subheadline, Eyebrow, Caption1 } from "../../components/ui/Typography";
import { useDesktopState } from "../../useDesktopState";
import { navigateToPath } from "../../router";

export interface ArtifactsRouteProps {
  /** Optional artifacts override (for testing) */
  artifacts?: DesktopArtifact[];
  /** Loading state override */
  loading?: boolean;
  /** Error state override */
  error?: string | null;
  /** Callback when artifact is selected */
  onSelectArtifact?: (artifactId: string) => void;
}

/**
 * ArtifactsRoute - Artifacts list and browse page
 *
 * Displays all user artifacts with:
 * - Search and filter functionality
 * - Paginated list view
 * - Quick access to create new artifacts
 * - Click to view artifact details
 */
export function ArtifactsRoute({
  artifacts: artifactsOverride,
  loading: loadingOverride,
  error: errorOverride,
  onSelectArtifact,
}: ArtifactsRouteProps) {
  const { t } = useTranslation("routes");
  const { artifacts: contextArtifacts, globalError, refreshArtifacts } = useDesktopState();

  const artifacts = artifactsOverride ?? contextArtifacts ?? [];
  const loading = loadingOverride ?? (artifacts.length === 0 && !globalError);
  const error = errorOverride ?? globalError;

  const [searchQuery, setSearchQuery] = useState("");
  const [sortBy, setSortBy] = useState<"updated" | "created" | "title">("updated");

  // Filter artifacts by search query
  const filteredArtifacts = useMemo(() => {
    if (!searchQuery.trim()) return artifacts;

    const query = searchQuery.toLowerCase();
    return artifacts.filter((artifact) => {
      const title = artifact.title?.toLowerCase() ?? "";
      return title.includes(query);
    });
  }, [artifacts, searchQuery]);

  // Sort artifacts
  const sortedArtifacts = useMemo(() => {
    const sorted = [...filteredArtifacts];
    switch (sortBy) {
      case "created":
        return sorted.sort((a, b) => b.createdAt - a.createdAt);
      case "title":
        return sorted.sort((a, b) => {
          const titleA = a.title ?? "";
          const titleB = b.title ?? "";
          return titleA.localeCompare(titleB);
        });
      case "updated":
      default:
        return sorted.sort((a, b) => b.updatedAt - a.updatedAt);
    }
  }, [filteredArtifacts, sortBy]);

  // Navigate to artifact detail
  const handleSelectArtifact = (artifactId: string) => {
    if (onSelectArtifact) {
      onSelectArtifact(artifactId);
    } else {
      navigateToPath(`/(app)/artifacts/${artifactId}`);
    }
  };

  // Navigate to create new artifact
  const handleCreateNew = () => {
    navigateToPath(`/(app)/artifacts/new`);
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
            <Eyebrow>{t("artifacts.header.eyebrow")}</Eyebrow>
            <Title2>{t("artifacts.title")}</Title2>
            <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
              {t("artifacts.description")}
            </Body>
          </div>

          <Button variant="primary" size="sm" onClick={handleCreateNew}>
            {t("artifacts.actions.new")}
          </Button>
        </div>

        {/* Search and Filter */}
        <div
          style={{
            display: "flex",
            gap: tokens.spacing[3],
            marginTop: tokens.spacing[4],
          }}
        >
          <Input
            type="text"
            placeholder={t("artifacts.search.placeholder")}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            style={{ flex: 1 }}
          />
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as "updated" | "created" | "title")}
            style={{
              padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
              backgroundColor: "var(--surface-secondary)",
              border: "1px solid var(--border-primary)",
              borderRadius: tokens.radii.md,
              color: "var(--text-primary)",
              fontSize: tokens.typography.fontSize.sm,
              cursor: "pointer",
            }}
          >
            <option value="updated">{t("artifacts.sort.updated")}</option>
            <option value="created">{t("artifacts.sort.created")}</option>
            <option value="title">{t("artifacts.sort.title")}</option>
          </select>
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
            <Button
              variant="ghost"
              size="sm"
              onClick={() => refreshArtifacts()}
              style={{ marginTop: tokens.spacing[3] }}
            >
              {t("artifacts.actions.retry")}
            </Button>
          </Card>
        )}

        {loading && !error && (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: tokens.spacing[4],
            }}
          >
            {[1, 2, 3, 4, 5].map((i) => (
              <ShimmerCard key={i} style={{ height: "80px" }} />
            ))}
          </div>
        )}

        {!loading && !error && sortedArtifacts.length === 0 && (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              textAlign: "center",
            }}
          >
            <Title2 color="secondary">{t("artifacts.empty.title")}</Title2>
            <Body color="tertiary" style={{ marginTop: tokens.spacing[2] }}>
              {searchQuery
                ? t("artifacts.empty.noResults")
                : t("artifacts.empty.description")}
            </Body>
            {!searchQuery && (
              <Button
                variant="primary"
                onClick={handleCreateNew}
                style={{ marginTop: tokens.spacing[4] }}
              >
                {t("artifacts.actions.createFirst")}
              </Button>
            )}
          </div>
        )}

        {!loading && !error && sortedArtifacts.length > 0 && (
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: tokens.spacing[3],
            }}
          >
            {sortedArtifacts.map((artifact) => (
              <ArtifactCard
                key={artifact.id}
                artifact={artifact}
                onClick={() => handleSelectArtifact(artifact.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

interface ArtifactCardProps {
  artifact: DesktopArtifact;
  onClick: () => void;
}

function ArtifactCard({ artifact, onClick }: ArtifactCardProps) {
  const { t } = useTranslation("routes");

  const formattedDate = useMemo(() => {
    const date = new Date(artifact.updatedAt);
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }, [artifact.updatedAt]);

  return (
    <Card
      variant="default"
      padding="none"
      onClick={onClick}
      style={{ cursor: "pointer" }}
    >
      <CardContent>
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            gap: tokens.spacing[4],
            padding: tokens.spacing[4],
          }}
        >
          <div style={{ flex: 1, minWidth: 0 }}>
            <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[2] }}>
              <Subheadline style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {artifact.title ?? t("artifacts.untitled")}
              </Subheadline>
              {artifact.draft && (
                <Badge variant="warning" size="sm">{t("artifacts.badges.draft")}</Badge>
              )}
              {!artifact.isDecrypted && (
                <Badge variant="secondary" size="sm">{t("artifacts.badges.locked")}</Badge>
              )}
            </div>
            <Caption1 color="tertiary" style={{ marginTop: tokens.spacing[1] }}>
              {t("artifacts.updated", { date: formattedDate })}
            </Caption1>
          </div>

          <Button variant="ghost" size="sm">
            {t("artifacts.actions.view")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}