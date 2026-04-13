import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Body, Title3, Caption1, Subheadline } from "../../components/ui/Typography";
import { Card, CardContent, Badge, ShimmerCard } from "../../components/ui";
import { tokens } from "../../design-system/tokens";
import { useDesktopState } from "../../useDesktopState";
import type { UserProfile } from "../../desktop-wire";
import { hrefForPath } from "../../router";

/**
 * FriendsRoute - Friends list page
 *
 * Displays the user's friends list with:
 * - Friend cards showing avatar, name, and username
 * - Navigation to user profile
 * - Remove friend functionality
 * - Link to search for new friends
 */
export function FriendsRoute() {
  const { t } = useTranslation("ui");
  const { status, friends, refreshFriends, removeFriend } = useDesktopState();

  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [removingId, setRemovingId] = useState<string | null>(null);

  const loadFriends = useCallback(async () => {
    if (status !== "ready") return;
    setLoading(true);
    try {
      await refreshFriends();
    } catch (err) {
      console.error("Failed to load friends:", err);
      setError(err instanceof Error ? err.message : "Failed to load friends");
    } finally {
      setLoading(false);
    }
  }, [refreshFriends, status]);

  useEffect(() => {
    loadFriends();
  }, [loadFriends]);

  const handleRemoveFriend = async (userId: string) => {
    setRemovingId(userId);
    setError(null);
    try {
      await removeFriend(userId);
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to remove friend";
      setError(message);
    } finally {
      setRemovingId(null);
    }
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
        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
          }}
        >
          <Title3>{t("friends.title")}</Title3>
          <a
            href={hrefForPath("/(app)/friends/search")}
            style={{
              padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
              backgroundColor: "var(--color-primary)",
              border: "none",
              borderRadius: tokens.radii.md,
              color: "white",
              cursor: "pointer",
              fontSize: tokens.typography.fontSize.sm,
              fontWeight: 600,
              textDecoration: "none",
            }}
          >
            {t("friends.findFriends")}
          </a>
        </div>
        <Body color="secondary" style={{ marginTop: tokens.spacing[2] }}>
          {t("friends.description")}
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
        {/* Error */}
        {error && (
          <Card variant="default" padding="md" style={{ borderColor: "var(--color-error)" }}>
            <Body style={{ color: "var(--color-error)" }}>{error}</Body>
          </Card>
        )}

        {/* Loading */}
        {loading && (
          <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[4] }}>
            <ShimmerCard style={{ height: "80px" }} />
            <ShimmerCard style={{ height: "80px" }} />
            <ShimmerCard style={{ height: "80px" }} />
          </div>
        )}

        {/* Empty state */}
        {!loading && friends.length === 0 && (
          <Card variant="default" padding="lg">
            <CardContent>
              <Body color="secondary" style={{ textAlign: "center" }}>
                {t("friends.noFriends")}
              </Body>
            </CardContent>
          </Card>
        )}

        {/* Friend list */}
        {!loading && friends.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
            {friends.map((friend) => (
              <FriendCard
                key={friend.id}
                friend={friend}
                isRemoving={removingId === friend.id}
                onRemove={() => handleRemoveFriend(friend.id)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

/**
 * Friend card component
 */
function FriendCard({
  friend,
  isRemoving,
  onRemove,
}: {
  friend: UserProfile;
  isRemoving: boolean;
  onRemove: () => void;
}) {
  const { t } = useTranslation("ui");

  const displayName = friend.firstName || friend.username || friend.id.slice(0, 8);
  const fullName = [friend.firstName, friend.lastName].filter(Boolean).join(" ") || null;

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
            href={hrefForPath(`/(app)/user/${friend.id}`)}
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
              {friend.avatar?.url ? (
                <img
                  src={friend.avatar.url}
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
              <Caption1 color="tertiary">@{friend.username}</Caption1>
            </div>
          </a>

          {/* Remove button */}
          <button
            onClick={(e) => {
              e.preventDefault();
              onRemove();
            }}
            disabled={isRemoving}
            style={{
              padding: `${tokens.spacing[2]} ${tokens.spacing[3]}`,
              backgroundColor: isRemoving ? "var(--surface-tertiary)" : "var(--color-error-muted)",
              border: `1px solid ${isRemoving ? "var(--border-primary)" : "var(--color-error)"}`,
              borderRadius: tokens.radii.md,
              color: isRemoving ? "var(--text-tertiary)" : "var(--color-error)",
              cursor: isRemoving ? "not-allowed" : "pointer",
              fontSize: tokens.typography.fontSize.xs,
              whiteSpace: "nowrap",
            }}
          >
            {isRemoving ? t("friends.removing") : t("friends.remove")}
          </button>
        </div>
      </CardContent>
    </Card>
  );
}

export default FriendsRoute;
