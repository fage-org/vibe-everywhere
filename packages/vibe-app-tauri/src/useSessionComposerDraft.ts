import { useEffect, useState } from "react";
import {
  clearSessionDraft,
  loadSessionDraft,
  saveSessionDraft,
} from "./session-drafts";

export function useSessionComposerDraft(activeSessionId: string | null) {
  const [composerValue, setComposerValue] = useState("");

  useEffect(() => {
    if (!activeSessionId) {
      setComposerValue("");
      return;
    }

    setComposerValue(loadSessionDraft(activeSessionId));
  }, [activeSessionId]);

  useEffect(() => {
    if (!activeSessionId) {
      return;
    }

    if (composerValue.trim()) {
      saveSessionDraft(activeSessionId, composerValue);
      return;
    }

    clearSessionDraft(activeSessionId);
  }, [activeSessionId, composerValue]);

  const clearComposerValue = () => {
    if (activeSessionId) {
      clearSessionDraft(activeSessionId);
    }
    setComposerValue("");
  };

  return {
    composerValue,
    setComposerValue,
    clearComposerValue,
  };
}
