import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { afterEach, describe, expect, it } from "vitest";
import { useSessionComposerDraft } from "./useSessionComposerDraft";

(globalThis as typeof globalThis & { IS_REACT_ACT_ENVIRONMENT?: boolean }).IS_REACT_ACT_ENVIRONMENT = true;

type HookValue = ReturnType<typeof useSessionComposerDraft>;

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

function HookProbe({
  sessionId,
  onValue,
}: {
  sessionId: string | null;
  onValue: (value: HookValue) => void;
}) {
  onValue(useSessionComposerDraft(sessionId));
  return null;
}

describe("useSessionComposerDraft", () => {
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

  it("restores the draft for the active session and clears it on demand", async () => {
    const store = installStorage();
    store.set("vibe-app-tauri.session-draft.session-1", JSON.stringify({ value: "Resume session" }));
    let latest!: HookValue;

    await act(async () => {
      renderer = create(<HookProbe sessionId="session-1" onValue={(value) => { latest = value; }} />);
    });

    expect(latest.composerValue).toBe("Resume session");

    await act(async () => {
      latest.clearComposerValue();
    });

    expect(latest.composerValue).toBe("");
    expect(store.has("vibe-app-tauri.session-draft.session-1")).toBe(false);
  });

  it("stores updates per session and clears state when no session is active", async () => {
    const store = installStorage();
    let latest!: HookValue;

    await act(async () => {
      renderer = create(<HookProbe sessionId="session-1" onValue={(value) => { latest = value; }} />);
    });

    await act(async () => {
      latest.setComposerValue("Ship AppV2 cleanup");
    });

    expect(store.get("vibe-app-tauri.session-draft.session-1")).toContain("Ship AppV2 cleanup");

    await act(async () => {
      renderer?.unmount();
      renderer = create(<HookProbe sessionId={null} onValue={(value) => { latest = value; }} />);
    });

    expect(latest.composerValue).toBe("");
  });
});
