import { beforeEach, describe, expect, it } from "vitest";
import {
  clearSessionPreferences,
  loadSessionPreferences,
  saveSessionPreferences,
} from "./session-preferences";

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

describe("session composer preferences", () => {
  const defaults = {
    permissionMode: "default",
    model: "gpt-5.4",
  };

  beforeEach(() => {
    installMockStorage();
  });

  it("loads defaults when no preferences are stored", () => {
    expect(loadSessionPreferences("session-1", defaults)).toEqual(defaults);
  });

  it("round-trips per-session preferences", () => {
    saveSessionPreferences("session-1", {
      permissionMode: "plan",
      model: "gpt-5.3-codex",
    });

    expect(loadSessionPreferences("session-1", defaults)).toEqual({
      permissionMode: "plan",
      model: "gpt-5.3-codex",
    });
  });

  it("keeps preferences isolated per session", () => {
    saveSessionPreferences("session-1", {
      permissionMode: "read-only",
      model: "gpt-5.4",
    });
    saveSessionPreferences("session-2", {
      permissionMode: "yolo",
      model: "gpt-5.3-codex",
    });

    expect(loadSessionPreferences("session-1", defaults)).toEqual({
      permissionMode: "read-only",
      model: "gpt-5.4",
    });
    expect(loadSessionPreferences("session-2", defaults)).toEqual({
      permissionMode: "yolo",
      model: "gpt-5.3-codex",
    });
  });

  it("ignores malformed stored values", () => {
    window.localStorage.setItem(
      "vibe-app-tauri.session-preferences.session-1",
      JSON.stringify({
        permissionMode: 7,
        model: "",
      }),
    );

    expect(loadSessionPreferences("session-1", defaults)).toEqual(defaults);
  });

  it("clears stored preferences", () => {
    saveSessionPreferences("session-1", defaults);

    clearSessionPreferences("session-1");

    expect(loadSessionPreferences("session-1", defaults)).toEqual(defaults);
  });
});
