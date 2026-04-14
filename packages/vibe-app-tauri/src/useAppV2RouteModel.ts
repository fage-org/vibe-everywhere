import { useMemo } from "react";
import type { ResolvedRoute } from "./router";

export type AppV2View =
  | "home"
  | "new-session"
  | "session-recent"
  | "session"
  | "settings"
  | "settings-account"
  | "settings-appearance"
  | "settings-ai-providers"
  | "settings-features"
  | "settings-language"
  | "settings-usage"
  | "settings-voice"
  | "restore"
  | "restore-manual"
  | "session-info"
  | "session-message"
  | "session-files"
  | "session-file"
  | "machine-detail"
  | "artifacts"
  | "artifact-detail"
  | "artifact-edit"
  | "artifact-new"
  | "terminal"
  | "terminal-connect"
  | "unsupported";

export type AppV2RouteModel = {
  view: AppV2View;
  activeSessionId: string | null;
  activeMessageId: string | null;
  activeFilePath: string | null;
  activeMachineId: string | null;
  activeArtifactId: string | null;
  canonicalPath: string;
  isSupported: boolean;
};

const SETTINGS_INDEX_KEYS = new Set([
  "settings-index",
  "settings-voice-language",
  "settings-connect-claude",
]);

const SETTINGS_SUBVIEW_MAP: Record<string, AppV2View> = {
  "settings-account": "settings-account",
  "settings-appearance": "settings-appearance",
  "settings-ai-providers": "settings-ai-providers",
  "settings-features": "settings-features",
  "settings-language": "settings-language",
  "settings-usage": "settings-usage",
  "settings-voice": "settings-voice",
};

const ARTIFACT_ROUTES_MAP: Record<string, AppV2View> = {
  "artifacts-index": "artifacts",
  "artifacts-new": "artifact-new",
  "artifacts-detail": "artifact-detail",
  "artifacts-edit": "artifact-edit",
};

const TERMINAL_ROUTES_MAP: Record<string, AppV2View> = {
  "terminal-index": "terminal",
  "terminal-connect": "terminal-connect",
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
      if (ARTIFACT_ROUTES_MAP[resolved.definition.key]) {
        return ARTIFACT_ROUTES_MAP[resolved.definition.key];
      }
      if (TERMINAL_ROUTES_MAP[resolved.definition.key]) {
        return TERMINAL_ROUTES_MAP[resolved.definition.key];
      }
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
    const isArtifactView = ["artifact-detail", "artifact-edit"].includes(view);
    return {
      view,
      activeSessionId: isSessionView ? resolved.params.id ?? null : null,
      activeMessageId: view === "session-message" ? resolved.params.messageId ?? null : null,
      activeFilePath: view === "session-file" ? resolved.searchParams.get("path") ?? null : null,
      activeMachineId: view === "machine-detail" ? resolved.params.id ?? null : null,
      activeArtifactId: isArtifactView ? resolved.params.id ?? null : null,
      canonicalPath: resolved.canonicalPath,
      isSupported: view !== "unsupported",
    };
  }, [resolved]);
}
