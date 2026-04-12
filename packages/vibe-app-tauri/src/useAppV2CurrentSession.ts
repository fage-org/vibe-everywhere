import { useEffect, useMemo } from "react";
import type { Message } from "./components/surfaces";
import type { AppShellState } from "./useAppShellState";

export function useAppV2CurrentSession(
  shell: AppShellState,
  activeSessionId: string | null,
  enabled: boolean,
) {
  useEffect(() => {
    if (enabled && activeSessionId) {
      void shell.loadMessages(activeSessionId);
    }
  }, [activeSessionId, enabled, shell]);

  const currentSession = useMemo(() => {
    if (!activeSessionId) {
      return null;
    }

    return shell.sessions.find((session) => session.id === activeSessionId) ?? null;
  }, [activeSessionId, shell.sessions]);

  const messages = useMemo<Message[]>(() => {
    if (!activeSessionId) {
      return [];
    }

    return (shell.sessionState[activeSessionId]?.items ?? []).map((item) => ({
      id: item.id,
      role: item.role === "assistant" || item.role === "user" || item.role === "system"
        ? item.role
        : "system",
      content: item.text,
      timestamp: new Date(item.createdAt),
      isStreaming: false,
    }));
  }, [activeSessionId, shell.sessionState]);

  return {
    currentSession,
    messages,
  };
}
