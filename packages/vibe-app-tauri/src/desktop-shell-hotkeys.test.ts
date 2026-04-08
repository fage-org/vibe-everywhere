import { describe, expect, it } from "vitest";
import {
  desktopHotkeyRoutes,
  resolveDesktopShellKeyAction,
} from "./desktop-shell-hotkeys";

describe("desktop shell hotkeys", () => {
  it("toggles the palette on Ctrl/Cmd+K", () => {
    expect(
      resolveDesktopShellKeyAction({
        key: "k",
        metaKey: false,
        ctrlKey: true,
        altKey: false,
        targetIsEditable: false,
      }),
    ).toEqual({ type: "toggle-palette" });
  });

  it("opens the palette on question mark only when not editing", () => {
    expect(
      resolveDesktopShellKeyAction({
        key: "?",
        metaKey: false,
        ctrlKey: false,
        altKey: false,
        targetIsEditable: false,
      }),
    ).toEqual({ type: "open-palette" });

    expect(
      resolveDesktopShellKeyAction({
        key: "?",
        metaKey: false,
        ctrlKey: false,
        altKey: false,
        targetIsEditable: true,
      }),
    ).toEqual({ type: "none" });
  });

  it("navigates on Alt+number shortcuts outside editable fields", () => {
    expect(
      resolveDesktopShellKeyAction({
        key: "2",
        metaKey: false,
        ctrlKey: false,
        altKey: true,
        targetIsEditable: false,
      }),
    ).toEqual({ type: "navigate", path: desktopHotkeyRoutes["2"] });
  });

  it("ignores Alt+number shortcuts inside editable fields", () => {
    expect(
      resolveDesktopShellKeyAction({
        key: "2",
        metaKey: false,
        ctrlKey: false,
        altKey: true,
        targetIsEditable: true,
      }),
    ).toEqual({ type: "none" });
  });

  it("closes the palette on Escape", () => {
    expect(
      resolveDesktopShellKeyAction({
        key: "Escape",
        metaKey: false,
        ctrlKey: false,
        altKey: false,
        targetIsEditable: false,
      }),
    ).toEqual({ type: "close-palette" });
  });
});
