export type NewSessionDraft = {
  workspace: string;
  model: string;
  title: string;
  prompt: string;
};

const NEW_SESSION_DRAFT_KEY = "vibe-app-tauri.new-session-draft";

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

export function loadNewSessionDraft(defaults: NewSessionDraft): NewSessionDraft {
  const raw = getStorage()?.getItem(NEW_SESSION_DRAFT_KEY);
  if (!raw) {
    return { ...defaults };
  }

  try {
    const parsed = JSON.parse(raw) as Partial<NewSessionDraft>;
    return {
      workspace:
        typeof parsed.workspace === "string" && parsed.workspace.trim()
          ? parsed.workspace
          : defaults.workspace,
      model:
        typeof parsed.model === "string" && parsed.model.trim()
          ? parsed.model
          : defaults.model,
      title:
        typeof parsed.title === "string" && parsed.title.trim()
          ? parsed.title
          : defaults.title,
      prompt:
        typeof parsed.prompt === "string" && parsed.prompt.trim()
          ? parsed.prompt
          : defaults.prompt,
    };
  } catch {
    return { ...defaults };
  }
}

export function saveNewSessionDraft(draft: NewSessionDraft): void {
  getStorage()?.setItem(NEW_SESSION_DRAFT_KEY, JSON.stringify(draft));
}

export function clearNewSessionDraft(): void {
  getStorage()?.removeItem(NEW_SESSION_DRAFT_KEY);
}
