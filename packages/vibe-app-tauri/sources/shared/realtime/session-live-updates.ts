export type TimelineMessageRecord = {
  id: string;
  createdAt: number;
  [key: string]: unknown;
};

export type SessionMessageAccumulator<T extends TimelineMessageRecord = TimelineMessageRecord> = {
  items: T[];
  loadedAt: number | null;
  lastSeq: number | null;
};

export type LiveMessageMergeResult<T extends TimelineMessageRecord = TimelineMessageRecord> =
  | { action: "ignore" }
  | { action: "refresh" }
  | {
      action: "apply";
      next: SessionMessageAccumulator<T>;
    };

export function mergeIncomingSessionMessages<T extends TimelineMessageRecord>(
  current: SessionMessageAccumulator<T> | undefined,
  incomingSeq: number,
  incomingMessages: T[],
  loadedAt: number,
): LiveMessageMergeResult<T> {
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
