import type { ConversationRecord, DeviceRecord, ProviderKind, TaskRecord } from "@/types";

export type ProjectDiscoverySource = "working_root" | "git_worktree" | "known_project";
export type ProjectAvailabilityState =
  | "available"
  | "offline"
  | "unreachable"
  | "history_only";

export type ProjectSummary = {
  key: string;
  deviceId: string;
  deviceName: string;
  cwd: string | null;
  repoRoot: string | null;
  repoCommonDir: string | null;
  title: string;
  pathLabel: string;
  updatedAtEpochMs: number;
  branchName: string | null;
  changedFilesCount: number;
  conversationCount: number;
  runningTaskCount: number;
  waitingInputCount: number;
  failedTaskCount: number;
  changedTaskCount: number;
  latestConversationId: string | null;
  latestTaskId: string | null;
  latestTaskStatus: TaskRecord["status"] | null;
  providers: ProviderKind[];
  discoverySource: ProjectDiscoverySource | null;
  lastVerifiedAtEpochMs: number | null;
  availabilityState: ProjectAvailabilityState;
  source: "inventory" | "history" | "merged";
};

export type HostSummary = {
  device: DeviceRecord;
  projectCount: number;
  runningTaskCount: number;
  waitingInputCount: number;
  failedTaskCount: number;
};

export type ProjectTreeNode = {
  id: string;
  label: string;
  kind: "group" | "project";
  project?: ProjectSummary;
  children: ProjectTreeNode[];
};

const DEFAULT_PROJECT_PATH = "__default__";

export function buildProjectKey(deviceId: string, cwd: string | null) {
  return `${deviceId}::${cwd ?? DEFAULT_PROJECT_PATH}`;
}

export function buildProjectRouteParam(cwd: string | null) {
  return cwd ? encodeURIComponent(cwd) : DEFAULT_PROJECT_PATH;
}

export function parseProjectRouteParam(value: string | string[] | undefined) {
  if (!value) {
    return null;
  }

  const normalized = Array.isArray(value) ? value.join("/") : value;
  return normalized === DEFAULT_PROJECT_PATH ? null : decodeURIComponent(normalized);
}

export type DiscoveredProjectRecord = {
  deviceId: string;
  cwd: string;
  repoRoot: string | null;
  repoCommonDir: string | null;
  pathLabel: string;
  title: string;
  updatedAtEpochMs: number;
  branchName: string | null;
  changedFilesCount: number;
  providers: ProviderKind[];
  discoverySource: ProjectDiscoverySource;
  lastVerifiedAtEpochMs: number | null;
  availabilityState: Exclude<ProjectAvailabilityState, "history_only">;
};

