import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Body, Title3, Caption1, Subheadline } from "../../components/ui/Typography";
import { Card, CardContent, Badge, ShimmerCard } from "../../components/ui";
import { tokens } from "../../design-system/tokens";
import { useDesktopState } from "../../useDesktopState";
import type { UserProfile } from "../../desktop-wire";

/**
 * UserDetailRoute - User profile page
 *
 * Displays user profile with:
 * - Avatar, name, username, bio
 * - Friend status badge
 * - Add/Remove friend actions
 */
export function UserDetailRoute({ userId }: { userId: string }) {
  const { t } = useTranslation("ui");
  const { status, userProfiles, addFriend, removeFriend, refreshFriends } = useDesktopState();

  const [user, setUser] = useState<UserProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [actioning, setActioning] = useState(false);

  const loadUser = useCallback(async () => {
    if (status !== "ready") return;

    setLoading(true);
    setError(null);

    // Check if user is in cache
    const cachedUser = userProfiles[userId];
    if (cachedUser) {
      setUser(cachedUser);
      setLoading(false);
      return;
    }

    // User not in cache - show placeholder
    setLoading(false);
  }, [userId, userProfiles, status]);

  useEffect(() => {
    loadUser();
  }, [loadUser]);

  const handleAddFriend = async () => {
    setActioning(true);
    setError(null);
    try {
      const updated = await addFriend(userId);
      if (updated) {
        setUser(updated);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to add friend";
      setError(message);
    } finally {
      setActioning(false);
    }
  };

  const handleRemoveFriend = async () => {
    setActioning(true);
    setError(null);
    try {
      const updated = await removeFriend(userId);
      if (updated) {
        setUser(updated);
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to remove friend";
      setError(message);
    } finally {
      setActioning(false);
    }
  };

  const getActionButton = () => {
    if (!user) return null;

    switch (user.status) {
      case "friend":
        return (
          <button
            onClick={handleRemoveFriend}
            disabled={actioning}
            style={{
              padding: `${tokens.spacing[3]} ${tokens.spacing[6]}`,
              backgroundColor: actioning ? "var(--surface-tertiary)" : "var(--color-error-muted)",
              border: `1px solid ${actioning ? "var(--border-primary)" : "var(--color-error)"}`,
              borderRadius: tokens.radii.md,
              color: actioning ? "var(--text-tertiary)" : "var(--color-error)",
              cursor: actioning ? "not-allowed" : "pointer",
              fontSize: tokens.typography.fontSize.base,
              fontWeight: 600,
            }}
          >
            {actioning ? t("friends.removing") : t("friends.removeFriend")}
          </button>
        );
      case "requested":
        return (
          <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: tokens.spacing[2] }}>
            <Badge variant="default">{t("friends.statusRequested")}</Badge>
            <button
              onClick={handleRemoveFriend}
              disabled={actioning}
              style={{
                padding: `${tokens.spacing[2]} ${tokens.spacing[4]}`,
                backgroundColor: "transparent",
                border: "1px solid var(--border-primary)",
                borderRadius: tokens.radii.md,
                color: "var(--text-secondary)",
                cursor: actioning ? "not-allowed" : "pointer",
                fontSize: tokens.typography.fontSize.sm,
              }}
            >
              {t("friends.cancelRequest")}
            </button>
          </div>
        );
      case "pending":
        return (
          <div style={{ display: "flex", gap: tokens.spacing[2] }}>
            <button
              onClick={handleAddFriend}
              disabled={actioning}
              style={{
                padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
                backgroundColor: actioning ? "var(--surface-tertiary)" : "var(--color-primary)",
                border: "none",
                borderRadius: tokens.radii.md,
                color: actioning ? "var(--text-tertiary)" : "white",
                cursor: actioning ? "not-allowed" : "pointer",
                fontSize: tokens.typography.fontSize.base,
                fontWeight: 600,
              }}
            >
              {actioning ? t("friends.accepting") : t("friends.acceptRequest")}
            </button>
            <button
              onClick={handleRemoveFriend}
              disabled={actioning}
              style={{
                padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
                backgroundColor: "var(--surface-secondary)",
                border: "1px solid var(--border-primary)",
                borderRadius: tokens.radii.md,
                color: "var(--text-primary)",
                cursor: actioning ? "not-allowed" : "pointer",
                fontSize: tokens.typography.fontSize.base,
              }}
            >
              {t("friends.declineRequest")}
            </button>
          </div>
        );
      default:
        return (
          <button
            onClick={handleAddFriend}
            disabled={actioning}
            style={{
              padding: `${tokens.spacing[3]} ${tokens.spacing[6]}`,
              backgroundColor: actioning ? "var(--surface-tertiary)" : "var(--color-primary)",
              border: "none",
              borderRadius: tokens.radii.md,
              color: actioning ? "var(--text-tertiary)" : "white",
              cursor: actioning ? "not-allowed" : "pointer",
              fontSize: tokens.typography.fontSize.base,
              fontWeight: 600,
            }}
          >
            {actioning ? t("friends.adding") : t("friends.addFriend")}
          </button>
        );
    }
  };

  if (loading) {
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          height: "100%",
          backgroundColor: "var(--bg-primary)",
          padding: tokens.spacing[6],
        }}
      >
        <ShimmerCard style={{ height: "200px" }} />
      </div>
    );
  }

  if (!user) {
    return (
      <div
        style={{
          display: "flex",
          flexDirection: "column",
          height: "100%",
          backgroundColor: "var(--bg-primary)",
          padding: tokens.spacing[6],
        }}
      >
        <Card variant="default" padding="lg">
          <CardContent>
            <Body color="secondary" style={{ textAlign: "center" }}>
              {t("friends.userNotFound")}
            </Body>
          </CardContent>
        </Card>
      </div>
    );
  }

  const displayName = user.firstName || user.username || user.id.slice(0, 8);
  const fullName = [user.firstName, user.lastName].filter(Boolean).join(" ") || null;

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
        <Title3>{t("friends.userProfile")}</Title3>
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

        {/* Profile card */}
        <Card variant="default" padding="lg">
          <CardContent>
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                gap: tokens.spacing[4],
              }}
            >
              {/* Avatar */}
              <div
                style={{
                  width: "96px",
                  height: "96px",
                  borderRadius: "50%",
                  backgroundColor: "var(--surface-tertiary)",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  overflow: "hidden",
                }}
              >
                {user.avatar?.url ? (
                  <img
                    src={user.avatar.url}
                    alt={displayName}
                    style={{ width: "100%", height: "100%", objectFit: "cover" }}
                  />
                ) : (
                  <span style={{ fontSize: "40px", color: "var(--text-secondary)" }}>
                    {displayName.charAt(0).toUpperCase()}
                  </span>
                )}
              </div>

              {/* Name */}
              <div style={{ textAlign: "center" }}>
                <Title3>{displayName}</Title3>
                {fullName && fullName !== displayName && (
                  <Body color="secondary">{fullName}</Body>
                )}
                <Caption1 color="tertiary" style={{ display: "block", marginTop: tokens.spacing[1] }}>
                  @{user.username}
                </Caption1>
              </div>

              {/* Status badge */}
              {user.status !== "none" && (
                <Badge
                  variant={user.status === "friend" ? "success" : "default"}
                >
                  {t(`friends.status${user.status.charAt(0).toUpperCase() + user.status.slice(1)}`)}
                </Badge>
              )}

              {/* Bio */}
              {user.bio && (
                <Card variant="default" padding="md" style={{ width: "100%" }}>
                  <Body>{user.bio}</Body>
                </Card>
              )}

              {/* Action */}
              {getActionButton()}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export default UserDetailRoute;