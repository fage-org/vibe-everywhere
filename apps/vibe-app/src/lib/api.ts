import type {
  AppConfig,
  AuditRecord,
  ConversationDetailResponse,
  ConversationInputRequest,
  ConversationRecord,
  CreateConversationPayload,
  CreateConversationResponse,
  CreatePortForwardPayload,
  CreateShellSessionPayload,
  CreateTaskPayload,
  DeviceRecord,
  GitCreateWorktreeRequest,
  GitCreateWorktreeResponse,
  GitDiffFileRequest,
  GitDiffFileResponse,
  GitInspectRequest,
  GitInspectResponse,
  GitRemoveWorktreeRequest,
  GitRemoveWorktreeResponse,
  PortForwardDetailResponse,
  PortForwardRecord,
  PortForwardStatus,
  ProviderKind,
  RespondConversationInputPayload,
  SendConversationMessagePayload,
  SendConversationMessageResponse,
  ServiceHealth,
  ShellSessionDetailResponse,
  ShellSessionRecord,
  ShellSessionStatus,
  TaskDetailResponse,
  TaskRecord,
  TaskStatus,
  WorkspaceBrowseRequest,
  WorkspaceBrowseResponse,
  WorkspaceFilePreviewRequest,
  WorkspaceFilePreviewResponse
} from "../types";
import { buildApiUrl } from "./runtime";

type RequestOptions = {
  accessToken?: string;
  init?: RequestInit;
};

type TaskQuery = {
  deviceId?: string;
  status?: TaskStatus | "all";
  provider?: ProviderKind;
  limit?: number;
};

type ShellSessionQuery = {
  deviceId?: string;
  status?: ShellSessionStatus | "all";
  limit?: number;
};

type PortForwardQuery = {
  deviceId?: string;
  status?: PortForwardStatus | "all";
  limit?: number;
};

type AuditQuery = {
  limit?: number;
};

type ConversationQuery = {
  deviceId?: string;
  archived?: boolean;
};

async function requestJson<T>(
  baseUrl: string,
  path: string,
  options?: RequestOptions
): Promise<T> {
  const accessToken = options?.accessToken?.trim();
  const response = await fetch(buildApiUrl(baseUrl, path), {
    headers: {
      "Content-Type": "application/json",
      ...(accessToken ? { Authorization: `Bearer ${accessToken}` } : {}),
      ...(options?.init?.headers ?? {})
    },
    ...options?.init
  });

  if (!response.ok) {
    let message = `${response.status} ${response.statusText}`;
    try {
      const errorBody = (await response.json()) as { message?: string };
      if (errorBody.message) {
        message = `${response.status} ${errorBody.message}`;
      }
    } catch {
      // Ignore non-JSON error bodies.
    }
    throw new Error(message);
  }

  return (await response.json()) as T;
}

function buildTaskPath(query?: TaskQuery) {
  const params = new URLSearchParams();
  if (query?.deviceId) {
    params.set("deviceId", query.deviceId);
  }
  if (query?.status && query.status !== "all") {
    params.set("status", query.status);
  }
  if (query?.provider) {
    params.set("provider", query.provider);
  }
  if (typeof query?.limit === "number") {
    params.set("limit", String(query.limit));
  }

  return params.size ? `/api/tasks?${params.toString()}` : "/api/tasks";
}

function buildShellSessionPath(query?: ShellSessionQuery) {
  const params = new URLSearchParams();
  if (query?.deviceId) {
    params.set("deviceId", query.deviceId);
  }
  if (query?.status && query.status !== "all") {
    params.set("status", query.status);
  }
  if (typeof query?.limit === "number") {
    params.set("limit", String(query.limit));
  }

  return params.size ? `/api/shell/sessions?${params.toString()}` : "/api/shell/sessions";
}

function buildPortForwardPath(query?: PortForwardQuery) {
  const params = new URLSearchParams();
  if (query?.deviceId) {
    params.set("deviceId", query.deviceId);
  }
  if (query?.status && query.status !== "all") {
    params.set("status", query.status);
  }
  if (typeof query?.limit === "number") {
    params.set("limit", String(query.limit));
  }

  return params.size ? `/api/port-forwards?${params.toString()}` : "/api/port-forwards";
}

function buildAuditPath(query?: AuditQuery) {
  const params = new URLSearchParams();
  if (typeof query?.limit === "number") {
    params.set("limit", String(query.limit));
  }

  return params.size ? `/api/audit/events?${params.toString()}` : "/api/audit/events";
}

