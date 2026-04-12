import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  beginLoopbackAccountLink,
  cancelLoopbackAccountLink,
  clearStoredCredentials,
  completeAccountLink,
  createAccount,
  createAccountLinkRequest,
  type DesktopArtifact,
  type DesktopMachine,
  describeSession,
  formatSecretKeyForBackup,
  getLoopbackAccountLinkStatus,
  loadServerUrl,
  loadStoredCredentials,
  openExternalUrl,
  restoreAccount,
  saveServerUrl,
  saveStoredCredentials,
  showDesktopNotification,
  type AccountProfile,
  type CreateSessionInput,
  type DesktopSession,
  type RemoteMessageRecord,
  type SessionBashRequest,
  type SessionBashResponse,
  type SessionReadFileResponse,
  type SessionRpcAck,
  type SessionAgentStateUpdate,
  type SessionMetadataUpdate,
  type SendMessageOptions,
  type Settings,
  type StoredCredentials,
  type UsageBucket,
  type UsagePeriod,
  type UserProfile,
  type UiMessage,
  Wave8Client,
  type FeedPostResponse,
} from "./wave8-client";
import { mergeIncomingSessionMessages } from "./session-live-updates";
import { removeDeletedSession, upsertRealtimeSession } from "./realtime-state";
import {
  buildGitDiffCommand,
  buildWorkspaceFilePath,
  decodeWorkspaceFileContent,
  parseSessionWorkspaceFiles,
  type SessionWorkspaceFile,
  type SessionWorkspaceFileContent,
} from "./session-files";
import {
  SessionBashResponseSchema,
  SessionReadFileResponseSchema,
  SessionRpcAckSchema,
  type SessionsRealtimeUpdate,
  SessionsRealtimeUpdateSchema,
  parseWithSchema,
  safeParseWithSchema,
} from "./wave8-wire";

type SocketClient = import("socket.io-client").Socket;

export type SessionUiState = {
  items: UiMessage[];
  loading: boolean;
  sending: boolean;
  aborting: boolean;
  error: string | null;
  loadedAt: number | null;
  lastSeq: number | null;
};

export type LinkUiState = {
  status: "idle" | "requesting" | "waiting" | "ready" | "error";
  linkUrl: string | null;
  qrSvg: string | null;
  error: string | null;
  attemptId: string | null;
  browserUrl: string | null;
};

type ErrorScope = "global" | "local";

const defaultSessionUiState: SessionUiState = {
  items: [],
  loading: false,
  sending: false,
  aborting: false,
  error: null,
  loadedAt: null,
  lastSeq: null,
};

function resolveActionError(error: unknown, fallback: string): string {
  return error instanceof Error ? error.message : fallback;
}

