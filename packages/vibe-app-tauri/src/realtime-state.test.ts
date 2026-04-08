import { describe, expect, it } from "vitest";
import { removeDeletedSession, upsertRealtimeSession } from "./realtime-state";

describe("realtime state helpers", () => {
  it("keeps updated sessions sorted by most recent activity", () => {
    const result = upsertRealtimeSession(
      [
        {
          id: "session-1",
          seq: 1,
          createdAt: 1,
          updatedAt: 10,
          active: false,
          activeAt: 10,
          metadata: null,
          metadataVersion: 1,
          agentState: null,
          agentStateVersion: 1,
          dataEncryptionKey: null,
        },
        {
          id: "session-2",
          seq: 2,
          createdAt: 2,
          updatedAt: 20,
          active: true,
          activeAt: 20,
          metadata: null,
          metadataVersion: 1,
          agentState: null,
          agentStateVersion: 1,
          dataEncryptionKey: null,
        },
      ],
      {
        id: "session-1",
        seq: 3,
        createdAt: 1,
        updatedAt: 30,
        active: true,
        activeAt: 30,
        metadata: null,
        metadataVersion: 2,
        agentState: null,
        agentStateVersion: 2,
        dataEncryptionKey: null,
      },
    );

    expect(result.map((session) => session.id)).toEqual(["session-1", "session-2"]);
    expect(result[0]?.updatedAt).toBe(30);
  });

  it("removes deleted sessions from both session inventory and cached session state", () => {
    const result = removeDeletedSession(
      [
        {
          id: "session-1",
          seq: 1,
          createdAt: 1,
          updatedAt: 1,
          active: true,
          activeAt: 1,
          metadata: null,
          metadataVersion: 1,
          agentState: null,
          agentStateVersion: 1,
          dataEncryptionKey: null,
        },
        {
          id: "session-2",
          seq: 2,
          createdAt: 2,
          updatedAt: 2,
          active: false,
          activeAt: 2,
          metadata: null,
          metadataVersion: 1,
          agentState: null,
          agentStateVersion: 1,
          dataEncryptionKey: null,
        },
      ],
      {
        "session-1": {
          items: [],
          loading: false,
          sending: false,
          error: null,
          loadedAt: 1,
          lastSeq: 1,
        },
        "session-2": {
          items: [],
          loading: false,
          sending: false,
          error: null,
          loadedAt: 2,
          lastSeq: 2,
        },
      },
      "session-1",
    );

    expect(result.sessions.map((session) => session.id)).toEqual(["session-2"]);
    expect(Object.keys(result.sessionState)).toEqual(["session-2"]);
  });
});
