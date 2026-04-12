import { useMemo } from "react";
import type { AppV2View } from "./useAppV2RouteModel";

type NavigationItem = {
  id: string;
  label: string;
  icon: string;
  badge?: number;
  state: "active" | "default";
  onClick: () => void;
};

type NavigationOptions = {
  view: AppV2View;
  unreadCount?: number;
  navigate: (path: string) => void;
  t: (key: string) => string;
};

export function useAppV2Navigation({
  view,
  unreadCount,
  navigate,
  t,
}: NavigationOptions) {
  return useMemo(() => {
    const primaryNavItems: NavigationItem[] = [
      {
        id: "home",
        label: t("components:nav.home"),
        icon: "🏠",
        state: view === "home" ? "active" : "default",
        onClick: () => navigate("/(app)/index"),
      },
      {
        id: "sessions",
        label: t("components:nav.sessions"),
        icon: "💬",
        state: view === "session" || view === "session-recent" ? "active" : "default",
        onClick: () => navigate("/(app)/session/recent"),
      },
      {
        id: "inbox",
        label: t("components:nav.inbox"),
        icon: "🔔",
        badge: unreadCount && unreadCount > 0 ? unreadCount : undefined,
        state: view === "inbox" ? "active" : "default",
        onClick: () => navigate("/(app)/inbox/index"),
      },
    ];

    const secondaryNavItems: NavigationItem[] = [
      {
        id: "settings",
        label: t("components:nav.settings"),
        icon: "⚙️",
        state: view === "settings" ? "active" : "default",
        onClick: () => navigate("/(app)/settings/index"),
      },
    ];

    const mobileActiveTab =
      view === "new-session" || view === "unsupported"
        ? "home"
        : view === "session-recent"
          ? "sessions"
          : view;

    const headerEyebrowKey =
      view === "new-session" || view === "unsupported" ? "components:nav.home" : `components:nav.${view}`;

    const headerTitle =
      view === "home"
        ? t("common:app.name")
        : view === "inbox"
          ? t("routes:inbox.title")
          : view === "settings"
            ? t("routes:settings.title")
            : view === "new-session"
              ? t("routes:home.actions.newSession")
              : view === "session-recent"
                ? t("routes:home.actions.resume")
                : t("common:app.name");

    return {
      primaryNavItems,
      secondaryNavItems,
      mobileActiveTab,
      headerEyebrowKey,
      headerTitle,
    };
  }, [navigate, t, unreadCount, view]);
}
