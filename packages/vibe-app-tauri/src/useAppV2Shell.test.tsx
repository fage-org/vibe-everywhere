import { act, create, type ReactTestRenderer } from "react-test-renderer";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useAppV2Shell, resolveAppV2View } from "./useAppV2Shell";
import type { AppShellState } from "./useAppShellState";
import type { ResolvedRoute } from "./router";

type HookValue = ReturnType<typeof useAppV2Shell>;

function createShellState(): AppShellState {
  return {
    status: "ready",
    serverUrl: "https://api.cluster-fluster.com",
    credentials: { token: "token", secret: "secret" },
    profile: {
      id: "user-1",
      firstName: "Avery",
      lastName: "Stone",
      username: "avery",
      connectedServices: ["anthropic"],
    },
    sessions: [
      {
        id: "session-1",
        seq: 1,
        createdAt: 1,
        updatedAt: 2,
        active: true,
        activeAt: 2,
        metadata: {
          name: "Demo Session",
          path: "/root/vibe-remote",
          host: "desktop",
          flavor: "codex",
          currentModelCode: "gpt-5.4",
          models: [
            { code: "gpt-5.4", value: "gpt-5.4", description: "default" },
          ],
        } as any,
        metadataVersion: 1,
        agentState: null,
        agentStateVersion: 1,
        dataEncryptionKey: null,
      },
    ],
    artifacts: [],
    machines: [],
    userProfiles: {},
    friends: [
      {
        id: "friend-1",
        firstName: "Sam",
        lastName: null,
        username: "sam",
        avatar: null,
        bio: null,
        status: "friend",
      },
    ],
    feedItems: [
      {
        id: "feed-1",
        body: { kind: "friend_request", uid: "friend-1" } as any,
        repeatKey: "friend_request_friend-1",
        cursor: "0-2",
        createdAt: 2,
      },
    ],
    sessionSummaries: [
      {
        session: {
          id: "session-1",
          seq: 1,
          createdAt: 1,
          updatedAt: 2,
          active: true,
          activeAt: 2,
          metadata: {
            name: "Demo Session",
            path: "/root/vibe-remote",
            host: "desktop",
            flavor: "codex",
            currentModelCode: "gpt-5.4",
            models: [
              { code: "gpt-5.4", value: "gpt-5.4", description: "default" },
            ],
          } as any,
          metadataVersion: 1,
          agentState: null,
          agentStateVersion: 1,
          dataEncryptionKey: null,
        },
        title: "Demo Session",
        subtitle: "desktop",
        detail: "Live session",
      },
    ],
    sessionState: {
      "session-1": {
        items: [
          {
            id: "message-1",
            localId: null,
            createdAt: 1,
            role: "assistant",
            title: "Assistant",
            text: "Inspect the live diff",
            rawType: "agent:assistant",
          },
        ],
        loading: false,
        sending: false,
        aborting: false,
        error: null,
        loadedAt: 1,
        lastSeq: 1,
      },
    },
    globalError: null,
    linkState: {
      status: "idle",
      linkUrl: null,
      qrSvg: null,
      error: null,
      attemptId: null,
      browserUrl: null,
    },
    backupKey: "backup-key",
    createFreshAccount: vi.fn(),
    restoreWithSecret: vi.fn(),
    startMobileLink: vi.fn(),
    cancelMobileLink: vi.fn(),
    refreshSessions: vi.fn(async () => []),
    refreshArtifacts: vi.fn(async () => []),
    refreshMachines: vi.fn(async () => []),
    refreshFriends: vi.fn(async () => []),
    refreshFeed: vi.fn(async () => []),
    loadMachine: vi.fn(async () => null),
    loadUserProfile: vi.fn(async () => null),
    loadArtifact: vi.fn(async () => null),
    loadUsage: vi.fn(async () => ({ usage: [], groupBy: "day", totalReports: 0 })),
    loadMessages: vi.fn(async () => null),
    loadSessionFiles: vi.fn(async () => ({ branch: "main", files: [], totalStaged: 0, totalUnstaged: 0 })),
    readSessionFile: vi.fn(async () => null),
    createSession: vi.fn(async () => ({ id: "session-2" })),
    createArtifact: vi.fn(),
    updateArtifact: vi.fn(),
    deleteArtifact: vi.fn(),
    deleteSession: vi.fn(),
    sendMessage: vi.fn(async () => undefined),
    abortSession: vi.fn(async () => undefined),
    logout: vi.fn(async () => undefined),
    updateServerUrl: vi.fn(async () => undefined),
    retryStoredSession: vi.fn(async () => undefined),
    storedSessionAvailable: true,
  } as unknown as AppShellState;
}

