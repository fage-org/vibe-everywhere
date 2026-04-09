import {
  act,
  create,
  type ReactTestRenderer,
} from "react-test-renderer";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const runtimeMocks = vi.hoisted(() => ({
  loadServerUrl: vi.fn(() => "https://api.cluster-fluster.com"),
  loadStoredCredentials: vi.fn(),
  saveStoredCredentials: vi.fn(async () => undefined),
  clearStoredCredentials: vi.fn(async () => undefined),
  saveServerUrl: vi.fn((value: string) => value),
  showDesktopNotification: vi.fn(async () => undefined),
  connect: vi.fn(),
}));

const socketMocks = vi.hoisted(() => {
  const createSocket = () => {
    const handlers = new Map<string, Array<(payload?: unknown) => void>>();
    return {
      on: vi.fn((event: string, handler: (payload?: unknown) => void) => {
        const existing = handlers.get(event) ?? [];
        handlers.set(event, [...existing, handler]);
      }),
      disconnect: vi.fn(),
      emitWithAck: vi.fn(),
      trigger(event: string, payload?: unknown) {
        for (const handler of handlers.get(event) ?? []) {
          handler(payload);
        }
      },
    };
  };

  return {
    createSocket,
    socket: null as any,
    io: vi.fn(),
  };
});

vi.mock("./wave8-client", async () => {
  const actual = await vi.importActual<typeof import("./wave8-client")>("./wave8-client");
  return {
    ...actual,
    loadServerUrl: runtimeMocks.loadServerUrl,
    loadStoredCredentials: runtimeMocks.loadStoredCredentials,
    saveStoredCredentials: runtimeMocks.saveStoredCredentials,
    clearStoredCredentials: runtimeMocks.clearStoredCredentials,
    saveServerUrl: runtimeMocks.saveServerUrl,
    showDesktopNotification: runtimeMocks.showDesktopNotification,
    Wave8Client: {
      connect: runtimeMocks.connect,
    },
  };
});

vi.mock("socket.io-client", () => ({
  io: socketMocks.io,
}));

import { useWave8Desktop } from "./useWave8Desktop";

type HookState = ReturnType<typeof useWave8Desktop>;

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

function buildSession(updatedAt = 200) {
  return {
    id: "session-1",
    seq: 1,
    createdAt: 100,
    updatedAt,
    active: true,
    activeAt: updatedAt,
    metadata: {
      path: "/root/vibe-remote",
      host: "desktop",
      name: "Rewrite Session",
      currentModelCode: "gpt-5.4",
      flavor: "wave9-tauri",
    },
    metadataVersion: 1,
    agentState: null,
    agentStateVersion: 1,
    dataEncryptionKey: "data-key-1",
  };
}

function buildMessage(id: string, createdAt: number, text: string) {
  return {
    id,
    localId: null,
    createdAt,
    role: "assistant" as const,
    title: "Assistant",
    text,
    rawType: "agent:assistant",
  };
}

function buildAccountSettings() {
  return {
    schemaVersion: 2,
    viewInline: false,
    inferenceOpenAIKey: null,
    expandTodos: true,
    showLineNumbers: true,
    showLineNumbersInToolViews: true,
    wrapLinesInDiffs: false,
    analyticsOptOut: false,
    experiments: false,
    alwaysShowContextSize: false,
    agentInputEnterToSend: true,
    avatarStyle: "brutalist",
    showFlavorIcons: false,
    compactSessionView: false,
    hideInactiveSessions: false,
    expResumeSession: false,
    reviewPromptAnswered: false,
    reviewPromptLikedApp: null,
    voiceAssistantLanguage: null,
    voiceCustomAgentId: null,
    voiceBypassToken: false,
    preferredLanguage: null,
    recentMachinePaths: [],
    lastUsedAgent: null,
    lastUsedPermissionMode: null,
    lastUsedModelMode: null,
    dismissedCLIWarnings: { perMachine: {}, global: {} },
  };
}

