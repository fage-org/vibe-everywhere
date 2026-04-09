export type RealtimeSessionRecord = {
  id: string;
  updatedAt: number;
  [key: string]: unknown;
};

export type RealtimeSessionUiState = {
  items: unknown[];
  loading: boolean;
  sending: boolean;
  error: string | null;
  loadedAt: number | null;
  lastSeq: number | null;
  [key: string]: unknown;
};

export function upsertRealtimeSession<T extends RealtimeSessionRecord>(
  sessions: T[],
  nextSession: T,
): T[] {
  const remaining = sessions.filter((session) => session.id !== nextSession.id);
  return [nextSession, ...remaining].sort((left, right) => right.updatedAt - left.updatedAt);
}

export function removeDeletedSession<
  T extends RealtimeSessionRecord,
  S extends RealtimeSessionUiState,
>(
  sessions: T[],
  sessionState: Record<string, S>,
  sessionId: string,
): {
  sessions: T[];
  sessionState: Record<string, S>;
} {
  const nextSessions = sessions.filter((session) => session.id !== sessionId);
  const nextSessionState = { ...sessionState };
  delete nextSessionState[sessionId];
  return {
    sessions: nextSessions,
    sessionState: nextSessionState,
  };
}
