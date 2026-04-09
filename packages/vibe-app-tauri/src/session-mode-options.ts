import type { SessionMetadata } from "./wave8-client";

export type SessionModeOption = {
  key: string;
  name: string;
  description: string | null;
};

function mapMetadataOptions(
  options?: Array<{
    code: string;
    value: string;
    description?: string | null;
  }>,
): SessionModeOption[] {
  return (options ?? []).map((option) => ({
    key: option.code,
    name: option.value,
    description: option.description ?? null,
  }));
}

function fallbackPermissionOptions(flavor: string | null | undefined): SessionModeOption[] {
  if (flavor === "codex") {
    return [
      { key: "default", name: "default", description: null },
      { key: "read-only", name: "read-only", description: null },
      { key: "safe-yolo", name: "safe-yolo", description: null },
      { key: "yolo", name: "yolo", description: null },
    ];
  }

  if (flavor === "gemini") {
    return [
      { key: "default", name: "default", description: null },
      { key: "auto_edit", name: "auto edit", description: null },
      { key: "yolo", name: "yolo", description: null },
      { key: "plan", name: "plan", description: null },
    ];
  }

  if (flavor === "openclaw") {
    return [
      { key: "default", name: "default", description: null },
      { key: "bypassPermissions", name: "bypass permissions", description: null },
    ];
  }

  return [
    { key: "default", name: "default", description: null },
    { key: "acceptEdits", name: "accept edits", description: null },
    { key: "plan", name: "plan", description: null },
    { key: "dontAsk", name: "don't ask", description: null },
    { key: "bypassPermissions", name: "bypass permissions", description: null },
  ];
}

function fallbackModelOptions(flavor: string | null | undefined): SessionModeOption[] {
  if (flavor === "codex") {
    return [
      { key: "default", name: "default model", description: null },
      { key: "gpt-5.4", name: "gpt-5.4", description: null },
      { key: "gpt-5.3-codex", name: "gpt-5.3-codex", description: null },
      { key: "gpt-5.2-codex", name: "gpt-5.2-codex", description: null },
    ];
  }

  if (flavor === "gemini") {
    return [
      { key: "gemini-2.5-pro", name: "gemini 2.5 pro", description: "most capable" },
      { key: "gemini-2.5-flash", name: "gemini 2.5 flash", description: "fast & efficient" },
      { key: "gemini-2.5-flash-lite", name: "gemini 2.5 flash lite", description: "fastest" },
    ];
  }

  return [
    { key: "default", name: "default model", description: null },
    { key: "opus", name: "opus 4.6", description: null },
    { key: "sonnet", name: "sonnet 4.6", description: null },
    { key: "haiku", name: "haiku 4.5", description: null },
  ];
}

export function getSessionPermissionOptions(metadata: SessionMetadata | null): SessionModeOption[] {
  const metadataOptions = mapMetadataOptions(metadata?.operatingModes);
  return metadataOptions.length > 0
    ? metadataOptions
    : fallbackPermissionOptions(metadata?.flavor ?? null);
}

export function getSessionModelOptions(metadata: SessionMetadata | null): SessionModeOption[] {
  const metadataOptions = mapMetadataOptions(metadata?.models);
  if (metadataOptions.length > 0) {
    return metadataOptions;
  }

  if (metadata?.currentModelCode) {
    return [
      {
        key: metadata.currentModelCode,
        name: metadata.currentModelCode,
        description: "current session model",
      },
    ];
  }

  return fallbackModelOptions(metadata?.flavor ?? null);
}

export function resolveSessionModeSelection(
  options: SessionModeOption[],
  preferredKeys: Array<string | null | undefined>,
): string {
  for (const key of preferredKeys) {
    if (!key) {
      continue;
    }
    const option = options.find((candidate) => candidate.key === key);
    if (option) {
      return option.key;
    }
  }

  return options[0]?.key ?? "default";
}
