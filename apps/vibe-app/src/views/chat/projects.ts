import type { ConversationRecord } from "@/types";

export const DEFAULT_PROJECT_KEY = "__default__";

export type ProjectListItem = {
  deviceId: string;
  key: string;
  cwd: string | null;
  name: string;
  topicCount: number;
  latestConversationId: string | null;
  latestConversationTitle: string;
  latestProvider: string;
  latestUpdatedAtEpochMs: number;
};

export function normalizeProjectCwd(value: string | null | undefined) {
  const normalized = value?.trim() ?? "";
  return normalized || null;
}

export function encodeProjectKey(value: string | null | undefined) {
  const cwd = normalizeProjectCwd(value);
  if (!cwd) {
    return DEFAULT_PROJECT_KEY;
  }

  return toBase64Url(cwd);
}

export function decodeProjectKey(value: string | string[] | undefined) {
  if (!value) {
    return null;
  }

  const key = Array.isArray(value) ? value[0] : value;
  if (!key || key === DEFAULT_PROJECT_KEY) {
    return null;
  }

  try {
    return normalizeProjectCwd(fromBase64Url(key));
  } catch {
    return null;
  }
}

export function conversationMatchesProject(
  conversation: ConversationRecord,
  deviceId: string,
  cwd: string | null,
) {
  return (
    conversation.deviceId === deviceId &&
    normalizeProjectCwd(conversation.cwd) === normalizeProjectCwd(cwd)
  );
}

export function collectProjectsForDevice(
  conversations: ConversationRecord[],
  deviceId: string,
) {
  const groups = new Map<string, ProjectListItem>();

  for (const conversation of conversations) {
    if (conversation.deviceId !== deviceId || conversation.archived) {
      continue;
    }

    const cwd = normalizeProjectCwd(conversation.cwd);
    const key = encodeProjectKey(cwd);
    const current = groups.get(key);
    const conversationTitle = conversation.title.trim() || "Untitled";

    if (!current) {
      groups.set(key, {
        deviceId,
        key,
        cwd,
        name: projectNameFromCwd(cwd),
        topicCount: 1,
        latestConversationId: conversation.id,
        latestConversationTitle: conversationTitle,
        latestProvider: conversation.provider,
        latestUpdatedAtEpochMs: conversation.updatedAtEpochMs,
      });
      continue;
    }

    current.topicCount += 1;
    if (conversation.updatedAtEpochMs > current.latestUpdatedAtEpochMs) {
      current.latestConversationId = conversation.id;
      current.latestConversationTitle = conversationTitle;
      current.latestProvider = conversation.provider;
      current.latestUpdatedAtEpochMs = conversation.updatedAtEpochMs;
    }
  }

  return [...groups.values()].sort(
    (left, right) => right.latestUpdatedAtEpochMs - left.latestUpdatedAtEpochMs,
  );
}

function projectNameFromCwd(cwd: string | null) {
  if (!cwd) {
    return "Default Workspace";
  }

  const trimmed = cwd.replace(/[\\/]+$/, "");
  const segments = trimmed.split(/[\\/]/).filter(Boolean);
  return segments[segments.length - 1] ?? cwd;
}

function toBase64Url(value: string) {
  const bytes = new TextEncoder().encode(value);
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }

  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function fromBase64Url(value: string) {
  const normalized = value.replace(/-/g, "+").replace(/_/g, "/");
  const padded = normalized.padEnd(Math.ceil(normalized.length / 4) * 4, "=");
  const binary = atob(padded);
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
}
