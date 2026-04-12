import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { afterEach, describe, expect, it } from "vitest";
import {
  APP_V2_NEW_SESSION_DEFAULT_DRAFT,
  useNewSessionDraftForm,
} from "./useNewSessionDraftForm";

(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

type HookValue = ReturnType<typeof useNewSessionDraftForm>;

function installStorage() {
  const store = new Map<string, string>();
  (globalThis as typeof globalThis & { window?: Window & typeof globalThis }).window = {
    localStorage: {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value);
      },
      removeItem: (key: string) => {
        store.delete(key);
      },
    },
  } as Window & typeof globalThis;

  return store;
}

function HookProbe({ onValue }: { onValue: (value: HookValue) => void }) {
  onValue(useNewSessionDraftForm());
  return null;
}

describe("useNewSessionDraftForm", () => {
  let renderer: ReactTestRenderer | null = null;

  afterEach(async () => {
    if (renderer) {
      await act(async () => {
        renderer?.unmount();
      });
    }
    renderer = null;
    Reflect.deleteProperty(globalThis, "window");
  });

  it("validates required workspace and prompt fields", async () => {
    installStorage();
    let latest!: HookValue;

    await act(async () => {
      renderer = create(<HookProbe onValue={(value) => { latest = value; }} />);
    });

    await act(async () => {
      expect(latest.validate()).toBeNull();
    });
    expect(latest.errors.workspace).toBe("Workspace is required.");
    expect(latest.errors.prompt).toBe("Initial prompt is required.");

    await act(async () => {
      latest.setWorkspace("/tmp/project");
    });

    await act(async () => {
      expect(latest.validate()).toBeNull();
    });
    expect(latest.errors.workspace).toBeUndefined();
    expect(latest.errors.prompt).toBe("Initial prompt is required.");
  });

  it("loads, persists, and resets the draft state", async () => {
    const store = installStorage();
    store.set("vibe-app-tauri.new-session-draft", JSON.stringify({
      workspace: "/tmp/existing",
      model: "gpt-5.4",
      title: "Existing",
      prompt: "Continue from last time",
    }));
    let latest!: HookValue;

    await act(async () => {
      renderer = create(<HookProbe onValue={(value) => { latest = value; }} />);
    });

    expect(latest.workspace).toBe("/tmp/existing");
    expect(latest.prompt).toBe("Continue from last time");

    await act(async () => {
      latest.setWorkspace("/tmp/next");
      latest.setPrompt("Ship the refactor");
    });

    expect(store.get("vibe-app-tauri.new-session-draft")).toContain("/tmp/next");
    expect(store.get("vibe-app-tauri.new-session-draft")).toContain("Ship the refactor");

    await act(async () => {
      latest.reset();
    });

    expect(store.has("vibe-app-tauri.new-session-draft")).toBe(false);
    expect(latest.workspace).toBe(APP_V2_NEW_SESSION_DEFAULT_DRAFT.workspace);
    expect(latest.prompt).toBe(APP_V2_NEW_SESSION_DEFAULT_DRAFT.prompt);
  });
});
