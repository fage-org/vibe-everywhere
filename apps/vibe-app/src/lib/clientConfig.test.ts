import { describe, expect, it } from "vitest";
import { normalizeModelId } from "@/lib/clientConfig";

describe("normalizeModelId", () => {
  it("fixes the common Codex gbt typo", () => {
    expect(normalizeModelId("codex", "gbt-5.4")).toBe("gpt-5.4");
  });

  it("keeps non-codex providers unchanged", () => {
    expect(normalizeModelId("claude_code", "gbt-5.4")).toBe("gbt-5.4");
  });

  it("trims surrounding whitespace", () => {
    expect(normalizeModelId("codex", "  gpt-5.4  ")).toBe("gpt-5.4");
  });
});
