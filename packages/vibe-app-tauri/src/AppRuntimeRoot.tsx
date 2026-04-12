import { App } from "./App";
import { AppV2 } from "./AppV2";
import { useDesktopRouter } from "./router";
import type { ResolvedRoute } from "./router";

const APP_V2_ROUTE_KEYS = new Set([
  "home",
  "inbox",
  "new-session",
  "session-recent",
  "session-detail",
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

const LEGACY_APP_ROUTE_KEYS = new Set([
  "restore-index",
  "restore-manual",
  "session-info",
  "session-message",
  "session-files",
  "session-file",
]);

export function shouldUseAppV2Root(resolved: ResolvedRoute): boolean {
  if (LEGACY_APP_ROUTE_KEYS.has(resolved.definition.key)) {
    return false;
  }

  if (APP_V2_ROUTE_KEYS.has(resolved.definition.key)) {
    return true;
  }

  return true;
}

export function AppRuntimeRoot() {
  const router = useDesktopRouter();

  if (!shouldUseAppV2Root(router.resolved)) {
    return <App />;
  }

  return <AppV2 />;
}