function buildConversationPath(query?: ConversationQuery) {
  const params = new URLSearchParams();
  if (query?.deviceId) {
    params.set("deviceId", query.deviceId);
  }
  if (typeof query?.archived === "boolean") {
    params.set("archived", String(query.archived));
  }

  return params.size
    ? `/api/conversations?${params.toString()}`
    : "/api/conversations";
}

export function fetchHealth(baseUrl: string) {
  return requestJson<ServiceHealth>(baseUrl, "/api/health");
}

export function fetchAppConfig(baseUrl: string) {
  return requestJson<AppConfig>(baseUrl, "/api/app-config");
}

export function fetchAuditEvents(baseUrl: string, accessToken: string, query?: AuditQuery) {
  return requestJson<AuditRecord[]>(baseUrl, buildAuditPath(query), {
    accessToken
  });
}

export function fetchDevices(baseUrl: string, accessToken: string) {
  return requestJson<DeviceRecord[]>(baseUrl, "/api/devices", {
    accessToken
  });
}

export function fetchTasks(baseUrl: string, accessToken: string, query?: TaskQuery) {
  return requestJson<TaskRecord[]>(baseUrl, buildTaskPath(query), {
    accessToken
  });
}

export function fetchConversations(
  baseUrl: string,
  accessToken: string,
  query?: ConversationQuery
) {
  return requestJson<ConversationRecord[]>(baseUrl, buildConversationPath(query), {
    accessToken
  });
}

export function fetchConversationDetail(
  baseUrl: string,
  conversationId: string,
  accessToken: string
) {
  return requestJson<ConversationDetailResponse>(
    baseUrl,
    `/api/conversations/${conversationId}`,
    {
      accessToken
    }
  );
}

export function createConversation(
  baseUrl: string,
  payload: CreateConversationPayload,
  accessToken: string
) {
  return requestJson<CreateConversationResponse>(baseUrl, "/api/conversations", {
    accessToken,
    init: {
      method: "POST",
      body: JSON.stringify(payload)
    }
  });
}

export function sendConversationMessage(
  baseUrl: string,
  conversationId: string,
  payload: SendConversationMessagePayload,
  accessToken: string
) {
  return requestJson<SendConversationMessageResponse>(
    baseUrl,
    `/api/conversations/${conversationId}/messages`,
    {
      accessToken,
      init: {
        method: "POST",
        body: JSON.stringify(payload)
      }
    }
  );
}

export function archiveConversation(
  baseUrl: string,
  conversationId: string,
  accessToken: string
) {
  return requestJson<ConversationRecord>(
    baseUrl,
    `/api/conversations/${conversationId}/archive`,
    {
      accessToken,
      init: {
        method: "POST"
      }
    }
  );
}

export function respondTaskInputRequest(
  baseUrl: string,
  taskId: string,
  requestId: string,
  payload: RespondConversationInputPayload,
  accessToken: string
) {
  return requestJson<ConversationInputRequest>(
    baseUrl,
    `/api/tasks/${taskId}/input-requests/${requestId}/respond`,
    {
      accessToken,
      init: {
        method: "POST",
        body: JSON.stringify(payload)
      }
    }
  );
}

export function fetchTaskDetail(baseUrl: string, taskId: string, accessToken: string) {
  return requestJson<TaskDetailResponse>(baseUrl, `/api/tasks/${taskId}`, {
    accessToken
  });
}

export async function createTask(
  baseUrl: string,
  payload: CreateTaskPayload,
  accessToken: string
) {
  const response = await requestJson<{ task: TaskRecord }>(baseUrl, "/api/tasks", {
    accessToken,
    init: {
      method: "POST",
      body: JSON.stringify(payload)
    }
  });

  return response.task;
}

export async function cancelTask(baseUrl: string, taskId: string, accessToken: string) {
  return requestJson<TaskDetailResponse>(baseUrl, `/api/tasks/${taskId}/cancel`, {
    accessToken,
    init: {
      method: "POST"
    }
  });
}

export function fetchShellSessions(
  baseUrl: string,
  accessToken: string,
  query?: ShellSessionQuery
) {
  return requestJson<ShellSessionRecord[]>(baseUrl, buildShellSessionPath(query), {
    accessToken
  });
}

