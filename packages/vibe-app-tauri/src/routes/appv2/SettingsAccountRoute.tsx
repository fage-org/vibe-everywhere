import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Body, Title3, Caption1 } from "../../components/ui/Typography";
import { Card, CardContent } from "../../components/ui";
import { tokens } from "../../design-system/tokens";
import { useDesktopState } from "../../useDesktopState";
import type { AccountProfile } from "../../desktop-client";

/**
 * SettingsAccountRoute - Account settings page
 *
 * Displays user profile information:
 * - Avatar with initials/gradient
 * - Name and username
 * - Connected services (AI providers)
 * - Account actions (sign out)
 */
export function SettingsAccountRoute() {
  const { t } = useTranslation("ui");
  const { profile, status, logout, refreshProfile } = useDesktopState();
  const [isSigningOut, setIsSigningOut] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);

  const handleSignOut = useCallback(async () => {
    if (isSigningOut) return;
    setIsSigningOut(true);
    try {
      await logout();
    } finally {
      setIsSigningOut(false);
    }
  }, [isSigningOut, logout]);

  const handleRefresh = useCallback(async () => {
    if (isRefreshing) return;
    setIsRefreshing(true);
    try {
      await refreshProfile();
    } finally {
      setIsRefreshing(false);
    }
  }, [isRefreshing, refreshProfile]);

  if (status !== "ready" || !profile) {
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
          <Card variant="default" padding="lg">
            <CardContent>
              <Body color="secondary" style={{ textAlign: "center" }}>
                {t("account.loading")}
              </Body>
            </CardContent>
          </Card>
        </div>
      </div>
    );
  }

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
        <Title3>{t("account.title")}</Title3>
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
        {/* Profile Card */}
        <Card variant="default" padding="lg">
          <CardContent>
            <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: tokens.spacing[4] }}>
              <AvatarDisplay profile={profile} />
              <div style={{ textAlign: "center" }}>
                <Title3 style={{ marginBottom: tokens.spacing[1] }}>
                  {getDisplayName(profile)}
                </Title3>
                {profile.username && (
                  <Caption1 color="secondary">@{profile.username}</Caption1>
                )}
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Account Info */}
        <Card variant="default" padding="md">
          <CardContent>
            <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
              <div style={{ display: "flex", alignItems: "center", gap: tokens.spacing[3] }}>
                <span style={{ fontSize: "20px" }}>📧</span>
                <div>
                  <Caption1 color="secondary">{t("account.accountId")}</Caption1>
                  <Body
                    style={{
                      fontFamily: tokens.typography.fontFamily.mono,
                      fontSize: tokens.typography.fontSize.sm,
                    }}
                  >
                    {profile.id.substring(0, 8)}...
                  </Body>
                </div>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Connected Services */}
        <Card variant="default" padding="md">
          <CardContent>
            <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
              <Caption1 style={{ fontWeight: 600, marginBottom: tokens.spacing[2] }}>
                {t("account.connectedServices")}
              </Caption1>
              {profile.connectedServices && profile.connectedServices.length > 0 ? (
                profile.connectedServices.map((service) => (
                  <div
                    key={service}
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: tokens.spacing[3],
                    }}
                  >
                    <span style={{ fontSize: "20px" }}>
                      {getServiceIcon(service)}
                    </span>
                    <Body>{getServiceDisplayName(service)}</Body>
                  </div>
                ))
              ) : (
                <Body color="secondary">{t("account.noConnectedServices")}</Body>
              )}
            </div>
          </CardContent>
        </Card>

        {/* Actions */}
        <div style={{ display: "flex", flexDirection: "column", gap: tokens.spacing[3] }}>
          <button
            onClick={handleRefresh}
            disabled={isRefreshing}
            style={{
              padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
              backgroundColor: "var(--surface-secondary)",
              border: "1px solid var(--border-primary)",
              borderRadius: tokens.radii.md,
              color: "var(--text-primary)",
              cursor: isRefreshing ? "wait" : "pointer",
              fontSize: tokens.typography.fontSize.base,
            }}
          >
            {isRefreshing ? t("account.refreshing") : t("account.refresh")}
          </button>
          <button
            onClick={handleSignOut}
            disabled={isSigningOut}
            style={{
              padding: `${tokens.spacing[3]} ${tokens.spacing[4]}`,
              backgroundColor: isSigningOut ? "var(--surface-tertiary)" : "var(--color-error)",
              border: "none",
              borderRadius: tokens.radii.md,
              color: "white",
              cursor: isSigningOut ? "wait" : "pointer",
              fontSize: tokens.typography.fontSize.base,
              fontWeight: 600,
            }}
          >
            {isSigningOut ? t("account.signingOut") : t("account.signOut")}
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * Avatar display component
 */
function AvatarDisplay({ profile }: { profile: AccountProfile }) {
  const initials = getInitials(profile);

  return (
    <div
      style={{
        width: "80px",
        height: "80px",
        borderRadius: "50%",
        background: getAvatarGradient(profile.id),
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        color: "white",
        fontSize: tokens.typography.fontSize.xl,
        fontWeight: 600,
      }}
    >
      {initials}
    </div>
  );
}

/**
 * Get display name from profile
 */
function getDisplayName(profile: AccountProfile): string {
  if (profile.firstName && profile.lastName) {
    return `${profile.firstName} ${profile.lastName}`;
  }
  if (profile.firstName) {
    return profile.firstName;
  }
  if (profile.lastName) {
    return profile.lastName;
  }
  return profile.username || "User";
}

/**
 * Get initials from profile
 */
function getInitials(profile: AccountProfile): string {
  const firstName = profile.firstName?.[0] || "";
  const lastName = profile.lastName?.[0] || "";
  if (firstName && lastName) {
    return `${firstName}${lastName}`.toUpperCase();
  }
  if (firstName) {
    return firstName.toUpperCase();
  }
  if (lastName) {
    return lastName.toUpperCase();
  }
  return (profile.username?.[0] || "U").toUpperCase();
}

/**
 * Generate a consistent gradient based on user ID
 */
function getAvatarGradient(id: string): string {
  // Use hash of ID to select from predefined gradients
  const hash = id.split("").reduce((acc, char) => acc + char.charCodeAt(0), 0);
  const gradients = [
    "linear-gradient(135deg, #667eea 0%, #764ba2 100%)",
    "linear-gradient(135deg, #f093fb 0%, #f5576c 100%)",
    "linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)",
    "linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)",
    "linear-gradient(135deg, #fa709a 0%, #fee140 100%)",
    "linear-gradient(135deg, #a8edea 0%, #fed6e3 100%)",
  ];
  return gradients[hash % gradients.length];
}

/**
 * Get icon for connected service
 */
function getServiceIcon(service: string): string {
  switch (service.toLowerCase()) {
    case "openai":
      return "🤖";
    case "anthropic":
      return "🧠";
    case "gemini":
      return "✨";
    case "github":
      return "🐙";
    default:
      return "🔗";
  }
}

/**
 * Get display name for connected service
 */
function getServiceDisplayName(service: string): string {
  switch (service.toLowerCase()) {
    case "openai":
      return "OpenAI";
    case "anthropic":
      return "Anthropic";
    case "gemini":
      return "Google Gemini";
    case "github":
      return "GitHub";
    default:
      return service;
  }
}

export default SettingsAccountRoute;