import { useMemo } from "react";
import type { ResolvedRoute } from "./router";

export type AppV2View =
  | "home"
  | "new-session"
  | "session-recent"
  | "session"
  | "settings"
  | "inbox"
  | "restore"
  | "restore-manual"
  | "session-info"
  | "session-message"
  | "session-files"
  | "session-file"
  | "unsupported";

export type AppV2RouteModel = {
  view: AppV2View;
  activeSessionId: string | null;
  activeMessageId: string | null;
  activeFilePath: string | null;
  canonicalPath: string;
  isSupported: boolean;
};

const SETTINGS_ROUTE_KEYS = new Set([
  "settings-index",
  "settings-account",
  "settings-appearance",
  "settings-features",
  "settings-language",
  "settings-usage",
  "settings-voice",
  "settings-voice-language",
  "settings-connect-claude",
]);

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
    default:
      if (SETTINGS_ROUTE_KEYS.has(resolved.definition.key)) {
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
      canonicalPath: resolved.canonicalPath,
      isSupported: view !== "unsupported",
    };
  }, [resolved]);
}