export function deriveProjectSummaries(
  devices: DeviceRecord[],
  conversations: ConversationRecord[],
  tasks: TaskRecord[],
  discoveredProjects: DiscoveredProjectRecord[] = []
) {
  const deviceById = new Map(devices.map((device) => [device.id, device]));
  const groups = new Map<string, ProjectSummary>();

  for (const discovered of discoveredProjects) {
    const device = deviceById.get(discovered.deviceId);
    if (!device) {
      continue;
    }

    const key = buildProjectKey(discovered.deviceId, discovered.cwd);
    groups.set(key, {
      key,
      deviceId: discovered.deviceId,
      deviceName: device.name,
      cwd: discovered.cwd,
      repoRoot: discovered.repoRoot,
      repoCommonDir: discovered.repoCommonDir,
      title: discovered.title,
      pathLabel: discovered.pathLabel,
      updatedAtEpochMs: discovered.updatedAtEpochMs,
      branchName: discovered.branchName,
      changedFilesCount: discovered.changedFilesCount,
      conversationCount: 0,
      runningTaskCount: 0,
      waitingInputCount: 0,
      failedTaskCount: 0,
      changedTaskCount: 0,
      latestConversationId: null,
      latestTaskId: null,
      latestTaskStatus: null,
      providers: discovered.providers,
      discoverySource: discovered.discoverySource,
      lastVerifiedAtEpochMs: discovered.lastVerifiedAtEpochMs,
      availabilityState: discovered.availabilityState,
      source: "inventory"
    });
  }

  for (const conversation of conversations) {
    const device = deviceById.get(conversation.deviceId);
    if (!device) {
      continue;
    }

    const key = buildProjectKey(conversation.deviceId, conversation.cwd);
    const existing = groups.get(key);
    const updatedAtEpochMs = Math.max(
      existing?.updatedAtEpochMs ?? 0,
      conversation.updatedAtEpochMs
    );
    const providers = new Set(existing?.providers ?? []);
    providers.add(conversation.provider);

    groups.set(key, {
      key,
      deviceId: conversation.deviceId,
      deviceName: device.name,
      cwd: conversation.cwd,
      repoRoot: existing?.repoRoot ?? null,
      repoCommonDir: existing?.repoCommonDir ?? null,
      title: inferProjectTitle(conversation.cwd),
      pathLabel: conversation.cwd ?? inferDefaultWorkspaceLabel(device),
      updatedAtEpochMs,
      branchName: existing?.branchName ?? null,
      changedFilesCount: existing?.changedFilesCount ?? 0,
      conversationCount: (existing?.conversationCount ?? 0) + 1,
      runningTaskCount: existing?.runningTaskCount ?? 0,
      waitingInputCount: existing?.waitingInputCount ?? 0,
      failedTaskCount: existing?.failedTaskCount ?? 0,
      changedTaskCount: existing?.changedTaskCount ?? 0,
      latestConversationId:
        selectLaterConversation(conversation.id, existing?.latestConversationId, conversations) ??
        conversation.id,
      latestTaskId: existing?.latestTaskId ?? conversation.latestTaskId,
      latestTaskStatus: existing?.latestTaskStatus ?? null,
      providers: [...providers],
      discoverySource: existing?.discoverySource ?? null,
      lastVerifiedAtEpochMs: existing?.lastVerifiedAtEpochMs ?? null,
      availabilityState: existing?.availabilityState ?? "history_only",
      source: existing ? "merged" : "history"
    });
  }

  for (const task of tasks) {
    const device = deviceById.get(task.deviceId);
    if (!device) {
      continue;
    }

    const key = buildProjectKey(task.deviceId, task.cwd);
    const existing = groups.get(key);
    const providers = new Set(existing?.providers ?? []);
    providers.add(task.provider);

    groups.set(key, {
      key,
      deviceId: task.deviceId,
      deviceName: device.name,
      cwd: task.cwd,
      repoRoot: existing?.repoRoot ?? null,
      repoCommonDir: existing?.repoCommonDir ?? null,
      title: inferProjectTitle(task.cwd),
      pathLabel: task.cwd ?? inferDefaultWorkspaceLabel(device),
      updatedAtEpochMs: Math.max(existing?.updatedAtEpochMs ?? 0, task.createdAtEpochMs),
      branchName: existing?.branchName ?? null,
      changedFilesCount: existing?.changedFilesCount ?? 0,
      conversationCount: existing?.conversationCount ?? 0,
      runningTaskCount:
        (existing?.runningTaskCount ?? 0) + (task.status === "running" ? 1 : 0),
      waitingInputCount:
        (existing?.waitingInputCount ?? 0) + (task.status === "waiting_input" ? 1 : 0),
      failedTaskCount: (existing?.failedTaskCount ?? 0) + (task.status === "failed" ? 1 : 0),
      changedTaskCount:
        (existing?.changedTaskCount ?? 0) +
        (task.status === "succeeded" || task.status === "failed" ? 1 : 0),
      latestConversationId: existing?.latestConversationId ?? task.conversationId,
      latestTaskId: chooseLatestTaskId(task, existing?.latestTaskId, tasks),
      latestTaskStatus: chooseLatestTaskStatus(task, existing?.latestTaskId, tasks),
      providers: [...providers],
      discoverySource: existing?.discoverySource ?? null,
      lastVerifiedAtEpochMs: existing?.lastVerifiedAtEpochMs ?? null,
      availabilityState: existing?.availabilityState ?? "history_only",
      source: existing ? "merged" : "history"
    });
  }

  return [...groups.values()].sort((left, right) => right.updatedAtEpochMs - left.updatedAtEpochMs);
}

export function deriveHostSummaries(
  devices: DeviceRecord[],
  projectSummaries: ProjectSummary[]
) {
  const grouped = new Map<string, HostSummary>();

  for (const device of devices) {
    grouped.set(device.id, {
      device,
      projectCount: 0,
      runningTaskCount: 0,
      waitingInputCount: 0,
      failedTaskCount: 0
    });
  }

  for (const project of projectSummaries) {
    const summary = grouped.get(project.deviceId);
    if (!summary) {
      continue;
    }

    summary.projectCount += 1;
    summary.runningTaskCount += project.runningTaskCount;
    summary.waitingInputCount += project.waitingInputCount;
    summary.failedTaskCount += project.failedTaskCount;
  }

  return [...grouped.values()].sort((left, right) => {
    if (left.device.online !== right.device.online) {
      return left.device.online ? -1 : 1;
    }
    return left.device.name.localeCompare(right.device.name);
  });
}

export function inferProjectTitle(cwd: string | null) {
  if (!cwd) {
    return "Default workspace";
  }

  const parts = cwd.split("/").filter(Boolean);
  return parts.length ? parts[parts.length - 1] : cwd;
}

