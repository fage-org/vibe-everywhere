import { useCallback } from "react";
import type { AppShellState } from "./useAppShellState";
import type { ResolvedRoute } from "./router";
import { useInboxFeedAdapter } from "./useInboxFeedAdapter";
import { useAppV2CurrentSession } from "./useAppV2CurrentSession";
import { useAppV2RouteModel, resolveAppV2View } from "./useAppV2RouteModel";
import { useAppV2SessionList } from "./useAppV2SessionList";

export { resolveAppV2View } from "./useAppV2RouteModel";
export type { AppV2View } from "./useAppV2RouteModel";

export function useAppV2Shell(
  shell: AppShellState,
  resolved: ResolvedRoute,
  navigate: (path: string) => void,
) {
  const routeModel = useAppV2RouteModel(resolved);
  const sessions = useAppV2SessionList(shell);
  const { currentSession, messages } = useAppV2CurrentSession(
    shell,
    routeModel.activeSessionId,
    routeModel.view === "session",
  );
  const notifications = useInboxFeedAdapter(shell, navigate);

  const startNewSession = useCallback(() => {
    navigate("/(app)/new/index");
  }, [navigate]);

  const openSession = useCallback((sessionId: string) => {
    navigate(`/(app)/session/${sessionId}`);
  }, [navigate]);

  const resumeLatestSession = useCallback(() => {
    const latest = shell.sessionSummaries[0]?.session.id;
    if (latest) {
      navigate(`/(app)/session/${latest}`);
      return;
    }

    navigate("/(app)/session/recent");
  }, [navigate, shell.sessionSummaries]);

  return {
    view: routeModel.view,
    activeSessionId: routeModel.activeSessionId,
    isSupported: routeModel.isSupported,
    sessions,
    currentSession,
    messages,
    notifications,
    unreadCount: undefined as number | undefined,
    isConnected: shell.status === "ready",
    isLoading: shell.status === "checking" || shell.status === "loading",
    errorMessage: shell.globalError,
    startNewSession,
    openSession,
    resumeLatestSession,
  };
}
