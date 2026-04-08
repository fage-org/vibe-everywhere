import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  approveTerminalConnection,
  buildUsageQueryParams,
  createAccountLinkRequest,
  formatSecretKeyForBackup,
  normalizeServerUrl,
  normalizeSecretKey,
  parseRawRecordToUiMessages,
  Wave8Client,
} from "./wave8-client";

describe("wave8 client helpers", () => {
  const originalFetch = globalThis.fetch;

  beforeEach(() => {
    globalThis.fetch = vi.fn() as typeof fetch;
  });

  afterEach(() => {
    globalThis.fetch = originalFetch;
    vi.restoreAllMocks();
  });

  it("round-trips formatted backup keys", () => {
    const secret = "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA";
    const formatted = formatSecretKeyForBackup(secret);

    expect(normalizeSecretKey(formatted)).toBe(secret);
  });

  it("parses user text records into timeline messages", () => {
    const messages = parseRawRecordToUiMessages("m1", null, 1, {
      role: "user",
      content: {
        type: "text",
        text: "hello world",
      },
    });

    expect(messages).toHaveLength(1);
    expect(messages[0].role).toBe("user");
    expect(messages[0].text).toBe("hello world");
  });

  it("parses session text envelopes into assistant timeline entries", () => {
    const messages = parseRawRecordToUiMessages("m2", null, 1, {
      role: "session",
      content: {
        data: {
          role: "agent",
          ev: {
            t: "text",
            text: "session hello",
          },
        },
      },
    });

    expect(messages[0].role).toBe("assistant");
    expect(messages[0].text).toContain("session hello");
  });

  it("parses assistant tool content into structured tool messages", () => {
    const messages = parseRawRecordToUiMessages("m3", null, 1, {
      role: "agent",
      content: {
        type: "output",
        data: {
          type: "assistant",
          message: {
            content: [
              {
                type: "tool_use",
                name: "read_file",
                input: { path: "/tmp/demo.txt" },
              },
              {
                type: "tool_result",
                content: [{ type: "text", text: "file contents" }],
              },
            ],
          },
        },
      },
    });

    expect(messages).toHaveLength(2);
    expect(messages[0].role).toBe("tool");
    expect(messages[0].title).toContain("read_file");
    expect(messages[1].text).toContain("file contents");
  });

  it("allows https backends and loopback http backends only", () => {
    expect(normalizeServerUrl("https://api.cluster-fluster.com/")).toBe(
      "https://api.cluster-fluster.com",
    );
    expect(normalizeServerUrl("http://127.0.0.1:8787")).toBe(
      "http://127.0.0.1:8787",
    );
    expect(() => normalizeServerUrl("http://example.com")).toThrow(
      "Server URL must use https unless it targets localhost",
    );
  });

  it("builds hourly usage query params for today and daily params for longer periods", () => {
    const today = buildUsageQueryParams("today", "session-1");
    const sevenDays = buildUsageQueryParams("7days");

    expect(today.groupBy).toBe("hour");
    expect(today.sessionId).toBe("session-1");
    expect(sevenDays.groupBy).toBe("day");
    expect(sevenDays.endTime).toBeGreaterThan(sevenDays.startTime);
  });

  it("skips terminal approval when the auth request is already authorized", async () => {
    vi.mocked(globalThis.fetch).mockResolvedValueOnce(
      new Response(JSON.stringify({ status: "authorized", supportsV2: true }), {
        status: 200,
      }),
    );

    const linkRequest = await createAccountLinkRequest();
    const publicKeyBase64Url = linkRequest.linkUrl.slice("vibe:///account?".length);

    await approveTerminalConnection(
      "https://api.cluster-fluster.com",
      {
        token: "token-a",
        secret: "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA",
      },
      publicKeyBase64Url,
    );

    expect(globalThis.fetch).toHaveBeenCalledTimes(1);
    expect(vi.mocked(globalThis.fetch).mock.calls[0]?.[0]).toContain(
      "/v1/auth/request/status",
    );
  });

  it("posts a terminal approval response when the auth request is pending", async () => {
    vi.mocked(globalThis.fetch)
      .mockResolvedValueOnce(
        new Response(JSON.stringify({ status: "pending", supportsV2: false }), {
          status: 200,
        }),
      )
      .mockResolvedValueOnce(new Response("", { status: 200 }));

    const linkRequest = await createAccountLinkRequest();
    const publicKeyBase64Url = linkRequest.linkUrl.slice("vibe:///account?".length);

    await approveTerminalConnection(
      "https://api.cluster-fluster.com",
      {
        token: "token-b",
        secret: "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA",
      },
      publicKeyBase64Url,
    );

    expect(globalThis.fetch).toHaveBeenCalledTimes(2);
    expect(vi.mocked(globalThis.fetch).mock.calls[1]?.[0]).toBe(
      "https://api.cluster-fluster.com/v1/auth/response",
    );

    const requestInit = vi.mocked(globalThis.fetch).mock.calls[1]?.[1] as RequestInit;
    expect(requestInit.method).toBe("POST");
    expect(requestInit.headers).toEqual({
      Authorization: "Bearer token-b",
      "Content-Type": "application/json",
    });

    const payload = JSON.parse(String(requestInit.body)) as {
      publicKey: string;
      response: string;
    };
    expect(payload.publicKey).toBeTruthy();
    expect(payload.response).toBeTruthy();
  });

  it("fetches and decrypts account settings", async () => {
    const client = await Wave8Client.connect("https://api.cluster-fluster.com", {
      token: "token-c",
      secret: "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA",
    });

    vi.mocked(globalThis.fetch).mockResolvedValueOnce(
      new Response(JSON.stringify({ success: true, version: 1 }), { status: 200 }),
    );
    const encryptedSettings = await client.updateAccountSettings(
      {
        schemaVersion: 2,
        viewInline: false,
        inferenceOpenAIKey: null,
        expandTodos: true,
        showLineNumbers: true,
        showLineNumbersInToolViews: false,
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
      },
      0,
      { preferredLanguage: "ja", showFlavorIcons: true },
    ).then(() => {
      const postCall = vi.mocked(globalThis.fetch).mock.calls.at(-1)?.[1] as RequestInit;
      return JSON.parse(String(postCall.body)).settings as string;
    });

    vi.mocked(globalThis.fetch).mockReset();
    vi.mocked(globalThis.fetch).mockResolvedValueOnce(
      new Response(
        JSON.stringify({
          settings: encryptedSettings,
          settingsVersion: 2,
        }),
        { status: 200 },
      ),
    );

    const result = await client.fetchAccountSettings();
    expect(result.version).toBe(2);
    expect(result.settings.preferredLanguage).toBe("ja");
    expect(result.settings.showFlavorIcons).toBe(true);
  });

  it("decodes encrypted account settings payloads for realtime updates", async () => {
    const client = await Wave8Client.connect("https://api.cluster-fluster.com", {
      token: "token-c2",
      secret: "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA",
    });

    vi.mocked(globalThis.fetch).mockResolvedValueOnce(
      new Response(JSON.stringify({ success: true, version: 1 }), { status: 200 }),
    );
    const encryptedSettings = await client.updateAccountSettings(
      {
        schemaVersion: 2,
        viewInline: false,
        inferenceOpenAIKey: null,
        expandTodos: true,
        showLineNumbers: true,
        showLineNumbersInToolViews: false,
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
      },
      0,
      { preferredLanguage: "ja" },
    ).then(() => {
      const postCall = vi.mocked(globalThis.fetch).mock.calls.at(-1)?.[1] as RequestInit;
      return JSON.parse(String(postCall.body)).settings as string;
    });

    const decoded = await client.decodeAccountSettingsPayload(encryptedSettings);
    expect(decoded.preferredLanguage).toBe("ja");
  });

  it("retries account settings updates after a version conflict", async () => {
    const client = await Wave8Client.connect("https://api.cluster-fluster.com", {
      token: "token-d",
      secret: "AQIDBAUGBwgJCgsMDQ4PEBESExQVFhcYGRobHB0eHyA",
    });

    const baseSettings = {
      schemaVersion: 2,
      viewInline: false,
      inferenceOpenAIKey: null,
      expandTodos: true,
      showLineNumbers: true,
      showLineNumbersInToolViews: false,
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

    vi.mocked(globalThis.fetch).mockResolvedValueOnce(
      new Response(JSON.stringify({ success: true, version: 1 }), { status: 200 }),
    );
    const serverEncryptedSettings = await client.updateAccountSettings(
      baseSettings,
      0,
      { showFlavorIcons: true },
    ).then(() => {
      const postCall = vi.mocked(globalThis.fetch).mock.calls.at(-1)?.[1] as RequestInit;
      return JSON.parse(String(postCall.body)).settings as string;
    });

    vi.mocked(globalThis.fetch).mockReset();
    vi.mocked(globalThis.fetch)
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            success: false,
            error: "version-mismatch",
            currentVersion: 3,
            currentSettings: serverEncryptedSettings,
          }),
          { status: 200 },
        ),
      )
      .mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            success: true,
            version: 4,
          }),
          { status: 200 },
        ),
      );

    const result = await client.updateAccountSettings(baseSettings, 2, {
      preferredLanguage: "ja",
    });

    expect(result.version).toBe(4);
    expect(result.settings.preferredLanguage).toBe("ja");
    expect(result.settings.showFlavorIcons).toBe(true);
    expect(globalThis.fetch).toHaveBeenCalledTimes(2);
  });
});