function createMockClient() {
  const initialState = {
    items: [buildMessage("m1", 101, "Initial backend message")],
    loadedAt: 1000,
    lastSeq: 1,
  };
  const sentState = {
    items: [
      buildMessage("m1", 101, "Initial backend message"),
      buildMessage("m2", 102, "Post-send backend reply"),
    ],
    loadedAt: 2000,
    lastSeq: 2,
  };
  const refreshedState = {
    items: [
      buildMessage("m1", 101, "Initial backend message"),
      buildMessage("m2", 102, "Post-send backend reply"),
      buildMessage("m3", 103, "Realtime follow-up"),
    ],
    loadedAt: 3000,
    lastSeq: 3,
  };

  const states = [initialState, sentState, refreshedState];
  const session = buildSession();
  const updatedSession = {
    ...session,
    seq: 4,
    updatedAt: 400,
    metadata: {
      ...session.metadata,
      name: "Renamed Session",
    },
  };

  return {
    session,
    updatedSession,
    listMessagesStates: {
      initialState,
      sentState,
      refreshedState,
    },
    client: {
      fetchProfile: vi.fn(async () => ({
        id: "user-1",
        firstName: "Avery",
        lastName: "Stone",
        username: "avery",
        connectedServices: ["anthropic"],
      })),
      fetchAccountSettings: vi.fn(async () => ({
        settings: buildAccountSettings(),
        version: 1,
      })),
      listSessions: vi.fn(async () => [buildSession()]),
      listArtifacts: vi.fn(async () => []),
      listMachines: vi.fn(async () => []),
      listFriends: vi.fn(async () => []),
      listFeed: vi.fn(async () => []),
      fetchMachine: vi.fn(async () => null),
      fetchUserProfile: vi.fn(async () => null),
      fetchArtifact: vi.fn(async () => null),
      updateAccountSettings: vi.fn(async () => ({
        settings: buildAccountSettings(),
        version: 2,
      })),
      queryUsage: vi.fn(async () => ({
        usage: [],
        groupBy: "day" as const,
        totalReports: 0,
      })),
      listMessages: vi.fn(async () => states.shift() ?? refreshedState),
      createSession: vi.fn(async () => buildSession(500)),
      createArtifact: vi.fn(),
      updateArtifact: vi.fn(),
      deleteArtifact: vi.fn(),
      sendMessage: vi.fn(async () => undefined),
      decodeMessage: vi.fn(async () => [buildMessage("m3", 103, "Realtime follow-up")]),
      applySessionUpdate: vi.fn(async () => updatedSession),
      encryptSessionRpcPayload: vi.fn(async (_sid: string, _dek: string | null, payload: object) =>
        JSON.stringify(payload),
      ),
      decryptSessionRpcPayload: vi.fn(async (_sid: string, _dek: string | null, payload: string) => {
        switch (payload) {
          case "rpc-1:session-1:bash":
            return {
              success: true,
              stdout: [
                "# branch.oid abcdef1234567890abcdef1234567890abcdef12",
                "# branch.head main",
                "1 .M N... 100644 100644 100644 abcdef1 abcdef1 src/App.tsx",
                "? docs/todo.md",
              ].join("\n"),
              stderr: "",
              exitCode: 0,
            };
          case "rpc-2:session-1:bash":
            return {
              success: true,
              stdout: "2\t1\tsrc/App.tsx\n---STAGED---\n",
              stderr: "",
              exitCode: 0,
            };
          case "rpc-3:session-1:readFile":
            return {
              success: true,
              content: Buffer.from("export const demo = 1;\n").toString("base64"),
            };
          case "rpc-4:session-1:bash":
            return {
              success: true,
              stdout:
                "diff --git a/src/App.tsx b/src/App.tsx\n--- a/src/App.tsx\n+++ b/src/App.tsx\n@@ -1 +1 @@\n-old\n+new\n",
              stderr: "",
              exitCode: 0,
            };
          case "rpc-5:session-1:abort":
            return {
              success: true,
            };
          default:
            throw new Error(`Unexpected RPC payload ${payload}`);
        }
      }),
    },
  };
}

