import { beforeEach, describe, expect, it } from "vitest";
import {
  clearNewSessionDraft,
  loadNewSessionDraft,
  saveNewSessionDraft,
  type NewSessionDraft,
} from "./new-session-draft";

function installMockStorage() {
  const store = new Map<string, string>();
  Object.defineProperty(globalThis, "window", {
    value: {
      localStorage: {
        getItem: (key: string) => store.get(key) ?? null,
        setItem: (key: string, value: string) => {
          store.set(key, value);
        },
        removeItem: (key: string) => {
          store.delete(key);
        },
      },
    },
    configurable: true,
    writable: true,
  });
}

const defaults: NewSessionDraft = {
  workspace: "/root/vibe-remote",
  model: "gpt-5.4",
  title: "Wave 9 Session",
  prompt: "Continue the parity work.",
};

describe("new session draft persistence", () => {
  beforeEach(() => {
    installMockStorage();
  });

  it("loads the defaults when no draft is stored", () => {
    expect(loadNewSessionDraft(defaults)).toEqual(defaults);
  });

  it("round-trips the stored draft", () => {
    const draft = {
      workspace: "/tmp/project",
      model: "gpt-5.3-codex",
      title: "Draft title",
      prompt: "Draft prompt",
    } satisfies NewSessionDraft;

    saveNewSessionDraft(draft);

    expect(loadNewSessionDraft(defaults)).toEqual(draft);
  });

  it("falls back to defaults for malformed values", () => {
    window.localStorage.setItem(
      "vibe-app-tauri.new-session-draft",
      JSON.stringify({
        workspace: 42,
        model: "",
        title: null,
        prompt: "Saved prompt",
      }),
    );

    expect(loadNewSessionDraft(defaults)).toEqual({
      ...defaults,
      prompt: "Saved prompt",
    });
  });

  it("clears the persisted draft", () => {
    saveNewSessionDraft(defaults);

    clearNewSessionDraft();

    expect(loadNewSessionDraft(defaults)).toEqual(defaults);
  });
});
