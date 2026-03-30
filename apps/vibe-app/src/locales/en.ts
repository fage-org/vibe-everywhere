const en = {
  app: {
    title: "Vibe Everywhere"
  },
  nav: {
    home: "Home",
    projects: "Projects",
    notifications: "Notifications",
    settings: "My"
  },
  common: {
    refresh: "Refresh",
    openProject: "Open project",
    viewDetails: "View details",
    all: "All",
    online: "Online",
    offline: "Offline",
    itemsCount: "{count} items",
    projectsCount: "{count} projects",
    refreshedAt: "Refreshed {value}"
  },
  shell: {
    badge: "AI Worktree",
    title: "Remote AI development across hosts and projects",
    serverError: "Server error",
    hostsOnline: "{count} hosts online",
    noHostOnline: "No host online",
    emptyServer: "Configure a relay in My to start browsing hosts and project conversations.",
    runningTasks: "Tasks running: {count}",
    attention: "Attention: {count}"
  },
  projectCard: {
    failed: "Failed",
    waiting: "Waiting",
    running: "Running",
    ready: "Ready",
    changedFiles: "{count} changed",
    inventoryOnly: "New entry",
    availability: {
      available: "Available",
      offline: "Host offline",
      unreachable: "Needs recheck",
      history_only: "History only"
    },
    discovery: {
      working_root: "Host inventory",
      git_worktree: "Git worktree",
      known_project: "Known project"
    },
    topics: "Topics",
    updated: "Updated"
  },
  home: {
    stats: {
      onlineHosts: "Online hosts",
      runningTasks: "Running tasks",
      needsAttention: "Needs attention"
    },
    continueWorkBadge: "Continue work",
    continueWorkEmptyTitle: "Connect a server and open a project.",
    continueWorkEmptySummary: "Once conversations start, the latest project returns here for quick follow-up from mobile.",
    runningNow: "Running now",
    noRunningTasks: "No active task right now. Recent project history and failed work will still appear below.",
    needsReview: "Needs review",
    noReviewProjects: "No failed task or pending confirmation at the moment.",
    recentProjects: "Recent projects"
  },
  projects: {
    title: "Browse by host and project",
    summary: "Move from host status to a concrete worktree-like project context without losing the current path.",
    searchPlaceholder: "Search host, project, or path",
    recentFilter: "Recent",
    hostEmptyOffline: "This host is offline, so its project inventory is not available right now.",
    hostEmptyNoWorkspace: "This host has no workspace root configured yet, so project discovery cannot start.",
    hostEmptyNoProjects: "No Git project was discovered on this host yet. Check the workspace root and repository layout.",
    empty: "No project matches the current filter."
  },
  notifications: {
    badge: "Notifications",
    title: "Return to work that needs a decision",
    summary: "Failed runs, provider questions, and completed tasks surface here so mobile users can re-enter the right project quickly.",
    unreadCount: "{count} unread",
    visibleCount: "{count} visible",
    preferencesTitle: "Project notification preferences",
    preferencesSummary: "Choose whether each project sends all completions or only failures and waiting-input events.",
    defaultPreferenceTitle: "Default preference",
    defaultPreferenceSummary: "Projects without an override inherit this notification policy.",
    preferenceInherited: "Inherited default: {value}",
    preferenceOverride: "Project override: {value}",
    preferenceReset: "Use default",
    preferencesEmpty: "Open some projects first, then their notification preferences will appear here.",
    preferenceImportant: "Failed + waiting",
    preferenceAll: "All activity",
    unread: "Unread",
    unreadSummary: "Open these first to return to work that still needs your decision.",
    recent: "Recent",
    recentSummary: "Already seen items stay here for quick re-entry.",
    newBadge: "New",
    actions: {
      conversation: "Open conversation",
      changes: "Open changes",
      logs: "Open logs"
    },
    empty: "Nothing new yet. Running, failed, and waiting-for-input work will appear here.",
    completed: "Completed"
  },
  settings: {
    badge: "My",
    title: "Server, appearance, and client preferences",
    summary: "The primary workflow stays in projects. Relay settings, locale, and theme stay here as secondary controls.",
    serverTitle: "Server settings",
    serverSummary: "Configure the relay address and control-plane token for this client.",
    relayUrl: "Relay URL",
    accessToken: "Access token",
    accessTokenPlaceholder: "Optional bearer token",
    save: "Save and refresh",
    currentServer: "Current server: {value}",
    notConfigured: "Not configured",
    language: "Language",
    theme: "Theme",
    policy: {
      badge: "Policy",
      title: "Execution policy center",
      summary: "Review which providers are currently visible and what each execution mode now enforces before you send work.",
      providerAvailability: "{online}/{available} online",
      empty: "No provider capability has been detected from the current hosts yet.",
      manageBadge: "Defaults",
      manageTitle: "Policy defaults",
      manageSummary: "These defaults are applied by the current client before per-project or per-task overrides take over.",
      defaultExecutionMode: "Default execution mode",
      defaultNotifications: "Default notification preference",
      sensitiveConfirm: "High-risk confirmation",
      confirmEnabled: "Enabled",
      confirmDisabled: "Disabled"
    },
    audit: {
      badge: "Audit",
      title: "Audit coverage",
      summary: "Current policy and audit coverage is visible here as a read-only summary before a fuller management surface exists.",
      coverageTitle: "Current coverage",
      manageBadge: "Records",
      manageTitle: "Global audit trail",
      manageSummary: "Review recent task, shell, and preview audit events from one global secondary surface.",
      empty: "No audit record matches the current filter.",
      filters: {
        task: "Task",
        shellPreview: "Shell + preview"
      },
      facts: {
        projectLogs: "Project logs show audit records ahead of raw runtime output for the active conversation.",
        taskLifecycle: "Task creation and cancellation actions are already recorded in the audit trail.",
        shellPreview: "Shell and preview lifecycle actions are recorded when those secondary tools are used.",
        secondarySurface: "This is a visibility center only for now; policy editing still lives in runtime defaults and secondary views."
      }
    }
  },
  workspace: {
    badge: "Project",
    title: "Project workspace",
    loadingPath: "Loading project path...",
    currentState: "Current state",
    tabs: {
      conversation: "Conversation",
      changes: "Changes",
      files: "Files",
      logs: "Logs"
    },
    desktop: {
      projects: "Host projects",
      hostTree: "Hosts and projects",
      hostEmpty: "No project is currently visible on this host.",
      worktreeTitle: "New worktree",
      worktreeSummary: "Create a sibling worktree and branch without leaving the current project.",
      worktreeBranch: "Branch name",
      worktreeBranchPlaceholder: "feature/mobile-review",
      worktreeDirectory: "Directory name",
      worktreeDestinationHint: "This creates ../{value} beside the current repository.",
      worktreeSubmit: "Create worktree",
      worktreeCreating: "Creating worktree...",
      worktreeCreated: "Worktree ../{value} is ready.",
      worktreeRemove: "Remove",
      worktreeRemoving: "Removing...",
      worktreeRemoved: "Worktree {value} was removed.",
      worktreeRemoveConfirm: "Remove worktree {value}? This deletes that sibling worktree directory.",
      worktreeBranchRequired: "Enter a branch name before creating a worktree.",
      worktreeDirectoryRequired: "Enter the destination directory name.",
      worktreeList: "Worktrees",
      worktreeCurrent: "Current",
      worktreeDetached: "Detached",
      worktreeStates: {
        current: "Current",
        detached: "Detached",
        inventory_missing: "Inventory missing",
        offline: "Host offline",
        unreachable: "Needs recheck",
        available: "Available",
        remove_failed: "Remove failed"
      }
    },
    metrics: {
      topics: "Topics {count}",
      running: "Running {count}",
      waiting: "Waiting {count}",
      branch: "Branch {value}",
      worktrees: "Worktrees {count}",
      changedFiles: "Changed files {count}",
      updated: "Updated {value}"
    },
    state: {
      failed: "Failure needs review",
      waiting: "Waiting for your input",
      running: "AI is working",
      ready: "Ready for a new turn"
    }
  },
  conversation: {
    topics: "Topics",
    emptyTitle: "No conversation yet",
    emptySummary: "This project is ready. Send the first request to start a conversation in {project}.",
    firstTurnTitle: "Start the first turn",
    firstTurnSummary: "Use the composer below to create the first AI task for {project}, then this transcript will stay attached to the project.",
    executionMode: {
      title: "Execution mode",
      readOnly: "Read only",
      readOnlySummary: "Inspect and explain without changing files.",
      workspaceWrite: "Workspace write",
      workspaceWriteSummary: "Allow file edits in the current workspace, but avoid tests unless needed later.",
      workspaceWriteAndTest: "Write + test",
      workspaceWriteAndTestSummary: "Allow file edits and focused verification commands."
    },
    executionModeMeta: {
      read_only: "read only",
      workspace_write: "workspace write",
      workspace_write_and_test: "write + test"
    },
    policyTitle: "Effective enforcement",
    policySummary: {
      acp: {
        readOnly: "ACP runtime blocks file writes and terminal sessions.",
        workspaceWrite: "ACP runtime allows workspace edits but blocks terminal test commands.",
        workspaceWriteAndTest: "ACP runtime allows workspace edits and focused terminal verification."
      },
      codex: {
        readOnly: "Codex runs with a native read-only sandbox and no approval prompts.",
        workspaceWrite: "Codex runs with a workspace-write sandbox and approval on untrusted actions.",
        workspaceWriteAndTest: "Codex runs with a workspace-write sandbox and auto-approves focused verification."
      },
      claude: {
        readOnly: "Claude runs in native plan mode and blocks edit/shell tools by default.",
        workspaceWrite: "Claude can edit the workspace, but common test commands are blocked by default.",
        workspaceWriteAndTest: "Claude can edit the workspace and test-command blocking is lifted for this task."
      },
      generic: {
        readOnly: "This mode is requested as read-only, but enforcement depends on the active provider.",
        workspaceWrite: "This mode allows workspace edits, while stronger enforcement depends on the active provider.",
        workspaceWriteAndTest: "This mode allows workspace edits and verification, while stronger enforcement depends on the active provider."
      }
    },
    statusLabel: {
      pending: "Pending",
      assigned: "Assigned",
      running: "Running",
      waiting_input: "Waiting input",
      cancel_requested: "Cancel requested",
      succeeded: "Succeeded",
      failed: "Failed",
      canceled: "Canceled"
    },
    statusSummary: {
      pending: "Task was queued and has not started yet.",
      assigned: "Task was assigned and is waiting to start.",
      running: "Task is actively running.",
      waiting_input: "Task is waiting for your reply before continuing.",
      cancel_requested: "Cancellation was requested and the runtime is winding down.",
      succeeded: "Task finished successfully.",
      failed: "Task failed before returning assistant text.",
      canceled: "Task was canceled before completion."
    },
    eventSummary: {
      idle: "No machine events were captured for this turn yet.",
      status: "{count} status update(s)",
      toolCalls: "{count} tool call(s)",
      toolOutputs: "{count} tool result(s)",
      errors: "{count} stderr event(s)"
    },
    eventStreamTitle: "Recent execution events",
    rawEventsTitle: "Detailed event output",
    waitingInput: "Waiting input",
    stopTask: "Stop task",
    retryTask: "Retry",
    explainResult: "Explain result",
    viewChanges: "View changes",
    viewLogs: "View logs",
    replyPlaceholder: "Type a reply",
    sendCustomReply: "Send custom reply",
    optionalModel: "Optional model",
    promptPlaceholder: "Tell the AI what to do in this project...",
    send: "Send",
    sensitiveConfirmTitle: "Sensitive action",
    sensitiveConfirmSummary: "This prompt looks like it may trigger a destructive or high-risk command.",
    sensitiveConfirmDetail: "Review the prompt and execution mode, then confirm if you still want to send it.",
    confirmSend: "Confirm and send",
    cancelConfirm: "Cancel"
  },
  changes: {
    branch: "Branch",
    changedFiles: "Changed files",
    aheadBehind: "Ahead / Behind",
    untracked: "Untracked",
    staged: "Staged",
    unstaged: "Unstaged",
    reviewTitle: "Review summary",
    reviewSummary: "Check scope and risk before reading the raw patch.",
    reviewNeedsAttention: "Needs review",
    reviewClean: "Clean",
    reviewUnavailableTitle: "Git context unavailable",
    reviewUnavailableSummary: "The current workspace did not return a reviewable Git state yet.",
    reviewCleanTitle: "No local changes",
    reviewCleanSummary: "This workspace is currently clean, so there is nothing to review.",
    reviewScopeTitle: "Change scope",
    reviewScopeSummary: "{count} file(s) changed on branch {branch}.",
    reviewConflictTitle: "Conflicts present",
    reviewConflictSummary: "{count} conflicted file(s) need explicit attention before approval.",
    reviewDeleteTitle: "Deletion risk",
    reviewDeleteSummary: "At least one file was deleted. Verify intent before continuing.",
    reviewNewFilesTitle: "New file review",
    reviewNewFilesSummary: "{count} newly added or untracked file(s) need a structure check.",
    reviewMixedTitle: "Mixed staged and unstaged changes",
    reviewMixedSummary: "This project contains both staged and unstaged edits. Review both sections carefully.",
    riskConflict: "Conflict",
    riskDelete: "Delete",
    riskNewFile: "New file",
    riskTypeChange: "Type change",
    riskStandard: "Standard",
    diffTitle: "File diff",
    diffEmpty: "Select a file to inspect its patch.",
    diffUnavailable: "No diff output is available for the selected file yet.",
    diffStaged: "Staged diff",
    diffUnstaged: "Unstaged diff",
    diffTruncated: "Diff output was truncated to keep the mobile review panel responsive.",
    binary: "Binary patch",
    noChanges: "No local Git changes were reported.",
    recentCommits: "Recent commits",
    noCommits: "No recent commit history is available."
  },
  files: {
    title: "Files",
    truncated: "truncated preview",
    empty: "Pick a file to preview it here."
  },
  logs: {
    empty: "No runtime logs are available yet.",
    errorSummaryTitle: "Error summary",
    errorSummaryBody: "Recent failed or stderr-heavy runs are grouped here first.",
    errorFallback: "This task failed without a structured error message.",
    noFilteredEvents: "No log events match the current filter for this task.",
    audit: {
      title: "Audit trail",
      summary: "User-visible control actions for this conversation and its tasks appear here first.",
      outcomes: {
        succeeded: "Succeeded",
        rejected: "Rejected",
        failed: "Failed"
      },
      actions: {
        device_registered: "Device registered",
        task_created: "Task created",
        task_canceled: "Task canceled",
        shell_session_created: "Shell session created",
        shell_session_closed: "Shell session closed",
        preview_created: "Preview created",
        preview_closed: "Preview closed"
      }
    },
    filters: {
      all: "All",
      errors: "Errors",
      tools: "Tools",
      provider: "Provider"
    }
  }
};

export default en;
