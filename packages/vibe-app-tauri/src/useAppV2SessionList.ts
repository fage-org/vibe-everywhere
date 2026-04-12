import { useMemo } from "react";
import type { Session } from "./components/surfaces";
import type { AppShellState } from "./useAppShellState";

export function useAppV2SessionList(shell: AppShellState): Session[] {
  return useMemo(() => {
    return shell.sessionSummaries.map(({ session, title, subtitle }) => {
      const latestMessage = shell.sessionState[session.id]?.items.at(-1)?.text;
      return {
        id: session.id,
        title,
        subtitle: subtitle || session.metadata?.path || undefined,
        lastMessage: latestMessage,
        lastActivityAt: new Date(session.updatedAt || session.createdAt),
        status: session.active ? "active" : "idle",
        unread: false,
        unreadCount: undefined,
        metadata: {
          model: session.metadata?.currentModelCode ?? undefined,
          workspace: session.metadata?.path ?? undefined,
        },
      };
    });
  }, [shell.sessionState, shell.sessionSummaries]);
}
