import { describe, expect, it } from "vitest";
import { buildResumeCommand, buildResumeCommandBlock } from "./resumeCommand";

describe("shared resume command helpers", () => {
  it("builds a resumable Claude command with directory change for POSIX shells", () => {
    expect(buildResumeCommand({
      path: "/tmp/project",
      os: "darwin",
      flavor: "claude",
      claudeSessionId: "93a9705e-bc6a-406d-8dce-8acc014dedbd",
    })).toBe(
      "cd '/tmp/project' && happy claude --resume 93a9705e-bc6a-406d-8dce-8acc014dedbd",
    );
  });

  it("builds copyable Windows Codex instructions", () => {
    expect(buildResumeCommandBlock({
      path: "C:\\Users\\test\\project",
      os: "win32",
      flavor: "codex",
      codexThreadId: "019ccca5-726b-7c61-b914-16de27dfab6e",
    })).toEqual({
      lines: [
        "Set-Location -LiteralPath 'C:\\Users\\test\\project'",
        "happy codex --resume 019ccca5-726b-7c61-b914-16de27dfab6e",
      ],
      copyText:
        "Set-Location -LiteralPath 'C:\\Users\\test\\project'\n"
        + "happy codex --resume 019ccca5-726b-7c61-b914-16de27dfab6e",
    });
  });

  it("returns null when no resumable identifier is available", () => {
    expect(buildResumeCommand({ path: "/tmp/project", flavor: "claude" })).toBeNull();
  });
});
