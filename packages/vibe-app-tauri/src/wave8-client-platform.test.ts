import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(),
}));

import { invoke, isTauri } from "@tauri-apps/api/core";
import {
  openTextFileDialog,
  saveTextFileDialog,
  showDesktopNotification,
} from "./wave8-client";

describe("wave8 desktop platform adapters", () => {
  beforeEach(() => {
    vi.stubGlobal("window", {});
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockReset();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("opens and saves text through Tauri invoke commands", async () => {
    vi.mocked(invoke).mockImplementation(async (command) => {
      if (command === "open_text_file_dialog") {
        return "backup-key";
      }
      if (command === "save_text_file_dialog") {
        return "/tmp/desktop-selection.txt";
      }
      return null;
    });

    await expect(openTextFileDialog("Load desktop backup key")).resolves.toBe("backup-key");
    await expect(
      saveTextFileDialog({
        title: "Save selected text",
        suggestedName: "desktop-selection.txt",
        contents: "hello",
      }),
    ).resolves.toBe("/tmp/desktop-selection.txt");

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("open_text_file_dialog", {
      title: "Load desktop backup key",
    });
    expect(vi.mocked(invoke)).toHaveBeenCalledWith("save_text_file_dialog", {
      title: "Save selected text",
      suggestedName: "desktop-selection.txt",
      contents: "hello",
    });
  });

  it("routes notifications through the desktop notification command in Tauri", async () => {
    await showDesktopNotification("Artifact updated", "Saved from the desktop editor.");

    expect(vi.mocked(invoke)).toHaveBeenCalledWith("show_desktop_notification", {
      title: "Artifact updated",
      body: "Saved from the desktop editor.",
    });
  });

  it("falls back to the browser Notification API outside Tauri", async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    const requestPermission = vi.fn(async () => "granted" as const);
    const notificationCtor = vi.fn();
    vi.stubGlobal("Notification", Object.assign(notificationCtor, {
      permission: "default",
      requestPermission,
    }));

    await showDesktopNotification("Desktop account restored", "The backup key worked.");

    expect(requestPermission).toHaveBeenCalled();
    expect(notificationCtor).toHaveBeenCalledWith("Desktop account restored", {
      body: "The backup key worked.",
    });
  });

  it("resolves null when the browser file dialog is canceled", async () => {
    vi.useFakeTimers();
    vi.mocked(isTauri).mockReturnValue(false);

    const listeners = new Map<string, () => void>();
    const input = {
      type: "",
      accept: "",
      style: {} as Record<string, string>,
      files: null as FileList | null,
      addEventListener: vi.fn(),
      remove: vi.fn(),
      click: vi.fn(() => {
        const focusHandler = listeners.get("focus");
        if (focusHandler) {
          focusHandler();
        }
      }),
    };

    vi.stubGlobal("window", {
      addEventListener: vi.fn((event: string, handler: () => void) => {
        listeners.set(event, handler);
      }),
      removeEventListener: vi.fn((event: string) => {
        listeners.delete(event);
      }),
      setTimeout,
      clearTimeout,
    });
    vi.stubGlobal("document", {
      body: {
        appendChild: vi.fn(),
      },
      createElement: vi.fn(() => input),
    });

    const pending = openTextFileDialog("Load desktop backup key");
    await vi.advanceTimersByTimeAsync(300);

    await expect(pending).resolves.toBeNull();
    expect(input.remove).toHaveBeenCalled();
    vi.useRealTimers();
  });
});
