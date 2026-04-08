import { afterEach, describe, expect, it, vi } from "vitest";
import { copyTextToClipboard } from "./wave8-client";

describe("wave8 clipboard helpers", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("copies text with the browser clipboard API when available", async () => {
    const writeText = vi.fn(async () => undefined);
    vi.stubGlobal("navigator", {
      clipboard: { writeText },
    });

    await copyTextToClipboard("desktop-backup-key");

    expect(writeText).toHaveBeenCalledWith("desktop-backup-key");
  });
});
