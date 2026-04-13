import { useMemo } from "react";
import type { ResolvedRoute } from "./router";

export type AppV2View =
  | "home"
  | "new-session"
  | "session-recent"
  | "session"
  | "settings"
  | "settings-appearance"
  | "settings-features"
  | "settings-language"
  | "settings-usage"
  | "inbox"
  | "restore"
  | "restore-manual"
  | "session-info"
  | "session-message"
  | "session-files"
  | "session-file"
  | "machine-detail"
  | "unsupported";

export type AppV2RouteModel = {
  view: AppV2View;
  activeSessionId: string | null;
  activeMessageId: string | null;
  activeFilePath: string | null;
  activeMachineId: string | null;
  canonicalPath: string;
  isSupported: boolean;
};

const SETTINGS_INDEX_KEYS = new Set([
  "settings-index",
  "settings-account",
  "settings-voice",
  "settings-voice-language",
  "settings-connect-claude",
]);

const SETTINGS_SUBVIEW_MAP: Record<string, AppV2View> = {
  "settings-appearance": "settings-appearance",
  "settings-features": "settings-features",
  "settings-language": "settings-language",
  "settings-usage": "settings-usage",
};

export function resolveAppV2View(resolved: ResolvedRoute): AppV2View {
  switch (resolved.definition.key) {
    case "home":
      return "home";
    case "new-session":
      return "new-session";
    case "session-recent":
      return "session-recent";
    case "session-detail":
      return "session";
    case "inbox":
      return "inbox";
    case "restore-index":
      return "restore";
    case "restore-manual":
      return "restore-manual";
    case "session-info":
      return "session-info";
    case "session-message":
      return "session-message";
    case "session-files":
      return "session-files";
    case "session-file":
      return "session-file";
    case "machine-detail":
      return "machine-detail";
    default:
      if (SETTINGS_SUBVIEW_MAP[resolved.definition.key]) {
        return SETTINGS_SUBVIEW_MAP[resolved.definition.key];
      }
      if (SETTINGS_INDEX_KEYS.has(resolved.definition.key)) {
        return "settings";
      }

      return "unsupported";
  }
}

export function useAppV2RouteModel(resolved: ResolvedRoute): AppV2RouteModel {
  return useMemo(() => {
    const view = resolveAppV2View(resolved);
    const isSessionView = ["session", "session-info", "session-message", "session-files", "session-file"].includes(view);
    return {
      view,
      activeSessionId: isSessionView ? resolved.params.id ?? null : null,
      activeMessageId: view === "session-message" ? resolved.params.messageId ?? null : null,
      activeFilePath: view === "session-file" ? resolved.searchParams.get("path") ?? null : null,
      activeMachineId: view === "machine-detail" ? resolved.params.id ?? null : null,
      canonicalPath: resolved.canonicalPath,
      isSupported: view !== "unsupported",
    };
  }, [resolved]);
}
