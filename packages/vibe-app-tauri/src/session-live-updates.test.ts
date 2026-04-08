import { describe, expect, it } from "vitest";
import {
  mergeIncomingSessionMessages,
  type SessionMessageAccumulator,
} from "./session-live-updates";

const baseState: SessionMessageAccumulator = {
  items: [
    {
      id: "m1",
      localId: null,
      createdAt: 1,
      role: "assistant",
      title: "Assistant",
      text: "hello",
      rawType: "agent:assistant",
    },
  ],
  loadedAt: 100,
  lastSeq: 1,
};

describe("session live updates", () => {
  it("appends contiguous live messages", () => {
    const result = mergeIncomingSessionMessages(
      baseState,
      2,
      [
        {
          id: "m2",
          localId: null,
          createdAt: 2,
          role: "assistant",
          title: "Assistant",
          text: "follow up",
          rawType: "agent:assistant",
        },
      ],
      200,
    );

    expect(result.action).toBe("apply");
    if (result.action !== "apply") {
      throw new Error("Expected live update to apply");
    }
    expect(result.next.lastSeq).toBe(2);
    expect(result.next.items.map((message) => message.id)).toEqual(["m1", "m2"]);
  });

  it("ignores duplicate or stale live messages", () => {
    const result = mergeIncomingSessionMessages(
      baseState,
      1,
      [
        {
          id: "m1",
          localId: null,
          createdAt: 1,
          role: "assistant",
          title: "Assistant",
          text: "hello",
          rawType: "agent:assistant",
        },
      ],
      200,
    );

    expect(result).toEqual({ action: "ignore" });
  });

  it("falls back to refresh when a sequence gap appears", () => {
    const result = mergeIncomingSessionMessages(
      baseState,
      3,
      [
        {
          id: "m3",
          localId: null,
          createdAt: 3,
          role: "assistant",
          title: "Assistant",
          text: "gap",
          rawType: "agent:assistant",
        },
      ],
      200,
    );

    expect(result).toEqual({ action: "refresh" });
  });

  it("requires the first loaded message to start at seq 1", () => {
    const result = mergeIncomingSessionMessages(
      {
        items: [],
        loadedAt: 100,
        lastSeq: null,
      },
      4,
      [
        {
          id: "m4",
          localId: null,
          createdAt: 4,
          role: "assistant",
          title: "Assistant",
          text: "unexpected start",
          rawType: "agent:assistant",
        },
      ],
      200,
    );

    expect(result).toEqual({ action: "refresh" });
  });
});
