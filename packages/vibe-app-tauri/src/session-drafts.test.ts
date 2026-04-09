import { beforeEach, describe, expect, it } from "vitest";
import {
  clearSessionDraft,
  loadSessionDraft,
  saveSessionDraft,
} from "./session-drafts";

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

describe("session draft persistence", () => {
  beforeEach(() => {
    installMockStorage();
  });

  it("loads an empty draft when none is stored", () => {
    expect(loadSessionDraft("session-1")).toBe("");
  });

  it("round-trips a session draft through local storage", () => {
    saveSessionDraft("session-1", "Resume the parity push");

    expect(loadSessionDraft("session-1")).toBe("Resume the parity push");
  });

  it("keeps drafts isolated per session", () => {
    saveSessionDraft("session-1", "Session one");
    saveSessionDraft("session-2", "Session two");

    expect(loadSessionDraft("session-1")).toBe("Session one");
    expect(loadSessionDraft("session-2")).toBe("Session two");
  });

  it("clears stored drafts after send or explicit reset", () => {
    saveSessionDraft("session-1", "Ship it");

    clearSessionDraft("session-1");

    expect(loadSessionDraft("session-1")).toBe("");
  });

  it("ignores malformed stored payloads", () => {
    window.localStorage.setItem(
      "vibe-app-tauri.session-draft.session-1",
      JSON.stringify({ value: 42 }),
    );

    expect(loadSessionDraft("session-1")).toBe("");
  });
});
