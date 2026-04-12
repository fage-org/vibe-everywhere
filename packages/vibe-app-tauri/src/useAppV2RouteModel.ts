import { useMemo } from "react";
import type { ResolvedRoute } from "./router";

export type AppV2View =
  | "home"
  | "new-session"
  | "session-recent"
  | "session"
  | "settings"
  | "inbox"
  | "unsupported";

export type AppV2RouteModel = {
  view: AppV2View;
  activeSessionId: string | null;
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
    return {
      view,
      activeSessionId: view === "session" ? resolved.params.id ?? null : null,
      canonicalPath: resolved.canonicalPath,
      isSupported: view !== "unsupported",
    };
  }, [resolved]);
}