export function fetchShellSessionDetail(
  baseUrl: string,
  sessionId: string,
  accessToken: string
) {
  return requestJson<ShellSessionDetailResponse>(baseUrl, `/api/shell/sessions/${sessionId}`, {
    accessToken
  });
}

export async function createShellSession(
  baseUrl: string,
  payload: CreateShellSessionPayload,
  accessToken: string
) {
  const response = await requestJson<{ session: ShellSessionRecord }>(
    baseUrl,
    "/api/shell/sessions",
    {
      accessToken,
      init: {
        method: "POST",
        body: JSON.stringify(payload)
      }
    }
  );

  return response.session;
}

export async function sendShellInput(
  baseUrl: string,
  sessionId: string,
  data: string,
  accessToken: string
) {
  return requestJson<ShellSessionDetailResponse>(
    baseUrl,
    `/api/shell/sessions/${sessionId}/input`,
    {
      accessToken,
      init: {
        method: "POST",
        body: JSON.stringify({ data })
      }
    }
  );
}

export async function closeShellSession(
  baseUrl: string,
  sessionId: string,
  accessToken: string
) {
  return requestJson<ShellSessionDetailResponse>(
    baseUrl,
    `/api/shell/sessions/${sessionId}/close`,
    {
      accessToken,
      init: {
        method: "POST"
      }
    }
  );
}

export function browseWorkspace(
  baseUrl: string,
  payload: WorkspaceBrowseRequest,
  accessToken: string
) {
  return requestJson<WorkspaceBrowseResponse>(baseUrl, "/api/workspace/browse", {
    accessToken,
    init: {
      method: "POST",
      body: JSON.stringify(payload)
    }
  });
}

export function previewWorkspaceFile(
  baseUrl: string,
  payload: WorkspaceFilePreviewRequest,
  accessToken: string
) {
  return requestJson<WorkspaceFilePreviewResponse>(baseUrl, "/api/workspace/preview", {
    accessToken,
    init: {
      method: "POST",
      body: JSON.stringify(payload)
    }
  });
}

export function inspectGitWorkspace(
  baseUrl: string,
  payload: GitInspectRequest,
  accessToken: string
) {
  return requestJson<GitInspectResponse>(baseUrl, "/api/git/inspect", {
    accessToken,
    init: {
      method: "POST",
      body: JSON.stringify(payload)
    }
  });
}

export function fetchGitDiffFile(
  baseUrl: string,
  payload: GitDiffFileRequest,
  accessToken: string
) {
  return requestJson<GitDiffFileResponse>(baseUrl, "/api/git/diff-file", {
    accessToken,
    init: {
      method: "POST",
      body: JSON.stringify(payload)
    }
  });
}

export function createGitWorktree(
  baseUrl: string,
  payload: GitCreateWorktreeRequest,
  accessToken: string
) {
  return requestJson<GitCreateWorktreeResponse>(baseUrl, "/api/git/worktrees", {
    accessToken,
    init: {
      method: "POST",
      body: JSON.stringify(payload)
    }
  });
}

export function removeGitWorktree(
  baseUrl: string,
  payload: GitRemoveWorktreeRequest,
  accessToken: string
) {
  return requestJson<GitRemoveWorktreeResponse>(baseUrl, "/api/git/worktrees/remove", {
    accessToken,
    init: {
      method: "POST",
      body: JSON.stringify(payload)
    }
  });
}

export function fetchPortForwards(
  baseUrl: string,
  accessToken: string,
  query?: PortForwardQuery
) {
  return requestJson<PortForwardRecord[]>(baseUrl, buildPortForwardPath(query), {
    accessToken
  });
}

export async function createPortForward(
  baseUrl: string,
  payload: CreatePortForwardPayload,
  accessToken: string
) {
  const response = await requestJson<{ forward: PortForwardRecord }>(
    baseUrl,
    "/api/port-forwards",
    {
      accessToken,
      init: {
        method: "POST",
        body: JSON.stringify({
          protocol: "tcp",
          ...payload
        })
      }
    }
  );

  return response.forward;
}

export async function closePortForward(
  baseUrl: string,
  forwardId: string,
  accessToken: string
) {
  return requestJson<PortForwardDetailResponse>(
    baseUrl,
    `/api/port-forwards/${forwardId}/close`,
    {
      accessToken,
      init: {
        method: "POST"
      }
    }
  );
}
