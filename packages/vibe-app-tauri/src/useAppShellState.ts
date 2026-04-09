import { useWave8Desktop, type LinkUiState, type SessionUiState } from "./useWave8Desktop";

export type { LinkUiState, SessionUiState };

export type AppShellState = ReturnType<typeof useWave8Desktop>;

export function useAppShellState(activeSessionId?: string | null): AppShellState {
  return useWave8Desktop(activeSessionId);
}
