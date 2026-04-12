import { useState } from "react";
import type { CreateSessionInput } from "./useNewSessionDraftForm";
import type { AppShellState } from "./useAppShellState";

export function useCreateSessionAction(
  shell: AppShellState,
  navigate: (path: string) => void,
  onCreated?: () => void,
) {
  const [isCreatingSession, setIsCreatingSession] = useState(false);
  const [createSessionError, setCreateSessionError] = useState<string | null>(null);

  const createSession = async (input: CreateSessionInput) => {
    setIsCreatingSession(true);
    setCreateSessionError(null);
    try {
      const session = await shell.createSession(input);
      onCreated?.();
      navigate(`/(app)/session/${session.id}`);
    } catch (error) {
      setCreateSessionError(
        error instanceof Error ? error.message : "Failed to create session",
      );
    } finally {
      setIsCreatingSession(false);
    }
  };

  return {
    isCreatingSession,
    createSessionError,
    setCreateSessionError,
    createSession,
  };
}
