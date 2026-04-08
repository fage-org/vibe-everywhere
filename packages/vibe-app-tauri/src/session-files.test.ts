import { describe, expect, it } from "vitest";
import {
  buildGitDiffCommand,
  buildWorkspaceFilePath,
  normalizeSessionRelativePath,
} from "./session-files";

describe("session file helpers", () => {
  it("normalizes valid relative file paths", () => {
    expect(normalizeSessionRelativePath("src/features/session.ts")).toBe(
      "src/features/session.ts",
    );
    expect(buildWorkspaceFilePath("/root/vibe-remote/", "src/features/session.ts")).toBe(
      "/root/vibe-remote/src/features/session.ts",
    );
  });

  it("rejects traversal or absolute file paths", () => {
    expect(() => normalizeSessionRelativePath("../secrets.txt")).toThrow(
      "traversal segments",
    );
    expect(() => normalizeSessionRelativePath("/etc/passwd")).toThrow(
      "workspace root",
    );
    expect(() => normalizeSessionRelativePath("src\\windows.ts")).toThrow(
      "unsupported characters",
    );
  });

  it("shell-escapes git diff file arguments", () => {
    expect(buildGitDiffCommand("src/it's-complicated.ts")).toBe(
      "git diff --no-ext-diff -- 'src/it'\"'\"'s-complicated.ts'",
    );
    expect(buildGitDiffCommand("src/$(echo hacked).ts")).toBe(
      "git diff --no-ext-diff -- 'src/$(echo hacked).ts'",
    );
  });
});
