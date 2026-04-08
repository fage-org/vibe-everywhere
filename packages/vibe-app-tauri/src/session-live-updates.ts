import type { UiMessage } from "./wave8-client";

export type SessionMessageAccumulator = {
  items: UiMessage[];
  loadedAt: number | null;
  lastSeq: number | null;
};

export type LiveMessageMergeResult =
  | { action: "ignore" }
  | { action: "refresh" }
  | {
      action: "apply";
      next: SessionMessageAccumulator;
    };

export function mergeIncomingSessionMessages(
  current: SessionMessageAccumulator | undefined,
  incomingSeq: number,
  incomingMessages: UiMessage[],
  loadedAt: number,
): LiveMessageMergeResult {
  if (!current?.loadedAt) {
    return { action: "refresh" };
  }

  const currentLastSeq = current.lastSeq;
  if (currentLastSeq === null) {
    if (incomingSeq !== 1) {
      return { action: "refresh" };
    }
  } else if (incomingSeq <= currentLastSeq) {
    return { action: "ignore" };
  } else if (incomingSeq !== currentLastSeq + 1) {
    return { action: "refresh" };
  }

  const byId = new Map(current.items.map((message) => [message.id, message]));
  for (const message of incomingMessages) {
    byId.set(message.id, message);
  }

  return {
    action: "apply",
    next: {
      items: Array.from(byId.values()).sort(
        (left, right) => left.createdAt - right.createdAt,
      ),
      loadedAt,
      lastSeq: incomingSeq,
    },
  };
}
