import type { ProviderKind } from "@/types";

export type ConfiguredProject = {
  id: string;
  name: string;
  deviceId: string;
  cwd: string;
  createdAtEpochMs: number;
  updatedAtEpochMs: number;
};

export type ModelProfile = {
  id: string;
  name: string;
  provider: ProviderKind;
  modelId: string;
  createdAtEpochMs: number;
  updatedAtEpochMs: number;
};

const CONFIGURED_PROJECTS_STORAGE_KEY = "vibe.everywhere.configuredProjects";
const MODEL_PROFILES_STORAGE_KEY = "vibe.everywhere.modelProfiles";
const SELECTED_PROJECT_STORAGE_KEY = "vibe.everywhere.selectedProjectId";
const SELECTED_PROVIDER_STORAGE_KEY = "vibe.everywhere.selectedProvider";
const SELECTED_MODEL_STORAGE_KEY = "vibe.everywhere.selectedModelProfileId";
const LAST_CONVERSATION_BY_SCOPE_STORAGE_KEY = "vibe.everywhere.lastConversationByScope";

export function loadConfiguredProjects() {
  return loadJsonArray<ConfiguredProject>(CONFIGURED_PROJECTS_STORAGE_KEY, isConfiguredProject);
}

export function persistConfiguredProjects(projects: ConfiguredProject[]) {
  window.localStorage.setItem(CONFIGURED_PROJECTS_STORAGE_KEY, JSON.stringify(projects));
}

export function loadModelProfiles() {
  return loadJsonArray<ModelProfile>(MODEL_PROFILES_STORAGE_KEY, isModelProfile);
}

export function persistModelProfiles(profiles: ModelProfile[]) {
  window.localStorage.setItem(MODEL_PROFILES_STORAGE_KEY, JSON.stringify(profiles));
}

export function loadSelectedProjectId() {
  return window.localStorage.getItem(SELECTED_PROJECT_STORAGE_KEY) ?? "";
}

export function persistSelectedProjectId(projectId: string) {
  persistString(SELECTED_PROJECT_STORAGE_KEY, projectId);
}

export function loadSelectedProvider() {
  const value = window.localStorage.getItem(SELECTED_PROVIDER_STORAGE_KEY) ?? "";
  return isProviderKind(value) ? value : "";
}

export function persistSelectedProvider(provider: ProviderKind | "") {
  persistString(SELECTED_PROVIDER_STORAGE_KEY, provider);
}

export function loadSelectedModelProfileId() {
  return window.localStorage.getItem(SELECTED_MODEL_STORAGE_KEY) ?? "";
}

export function persistSelectedModelProfileId(modelProfileId: string) {
  persistString(SELECTED_MODEL_STORAGE_KEY, modelProfileId);
}

export function loadLastConversationIdByScope() {
  const raw = window.localStorage.getItem(LAST_CONVERSATION_BY_SCOPE_STORAGE_KEY);
  if (!raw) {
    return {} as Record<string, string>;
  }

  try {
    const parsed = JSON.parse(raw) as Record<string, unknown>;
    return Object.fromEntries(
      Object.entries(parsed).filter(
        (entry): entry is [string, string] =>
          typeof entry[0] === "string" && typeof entry[1] === "string"
      )
    );
  } catch {
    return {} as Record<string, string>;
  }
}

export function persistLastConversationIdByScope(lastConversationIdByScope: Record<string, string>) {
  window.localStorage.setItem(
    LAST_CONVERSATION_BY_SCOPE_STORAGE_KEY,
    JSON.stringify(lastConversationIdByScope)
  );
}

export function buildConversationScopeKey(projectId: string, provider: ProviderKind) {
  return `${projectId}::${provider}`;
}

function loadJsonArray<T>(key: string, predicate: (value: unknown) => value is T) {
  const raw = window.localStorage.getItem(key);
  if (!raw) {
    return [] as T[];
  }

  try {
    const parsed = JSON.parse(raw) as unknown[];
    return Array.isArray(parsed) ? parsed.filter(predicate) : [];
  } catch {
    return [] as T[];
  }
}

function persistString(key: string, value: string) {
  if (value.trim()) {
    window.localStorage.setItem(key, value);
    return;
  }

  window.localStorage.removeItem(key);
}

function isConfiguredProject(value: unknown): value is ConfiguredProject {
  if (!value || typeof value !== "object") {
    return false;
  }

  const candidate = value as Partial<ConfiguredProject>;
  return (
    typeof candidate.id === "string" &&
    typeof candidate.name === "string" &&
    typeof candidate.deviceId === "string" &&
    typeof candidate.cwd === "string" &&
    typeof candidate.createdAtEpochMs === "number" &&
    typeof candidate.updatedAtEpochMs === "number"
  );
}

function isModelProfile(value: unknown): value is ModelProfile {
  if (!value || typeof value !== "object") {
    return false;
  }

  const candidate = value as Partial<ModelProfile>;
  return (
    typeof candidate.id === "string" &&
    typeof candidate.name === "string" &&
    isProviderKind(candidate.provider) &&
    typeof candidate.modelId === "string" &&
    typeof candidate.createdAtEpochMs === "number" &&
    typeof candidate.updatedAtEpochMs === "number"
  );
}

function isProviderKind(value: unknown): value is ProviderKind {
  return value === "codex" || value === "claude_code" || value === "open_code";
}