export function buildProjectTree(projects: ProjectSummary[], workspaceRoot?: string | null) {
  const root: ProjectTreeNode[] = [];
  const groupIndex = new Map<string, ProjectTreeNode>();
  const worktreeFamilyCounts = new Map<string, number>();

  for (const project of projects) {
    if (!project.repoCommonDir) {
      continue;
    }
    worktreeFamilyCounts.set(
      project.repoCommonDir,
      (worktreeFamilyCounts.get(project.repoCommonDir) ?? 0) + 1
    );
  }

  for (const project of projects) {
    const familyCount = project.repoCommonDir
      ? worktreeFamilyCounts.get(project.repoCommonDir) ?? 0
      : 0;
    if (project.repoCommonDir && familyCount > 1) {
      const familyId = `repo:${project.repoCommonDir}`;
      let familyGroup = groupIndex.get(familyId);
      if (!familyGroup) {
        familyGroup = {
          id: familyId,
          label: inferProjectTitle(project.repoRoot ?? project.title),
          kind: "group",
          children: []
        };
        root.push(familyGroup);
        groupIndex.set(familyId, familyGroup);
      }

      familyGroup.children.push({
        id: `project:${project.key}`,
        label: inferProjectTitle(project.cwd),
        kind: "project",
        project,
        children: []
      });
      continue;
    }

    const segments = projectPathSegments(project, workspaceRoot);
    if (!segments.length) {
      root.push({
        id: `project:${project.key}`,
        label: project.title,
        kind: "project",
        project,
        children: []
      });
      continue;
    }

    let currentNodes = root;
    let prefix = "";
    for (let index = 0; index < segments.length - 1; index += 1) {
      prefix = `${prefix}/${segments[index]}`;
      const groupId = `group:${prefix}`;
      let group = groupIndex.get(groupId);
      if (!group) {
        group = {
          id: groupId,
          label: segments[index],
          kind: "group",
          children: []
        };
        currentNodes.push(group);
        groupIndex.set(groupId, group);
      }
      currentNodes = group.children;
    }

    currentNodes.push({
      id: `project:${project.key}`,
      label: project.title,
      kind: "project",
      project,
      children: []
    });
  }

  sortProjectTree(root);
  return root;
}

function inferDefaultWorkspaceLabel(device: DeviceRecord) {
  return (
    device.metadata.workspace_root ??
    device.metadata.working_root ??
    device.metadata.cwd ??
    "Default workspace"
  );
}

function projectPathSegments(project: ProjectSummary, workspaceRoot?: string | null) {
  if (!project.cwd) {
    return [];
  }

  const cwdParts = normalizePathSegments(project.cwd);
  const rootParts = normalizePathSegments(workspaceRoot ?? "");
  const relativeParts =
    rootParts.length && cwdParts.slice(0, rootParts.length).join("/") === rootParts.join("/")
      ? cwdParts.slice(rootParts.length)
      : cwdParts;

  return relativeParts.length ? relativeParts : [project.title];
}

function normalizePathSegments(path: string) {
  return path
    .split("/")
    .map((segment) => segment.trim())
    .filter(Boolean);
}

function sortProjectTree(nodes: ProjectTreeNode[]) {
  nodes.sort((left, right) => {
    if (left.kind !== right.kind) {
      return left.kind === "group" ? -1 : 1;
    }
    return left.label.localeCompare(right.label);
  });

  for (const node of nodes) {
    sortProjectTree(node.children);
  }
}

function chooseLatestTaskId(task: TaskRecord, currentId: string | null | undefined, tasks: TaskRecord[]) {
  if (!currentId) {
    return task.id;
  }

  const current = tasks.find((entry) => entry.id === currentId);
  if (!current || task.createdAtEpochMs >= current.createdAtEpochMs) {
    return task.id;
  }

  return current.id;
}

function chooseLatestTaskStatus(
  task: TaskRecord,
  currentId: string | null | undefined,
  tasks: TaskRecord[]
) {
  if (!currentId) {
    return task.status;
  }

  const current = tasks.find((entry) => entry.id === currentId);
  if (!current || task.createdAtEpochMs >= current.createdAtEpochMs) {
    return task.status;
  }

  return current.status;
}

function selectLaterConversation(
  candidateId: string,
  currentId: string | null | undefined,
  conversations: ConversationRecord[]
) {
  if (!currentId) {
    return candidateId;
  }

  const current = conversations.find((entry) => entry.id === currentId);
  const candidate = conversations.find((entry) => entry.id === candidateId);
  if (!candidate) {
    return currentId;
  }
  if (!current || candidate.updatedAtEpochMs >= current.updatedAtEpochMs) {
    return candidate.id;
  }

  return current.id;
}
