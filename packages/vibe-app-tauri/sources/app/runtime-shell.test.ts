import { describe, expect, it } from "vitest";
import {
  buildRuntimeDocumentTitle,
  buildRuntimeMetaDescription,
  resolveRuntimeShellCopy,
} from "./runtime-shell";

describe("runtime shell copy", () => {
  it("resolves Android-specific shell labels", () => {
    const copy = resolveRuntimeShellCopy("mobile");

    expect(copy.entryEyebrow).toBe("Android entry");
    expect(copy.primaryNavLabel).toBe("Primary Android routes");
    expect(copy.configTitle).toBe("Android Configuration");
  });

  it("builds runtime-specific document titles", () => {
    expect(buildRuntimeDocumentTitle("desktop", "Settings")).toBe(
      "Settings | Vibe Desktop",
    );
    expect(buildRuntimeDocumentTitle("mobile", "Restore")).toBe(
      "Restore | Vibe Android",
    );
    expect(buildRuntimeDocumentTitle("browser")).toBe("Vibe Browser Export");
  });

  it("keeps browser export metadata explicit", () => {
    expect(buildRuntimeMetaDescription("browser")).toContain("retained browser export");
  });
});
