import { renderToStaticMarkup } from "react-dom/server";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { DesktopArtifact } from "./wave8-client";

const mockDesktopState = vi.hoisted(() => ({
  value: null as any,
}));

vi.mock("./useWave8Desktop", () => ({
  useWave8Desktop: () => mockDesktopState.value,
}));

import { DesktopShell } from "./App";

function createDesktopState() {
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
        },
        metadataVersion: 1,
        agentState: null,
        agentStateVersion: 1,
        dataEncryptionKey: null,
      },
    ],
    artifacts: [
      {
        id: "artifact-1",
        title: "Release Notes",
        body: "Desktop artifact body",
        sessions: ["session-1"],
        draft: false,
        headerVersion: 1,
        bodyVersion: 1,
        seq: 1,
        createdAt: 1,
        updatedAt: 2,
        isDecrypted: true,
      },
    ] as DesktopArtifact[],
    machines: [
      {
        id: "machine-1",
        seq: 1,
        createdAt: 1,
        updatedAt: 2,
        active: true,
        activeAt: 2,
        metadata: {
          host: "desktop",
          displayName: "Desktop Machine",
          platform: "Linux",
          homeDir: "/root",
        },
        metadataVersion: 1,
        daemonState: null,
        daemonStateVersion: 1,
      },
    ],
    userProfiles: {},
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
    sessionState: {},
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
    loadMachine: vi.fn(async () => null),
    loadUserProfile: vi.fn(async () => null),
    loadArtifact: vi.fn(async () => null),
    loadUsage: vi.fn(async () => ({
      usage: [
        {
          timestamp: 1_710_000_000,
          tokens: { "gpt-5.4": 1200 },
          cost: { "gpt-5.4": 0.42 },
          reportCount: 3,
        },
      ],
      groupBy: "day",
      totalReports: 3,
    })),
    loadMessages: vi.fn(async () => null),
    loadSessionFiles: vi.fn(async () => ({
      branch: "main",
      files: [],
      totalStaged: 0,
      totalUnstaged: 0,
    })),
    readSessionFile: vi.fn(async () => ({
      relativePath: "src/App.tsx",
      absolutePath: "/root/vibe-remote/src/App.tsx",
      content: "demo",
      diff: null,
      isBinary: false,
      language: "typescript",
    })),
    createSession: vi.fn(),
    createArtifact: vi.fn(),
    updateArtifact: vi.fn(),
    deleteArtifact: vi.fn(),
    sendMessage: vi.fn(),
    logout: vi.fn(),
    updateServerUrl: vi.fn(),
    retryStoredSession: vi.fn(),
    storedSessionAvailable: true,
  };
}

describe("DesktopShell authenticated routes", () => {
  beforeEach(() => {
    mockDesktopState.value = createDesktopState();
  });

  it("renders an authenticated artifact detail route with desktop save controls", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/artifacts/artifact-1"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Artifact detail");
    expect(html).toContain("Save body to file");
    expect(html).toContain("Desktop artifact body");
  });

  it("keeps artifact export disabled while the body is still loading", () => {
    const state = createDesktopState();
    const { body: _ignoredBody, ...artifactWithoutBody } = state.artifacts[0];
    state.artifacts = [
      artifactWithoutBody,
    ];
    mockDesktopState.value = state;

    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/artifacts/artifact-1"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Loading body...");
    expect(html).toContain("disabled");
  });

  it("keeps artifact export disabled when the body is present but not decryptable", () => {
    const state = createDesktopState();
    state.artifacts = [
      {
        ...state.artifacts[0],
        body: null,
        isDecrypted: false,
      },
    ];
    mockDesktopState.value = state;

    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/artifacts/artifact-1"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Encrypted body");
    expect(html).toContain("disabled");
  });

  it("renders an authenticated session file deep link without the empty-state fallback", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/session/session-1/file?path=src%2FApp.tsx"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Session file");
    expect(html).toContain("App.tsx");
    expect(html).not.toContain("Open the live files inventory first");
  });

  it("renders the Claude connect route with an explicit terminal handoff command", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/settings/connect/claude"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("vibe connect claude");
    expect(html).toContain("Copy terminal command");
    expect(html).toContain("Connected status");
  });

  it("renders the terminal connect route with live approval controls", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/terminal/connect?key=demo-terminal-key"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Approve terminal connection");
    expect(html).toContain("demo-terminal-key");
    expect(html).toContain("Terminal auth URL or public key");
  });

  it("renders the voice route with an explicit custom agent save action", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/settings/voice"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Custom agent ID");
    expect(html).toContain("Save custom agent ID");
    expect(html).not.toContain("value=\"agent-");
  });

  it("renders the usage route with live backend usage sections", () => {
    const html = renderToStaticMarkup(
      <DesktopShell
        path="/(app)/settings/usage"
        commandOpen={false}
        onNavigate={() => undefined}
        onCommandOpen={() => undefined}
        onCommandClose={() => undefined}
      />,
    );

    expect(html).toContain("Live query");
    expect(html).toContain("Top models");
    expect(html).toContain("/v1/usage/query");
  });
});
