const SESSION_DRAFT_PREFIX = "vibe-app-tauri.session-draft.";

type StoredSessionDraft = {
  value: string;
};

function getStorage(): Storage | null {
  if (typeof window === "undefined") {
    return null;
  }

  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function buildSessionDraftStorageKey(sessionId: string): string {
  return `${SESSION_DRAFT_PREFIX}${sessionId}`;
}

export function loadSessionDraft(sessionId: string): string {
  const raw = getStorage()?.getItem(buildSessionDraftStorageKey(sessionId));
  if (!raw) {
    return "";
  }

  try {
    const parsed = JSON.parse(raw) as StoredSessionDraft;
    return typeof parsed.value === "string" ? parsed.value : "";
  } catch {
    return "";
  }
}

export function saveSessionDraft(sessionId: string, value: string): void {
  getStorage()?.setItem(
    buildSessionDraftStorageKey(sessionId),
    JSON.stringify({ value } satisfies StoredSessionDraft),
  );
}

export function clearSessionDraft(sessionId: string): void {
  getStorage()?.removeItem(buildSessionDraftStorageKey(sessionId));
}
