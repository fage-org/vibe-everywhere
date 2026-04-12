import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  clearNewSessionDraft,
  loadNewSessionDraft,
  saveNewSessionDraft,
  type NewSessionDraft,
} from "./new-session-draft";

export type CreateSessionInput = NewSessionDraft;

export type NewSessionDraftForm = {
  workspace: string;
  model: string;
  title: string;
  prompt: string;
};

export type NewSessionDraftErrors = {
  workspace?: string;
  prompt?: string;
};

export const APP_V2_NEW_SESSION_DEFAULT_DRAFT: NewSessionDraft = {
  workspace: "",
  model: "gpt-5.4",
  title: "",
  prompt: "",
};

export function useNewSessionDraftForm(
  defaults: NewSessionDraft = APP_V2_NEW_SESSION_DEFAULT_DRAFT,
) {
  const { t } = useTranslation("routes");
  const translateOrDefault = (key: string, fallback: string) => {
    const translated = t(key);
    return translated === key ? fallback : translated;
  };
  const initialDraftRef = useRef<NewSessionDraft | null>(null);
  const skipNextPersistRef = useRef(false);
  if (!initialDraftRef.current) {
    initialDraftRef.current = loadNewSessionDraft(defaults);
  }

  const [workspace, setWorkspace] = useState(initialDraftRef.current.workspace);
  const [model, setModel] = useState(initialDraftRef.current.model);
  const [title, setTitle] = useState(initialDraftRef.current.title);
  const [prompt, setPrompt] = useState(initialDraftRef.current.prompt);
  const [errors, setErrors] = useState<NewSessionDraftErrors>({});

  useEffect(() => {
    if (skipNextPersistRef.current) {
      skipNextPersistRef.current = false;
      return;
    }

    saveNewSessionDraft({
      workspace,
      model,
      title,
      prompt,
    });
  }, [model, prompt, title, workspace]);

  const validate = (): CreateSessionInput | null => {
    const nextErrors: NewSessionDraftErrors = {};
    if (!workspace.trim()) {
      nextErrors.workspace = translateOrDefault(
        "newSession.validation.workspaceRequired",
        "Workspace is required.",
      );
    }

    if (!prompt.trim()) {
      nextErrors.prompt = translateOrDefault(
        "newSession.validation.promptRequired",
        "Initial prompt is required.",
      );
    }

    if (Object.keys(nextErrors).length > 0) {
      setErrors(nextErrors);
      return null;
    }

    setErrors({});
    return {
      workspace: workspace.trim(),
      model: model.trim() || defaults.model,
      title: title.trim(),
      prompt: prompt.trim(),
    };
  };

  const reset = () => {
    skipNextPersistRef.current = true;
    clearNewSessionDraft();
    setWorkspace(defaults.workspace);
    setModel(defaults.model);
    setTitle(defaults.title);
    setPrompt(defaults.prompt);
    setErrors({});
  };

  return {
    workspace,
    setWorkspace,
    model,
    setModel,
    title,
    setTitle,
    prompt,
    setPrompt,
    errors,
    setErrors,
    validate,
    reset,
  };
}
