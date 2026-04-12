import { useEffect, useMemo, useState } from "react";
import { getSessionModelOptions, resolveSessionModeSelection } from "./session-mode-options";
import { loadSessionPreferences, saveSessionPreferences } from "./session-preferences";
import type { DesktopSession, SendMessageOptions } from "./wave8-client";

type SessionModelOption = {
  id: string;
  name: string;
};

export function useAppV2ComposerPreferences(
  activeSessionId: string | null,
  currentSession: DesktopSession | null,
) {
  const modelOptions = useMemo(
    () => getSessionModelOptions(currentSession?.metadata ?? null),
    [currentSession?.metadata],
  );

  const defaultPreferences = useMemo(
    () => ({
      permissionMode: "default",
      model: resolveSessionModeSelection(modelOptions, [
        currentSession?.metadata?.currentModelCode,
        "default",
      ]),
    }),
    [currentSession?.metadata?.currentModelCode, modelOptions],
  );

  const [composerPreferences, setComposerPreferences] = useState(defaultPreferences);

  useEffect(() => {
    if (!activeSessionId) {
      setComposerPreferences(defaultPreferences);
      return;
    }

    setComposerPreferences(loadSessionPreferences(activeSessionId, defaultPreferences));
  }, [activeSessionId, defaultPreferences]);

  useEffect(() => {
    if (!activeSessionId) {
      return;
    }

    saveSessionPreferences(activeSessionId, composerPreferences);
  }, [activeSessionId, composerPreferences]);

  const models = useMemo<SessionModelOption[]>(
    () =>
      modelOptions.map((model) => ({
        id: model.key,
        name: model.description ? `${model.name} - ${model.description}` : model.name,
      })),
    [modelOptions],
  );

  const sendMessageOptions = useMemo<SendMessageOptions>(
    () => ({
      model: composerPreferences.model === "default" ? null : composerPreferences.model,
    }),
    [composerPreferences.model],
  );

  return {
    models,
    selectedModel: composerPreferences.model,
    setSelectedModel: (model: string) => {
      setComposerPreferences((current) => ({
        ...current,
        model,
      }));
    },
    sendMessageOptions,
  };
}
