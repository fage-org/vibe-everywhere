const en = {
  app: {
    name: "Vibe Everywhere",
    title: "Vibe Everywhere"
  },
  locale: {
    label: "Language",
    zhCN: "中文",
    en: "English"
  },
  theme: {
    label: "Theme",
    system: "System",
    light: "Light",
    dark: "Dark"
  },
  common: {
    connect: "Connect",
    refresh: "Refresh",
    cancel: "Cancel",
    close: "Close",
    online: "online",
    offline: "offline",
    status: "Status",
    scope: "Scope",
    provider: "Provider",
    title: "Title",
    model: "Model",
    prompt: "Prompt",
    protocol: "Protocol",
    created: "Created",
    started: "Started",
    finished: "Finished",
    pending: "Pending",
    allDevices: "All devices",
    selectedDevice: "Selected device",
    allStatus: "All status",
    sendCommand: "Send Command",
    optionalAccessToken: "optional access token",
    useAgentWorkingRoot: "Use agent working root",
    defaultModel: "Default model",
    waiting: "Waiting"
  },
  connectionState: {
    disconnected: "disconnected",
    connecting: "connecting",
    connected: "connected"
  },
  taskStatus: {
    pending: "Pending",
    assigned: "Assigned",
    running: "Running",
    cancel_requested: "Cancel requested",
    succeeded: "Succeeded",
    failed: "Failed",
    canceled: "Canceled"
  },
  shellStatus: {
    pending: "Pending",
    active: "Active",
    close_requested: "Close requested",
    succeeded: "Succeeded",
    failed: "Failed",
    closed: "Closed"
  },
  portForwardStatus: {
    pending: "Pending",
    active: "Active",
    close_requested: "Close requested",
    closed: "Closed",
    failed: "Failed"
  },
  eventKind: {
    system: "System",
    status: "Status",
    provider_stdout: "Provider stdout",
    provider_stderr: "Provider stderr",
    assistant_delta: "Assistant delta",
    tool_call: "Tool call",
    tool_output: "Tool output"
  },
  stream: {
    stdin: "stdin",
    stdout: "stdout",
    stderr: "stderr",
    system: "system"
  },
  transport: {
    relay_tunnel: "relay tunnel",
    overlay_proxy: "overlay proxy"
  },
  gitFileStatus: {
    modified: "Modified",
    added: "Added",
    deleted: "Deleted",
    renamed: "Renamed",
    copied: "Copied",
    updated_but_unmerged: "Conflicted",
    untracked: "Untracked",
    type_changed: "Type changed",
    unknown: "Unknown"
  },
  auth: {
    required: "auth required",
    optional: "auth optional"
  },
  role: {
    owner: "Owner",
    admin: "Admin",
    member: "Member",
    viewer: "Viewer",
    agent: "Agent"
  },
  authMode: {
    disabled: "Disabled",
    access_token: "Access Token"
  },
  storageKind: {
    file: "File",
    memory: "Memory",
    external: "External"
  },
  deploymentMode: {
    self_hosted: "Self-Hosted",
    hosted_compatible: "Hosted-Compatible"
  },
  notificationChannel: {
    in_app: "In-App",
    system: "System"
  },
  auditAction: {
    device_registered: "Device Registered",
    task_created: "Task Created",
    task_canceled: "Task Canceled",
    shell_session_created: "Shell Session Created",
    shell_session_closed: "Shell Session Closed",
    preview_created: "Preview Created",
    preview_closed: "Preview Closed"
  },
  auditOutcome: {
    succeeded: "Succeeded",
    rejected: "Rejected",
    failed: "Failed"
  },
  platform: {
    client: {
      web: "Web",
      tauri_desktop: "Desktop",
      android: "Android"
    }
  },
  error: {
    targetPortInvalid: "Target port must be an integer between 1 and 65535."
  },
  dashboard: {
    heroBadge: "Vue + Tauri 2 + Rust Relay",
    heroDescription:
      "An AI-session-first remote development control plane. Pick a device, launch an AI session, supervise the output, then drop into terminal or preview tunnels only when advanced diagnosis is needed.",
    relayTitle: "Relay",
    relayBaseUrl: "Relay Base URL",
    mobileLoopbackWarning:
      "Mobile clients cannot connect to localhost / 127.0.0.1. Use the relay machine's LAN IP or an HTTPS public address instead.",
    mobileRemoteHint:
      "On mobile, enter the relay machine's LAN IP or an HTTPS public address.",
    activity: {
      title: "Activity Center",
      summary: "{unread} unread of {total} total",
      markAllRead: "Mark All Read",
      empty: "Task completions and preview readiness will appear here.",
      unread: "Unread",
      categories: {
        taskSucceeded: "Task Succeeded",
        taskFailed: "Task Failed",
        taskCanceled: "Task Canceled",
        previewReady: "Preview Ready",
        previewFailed: "Preview Failed"
      },
      messages: {
        taskSucceeded: "\"{title}\" finished on device {deviceId}.",
        taskFailed: "\"{title}\" failed on device {deviceId}.",
        taskCanceled: "\"{title}\" was canceled on device {deviceId}.",
        previewReady: "Preview {host}:{port} is ready on device {deviceId}.",
        previewFailed: "Preview {host}:{port} failed on device {deviceId}."
      }
    },
    deployment: {
      title: "Deployment Surface",
      mode: "Deployment Mode",
      authMode: "Auth Mode",
      storageKind: "Storage",
      currentClient: "Current Client",
      relayOrigin: "Relay Public Origin",
      documentation: "Self-Hosted Docs",
      guidance: {
        selfHosted:
          "This control plane is operating in self-hosted mode. Keep relay origins, tokens, and forward hosts configurable per environment.",
        hostedCompatible:
          "This relay is exposing a hosted-compatible surface. Avoid depending on loopback or single-machine assumptions in clients.",
        explicitRemote:
          "This client should use an explicit remote relay URL. Avoid localhost and prefer the relay machine's LAN or public origin."
      }
    },
    platform: {
      title: "Client Capability Matrix",
      current: "Current",
      available: "Available",
      mobileOptimized: "Mobile Optimized",
      desktopOptimized: "Desktop Optimized",
      systemNotifications: "System Notifications",
      inAppOnly: "In-App Notifications",
      persistedConfig: "Persisted Runtime Config",
      explicitRelay: "Explicit Remote Relay",
      loopbackFriendly: "Loopback Friendly"
    },
    governance: {
      title: "Governance And Audit",
      description:
        "Show the current actor, tenant scope, notification channels, and recent audit events directly in the control plane.",
      tenant: "Tenant",
      user: "User",
      role: "Role",
      notificationChannels: "Notification Channels",
      auditTitle: "Audit Events",
      auditEmpty: "No audit events are available yet for the current tenant."
    },
    stats: {
      onlineDevices: "Online Devices",
      devices: "Devices",
      aiSessions: "AI Sessions",
      advancedTools: "Advanced Tools"
    },
    devices: {
      title: "Devices",
      registered: "{count} registered devices",
      sessions: "Sessions {count}",
      terminals: "Terminal {count}"
    },
    sessions: {
      title: "AI Sessions",
      visibleSummary: "{visible} visible of {total} total",
      empty:
        "No AI sessions match the current filters. Pick a device and start a new session first.",
      start: "Start AI Session",
      cancelSession: "Cancel Session",
      newOnDevice: "New AI Session on {name}",
      readySelected:
        "The device is selected. Fill in the prompt to create an AI session and supervise the event stream, workspace, and follow-up tools here.",
      readyEmpty: "Pick an online device first, then start a new AI session."
    },
    workspace: {
      title: "Session Workspace",
      newTitle: "New AI Session",
      newDescription:
        "Describe the goal, working directory, and optional model override for the selected device.",
      currentTitle: "Current Session",
      promptTitle: "Prompt",
      waitingEvents: "The session was created and is waiting for provider events.",
      workingDirectory: "Working Directory",
      sessionMetrics: "Session Metrics",
      device: "Device",
      workingRoot: "Working Root",
      deviceCapacity: "Device Capacity",
      providers: "{count} providers",
      capacitySummary: "{sessions} sessions · {terminals} terminals · {previews} previews",
      relativePathHint: "Use a relative path in session CWD to stay inside this workspace.",
      exitCode: "Exit {code}",
      eventsSummary: "{count} events · Device {deviceId}",
      supervision: {
        title: "Session Supervision",
        description:
          "Combine provider output signals with Git context before opening terminal or preview.",
        summary: {
          running:
            "The AI session is still running. {changed} changed files are currently visible in this workspace scope.",
          succeededWithChanges:
            "The AI session completed and left {changed} changed files for review on branch {branch}.",
          succeededClean:
            "The AI session completed without visible Git changes in the current workspace scope.",
          failedWithChanges:
            "The AI session failed, but {changed} changed files still remain in the current workspace scope.",
          failedClean:
            "The AI session failed and no Git changes are visible in the current workspace scope.",
          noGit:
            "Git supervision is not available for this workspace yet, so rely on the event stream and workspace preview."
        },
        counts: {
          assistant: "Assistant Messages",
          tool: "Tool Activity",
          stderr: "Provider Errors",
          changed: "Changed Files"
        }
      },
      browser: {
        title: "Workspace Browser",
        description:
          "Browse the selected session workspace without dropping into terminal.",
        unsupported:
          "The selected device does not advertise workspace browse capability yet.",
        loading: "Loading workspace...",
        empty: "This folder is empty.",
        up: "Up",
        root: "Workspace Root",
        path: "Current Path",
        entries: "{count} entries",
        kind: {
          directory: "Directory",
          file: "File"
        },
        sizeLabel: "File Size",
        size: "{size} bytes",
        lines: "Lines",
        previewTitle: "File Preview",
        previewLoading: "Loading file preview...",
        previewEmpty: "Select a file to preview it here.",
        binaryNotice: "This file is binary or non-text and cannot be previewed inline.",
        truncated: "Preview truncated to keep the control plane responsive."
      },
      git: {
        title: "Git Inspect",
        description:
          "Inspect branch state, changed files, and recent commits without dropping into terminal.",
        unsupported: "The selected device does not advertise Git inspect capability yet.",
        loading: "Loading Git context...",
        empty: "Select a device to inspect repository state.",
        workspaceRoot: "Workspace Scope",
        repoRoot: "Repository Root",
        scopePath: "Scope Path",
        branch: "Branch",
        upstream: "Upstream",
        drift: "Upstream Drift",
        noUpstream: "No upstream",
        driftSummary: "ahead {ahead} · behind {behind}",
        state: {
          not_repository: "The current workspace is not inside a Git repository.",
          git_unavailable: "Git is not available on the selected device."
        },
        clean: "No changed files are visible in the current workspace scope.",
        noCommits: "This repository does not have any commits yet.",
        changedFilesTitle: "Changed Files",
        recentCommitsTitle: "Recent Commits",
        preview: "Preview",
        deletedPreviewUnavailable: "Deleted file cannot be previewed.",
        stats: {
          changedFiles: "{count} changed files",
          stagedFiles: "{count} staged",
          unstagedFiles: "{count} unstaged",
          untrackedFiles: "{count} untracked",
          conflictedFiles: "{count} conflicted",
          stagedLines: "+{additions} / -{deletions} staged",
          unstagedLines: "+{additions} / -{deletions} unstaged"
        }
      }
    },
    fields: {
      accessToken: "Access Token",
      provider: "Provider",
      title: "Title",
      sessionCwd: "Session CWD",
      model: "Model",
      prompt: "Prompt",
      terminalCwd: "Terminal CWD",
      targetHost: "Service Host",
      targetPort: "Service Port"
    },
    placeholders: {
      selectProvider: "Select provider",
      sessionTitle: "Ad hoc AI session",
      sessionCwd: "repo/path or /absolute/path",
      model: "optional model override",
      prompt:
        "Tell Codex / Claude Code / OpenCode what to do on the selected device.",
      terminalCwd: "optional working dir",
      terminalInput: "pwd",
      targetHost: "127.0.0.1",
      targetPort: "3000"
    },
    composerNote:
      "Codex / OpenCode prefer ACP. Claude Code still runs through CLI. Terminal and Preview remain below as advanced tools.",
    advanced: {
      badge: "Advanced Tools",
      title: "Terminal And Preview",
      description:
        "These capabilities stay available for environment checks, manual fallback, and preview access, but they are no longer the homepage's primary workflow."
    },
    terminal: {
      title: "Terminal",
      visibleSummary: "{visible} visible of {total} total · WS {state}",
      open: "Open Terminal",
      empty: "No terminal sessions match the current filters.",
      waiting: "The session was created and is waiting for the agent to start the terminal.",
      select: "Pick a terminal session or create a new one for the selected device.",
      noCapability:
        "The selected device does not advertise shell capability, so a terminal session cannot be created.",
      note:
        "Use this for environment checks, manual fallback, and advanced diagnosis. It is not the default product path.",
      detailTitle: "Terminal {id}",
      detailSummary: "{status} · started {time}"
    },
    preview: {
      title: "Preview",
      description:
        "Expose a local web app or HTTP service from the selected device without starting from raw tunnel details.",
      visibleSummary: "{visible} visible of {total} total",
      open: "Create Preview",
      launchTitle: "Launch Preview From This Workspace",
      launchDescription:
        "Use the current device and session context to expose a local HTTP service immediately. Detailed history and raw connection data stay in the advanced panel below.",
      empty:
        "There are no matching previews yet. Select a device and enter the local service host and port to create one.",
      select: "Pick a preview to inspect it here, or create a new one first.",
      note:
        "Preview is the product-facing path for common local web services. Raw relay and target endpoint details remain below for advanced debugging.",
      serviceHint:
        "Start with the common case: a local HTTP service on the selected device. Keep the host editable for non-localhost services.",
      serviceTarget: "Service {host}:{port}",
      previewUrl: "Preview URL",
      openLink: "Open In Browser",
      ready:
        "The preview relay is active. Open it in the browser or inspect the advanced connection details below.",
      waiting:
        "The preview is still being provisioned. Wait for it to become active before opening it.",
      mobileWarning:
        "This preview currently points to a loopback host and will not open from mobile. Reconfigure the relay forward host instead.",
      advancedTitle: "Advanced Connection Details",
      advancedDescription:
        "Raw relay and target endpoints stay here for self-hosted networking checks and low-level troubleshooting.",
      transportLabel: "Transport",
      relayEndpoint: "Relay Endpoint",
      targetEndpoint: "Target Endpoint",
      detailTitle: "Preview {id}"
    }
  }
} as const

export default en
