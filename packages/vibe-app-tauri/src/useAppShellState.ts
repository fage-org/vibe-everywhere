import { useDesktopState, type LinkUiState, type SessionUiState } from "./useDesktopState";

export type { LinkUiState, SessionUiState };

export type AppShellState = ReturnType<typeof useDesktopState>;

export function useAppShellState(activeSessionId?: string | null): AppShellState {
  return useDesktopState(activeSessionId);
}
