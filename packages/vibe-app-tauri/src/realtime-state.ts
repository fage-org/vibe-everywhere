import type { DesktopSession } from "./wave8-client";
import type { SessionUiState } from "./useWave8Desktop";

export function upsertRealtimeSession(
  sessions: DesktopSession[],
  nextSession: DesktopSession,
): DesktopSession[] {
  const remaining = sessions.filter((session) => session.id !== nextSession.id);
  return [nextSession, ...remaining].sort((left, right) => right.updatedAt - left.updatedAt);
}

export function removeDeletedSession(
  sessions: DesktopSession[],
  sessionState: Record<string, SessionUiState>,
  sessionId: string,
): {
  sessions: DesktopSession[];
  sessionState: Record<string, SessionUiState>;
} {
  const nextSessions = sessions.filter((session) => session.id !== sessionId);
  const nextSessionState = { ...sessionState };
  delete nextSessionState[sessionId];
  return {
    sessions: nextSessions,
    sessionState: nextSessionState,
  };
}
