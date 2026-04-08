import { DEFAULT_PATH } from "./router";

export const desktopHotkeyRoutes: Record<string, string> = {
  "1": DEFAULT_PATH,
  "2": "/(app)/inbox/index",
  "3": "/(app)/new/index",
  "4": "/(app)/session/recent",
  "5": "/(app)/settings/index",
  "6": "/(app)/restore/index",
};

export type DesktopShellKeyAction =
  | { type: "none" }
  | { type: "toggle-palette" }
  | { type: "open-palette" }
  | { type: "close-palette" }
  | { type: "navigate"; path: string };

export function isEditableTarget(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  );
}

export function resolveDesktopShellKeyAction(input: {
  key: string;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
  targetIsEditable: boolean;
}): DesktopShellKeyAction {
  const lowerKey = input.key.toLowerCase();

  if ((input.metaKey || input.ctrlKey) && lowerKey === "k") {
    return { type: "toggle-palette" };
  }

  if (!input.targetIsEditable && input.key === "?") {
    return { type: "open-palette" };
  }

  if (input.key === "Escape") {
    return { type: "close-palette" };
  }

  if (input.targetIsEditable) {
    return { type: "none" };
  }

  if (input.altKey && !input.ctrlKey && !input.metaKey) {
    const path = desktopHotkeyRoutes[lowerKey];
    if (path) {
      return { type: "navigate", path };
    }
  }

  return { type: "none" };
}
