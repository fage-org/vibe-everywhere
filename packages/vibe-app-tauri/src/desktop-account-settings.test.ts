import { describe, expect, it, vi } from "vitest";
import {
  mapDesktopPreferencePatchToAccountSettings,
  runOptimisticAccountSync,
} from "./desktop-account-settings";

describe("desktop account settings helpers", () => {
  it("maps a language preference patch to account settings", () => {
    expect(
      mapDesktopPreferencePatchToAccountSettings({
        languagePatch: { appLanguage: "ja" },
      }),
    ).toEqual({
      preferredLanguage: "ja",
    });
  });

  it("rolls back optimistic state when account settings sync fails", async () => {
    const syncAccountSettings = vi.fn(async () => {
      throw new Error("sync failed");
    });

    const result = await runOptimisticAccountSync({
      previousState: { customAgentId: null },
      nextState: { customAgentId: "agent-1" },
      accountPatch: { voiceCustomAgentId: "agent-1" },
      syncAccountSettings,
    });

    expect(result.rolledBack).toBe(true);
    expect(result.finalState).toEqual({ customAgentId: null });
    expect(result.error).toBe("sync failed");
  });

  it("keeps optimistic state when sync succeeds", async () => {
    const syncAccountSettings = vi.fn(async () => undefined);

    const result = await runOptimisticAccountSync({
      previousState: { appLanguage: "en" },
      nextState: { appLanguage: "ja" },
      accountPatch: { preferredLanguage: "ja" },
      syncAccountSettings,
    });

    expect(result.rolledBack).toBe(false);
    expect(result.finalState).toEqual({ appLanguage: "ja" });
    expect(result.error).toBeNull();
  });
});