function createResolvedRoute(
  key: string,
  section: string,
  params: Record<string, string> = {},
): ResolvedRoute {
  return {
    definition: {
      key,
      label: key,
      title: key,
      pattern: "/(app)/index",
      examplePath: "/(app)/index",
      summary: key,
      promotionClass: "P0",
      ownerModule: "desktop-shell-and-routing" as any,
      section: section as any,
      status: "wired",
    },
    params,
    canonicalPath: "/(app)/index",
    searchParams: new URLSearchParams(),
  };
}

function HookProbe({
  shell,
  resolved,
  navigate,
  onValue,
}: {
  shell: AppShellState;
  resolved: ResolvedRoute;
  navigate: (path: string) => void;
  onValue: (value: HookValue) => void;
}) {
  onValue(useAppV2Shell(shell, resolved, navigate));
  return null;
}

describe("useAppV2Shell", () => {
  let renderer: ReactTestRenderer | null = null;

  afterEach(async () => {
    if (renderer) {
      await act(async () => {
        renderer?.unmount();
      });
    }
    renderer = null;
  });

  it("maps route keys into AppV2 views", () => {
    expect(resolveAppV2View(createResolvedRoute("home", "Landing"))).toBe("home");
    expect(resolveAppV2View(createResolvedRoute("new-session", "Session"))).toBe("new-session");
    expect(resolveAppV2View(createResolvedRoute("session-detail", "Session", { id: "session-1" }))).toBe("session");
    expect(resolveAppV2View(createResolvedRoute("settings-index", "Settings"))).toBe("settings");
    expect(resolveAppV2View(createResolvedRoute("inbox", "Session"))).toBe("inbox");
    expect(resolveAppV2View(createResolvedRoute("artifacts-index", "Artifacts"))).toBe("unsupported");
  });

  it("routes home actions through canonical app paths", async () => {
    const shell = createShellState();
    const navigate = vi.fn();
    let latest!: HookValue;

    await act(async () => {
      renderer = create(
        <HookProbe
          shell={shell}
          resolved={createResolvedRoute("home", "Landing")}
          navigate={navigate}
          onValue={(value) => { latest = value; }}
        />,
      );
    });

    latest.startNewSession();
    latest.resumeLatestSession();

    expect(navigate).toHaveBeenNthCalledWith(1, "/(app)/new/index");
    expect(navigate).toHaveBeenNthCalledWith(2, "/(app)/session/session-1");
  });

  it("derives the current session and loads messages from route state", async () => {
    const shell = createShellState();
    const navigate = vi.fn();
    let latest!: HookValue;

    await act(async () => {
      renderer = create(
        <HookProbe
          shell={shell}
          resolved={createResolvedRoute("session-detail", "Session", { id: "session-1" })}
          navigate={navigate}
          onValue={(value) => { latest = value; }}
        />,
      );
    });

    expect(shell.loadMessages).toHaveBeenCalledWith("session-1");
    expect(latest.currentSession?.id).toBe("session-1");
    expect(latest.messages[0]?.content).toBe("Inspect the live diff");
    expect(latest.sessions[0]?.metadata?.model).toBe("gpt-5.4");
    expect(latest.unreadCount).toBeUndefined();
  });
});
