import { useMemo } from "react";
import type { QuickAction, StatItem } from "./components/routes";
import type { UseLanguageReturn } from "./hooks/useLanguage";

type Options = {
  t: UseLanguageReturn["t"];
  sessionCount: number;
  messageCount: number;
  activeSessionCount: number;
  onStartNewSession: () => void;
  onResumeLatestSession: () => void;
  onOpenSettings: () => void;
};

export function useAppV2HomeViewModel({
  t,
  sessionCount,
  messageCount,
  activeSessionCount,
  onStartNewSession,
  onResumeLatestSession,
  onOpenSettings,
}: Options) {
  const quickActions = useMemo((): QuickAction[] => {
    return [
      {
        id: "new-session",
        label: t("routes:home.actions.newSession"),
        description: t("routes:home.quickActionDescriptions.newSession"),
        icon: "💬",
        onClick: onStartNewSession,
      },
      {
        id: "resume",
        label: t("routes:home.actions.resume"),
        description: t("routes:home.quickActionDescriptions.resume"),
        icon: "▶️",
        onClick: onResumeLatestSession,
      },
      {
        id: "settings",
        label: t("routes:home.actions.settings"),
        description: t("routes:home.quickActionDescriptions.settings"),
        icon: "⚙️",
        onClick: onOpenSettings,
      },
    ];
  }, [onOpenSettings, onResumeLatestSession, onStartNewSession, t]);

  const stats = useMemo((): StatItem[] => {
    return [
      { label: t("routes:home.sections.recentSessions"), value: sessionCount },
      { label: t("components:composer.send"), value: messageCount },
      { label: t("routes:home.stats.active"), value: activeSessionCount },
    ];
  }, [activeSessionCount, messageCount, sessionCount, t]);

  return {
    quickActions,
    stats,
  };
}
