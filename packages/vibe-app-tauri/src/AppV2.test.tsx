import { act, create } from "react-test-renderer";
import { afterEach, describe, expect, it, vi } from "vitest";

const appV2Mocks = vi.hoisted(() => ({
  shell: null as any,
  router: null as any,
  profile: {
    appEnv: "development",
    devHost: "0.0.0.0",
    devPort: 1420,
    mode: "test-mobile",
    outDir: "dist/mobile",
    runtimeTarget: "mobile",
    surfaceKey: "mobileAndroid",
  },
}));

vi.mock("./useAppShellState", () => ({
  useAppShellState: () => appV2Mocks.shell,
}));

vi.mock("./router", async () => {
  const actual = await vi.importActual<typeof import("./router")>("./router");
  return {
    ...actual,
    useDesktopRouter: () => appV2Mocks.router,
  };
});

vi.mock("../sources/app/providers/RuntimeBootstrapProvider", () => ({
  useRuntimeBootstrapProfile: () => appV2Mocks.profile,
}));

import { AppV2 } from "./AppV2";

(
  globalThis as typeof globalThis & {
    IS_REACT_ACT_ENVIRONMENT?: boolean;
    window?: Window & typeof globalThis;
    document?: Document;
  }
).IS_REACT_ACT_ENVIRONMENT = true;

function createShellState() {
  return {
    status: "ready" as const,
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
        },
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
        body: { kind: "friend_request", uid: "friend-1" },
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
          },
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
    createSession: vi.fn(async () => ({
      id: "session-2",
      seq: 1,
      createdAt: 1,
      updatedAt: 2,
      active: true,
      activeAt: 2,
      metadata: {
        name: "New Session",
        path: "/root/vibe-remote",
        host: "desktop",
        flavor: "codex",
        currentModelCode: "gpt-5.4",
        models: [{ code: "gpt-5.4", value: "gpt-5.4", description: "default" }],
      },
      metadataVersion: 1,
      agentState: null,
      agentStateVersion: 1,
      dataEncryptionKey: null,
    })),
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
  };
}

function nodeHasText(node: any, text: string): boolean {
  if (typeof node === "string") {
    return node.includes(text);
  }

  if (Array.isArray(node)) {
    return node.some((child) => nodeHasText(child, text));
  }

  if (node && typeof node === "object" && "children" in node) {
    return nodeHasText((node as { children?: unknown }).children, text);
  }

  return false;
}

function testNodeText(node: any): string {
  if (typeof node === "string") {
    return node;
  }

  if (Array.isArray(node)) {
    return node.map((child) => testNodeText(child)).join(" ");
  }

  if (node && typeof node === "object" && "children" in node) {
    return testNodeText((node as { children?: unknown }).children);
  }

  return "";
}