export function useWave8Desktop(activeSessionId?: string | null) {
  const clientRef = useRef<Wave8Client | null>(null);
  const linkAbortRef = useRef<AbortController | null>(null);
  const socketRef = useRef<SocketClient | null>(null);
  const activeSessionIdRef = useRef<string | null>(activeSessionId ?? null);
  const loadedSessionsRef = useRef<Set<string>>(new Set());
  const sessionsRef = useRef<DesktopSession[]>([]);
  const sessionStateRef = useRef<Record<string, SessionUiState>>({});
  const refreshSessionsRef = useRef<(() => Promise<DesktopSession[]>) | null>(null);
  const loadMessagesRef = useRef<(typeof loadMessages) | null>(null);
  const [serverUrl, setServerUrlState] = useState(() => loadServerUrl());
  const [status, setStatus] = useState<"checking" | "signed-out" | "loading" | "ready">(
    "checking",
  );
  const [credentials, setCredentials] = useState<StoredCredentials | null>(null);
  const [profile, setProfile] = useState<AccountProfile | null>(null);
  const [accountSettings, setAccountSettings] = useState<Settings | null>(null);
  const [accountSettingsVersion, setAccountSettingsVersion] = useState<number | null>(null);
  const [sessions, setSessions] = useState<DesktopSession[]>([]);
  const [artifacts, setArtifacts] = useState<DesktopArtifact[]>([]);
  const [machines, setMachines] = useState<DesktopMachine[]>([]);
  const [userProfiles, setUserProfiles] = useState<Record<string, UserProfile>>({});
  const [friends, setFriends] = useState<UserProfile[]>([]);
  const [feedItems, setFeedItems] = useState<FeedPostResponse[]>([]);
  const [sessionState, setSessionState] = useState<Record<string, SessionUiState>>({});
  const [globalError, setGlobalError] = useState<string | null>(null);
  const [linkState, setLinkState] = useState<LinkUiState>({
    status: "idle",
    linkUrl: null,
    qrSvg: null,
    error: null,
    attemptId: null,
    browserUrl: null,
  });
  const [backupKey, setBackupKey] = useState<string | null>(null);
  const [storedSessionAvailable, setStoredSessionAvailable] = useState(false);

  useEffect(() => {
    activeSessionIdRef.current = activeSessionId ?? null;
  }, [activeSessionId]);

  useEffect(() => {
    loadedSessionsRef.current = new Set(
      Object.entries(sessionState)
        .filter(([, value]) => !!value.loadedAt)
        .map(([sessionId]) => sessionId),
    );
  }, [sessionState]);

  useEffect(() => {
    sessionStateRef.current = sessionState;
  }, [sessionState]);

  useEffect(() => {
    sessionsRef.current = sessions;
  }, [sessions]);

  const applySessionState = useCallback(
    (sessionId: string, patch: Partial<SessionUiState>) => {
      setSessionState((current) => ({
        ...current,
        [sessionId]: {
          ...defaultSessionUiState,
          ...current[sessionId],
          ...patch,
        },
      }));
    },
    [],
  );

  const connectClient = useCallback(
    async (nextCredentials: StoredCredentials, nextServerUrl: string) => {
      const client = await Wave8Client.connect(nextServerUrl, nextCredentials);
      clientRef.current = client;
      return client;
    },
    [],
  );

  const hydrate = useCallback(
    async (nextCredentials: StoredCredentials, nextServerUrl: string) => {
      setStatus("loading");
      setGlobalError(null);
      const client = await connectClient(nextCredentials, nextServerUrl);
      const [nextProfile, nextAccountSettings, nextSessions, nextArtifacts, nextMachines, nextFriends, nextFeedItems] =
        await Promise.all([
        client.fetchProfile(),
        client.fetchAccountSettings(),
        client.listSessions(),
        client.listArtifacts(),
        client.listMachines(),
        client.listFriends(),
        client.listFeed(),
        ]);
      setCredentials(nextCredentials);
      setProfile(nextProfile);
      setAccountSettings(nextAccountSettings.settings);
      setAccountSettingsVersion(nextAccountSettings.version);
      setSessions(nextSessions);
      setArtifacts(nextArtifacts);
      setMachines(nextMachines);
      setFriends(nextFriends);
      setFeedItems(nextFeedItems);
      setStatus("ready");
      return {
        client,
        sessions: nextSessions,
        artifacts: nextArtifacts,
        machines: nextMachines,
        friends: nextFriends,
        feedItems: nextFeedItems,
      };
    },
    [connectClient],
  );

  const bootFromStorage = useCallback(async () => {
    const stored = await loadStoredCredentials();
    if (!stored) {
      clientRef.current = null;
      setCredentials(null);
      setProfile(null);
      setAccountSettings(null);
      setAccountSettingsVersion(null);
      setSessions([]);
      setArtifacts([]);
      setMachines([]);
      setUserProfiles({});
      setFriends([]);
      setFeedItems([]);
      setSessionState({});
      setStoredSessionAvailable(false);
      setGlobalError(null);
      setStatus("signed-out");
      return;
    }

    setStoredSessionAvailable(true);
    try {
      await hydrate(stored, serverUrl);
    } catch (error) {
      clientRef.current = null;
      setCredentials(null);
      setProfile(null);
      setAccountSettings(null);
      setAccountSettingsVersion(null);
      setSessions([]);
      setArtifacts([]);
      setMachines([]);
      setUserProfiles({});
      setFriends([]);
      setFeedItems([]);
      setSessionState({});
      setStatus("signed-out");
      setGlobalError(error instanceof Error ? error.message : "Failed to restore desktop session");
    }
  }, [hydrate, serverUrl]);

  useEffect(() => {
    void bootFromStorage();
  }, [bootFromStorage]);

  const refreshSessions = useCallback(async (scope: ErrorScope = "global") => {
    if (!clientRef.current) {
      return [];
    }

    try {
      const nextSessions = await clientRef.current.listSessions();
      setSessions(nextSessions);
      return nextSessions;
    } catch (error) {
      if (scope === "global") {
        setGlobalError(error instanceof Error ? error.message : "Failed to refresh sessions");
      }
      throw error;
    }
  }, []);

  const refreshProfile = useCallback(async () => {
    const client = clientRef.current;
    if (!client) {
      return null;
    }

    try {
      const nextProfile = await client.fetchProfile();
      setProfile(nextProfile);
      return nextProfile;
    } catch (error) {
      setGlobalError(error instanceof Error ? error.message : "Failed to refresh profile");
      throw error;
    }
  }, []);

  const refreshAccountSettings = useCallback(async () => {
    const client = clientRef.current;
    if (!client) {
      return null;
    }

    try {
      const next = await client.fetchAccountSettings();
      setAccountSettings(next.settings);
      setAccountSettingsVersion(next.version);
      return next;
    } catch (error) {
      setGlobalError(error instanceof Error ? error.message : "Failed to refresh account settings");
      throw error;
    }
  }, []);

  const updateAccountSettings = useCallback(
    async (patch: Partial<Settings>) => {
      const client = clientRef.current;
      if (!client) {
        throw new Error("Sign in first");
      }

      const currentSnapshot =
        accountSettings && accountSettingsVersion !== null
          ? {
              settings: accountSettings,
              version: accountSettingsVersion,
            }
          : await refreshAccountSettings();

      if (!currentSnapshot) {
        throw new Error("Account settings are unavailable");
      }

      const next = await client.updateAccountSettings(
        currentSnapshot.settings,
        currentSnapshot.version,
        patch,
      );
      setAccountSettings(next.settings);
      setAccountSettingsVersion(next.version);
      return next;
    },
    [accountSettings, accountSettingsVersion, refreshAccountSettings],
  );

  const refreshArtifacts = useCallback(async () => {
    if (!clientRef.current) {
      return [];
    }

    try {
      const nextArtifacts = await clientRef.current.listArtifacts();
      setArtifacts(nextArtifacts);
      return nextArtifacts;
    } catch (error) {
      setGlobalError(error instanceof Error ? error.message : "Failed to refresh artifacts");
      throw error;
    }
  }, []);

  const refreshFriends = useCallback(async (scope: ErrorScope = "global") => {
    if (!clientRef.current) {
      return [];
    }

    try {
      const nextFriends = await clientRef.current.listFriends();
      setFriends(nextFriends);
      return nextFriends;
    } catch (error) {
      if (scope === "global") {
        setGlobalError(error instanceof Error ? error.message : "Failed to refresh friends");
      }
      throw error;
    }
  }, []);

  const refreshFeed = useCallback(async (scope: ErrorScope = "global") => {
    if (!clientRef.current) {
      return [];
    }

    try {
      const nextFeedItems = await clientRef.current.listFeed();
      setFeedItems(nextFeedItems);
      return nextFeedItems;
    } catch (error) {
      if (scope === "global") {
        setGlobalError(error instanceof Error ? error.message : "Failed to refresh feed");
      }
      throw error;
    }
  }, []);

  const loadArtifact = useCallback(
    async (artifactId: string) => {
      const client = clientRef.current;
      if (!client) {
        return null;
      }

      const artifact = await client.fetchArtifact(artifactId);
      if (!artifact) {
        return null;
      }

      setArtifacts((current) => {
        const remaining = current.filter((item) => item.id !== artifact.id);
        return [artifact, ...remaining].sort((left, right) => right.updatedAt - left.updatedAt);
      });
      return artifact;
    },
    [],
  );

  const refreshMachines = useCallback(async () => {
    if (!clientRef.current) {
      return [];
    }

    try {
      const nextMachines = await clientRef.current.listMachines();
      setMachines(nextMachines);
      return nextMachines;
    } catch (error) {
      setGlobalError(error instanceof Error ? error.message : "Failed to refresh machines");
      throw error;
    }
  }, []);

  const loadMachine = useCallback(async (machineId: string) => {
    const client = clientRef.current;
    if (!client) {
      return null;
    }

    const machine = await client.fetchMachine(machineId);
    if (!machine) {
      return null;
    }

    setMachines((current) => {
      const remaining = current.filter((item) => item.id !== machine.id);
      return [machine, ...remaining].sort((left, right) => right.activeAt - left.activeAt);
    });
    return machine;
  }, []);

  const loadUserProfile = useCallback(async (userId: string) => {
    const client = clientRef.current;
    if (!client) {
      return null;
    }

    const profile = await client.fetchUserProfile(userId);
    if (!profile) {
      return null;
    }

    setUserProfiles((current) => ({
      ...current,
      [profile.id]: profile,
    }));
    return profile;
  }, []);

  const searchUsers = useCallback(async (query: string) => {
    const client = clientRef.current;
    if (!client) {
      return [];
    }

    const users = await client.searchUsers(query);
    setUserProfiles((current) => {
      const next = { ...current };
      for (const user of users) {
        next[user.id] = user;
      }
      return next;
    });
    return users;
  }, []);

  const addFriend = useCallback(async (userId: string) => {
    const client = clientRef.current;
    if (!client) {
      throw new Error("Sign in first");
    }

    const updated = await client.addFriend(userId);
    await Promise.all([refreshFriends("local"), refreshFeed("local")]);
    if (updated) {
      setUserProfiles((current) => ({
        ...current,
        [updated.id]: updated,
      }));
    }
    return updated;
  }, [refreshFeed, refreshFriends]);

  const removeFriend = useCallback(async (userId: string) => {
    const client = clientRef.current;
    if (!client) {
      throw new Error("Sign in first");
    }

    const updated = await client.removeFriend(userId);
    await Promise.all([refreshFriends("local"), refreshFeed("local")]);
    if (updated) {
      setUserProfiles((current) => ({
        ...current,
        [updated.id]: updated,
      }));
    }
    return updated;
  }, [refreshFeed, refreshFriends]);

  const loadUsage = useCallback(
    async (period: UsagePeriod, sessionId?: string) => {
      const client = clientRef.current;
      if (!client) {
        throw new Error("Sign in first");
      }

      return client.queryUsage(period, sessionId);
    },
    [],
  );

  const loadMessages = useCallback(
    async (sessionId: string, force = false) => {
      const client = clientRef.current;
      if (!client) {
        return null;
      }

      const knownState = sessionState[sessionId];
      if (!force && knownState?.loadedAt && !knownState.loading) {
        return knownState;
      }

      let targetSession = sessions.find((session) => session.id === sessionId) ?? null;
      if (!targetSession) {
        const nextSessions = await refreshSessions("local");
        targetSession = nextSessions.find((session) => session.id === sessionId) ?? null;
      }
      if (!targetSession) {
        applySessionState(sessionId, {
          error: `Session ${sessionId} not found`,
          loading: false,
        });
        return null;
      }

      applySessionState(sessionId, { loading: true, error: null });
      try {
        const nextState = await client.listMessages(targetSession);
        applySessionState(sessionId, {
          items: nextState.items,
          loading: false,
          error: null,
          loadedAt: nextState.loadedAt,
          lastSeq: nextState.lastSeq,
        });
        return nextState;
      } catch (error) {
        applySessionState(sessionId, {
          loading: false,
          error: error instanceof Error ? error.message : "Failed to load session messages",
        });
        return null;
      }
    },
    [applySessionState, refreshSessions, sessionState, sessions],
  );

  const loginWithCredentials = useCallback(
    async (nextCredentials: StoredCredentials, nextBackupKey?: string | null) => {
      await saveStoredCredentials(nextCredentials);
      setStoredSessionAvailable(true);
      const result = await hydrate(nextCredentials, serverUrl);
      setBackupKey(nextBackupKey ?? formatSecretKeyForBackup(nextCredentials.secret));
      return result.sessions;
    },
    [hydrate, serverUrl],
  );

  const createFreshAccount = useCallback(async () => {
    setGlobalError(null);
    try {
      const created = await createAccount(serverUrl);
      await loginWithCredentials(created.credentials, created.backupKey);
      void showDesktopNotification(
        "Desktop account ready",
        "A fresh Vibe desktop account was created on this machine.",
      ).catch(() => undefined);
    } catch (error) {
      setGlobalError(resolveActionError(error, "Failed to create account"));
      throw error;
    }
  }, [loginWithCredentials, serverUrl]);

  const restoreWithSecret = useCallback(
    async (secret: string) => {
      setGlobalError(null);
      try {
        const restored = await restoreAccount(serverUrl, secret);
        await loginWithCredentials(restored);
        void showDesktopNotification(
          "Desktop account restored",
          "The backup key restored this desktop account successfully.",
        ).catch(() => undefined);
      } catch (error) {
        setGlobalError(resolveActionError(error, "Failed to restore account"));
        throw error;
      }
    },
    [loginWithCredentials, serverUrl],
  );

  const cancelMobileLink = useCallback(() => {
    const attemptId = linkState.attemptId;
    linkAbortRef.current?.abort();
    linkAbortRef.current = null;
    if (attemptId) {
      void cancelLoopbackAccountLink(attemptId);
    }
    setLinkState({
      status: "idle",
      linkUrl: null,
      qrSvg: null,
      error: null,
      attemptId: null,
      browserUrl: null,
    });
  }, [linkState.attemptId]);

  const startMobileLink = useCallback(async () => {
    cancelMobileLink();
    setGlobalError(null);
    setLinkState({
      status: "requesting",
      linkUrl: null,
      qrSvg: null,
      error: null,
      attemptId: null,
      browserUrl: null,
    });

    let activeAttemptId: string | null = null;
    try {
      const request = await createAccountLinkRequest();
      const attempt = await beginLoopbackAccountLink(serverUrl, request);
      activeAttemptId = attempt.attemptId;
      const QRCode = await import("qrcode");
      const qrSvg = await QRCode.toString(request.linkUrl, {
        type: "svg",
        margin: 1,
        width: 220,
        color: {
          dark: "#7be2c4",
          light: "#08131c",
        },
      });
      const abortController = new AbortController();
      linkAbortRef.current = abortController;
      setLinkState({
        status: "waiting",
        linkUrl: request.linkUrl,
        qrSvg,
        error: null,
        attemptId: attempt.attemptId,
        browserUrl: attempt.browserUrl,
      });

      await openExternalUrl(attempt.browserUrl);

      while (!abortController.signal.aborted) {
        const snapshot = await getLoopbackAccountLinkStatus(attempt.attemptId);
        if (snapshot.status === "completed") {
          const linkedCredentials = await completeAccountLink(serverUrl, request);
          await loginWithCredentials(linkedCredentials);
          void showDesktopNotification(
            "Desktop link approved",
            "This desktop is now linked to the approved Vibe account.",
          ).catch(() => undefined);
          setLinkState({
            status: "ready",
            linkUrl: request.linkUrl,
            qrSvg,
            error: null,
            attemptId: attempt.attemptId,
            browserUrl: attempt.browserUrl,
          });
          return;
        }

        if (snapshot.status === "failed") {
          throw new Error(snapshot.error);
        }

        if (snapshot.status === "canceled" || snapshot.status === "not_found") {
          throw new Error("Desktop auth callback was canceled");
        }

        await new Promise<void>((resolve, reject) => {
          const timeoutId = window.setTimeout(resolve, 1000);
          abortController.signal.addEventListener(
            "abort",
            () => {
              window.clearTimeout(timeoutId);
              reject(new Error("Canceled"));
            },
            { once: true },
          );
        });
      }
    } catch (error) {
      if (error instanceof Error && error.message === "Canceled") {
        return;
      }
      if (linkAbortRef.current) {
        linkAbortRef.current.abort();
        linkAbortRef.current = null;
      }
      if (activeAttemptId) {
        void cancelLoopbackAccountLink(activeAttemptId);
      }
      const errorMessage = resolveActionError(error, "Failed to link account");
      setLinkState({
        status: "error",
        linkUrl: null,
        qrSvg: null,
        error: errorMessage,
        attemptId: null,
        browserUrl: null,
      });
      throw error;
    }
  }, [cancelMobileLink, loginWithCredentials, serverUrl]);

  const retryStoredSession = useCallback(async () => {
    await bootFromStorage();
  }, [bootFromStorage]);

  const logout = useCallback(async () => {
    cancelMobileLink();
    socketRef.current?.disconnect();
    socketRef.current = null;
    let logoutError: string | null = null;
    try {
      await clearStoredCredentials();
    } catch (error) {
      logoutError = resolveActionError(error, "Failed to clear stored desktop credentials");
    }
    clientRef.current = null;
    setCredentials(null);
    setProfile(null);
    setAccountSettings(null);
    setAccountSettingsVersion(null);
    setSessions([]);
    setArtifacts([]);
    setMachines([]);
    setUserProfiles({});
    setFriends([]);
    setFeedItems([]);
    setSessionState({});
    setBackupKey(null);
    setStoredSessionAvailable(false);
    setGlobalError(logoutError);
    setStatus("signed-out");
  }, [cancelMobileLink]);

  const updateServerUrl = useCallback(
    async (nextUrl: string) => {
      const normalized = saveServerUrl(nextUrl);
      setServerUrlState(normalized);
      if (credentials) {
        await hydrate(credentials, normalized);
      }
    },
    [credentials, hydrate],
  );

  const createSession = useCallback(
    async (input: CreateSessionInput) => {
      const client = clientRef.current;
      if (!client) {
        throw new Error("Sign in first");
      }
      const session = await client.createSession(input);
      setSessions((current) => [session, ...current.filter((item) => item.id !== session.id)]);
      await loadMessages(session.id, true);
      await refreshSessions();
      return session;
    },
    [loadMessages, refreshSessions],
  );

  const createArtifact = useCallback(
    async (input: {
      title: string | null;
      body: string | null;
      sessions?: string[];
      draft?: boolean;
    }) => {
      const client = clientRef.current;
      if (!client) {
        throw new Error("Sign in first");
      }

      const artifact = await client.createArtifact(input);
      setArtifacts((current) => [artifact, ...current.filter((item) => item.id !== artifact.id)]);
      return artifact;
    },
    [],
  );

  const updateArtifact = useCallback(
    async (
      artifactId: string,
      input: {
        title: string | null;
        body: string | null;
        sessions?: string[];
        draft?: boolean;
      },
    ) => {
      const client = clientRef.current;
      if (!client) {
        throw new Error("Sign in first");
      }

      const currentArtifact =
        artifacts.find((artifact) => artifact.id === artifactId) ??
        (await loadArtifact(artifactId));
      if (!currentArtifact) {
        throw new Error(`Artifact ${artifactId} not found`);
      }

      const updated = await client.updateArtifact(currentArtifact, input);
      setArtifacts((current) =>
        current
          .map((artifact) => (artifact.id === artifactId ? updated : artifact))
          .sort((left, right) => right.updatedAt - left.updatedAt),
      );
      return updated;
    },
    [artifacts, loadArtifact],
  );

  const deleteArtifact = useCallback(async (artifactId: string) => {
    const client = clientRef.current;
    if (!client) {
      throw new Error("Sign in first");
    }

    await client.deleteArtifact(artifactId);
    setArtifacts((current) => current.filter((artifact) => artifact.id !== artifactId));
  }, []);

  const deleteSession = useCallback(async (sessionId: string) => {
    const client = clientRef.current;
    if (!client) {
      throw new Error("Sign in first");
    }

    setGlobalError(null);
    try {
      await client.deleteSession(sessionId);
      setSessions((current) => current.filter((session) => session.id !== sessionId));
      setSessionState((current) => {
        const next = { ...current };
        delete next[sessionId];
        return next;
      });
    } catch (error) {
      setGlobalError(resolveActionError(error, "Failed to delete session"));
      throw error;
    }
  }, []);

  const sendMessage = useCallback(
    async (sessionId: string, text: string, options?: SendMessageOptions) => {
      const trimmed = text.trim();
      if (!trimmed) {
        return;
      }

      const client = clientRef.current;
      if (!client) {
        throw new Error("Sign in first");
      }

      const session = sessions.find((item) => item.id === sessionId);
      if (!session) {
        throw new Error(`Session ${sessionId} not found`);
      }

      applySessionState(sessionId, { sending: true, error: null });
      try {
        await client.sendMessage(sessionId, trimmed, session.dataEncryptionKey, options);
        await Promise.all([refreshSessions("local"), loadMessages(sessionId, true)]);
        applySessionState(sessionId, { sending: false });
      } catch (error) {
        applySessionState(sessionId, {
          sending: false,
          error: error instanceof Error ? error.message : "Failed to send message",
        });
        throw error;
      }
    },
    [applySessionState, loadMessages, refreshSessions, sessions],
  );

  const sessionRpc = useCallback(
    async <TParams extends object, TResult>(
      sessionId: string,
      method: string,
      params: TParams,
      parser: (value: unknown) => TResult,
    ): Promise<TResult> => {
      const socket = socketRef.current;
      const client = clientRef.current;
      const session = sessionsRef.current.find((item) => item.id === sessionId) ?? null;
      if (!socket || !client || !session) {
        throw new Error("Session RPC is unavailable");
      }

      const encryptedParams = await client.encryptSessionRpcPayload(
        sessionId,
        session.dataEncryptionKey,
        params,
      );
      const ack = parseWithSchema(
        SessionRpcAckSchema,
        await socket.emitWithAck("rpc-call", {
          method: `${sessionId}:${method}`,
          params: encryptedParams,
        }),
        `Session RPC ack for ${sessionId}:${method}`,
      ) as SessionRpcAck;

      if (!ack.ok) {
        throw new Error(ack.error || `Session RPC ${method} failed`);
      }

      const decrypted = await client.decryptSessionRpcPayload(
        sessionId,
        session.dataEncryptionKey,
        ack.result,
      );
      return parser(decrypted);
    },
    [],
  );

  const abortSession = useCallback(
    async (sessionId: string) => {
      applySessionState(sessionId, { aborting: true, error: null });

      try {
        await sessionRpc<{ reason: string }, unknown>(
          sessionId,
          "abort",
          {
            reason:
              "The user canceled the current turn. Stop the in-flight work and wait for the next instruction.",
          },
          (value) => value,
        );
        applySessionState(sessionId, { aborting: false });
      } catch (error) {
        applySessionState(sessionId, {
          aborting: false,
          error: error instanceof Error ? error.message : "Failed to abort session turn",
        });
        throw error;
      }
    },
    [applySessionState, sessionRpc],
  );

  const loadSessionFiles = useCallback(
    async (sessionId: string): Promise<{
      branch: string | null;
      files: SessionWorkspaceFile[];
      totalStaged: number;
      totalUnstaged: number;
    }> => {
      const session = sessionsRef.current.find((item) => item.id === sessionId) ?? null;
      const workspaceRoot = session?.metadata?.path;
      if (!workspaceRoot) {
        throw new Error("Session workspace path is unavailable");
      }

      const statusResult = await sessionRpc<SessionBashRequest, SessionBashResponse>(
        sessionId,
        "bash",
        {
          command: "git status --porcelain=v2 --branch --untracked-files=all",
          cwd: workspaceRoot,
          timeout: 10000,
        },
        (value) =>
          parseWithSchema(
            SessionBashResponseSchema,
            value,
            `Session bash response for git status in ${sessionId}`,
          ),
      );

      if (!statusResult.success || statusResult.exitCode !== 0) {
        throw new Error(statusResult.error || statusResult.stderr || "Failed to inspect git status");
      }

      const diffStatResult = await sessionRpc<SessionBashRequest, SessionBashResponse>(
        sessionId,
        "bash",
        {
          command: "git diff --numstat HEAD && echo \"---STAGED---\" && git diff --cached --numstat",
          cwd: workspaceRoot,
          timeout: 10000,
        },
        (value) =>
          parseWithSchema(
            SessionBashResponseSchema,
            value,
            `Session bash response for git diff stats in ${sessionId}`,
          ),
      );

      const diffOutput = diffStatResult.success ? diffStatResult.stdout : "";
      return parseSessionWorkspaceFiles(statusResult.stdout, diffOutput, workspaceRoot);
    },
    [sessionRpc],
  );

  const readSessionFile = useCallback(
    async (sessionId: string, relativePath: string): Promise<SessionWorkspaceFileContent> => {
      const session = sessionsRef.current.find((item) => item.id === sessionId) ?? null;
      const workspaceRoot = session?.metadata?.path;
      if (!workspaceRoot) {
        throw new Error("Session workspace path is unavailable");
      }

      const readFileResponse = await sessionRpc<{ path: string }, SessionReadFileResponse>(
        sessionId,
        "readFile",
        {
          path: buildWorkspaceFilePath(workspaceRoot, relativePath),
        },
        (value) =>
          parseWithSchema(
            SessionReadFileResponseSchema,
            value,
            `Session readFile response for ${relativePath}`,
          ),
      );

      if (!readFileResponse.success || !readFileResponse.content) {
        throw new Error(readFileResponse.error || `Failed to read ${relativePath}`);
      }

      const diffResponse = await sessionRpc<SessionBashRequest, SessionBashResponse>(
        sessionId,
        "bash",
        {
          command: buildGitDiffCommand(relativePath),
          cwd: workspaceRoot,
          timeout: 5000,
        },
        (value) =>
          parseWithSchema(
            SessionBashResponseSchema,
            value,
            `Session bash response for file diff ${relativePath}`,
          ),
      ).catch(() => ({
        success: false,
        stdout: "",
        stderr: "",
        exitCode: -1,
      }));

      return decodeWorkspaceFileContent(
        relativePath,
        workspaceRoot,
        readFileResponse.content,
        diffResponse.success ? diffResponse.stdout || null : null,
      );
    },
    [sessionRpc],
  );

  const sessionSummaries = useMemo(
    () => sessions.map((session) => ({ session, ...describeSession(session) })),
    [sessions],
  );

  useEffect(() => {
    refreshSessionsRef.current = refreshSessions;
  }, [refreshSessions]);

  useEffect(() => {
    loadMessagesRef.current = loadMessages;
  }, [loadMessages]);

  useEffect(() => {
    if (!credentials) {
      socketRef.current?.disconnect();
      socketRef.current = null;
      return undefined;
    }
    let disposed = false;
    let socket: SocketClient | null = null;

    const refreshActiveSession = (sessionId: string | null | undefined) => {
      if (!sessionId) {
        return;
      }
      const shouldReload =
        activeSessionIdRef.current === sessionId || loadedSessionsRef.current.has(sessionId);
      if (shouldReload) {
        void loadMessagesRef.current?.(sessionId, true);
      }
    };

    void import("socket.io-client").then(({ io }) => {
      if (disposed) {
        return;
      }

      socket = io(serverUrl, {
        path: "/v1/updates",
        auth: {
          token: credentials.token,
          clientType: "user-scoped",
        },
        transports: ["websocket"],
        reconnection: true,
        reconnectionDelay: 1000,
        reconnectionDelayMax: 5000,
        reconnectionAttempts: Infinity,
      });

      socketRef.current = socket;

      socket.on("connect", () => {
        void refreshSessionsRef.current?.();
        void refreshArtifacts();
        void refreshProfile();
        void refreshAccountSettings();
        void refreshFriends();
        void refreshFeed();
        refreshActiveSession(activeSessionIdRef.current);
      });

      socket.on("update", (rawPayload: unknown) => {
        const payload: SessionsRealtimeUpdate | null = safeParseWithSchema(
          SessionsRealtimeUpdateSchema,
          rawPayload,
        );
        if (!payload) {
          setGlobalError("Received an invalid realtime update payload");
          return;
        }

        const eventType = payload.body.t;
        let sessionId: string | null = null;
        if (eventType === "new-message" || eventType === "delete-session") {
          sessionId = payload.body.sid;
        } else if (eventType === "new-session" || eventType === "update-session") {
          sessionId = payload.body.id;
        }

        if (eventType === "new-session") {
          void refreshSessionsRef.current?.();
        }

        if (eventType === "delete-session" && sessionId) {
          setSessions((current) =>
            removeDeletedSession(current, sessionStateRef.current, sessionId).sessions,
          );
          setSessionState((current) =>
            removeDeletedSession(sessionsRef.current, current, sessionId).sessionState,
          );
          loadedSessionsRef.current.delete(sessionId);
          if (activeSessionIdRef.current === sessionId) {
            setGlobalError(`Session ${sessionId} is no longer available`);
          }
          return;
        }

        if (eventType === "update-account") {
          const accountUpdate = payload.body;

          if (accountUpdate.settings && clientRef.current) {
            void clientRef.current
              .decodeAccountSettingsPayload(accountUpdate.settings.value ?? null)
              .then((settings) => {
                setAccountSettings(settings);
                setAccountSettingsVersion(accountUpdate.settings?.version ?? null);
              })
              .catch(() => {
                void refreshAccountSettings();
              });
          }

          if (
            accountUpdate.firstName !== undefined ||
            accountUpdate.lastName !== undefined ||
            accountUpdate.avatar !== undefined ||
            accountUpdate.github !== undefined
          ) {
            void refreshProfile();
          }
          return;
        }

        if (
          eventType === "new-artifact" ||
          eventType === "update-artifact" ||
          eventType === "delete-artifact"
        ) {
          void refreshArtifacts();
          return;
        }

        if (eventType === "update-machine") {
          void refreshMachines();
          return;
        }

        if (eventType === "relationship-updated") {
          void refreshFriends();
          return;
        }

        if (eventType === "new-feed-post") {
          void refreshFeed();
          return;
        }

        if (eventType === "new-message" && sessionId) {
          const incomingMessage: RemoteMessageRecord = payload.body.message;
          const targetSession =
            sessionsRef.current.find((session) => session.id === sessionId) ?? null;
          const incomingSeq = incomingMessage.seq;

          if (targetSession) {
            setSessions((current) =>
              current
                .map((session) =>
                  session.id === sessionId
                    ? {
                        ...session,
                        seq: Math.max(session.seq, payload.seq ?? session.seq),
                        updatedAt:
                          payload.createdAt ?? incomingMessage.createdAt ?? session.updatedAt,
                      }
                    : session,
                )
                .sort((left, right) => right.updatedAt - left.updatedAt),
            );
          } else {
            void refreshSessionsRef.current?.();
          }

          const shouldAppend =
            !!targetSession &&
            (activeSessionIdRef.current === sessionId || loadedSessionsRef.current.has(sessionId));

          if (shouldAppend && clientRef.current) {
            void clientRef.current
              .decodeMessage(sessionId, targetSession.dataEncryptionKey, incomingMessage)
              .then((messages) => {
                const merged = mergeIncomingSessionMessages(
                  sessionStateRef.current[sessionId],
                  incomingSeq,
                  messages,
                  Date.now(),
                );

                if (merged.action === "apply") {
                  applySessionState(sessionId, merged.next);
                  return;
                }

                if (merged.action === "refresh") {
                  refreshActiveSession(sessionId);
                }
              })
              .catch(() => {
                refreshActiveSession(sessionId);
              });
          }
          return;
        }

        if (eventType === "update-session" && sessionId) {
          const targetSession =
            sessionsRef.current.find((session) => session.id === sessionId) ?? null;
          if (!targetSession || !clientRef.current) {
            void refreshSessionsRef.current?.();
            refreshActiveSession(sessionId);
            return;
          }

          void clientRef.current
            .applySessionUpdate(targetSession, {
              metadata: payload.body.metadata,
              agentState: payload.body.agentState,
              seq: payload.seq,
              updatedAt: payload.createdAt,
            })
            .then((nextSession) => {
              setSessions((current) => upsertRealtimeSession(current, nextSession));
            })
            .catch(() => {
              void refreshSessionsRef.current?.();
            });

          refreshActiveSession(sessionId);
        }
      });
    });

    return () => {
      disposed = true;
      socket?.disconnect();
      if (socketRef.current === socket) {
        socketRef.current = null;
      }
    };
  }, [credentials, refreshAccountSettings, refreshArtifacts, refreshMachines, refreshProfile, serverUrl]);

  return {
    status,
    serverUrl,
    credentials,
    profile,
    accountSettings,
    accountSettingsVersion,
    sessions,
    artifacts,
    machines,
    userProfiles,
    friends,
    feedItems,
    sessionSummaries,
    sessionState,
    globalError,
    linkState,
    backupKey: backupKey ?? (credentials ? formatSecretKeyForBackup(credentials.secret) : null),
    createFreshAccount,
    restoreWithSecret,
    startMobileLink,
    cancelMobileLink,
    refreshSessions,
    refreshAccountSettings,
    refreshArtifacts,
    refreshMachines,
    refreshFriends,
    refreshFeed,
    loadMachine,
    loadUserProfile,
    searchUsers,
    addFriend,
    removeFriend,
    loadArtifact,
    loadUsage,
    loadMessages,
    loadSessionFiles,
    readSessionFile,
    createSession,
    createArtifact,
    updateArtifact,
    deleteArtifact,
    deleteSession,
    sendMessage,
    abortSession,
    logout,
    updateServerUrl,
    updateAccountSettings,
    retryStoredSession,
    storedSessionAvailable,
  };
}
