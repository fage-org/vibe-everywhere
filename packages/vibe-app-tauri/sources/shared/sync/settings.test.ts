import { describe, expect, it } from "vitest";
import { applySettings, settingsDefaults, settingsParse } from "./settings";

describe("shared settings helpers", () => {
  it("fills defaults and preserves unknown fields", () => {
    const parsed = settingsParse({
      preferredLanguage: "ja",
      customFlag: "keep-me",
    });

    expect(parsed.preferredLanguage).toBe("ja");
    expect(parsed.customFlag).toBe("keep-me");
    expect(parsed.avatarStyle).toBe(settingsDefaults.avatarStyle);
  });

  it("migrates legacy zh language values", () => {
    const parsed = settingsParse({ preferredLanguage: "zh" });

    expect(parsed.preferredLanguage).toBe("zh-Hans");
  });

  it("applies partial updates on top of current settings", () => {
    const updated = applySettings(settingsDefaults, {
      showFlavorIcons: true,
      preferredLanguage: "it",
    });

    expect(updated.showFlavorIcons).toBe(true);
    expect(updated.preferredLanguage).toBe("it");
    expect(updated.avatarStyle).toBe(settingsDefaults.avatarStyle);
  });
});
