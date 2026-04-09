export type SessionComposerPreferences = {
  permissionMode: string;
  model: string;
};

const SESSION_PREFERENCES_PREFIX = "vibe-app-tauri.session-preferences.";

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

function buildSessionPreferencesKey(sessionId: string): string {
  return `${SESSION_PREFERENCES_PREFIX}${sessionId}`;
}

export function loadSessionPreferences(
  sessionId: string,
  defaults: SessionComposerPreferences,
): SessionComposerPreferences {
  const raw = getStorage()?.getItem(buildSessionPreferencesKey(sessionId));
  if (!raw) {
    return { ...defaults };
  }

  try {
    const parsed = JSON.parse(raw) as Partial<SessionComposerPreferences>;
    return {
      permissionMode:
        typeof parsed.permissionMode === "string" && parsed.permissionMode.trim()
          ? parsed.permissionMode
          : defaults.permissionMode,
      model:
        typeof parsed.model === "string" && parsed.model.trim()
          ? parsed.model
          : defaults.model,
    };
  } catch {
    return { ...defaults };
  }
}

export function saveSessionPreferences(
  sessionId: string,
  preferences: SessionComposerPreferences,
): void {
  getStorage()?.setItem(buildSessionPreferencesKey(sessionId), JSON.stringify(preferences));
}

export function clearSessionPreferences(sessionId: string): void {
  getStorage()?.removeItem(buildSessionPreferencesKey(sessionId));
}