function HookProbe({
  activeSessionId,
  onValue,
}: {
  activeSessionId?: string | null;
  onValue: (value: HookState) => void;
}) {
  onValue(useWave8Desktop(activeSessionId));
  return null;
}

async function flushPromises() {
  await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 0));
}

async function waitFor(
  predicate: () => boolean,
  label: string,
) {
  for (let attempt = 0; attempt < 30; attempt += 1) {
    if (predicate()) {
      return;
    }
    await act(async () => {
      await flushPromises();
    });
  }

  throw new Error(`Timed out waiting for ${label}`);
}

describe("useWave8Desktop", () => {
  let renderer: ReactTestRenderer | null = null;
  let latest: HookState | null = null;

  beforeEach(() => {
    latest = null;
    socketMocks.socket = socketMocks.createSocket();
    socketMocks.io.mockReset();
    socketMocks.io.mockReturnValue(socketMocks.socket);
    runtimeMocks.loadServerUrl.mockReset();
    runtimeMocks.loadServerUrl.mockReturnValue("https://api.cluster-fluster.com");
    runtimeMocks.loadStoredCredentials.mockReset();
    runtimeMocks.loadStoredCredentials.mockResolvedValue({
      token: "token-1",
      secret: "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA",
    });
    runtimeMocks.saveStoredCredentials.mockClear();
    runtimeMocks.clearStoredCredentials.mockClear();
    runtimeMocks.saveServerUrl.mockClear();
    runtimeMocks.showDesktopNotification.mockClear();
  });

  afterEach(async () => {
    if (renderer) {
      await act(async () => {
        renderer?.unmount();
        await flushPromises();
      });
    }
    renderer = null;
    latest = null;
    vi.clearAllMocks();
  });

  it("boots authenticated state and completes the session runtime chain", async () => {
    const mock = createMockClient();
    runtimeMocks.connect.mockResolvedValue(mock.client);

    await act(async () => {
      renderer = create(
        <HookProbe activeSessionId="session-1" onValue={(value) => { latest = value; }} />,
      );
      await flushPromises();
    });

    await waitFor(
      () => latest?.status === "ready" && latest?.sessions.length === 1,
      "authenticated bootstrap",
    );

    expect(runtimeMocks.connect).toHaveBeenCalledWith(
      "https://api.cluster-fluster.com",
      expect.objectContaining({ token: "token-1" }),
    );
    expect(latest?.sessionSummaries[0]?.title).toBe("vibe-remote");

    const desktop = latest;
    if (!desktop) {
      throw new Error("Hook state did not initialize");
    }

    await act(async () => {
      await desktop.loadMessages("session-1");
      await flushPromises();
    });

    expect(mock.client.listMessages).toHaveBeenCalledTimes(1);
    expect(latest?.sessionState["session-1"]).toMatchObject({
      lastSeq: 1,
      loading: false,
      error: null,
    });
    expect(latest?.sessionState["session-1"]?.items.map((message) => message.id)).toEqual(["m1"]);

    await act(async () => {
      await desktop.sendMessage("session-1", "Ship the rewrite", {
        permissionMode: "plan",
        model: "gpt-5.4",
      });
      await flushPromises();
    });

    expect(mock.client.sendMessage).toHaveBeenCalledWith(
      "session-1",
      "Ship the rewrite",
      "data-key-1",
      {
        permissionMode: "plan",
        model: "gpt-5.4",
      },
    );
    expect(mock.client.listSessions).toHaveBeenCalledTimes(2);
    expect(mock.client.listMessages).toHaveBeenCalledTimes(2);
    expect(latest?.sessionState["session-1"]?.items.map((message) => message.id)).toEqual([
      "m1",
      "m2",
    ]);

    let rpcCount = 0;
    socketMocks.socket.emitWithAck.mockImplementation(
      async (_event: string, payload: { method: string }) => ({
        ok: true,
        result: `rpc-${++rpcCount}:${payload.method}`,
      }),
    );

    const inventory = await desktop.loadSessionFiles("session-1");
    expect(inventory).toMatchObject({
      branch: "main",
      totalStaged: 0,
      totalUnstaged: 2,
    });
    expect(inventory?.files.map((file) => file.relativePath)).toEqual([
      "src/App.tsx",
      "docs/todo.md",
    ]);

    const file = await desktop.readSessionFile("session-1", "src/App.tsx");
    expect(file).toMatchObject({
      relativePath: "src/App.tsx",
      isBinary: false,
      language: "typescript",
    });
    expect(file?.content).toContain("export const demo = 1;");
    expect(file?.diff).toContain("+new");

    rpcCount = 4;
    socketMocks.socket.emitWithAck.mockImplementation(
      async (_event: string, payload: { method: string }) => ({
        ok: true,
        result: `rpc-${++rpcCount}:${payload.method}`,
      }),
    );

    await act(async () => {
      await desktop.abortSession("session-1");
      await flushPromises();
    });

    expect(latest?.sessionState["session-1"]?.aborting).toBe(false);

    await act(async () => {
      socketMocks.socket.trigger("update", {
        id: "update-1",
        seq: 3,
        createdAt: 300,
        body: {
          t: "new-message",
          sid: "session-1",
          message: {
            id: "remote-3",
            seq: 3,
            localId: null,
            content: {
              c: "ciphertext",
              t: "encrypted",
            },
            createdAt: 103,
            updatedAt: 103,
          },
        },
      });
      await flushPromises();
    });

    await waitFor(
      () => latest?.sessionState["session-1"]?.lastSeq === 3,
      "realtime message append",
    );
    expect(mock.client.decodeMessage).toHaveBeenCalledWith(
      "session-1",
      "data-key-1",
      expect.objectContaining({ id: "remote-3", seq: 3 }),
    );
    expect(latest?.sessionState["session-1"]?.items.map((message) => message.id)).toEqual([
      "m1",
      "m2",
      "m3",
    ]);

    await act(async () => {
      socketMocks.socket.trigger("update", {
        id: "update-2",
        seq: 4,
        createdAt: 400,
        body: {
          t: "update-session",
          id: "session-1",
          metadata: {
            value: "encrypted-metadata",
            version: 2,
          },
          agentState: null,
        },
      });
      await flushPromises();
    });

    await waitFor(
      () => latest?.sessions[0]?.metadata?.name === "Renamed Session",
      "session metadata refresh",
    );
    expect(mock.client.applySessionUpdate).toHaveBeenCalledWith(
      expect.objectContaining({ id: "session-1" }),
      expect.objectContaining({
        metadata: {
          value: "encrypted-metadata",
          version: 2,
        },
        seq: 4,
        updatedAt: 400,
      }),
    );
  });

  it("surfaces invalid realtime payloads as a global error", async () => {
    const mock = createMockClient();
    runtimeMocks.connect.mockResolvedValue(mock.client);

    await act(async () => {
      renderer = create(
        <HookProbe activeSessionId="session-1" onValue={(value) => { latest = value; }} />,
      );
      await flushPromises();
    });

    await waitFor(
      () => latest?.status === "ready",
      "authenticated bootstrap",
    );

    await act(async () => {
      socketMocks.socket.trigger("update", {
        id: "broken-update",
        seq: 5,
        createdAt: 500,
        body: {
          t: "new-message",
          sid: "session-1",
          message: {
            id: "bad-message",
            seq: "broken",
          },
        },
      });
      await flushPromises();
    });

    expect(latest?.globalError).toBe("Received an invalid realtime update payload");
  });
});
