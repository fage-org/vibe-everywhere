import { describe, expect, it } from "vitest";
import {
  SessionMessagesResponseSchema,
  SessionsRealtimeUpdateSchema,
  parseWithSchema,
  safeParseWithSchema,
} from "./wave8-wire";

describe("wave8 wire schemas", () => {
  it("accepts valid realtime new-message payloads through shared compatibility schemas", () => {
    const parsed = parseWithSchema(
      SessionsRealtimeUpdateSchema,
      {
        id: "update-1",
        seq: 9,
        createdAt: 123456,
        body: {
          t: "new-message",
          sid: "session-1",
          message: {
            id: "message-1",
            seq: 4,
            localId: null,
            content: {
              c: "ciphertext",
              t: "encrypted",
            },
            createdAt: 123400,
            updatedAt: 123401,
          },
        },
      },
      "Realtime update payload",
    );

    expect(parsed.body.t).toBe("new-message");
    if (parsed.body.t !== "new-message") {
      throw new Error("Expected new-message payload");
    }
    expect(parsed.body.message.seq).toBe(4);
  });

  it("rejects invalid realtime payloads instead of trusting unchecked casts", () => {
    const parsed = safeParseWithSchema(SessionsRealtimeUpdateSchema, {
      id: "update-2",
      seq: 10,
      createdAt: 123500,
      body: {
        t: "new-message",
        sid: "session-2",
        message: {
          id: "message-2",
          seq: "broken",
        },
      },
    });

    expect(parsed).toBeNull();
  });

  it("validates paginated message responses", () => {
    const parsed = parseWithSchema(
      SessionMessagesResponseSchema,
      {
        messages: [
          {
            id: "message-3",
            seq: 1,
            localId: null,
            content: {
              c: "ciphertext",
              t: "encrypted",
            },
            createdAt: 10,
            updatedAt: 11,
          },
        ],
        hasMore: false,
      },
      "Session messages page",
    );

    expect(parsed.hasMore).toBe(false);
    expect(parsed.messages).toHaveLength(1);
  });

  it("accepts update-account payloads used for account settings realtime refresh", () => {
    const parsed = parseWithSchema(
      SessionsRealtimeUpdateSchema,
      {
        id: "update-3",
        seq: 11,
        createdAt: 123600,
        body: {
          t: "update-account",
          id: "user-1",
          settings: {
            value: "ciphertext",
            version: 5,
          },
        },
      },
      "Realtime account update payload",
    );

    expect(parsed.body.t).toBe("update-account");
    if (parsed.body.t !== "update-account") {
      throw new Error("Expected update-account payload");
    }
    expect(parsed.body.settings?.version).toBe(5);
  });
});
