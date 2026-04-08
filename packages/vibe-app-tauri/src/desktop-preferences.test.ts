import { beforeEach, describe, expect, it } from "vitest";
import {
  defaultAppearanceSettings,
  defaultLanguageSettings,
  defaultVoiceSettings,
  loadAppearanceSettings,
  loadLanguageSettings,
  loadVoiceSettings,
  resolveDesktopThemePreference,
  saveAppearanceSettings,
  saveLanguageSettings,
  saveVoiceSettings,
} from "./desktop-preferences";

function installMockStorage() {
  const store = new Map<string, string>();
  Object.defineProperty(globalThis, "window", {
    value: {
      localStorage: {
        getItem: (key: string) => store.get(key) ?? null,
        setItem: (key: string, value: string) => {
          store.set(key, value);
        },
        removeItem: (key: string) => {
          store.delete(key);
        },
      },
    },
    configurable: true,
    writable: true,
  });
}

describe("desktop preferences", () => {
  beforeEach(() => {
    installMockStorage();
  });

  it("loads defaults when no appearance settings are stored", () => {
    expect(loadAppearanceSettings()).toEqual(defaultAppearanceSettings);
  });

  it("round-trips appearance settings through storage", () => {
    saveAppearanceSettings({
      ...defaultAppearanceSettings,
      themePreference: "dark",
      density: "compact",
      showFlavorIcons: true,
      wrapLinesInDiffs: true,
    });

    expect(loadAppearanceSettings()).toEqual({
      ...defaultAppearanceSettings,
      themePreference: "dark",
      density: "compact",
      showFlavorIcons: true,
      wrapLinesInDiffs: true,
    });
  });

  it("ignores invalid stored appearance values", () => {
    window.localStorage.setItem(
      "vibe-app-tauri.appearance-settings",
      JSON.stringify({
        themePreference: "sepia",
        density: "wide",
        compactSessionView: "yes",
        avatarStyle: "glass",
      }),
    );

    expect(loadAppearanceSettings()).toEqual(defaultAppearanceSettings);
  });

  it("round-trips voice settings through storage", () => {
    saveVoiceSettings({
      assistantLanguage: "ja",
      customAgentId: "voice-agent-1",
      bypassToken: true,
    });

    expect(loadVoiceSettings()).toEqual({
      assistantLanguage: "ja",
      customAgentId: "voice-agent-1",
      bypassToken: true,
    });
  });

  it("loads defaults when no voice settings are stored", () => {
    expect(loadVoiceSettings()).toEqual(defaultVoiceSettings);
  });

  it("round-trips language settings through storage", () => {
    saveLanguageSettings({
      appLanguage: "ja",
    });

    expect(loadLanguageSettings()).toEqual({
      appLanguage: "ja",
    });
  });

  it("loads defaults when no language settings are stored", () => {
    expect(loadLanguageSettings()).toEqual(defaultLanguageSettings);
  });

  it("resolves adaptive theme against the system preference", () => {
    expect(resolveDesktopThemePreference("adaptive", true)).toBe("dark");
    expect(resolveDesktopThemePreference("adaptive", false)).toBe("light");
    expect(resolveDesktopThemePreference("light", true)).toBe("light");
  });
});
