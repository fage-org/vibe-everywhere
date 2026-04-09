import { describe, expect, it } from "vitest";
import {
  getSessionModelOptions,
  getSessionPermissionOptions,
  resolveSessionModeSelection,
} from "./session-mode-options";

describe("session mode options", () => {
  it("uses metadata-backed model options when available", () => {
    expect(
      getSessionModelOptions({
        path: "/root/vibe-remote",
        host: "desktop",
        models: [
          { code: "gpt-5.4", value: "gpt-5.4", description: "default" },
          { code: "gpt-5.3-codex", value: "gpt-5.3-codex", description: "fast" },
        ],
      }),
    ).toEqual([
      { key: "gpt-5.4", name: "gpt-5.4", description: "default" },
      { key: "gpt-5.3-codex", name: "gpt-5.3-codex", description: "fast" },
    ]);
  });

  it("falls back to flavor defaults for permission options", () => {
    expect(
      getSessionPermissionOptions({
        path: "/root/vibe-remote",
        host: "desktop",
        flavor: "codex",
      }),
    ).toEqual([
      { key: "default", name: "default", description: null },
      { key: "read-only", name: "read-only", description: null },
      { key: "safe-yolo", name: "safe-yolo", description: null },
      { key: "yolo", name: "yolo", description: null },
    ]);
  });

  it("prefers stored keys before metadata defaults", () => {
    expect(
      resolveSessionModeSelection(
        [
          { key: "default", name: "default", description: null },
          { key: "plan", name: "plan", description: null },
        ],
        ["plan", "default"],
      ),
    ).toBe("plan");
  });
});