function installAppV2BrowserStubs() {
  const store = new Map<string, string>();
  vi.stubGlobal("document", {
    documentElement: {
      setAttribute: vi.fn(),
      style: {},
    },
  });
  vi.stubGlobal("window", {
    innerWidth: 390,
    matchMedia: vi.fn(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
    localStorage: {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => {
        store.set(key, value);
      },
      removeItem: (key: string) => {
        store.delete(key);
      },
    },
  });

  return store;
}

function createRouter(path: string) {
  const routeMap: Record<string, { key: string; section: string; params?: Record<string, string> }> = {
    "/(app)/index": { key: "home", section: "Landing" },
    "/(app)/new/index": { key: "new-session", section: "Session" },
    "/(app)/session/recent": { key: "session-recent", section: "Session" },
    "/(app)/session/session-1": { key: "session-detail", section: "Session", params: { id: "session-1" } },
    "/(app)/inbox/index": { key: "inbox", section: "Session" },
    "/(app)/settings/index": { key: "settings-index", section: "Settings" },
    "/(app)/artifacts/index": { key: "artifacts-index", section: "Artifacts" },
  };
  const current = routeMap[path];
  if (!current) {
    throw new Error(`Unhandled test path: ${path}`);
  }

  return {
    path,
    navigate: vi.fn(),
    resolved: {
      definition: {
        key: current.key,
        label: current.key,
        title: current.key,
        pattern: path,
        examplePath: path,
        summary: current.key,
        promotionClass: "P0",
        ownerModule: "desktop-shell-and-routing",
        section: current.section,
        status: "wired",
      },
      params: current.params ?? {},
      canonicalPath: path,
      searchParams: new URLSearchParams(),
    },
  };
}

describe("AppV2", () => {
  let renderer: any = null;

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  afterEach(async () => {
    if (renderer) {
      await act(async () => {
        renderer?.unmount();
      });
    }
    renderer = null;
    vi.clearAllMocks();
  });

  it("routes the home primary action into the new-session view instead of calling createSession directly", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/index");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const mountedRenderer = renderer!;
    const newSessionButton = mountedRenderer.root.find(
      (node: any) => node.type === "button" && testNodeText(node.children).includes("Start New Session"),
    );

    await act(async () => {
      newSessionButton.props.onClick();
    });

    expect(router.navigate).toHaveBeenCalledWith("/(app)/new/index");
    expect(shell.createSession).not.toHaveBeenCalled();
  });

  it("routes the home resume action to the latest session", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/index");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const mountedRenderer = renderer!;
    const resumeCard = mountedRenderer.root.find(
      (node: any) =>
        node.props?.style?.cursor === "pointer"
        && testNodeText(node.children).includes("Resume Session"),
    );

    await act(async () => {
      resumeCard.props.onClick();
    });

    expect(router.navigate).toHaveBeenCalledWith("/(app)/session/session-1");
  });

  it("renders the session route from real sessionState messages", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/session/session-1");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    expect(shell.loadMessages).toHaveBeenCalledWith("session-1");
    const mountedRenderer = renderer!;
    expect(JSON.stringify(mountedRenderer.toJSON())).toContain("Inspect the live diff");
    expect(JSON.stringify(mountedRenderer.toJSON())).toContain("vibe-remote");
  });

  it("renders inbox from feedItems without fake unread controls", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/inbox/index");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const mountedRenderer = renderer!;
    const snapshot = JSON.stringify(mountedRenderer.toJSON());
    expect(snapshot).toContain("Friend request from Sam");
    expect(snapshot).toContain("Open @sam to review the incoming social request.");
    expect(snapshot).not.toContain("Mark all as read");
    expect(snapshot).not.toContain("1 unread");
  });

  it("renders an explicit unsupported surface for non-productized routes", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/artifacts/index");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const snapshot = JSON.stringify(renderer!.toJSON());
    expect(snapshot).toContain("Route not yet productized");
    expect(snapshot).toContain("active Wave 10 surface");
  });

  it("renders settings as immediate persistence instead of a fake save workflow", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/settings/index");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const snapshot = JSON.stringify(renderer!.toJSON());
    expect(snapshot).toContain("Settings in the AppV2 shell persist immediately when changed.");
    expect(snapshot).not.toContain("Save Changes");
  });

  it("routes the home view-all action to recent sessions", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/index");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const mountedRenderer = renderer!;
    const viewAllButton = mountedRenderer.root.find(
      (node: any) => node.type === "button" && testNodeText(node.children).includes("View All"),
    );

    await act(async () => {
      viewAllButton.props.onClick();
    });

    expect(router.navigate).toHaveBeenCalledWith("/(app)/session/recent");
  });

  it("restores the session draft for the active AppV2 session route", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/session/session-1");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    const storage = installAppV2BrowserStubs();
    storage.set(
      "vibe-app-tauri.session-draft.session-1",
      JSON.stringify({ value: "Resume the AppV2 rollout" }),
    );

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const snapshot = JSON.stringify(renderer!.toJSON());
    expect(snapshot).toContain("Resume the AppV2 rollout");
  });

  it("sends a message through the active session and clears the stored draft", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/session/session-1");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    const storage = installAppV2BrowserStubs();
    storage.set(
      "vibe-app-tauri.session-draft.session-1",
      JSON.stringify({ value: "Ship the AppV2 session flow" }),
    );

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const mountedRenderer = renderer!;
    const composerInput = mountedRenderer.root.find(
      (node: any) => node.type === "textarea" && node.props.placeholder?.includes("Message"),
    );

    await act(async () => {
      composerInput.props.onChange({
        target: { value: "Ship the AppV2 session flow", scrollHeight: 24, style: {} },
      });
    });

    const sendButton = mountedRenderer.root.find(
      (node: any) =>
        node.type === "button"
        && node.props.style?.width === "32px"
        && node.props.style?.height === "32px",
    );

    await act(async () => {
      await sendButton.props.onClick();
    });

    expect(shell.sendMessage).toHaveBeenCalledWith("session-1", "Ship the AppV2 session flow", {
      model: "gpt-5.4",
    });
    expect(storage.get("vibe-app-tauri.session-draft.session-1")).toBeUndefined();
  });

  it("applies a changed model selection to the next outbound message", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/session/session-1");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const mountedRenderer = renderer!;
    const modelToggle = mountedRenderer.root.find(
      (node: any) =>
        node.type === "button"
        && testNodeText(node.children).includes("gpt-5.4"),
    );

    await act(async () => {
      modelToggle.props.onClick();
    });

    const modelOptions = mountedRenderer.root.findAll(
      (node: any) =>
        node.type === "button"
        && testNodeText(node.children).includes("gpt-5.4 - default"),
    );
    const modelOption = modelOptions.at(-1);

    await act(async () => {
      modelOption?.props.onClick();
    });

    const composerInput = mountedRenderer.root.find(
      (node: any) => node.type === "textarea" && node.props.placeholder?.includes("Message"),
    );

    await act(async () => {
      composerInput.props.onChange({
        target: { value: "Send with the selected model", scrollHeight: 24, style: {} },
      });
    });

    const sendButton = mountedRenderer.root.find(
      (node: any) =>
        node.type === "button"
        && node.props.style?.width === "32px"
        && node.props.style?.height === "32px",
    );

    await act(async () => {
      await sendButton.props.onClick();
    });

    expect(shell.sendMessage).toHaveBeenCalledWith("session-1", "Send with the selected model", {
      model: "gpt-5.4",
    });
  });

  it("renders session composer capability limits explicitly instead of fake attachment buttons", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/session/session-1");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const snapshot = JSON.stringify(renderer!.toJSON());
    expect(snapshot).toContain("Attachments and media insertion are not yet available in the AppV2 session shell.");
    expect(snapshot).not.toContain("📎");
    expect(snapshot).not.toContain("🖼️");
  });

  it("marks the sessions tab active for the recent-sessions route in mobile shell", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/session/recent");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="mobile" />);
    });

    const snapshot = JSON.stringify(renderer!.toJSON());
    expect(snapshot).toContain("\"Sessions\"");
    expect(snapshot).toContain("var(--color-primary)");
  });

  it("shows the desktop shell as connected when status is ready", async () => {
    const shell = createShellState();
    const router = createRouter("/(app)/settings/index");
    appV2Mocks.shell = shell;
    appV2Mocks.router = router;
    installAppV2BrowserStubs();
    (window as typeof window).innerWidth = 1280;

    await act(async () => {
      renderer = create(<AppV2 runtimeTarget="desktop" />);
    });

    const snapshot = JSON.stringify(renderer!.toJSON());
    expect(snapshot).toContain("Connected");
    expect(snapshot).toContain("var(--color-success)");
  });
});
