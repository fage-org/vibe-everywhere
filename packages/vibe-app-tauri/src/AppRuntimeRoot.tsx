import { AppV2 } from "./AppV2";
import { useDesktopRouter } from "./router";

export function AppRuntimeRoot() {
  const router = useDesktopRouter();

  // AppV2 is now the default shell for all routes
  return <AppV2 />;
}

/**
 * Legacy routing logic removed - AppV2 is now the sole UI shell.
 *
 * Historical note: Prior to this change, LEGACY_APP_ROUTE_KEYS routed to
 * the old App.tsx component. All routes now use AppV2.
 *
 * Archived route keys:
 * - restore-index (now handled by RestoreRoute in AppV2)
 * - restore-manual (now handled by ManualRestoreRoute in AppV2)
 * - session-info (now handled by SessionInfoRoute in AppV2)
 * - session-message (now handled by SessionMessageRoute in AppV2)
 * - session-files (now handled by SessionFilesRoute in AppV2)
 * - session-file (now handled by SessionFileRoute in AppV2)
 */