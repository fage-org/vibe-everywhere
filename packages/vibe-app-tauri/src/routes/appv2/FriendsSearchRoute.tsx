import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Body, Title3, Caption1, Subheadline } from "../../components/ui/Typography";
import { Card, CardContent, Badge, ShimmerCard } from "../../components/ui";
import { tokens } from "../../design-system/tokens";
import { useDesktopState } from "../../useDesktopState";
import type { UserProfile } from "../../desktop-wire";
import { hrefForPath } from "../../router";

/**
 * FriendsSearchRoute - Search for users to add as friends
 *
 * Features:
 * - Search input with real-time results
 * - User cards with avatar, name, and status
 * - Add friend button based on relationship status
 * - Navigation to user profile
 */
export function FriendsSearchRoute() {
  const { t } = useTranslation("ui");
  const { status, searchUsers, addFriend, userProfiles } = useDesktopState();

  const [query, setQuery] = useState("");
  const [results, setResults] = useState<UserProfile[]>([]);
  const [searching, setSearching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [addingId, setAddingId] = useState<string | null>(null);

  const handleSearch = useCallback(async () => {
    if (!query.trim() || status !== "ready") return;

    setSearching(true);
    setError(null);
    try {
      const users = await searchUsers(query.trim());
      setResults(users);
    } catch (err) {
      console.error("Failed to search users:", err);
      setError(err instanceof Error ? err.message : "Failed to search users");
    } finally {
      setSearching(false);
    }
  }, [query, searchUsers, status]);

  const handleAddFriend = async (userId: string) => {
    setAddingId(userId);
    setError(null);
    try {
      await addFriend(userId);
      // Update the results to reflect new status
      setResults((prev) =>
        prev.map((user) => {
          if (user.id === userId) {
            return { ...user, status: "requested" as const };
          }
          return user;
        }),
      );
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to add friend";
      setError(message);
    } finally {
      setAddingId(null);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    handleSearch();
  };

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
        <Title3>{t("friends.searchTitle")}</Title3>
        <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
          {t("friends.searchDescription")}
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
        {/* Search form */}
        <form onSubmit={handleSubmit}>
          <div style={{ display: "flex", gap: tokens.spacing[2] }}>
            <input
              type="text"
              placeholder={t("friends.searchPlaceholder")}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              style={{
                flex: 1,
                padding: tokens.spacing[3],
                backgroundColor: "var(--surface-secondary)",
                border: "1px solid var(--border-primary)",
                borderRadius: tokens.radii.md,
                color: "var(--text-primary)",
                fontSize: tokens.typography.fontSize.base,
              }}
            />
            <button
              type="submit"
              disabled={searching || !query.trim()}
              style={{
                padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
                backgroundColor: searching || !query.trim() ? "var(--surface-tertiary)" : "var(--color-primary)",
                border: "none",
                borderRadius: tokens.radii.md,
                color: searching || !query.trim() ? "var(--text-tertiary)" : "white",
                cursor: searching || !query.trim() ? "not-allowed" : "pointer",
                fontSize: tokens.typography.fontSize.base,
                fontWeight: 600,
              }}
            >
              {searching ? t("friends.searching") : t("friends.search")}
            </button>
          </div>
        </form>

        {/* Error */}
        {error && (
          <Card variant="default" padding="md" style={{ borderColor: "var(--color-error)" }}>
            <Body style={{ color: "var(--color-error)" }}>{error}</Body>
          </Card>
        )}

        {/* Loading */}
        {searching && (
          <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[4] }}>
            <ShimmerCard style={{ height: "80px" }} />
            <ShimmerCard style={{ height: "80px" }} />
            <ShimmerCard style={{ height: "80px" }} />
          </div>
        )}

        {/* Empty state */}
        {!searching && results.length === 0 && query.trim() && (
          <Card variant="default" padding="lg">
            <CardContent>
              <Body color="secondary" style={{ textAlign: "center" }}>
                {t("friends.noResults")}
              </Body>
            </CardContent>
          </Card>
        )}

        {/* Results */}
        {!searching && results.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
            {results.map((user) => (
              <UserSearchCard
                key={user.id}
                user={user}
                isAdding={addingId === user.id}
                onAdd={() => handleAddFriend(user.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * User search result card
 */
function UserSearchCard({
  user,
  isAdding,
  onAdd,
}: {
  user: UserProfile;
  isAdding: boolean;
  onAdd: () => void;
}) {
  const { t } = useTranslation("ui");

  const displayName = user.firstName || user.username || user.id.slice(0, 8);
  const fullName = [user.firstName, user.lastName].filter(Boolean).join(" ") || null;

  const getActionLabel = () => {
    switch (user.status) {
      case "friend":
        return t("friends.statusFriend");
      case "requested":
        return t("friends.statusRequested");
      case "pending":
        return t("friends.statusPending");
      default:
        return t("friends.addFriend");
    }
  };

  const canAdd = user.status === "none" || user.status === "rejected";

  return (
    <Card variant="default" padding="md">
      <CardContent>
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          {/* User info */}
          <a
            href={hrefForPath(`/(app)/user/${user.id}`)}
            style={{
              display: "flex",
              alignItems: "center",
              gap: tokens.spacing[3],
              textDecoration: "none",
              color: "inherit",
              flex: 1,
            }}
          >
            {/* Avatar */}
            <div
              style={{
                width: "48px",
                height: "48px",
                borderRadius: "50%",
                backgroundColor: "var(--surface-tertiary)",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                overflow: "hidden",
                flexShrink: 0,
              }}
            >
              {user.avatar?.url ? (
                <img
                  src={user.avatar.url}
                  alt={displayName}
                  style={{ width: "100%", height: "100%", objectFit: "cover" }}
                />
              ) : (
                <span style={{ fontSize: "20px", color: "var(--text-secondary)" }}>
                  {displayName.charAt(0).toUpperCase()}
                </span>
              )}
            </div>

            {/* Name and username */}
            <div style={{ minWidth: 0 }}>
              <Subheadline bold style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                {displayName}
              </Subheadline>
              {fullName && fullName !== displayName && (
                <Caption1 color="secondary" style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                  {fullName}
                </Caption1>
              )}
              <Caption1 color="tertiary">@{user.username}</Caption1>
            </div>
          </a>

          {/* Action button */}
          {canAdd ? (
            <button
              onClick={(e) => {
                e.preventDefault();
                onAdd();
              }}
              disabled={isAdding}
              style={{
                padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
                backgroundColor: isAdding ? "var(--surface-tertiary)" : "var(--color-primary)",
                border: "none",
                borderRadius: tokens.radii.md,
                color: isAdding ? "var(--text-tertiary)" : "white",
                cursor: isAdding ? "not-allowed" : "pointer",
                fontSize: tokens.typography.fontSize.xs,
                fontWeight: 600,
                whiteSpace: "nowrap",
              }}
            >
              {isAdding ? t("friends.adding") : getActionLabel()}
            </button>
          ) : (
            <Badge
              variant={user.status === "friend" ? "success" : "default"}
            >
              {getActionLabel()}
            </Badge>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

export default FriendsSearchRoute;
