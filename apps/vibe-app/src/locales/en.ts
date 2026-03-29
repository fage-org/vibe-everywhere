const en = {
  app: {
    name: "Vibe Everywhere",
    title: "Vibe Everywhere",
  },
  locale: {
    label: "Language",
    zhCN: "中文",
    en: "English",
  },
  theme: {
    label: "Theme",
    system: "System",
    light: "Light",
    dark: "Dark",
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
    waiting: "Waiting",
  },
  connectionState: {
    disconnected: "disconnected",
    connecting: "connecting",
    connected: "connected",
  },
  taskStatus: {
    pending: "Pending",
    assigned: "Assigned",
    running: "Running",
    waiting_input: "Waiting for input",
    cancel_requested: "Cancel requested",
    succeeded: "Succeeded",
    failed: "Failed",
    canceled: "Canceled",
  },
  shellStatus: {
    pending: "Pending",
    active: "Active",
    close_requested: "Close requested",
    succeeded: "Succeeded",
    failed: "Failed",
    closed: "Closed",
  },
  portForwardStatus: {
    pending: "Pending",
    active: "Active",
    close_requested: "Close requested",
    closed: "Closed",
    failed: "Failed",
  },
  eventKind: {
    system: "System",
    status: "Status",
    provider_stdout: "Provider stdout",
    provider_stderr: "Provider stderr",
    assistant_delta: "Assistant delta",
    tool_call: "Tool call",
    tool_output: "Tool output",
  },
  stream: {
    stdin: "stdin",
    stdout: "stdout",
    stderr: "stderr",
    system: "system",
  },
  transport: {
    relay_tunnel: "relay tunnel",
    overlay_proxy: "overlay proxy",
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
    unknown: "Unknown",
  },
  auth: {
    required: "auth required",
    optional: "auth optional",
  },
  role: {
    owner: "Owner",
    admin: "Admin",
    member: "Member",
    viewer: "Viewer",
    agent: "Agent",
  },
  authMode: {
    disabled: "Disabled",
    access_token: "Access Token",
  },
  storageKind: {
    file: "File",
    memory: "Memory",
    external: "External",
  },
  deploymentMode: {
    self_hosted: "Self-Hosted",
    hosted_compatible: "Hosted-Compatible",
  },
  notificationChannel: {
    in_app: "In-App",
    system: "System",
  },
  auditAction: {
    device_registered: "Device Registered",
    task_created: "Task Created",
    task_canceled: "Task Canceled",
    shell_session_created: "Shell Session Created",
    shell_session_closed: "Shell Session Closed",
    preview_created: "Preview Created",
    preview_closed: "Preview Closed",
  },
  auditOutcome: {
    succeeded: "Succeeded",
    rejected: "Rejected",
    failed: "Failed",
  },
  platform: {
    client: {
      web: "Web",
      tauri_desktop: "Desktop",
      android: "Android",
    },
  },
  error: {
    targetPortInvalid: "Target port must be an integer between 1 and 65535.",
  },
  appShell: {
    nav: {
      chat: "Chat",
      devices: "Devices",
      menu: "Menu",
    },
  },
  chatHome: {
    badge: "Remote Projects",
    title: "Pick a device and reopen the project where the AI work lives.",
    summary:
      "The home surface lists each connected device and the project folders that already have conversation history. Open a project to continue its topics like a chat app instead of returning to a control dashboard.",
    connectionState: "Relay",
    onlineDevices: "Online devices",
    totalProjects: "Projects",
    serverSettings: "Server settings",
    deviceProjectCount: "{count} projects",
    defaultWorkspaceEntry: "Default workspace",
    defaultWorkspacePath: "Default workspace",
    topicCount: "{count} topics",
    emptyProjects: "No project history yet. Start from the device's default workspace.",
  },
  chatProject: {
    badge: "Project Chat",
    title: "Project chat",
    defaultWorkspaceTitle: "Default Workspace",
    defaultWorkspacePath: "Default workspace",
    newTopic: "New topic",
    archive: "Archive",
    topicCount: "{count} topics",
    traceCount: "{count} trace events",
    waitingInput: "Waiting for your reply.",
    traceOnlyFailed: "The run failed. Open trace tools from secondary surfaces if you need details.",
    traceOnlyCompleted: "Run completed without assistant text.",
    traceOnlyCanceled: "Run canceled.",
    generating: "Generating...",
    emptyBadge: "Ready",
    emptyTitle: "Start the first topic in this project.",
    emptySummary:
      "This project keeps the work directory stable while topics stay separate. Send a prompt below to create the first conversation.",
    inputRequestBadge: "Needs Your Input",
    customReply: "Custom reply",
    customReplyPlaceholder: "Type a free-form reply for the provider.",
    submitReply: "Submit reply",
    modelPlaceholder: "Optional model override",
    promptPlaceholder: "Describe the next task for this project...",
    followupSummary: "Follow-up messages continue the selected topic in the same provider thread.",
    newTopicSummary: "New topics inherit this device and project folder.",
    send: "Send",
    historyTitle: "Topic history",
    historySummary: "Only topics from the current device and project are shown here.",
    emptyHistory: "There are no topics in this project yet.",
    userTurn: "You",
  },
  menuPage: {
    badge: "Menu",
    title: "Menu",
    summary:
      "Settings and secondary capabilities live here so the main product flow stays focused on device and project chats.",
    serverSettingsTitle: "Settings",
    serverSettingsSummary:
      "Configure the relay address, access token, language, and theme.",
    secondaryToolsTitle: "Secondary tools",
    secondaryToolsSummary:
      "Terminal, preview, and low-level diagnostics remain secondary surfaces rather than top-level navigation.",
    secondaryToolsBadge: "Secondary only",
  },
  settingsPage: {
    badge: "Settings",
    title: "Server Settings",
    summary:
      "Configure how this client connects to the relay and keep appearance preferences in one place.",
    serverTitle: "Relay Connection",
    serverSummary: "Relay URL and access token follow the persisted client configuration.",
    saveAndConnect: "Save and connect",
    localeTitle: "Language",
    themeTitle: "Theme",
    currentServer: "Current relay",
    notConnected: "Not connected",
  },
  dashboard: {
    nav: {
      sessions: "AI Sessions",
      devices: "Devices",
      advanced: "Advanced",
    },
    navDescriptions: {
      sessions:
        "Connect once, pick a device, start AI work, and review the outcome in one flow.",
      devices:
        "Inspect device inventory, runtime state, deployment metadata, and provider readiness.",
      advanced:
        "Use terminal and preview tools only when deeper diagnosis is required.",
    },
    shell: {
      badge: "Operator Console",
      description:
        "Route-backed sections keep the core workflows separated, while the same information architecture adapts to desktop and mobile.",
      connectionState: "Relay Stream",
      selectedDevice: "Selected Device",
      noDeviceSelected: "No device selected",
    },
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
        previewFailed: "Preview Failed",
      },
      messages: {
        taskSucceeded: '"{title}" finished on device {deviceId}.',
        taskFailed: '"{title}" failed on device {deviceId}.',
        taskCanceled: '"{title}" was canceled on device {deviceId}.',
        previewReady: "Preview {host}:{port} is ready on device {deviceId}.",
        previewFailed: "Preview {host}:{port} failed on device {deviceId}.",
      },
    },
    deployment: {
      title: "Deployment Surface",
      summary:
        "Review the current deployment metadata and connection endpoints here. Detailed self-hosted guidance stays in documentation.",
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
          "This client should use an explicit remote relay URL. Avoid localhost and prefer the relay machine's LAN or public origin.",
      },
    },
    platform: {
      title: "Current Client",
      summary:
        "Platform capabilities are shown from the detected runtime only. Other platforms stay hidden on the main surface to avoid reading this as a switcher.",
      currentClientLabel: "Currently In Use",
      currentlyUsing: "In Use",
      mobileOptimized: "Mobile Optimized",
      desktopOptimized: "Desktop Optimized",
      systemNotifications: "System Notifications",
      inAppOnly: "In-App Notifications",
      persistedConfig: "Persisted Runtime Config",
      sessionOnlyConfig: "Session-Only Config",
      explicitRelay: "Explicit Remote Relay",
      loopbackFriendly: "Loopback Friendly",
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
      auditEmpty: "No audit events are available yet for the current tenant.",
    },
    stats: {
      onlineDevices: "Online Devices",
      devices: "Devices",
      aiSessions: "AI Sessions",
      advancedTools: "Advanced Tools",
      unreadActivity: "Unread Activity",
    },
    devices: {
      title: "Devices",
      registered: "{count} registered devices",
      sessions: "Sessions {count}",
      terminals: "Terminal {count}",
      previews: "Preview {count}",
      availableProviders: "Available Providers",
      inventoryTitle: "Device Inventory",
      inventoryDescription:
        "Choose a device to inspect its runtime profile before launching AI work or advanced tools.",
      runtimeTitle: "Runtime Profile",
      runtimeDescription:
        "Review platform, working root, last heartbeat, and overlay connectivity for the selected device.",
      platform: "Platform",
      workingRoot: "Working Root",
      lastSeen: "Last Seen",
      currentTask: "Current Task",
      overlayMode: "Overlay Mode",
      overlayState: "Overlay State",
      overlayRelay: "Overlay Relay",
      capabilitiesTitle: "Capabilities And Providers",
      capabilitiesDescription:
        "Keep raw capability labels visible so the current runtime behavior stays aligned with what the agent actually reports.",
      capabilitiesLabel: "Reported Capabilities",
      capabilitiesEmpty:
        "This device has not reported any capability tags yet.",
      providersTitle: "Provider Availability",
      providerAvailable: "Available",
      providerUnavailable: "Unavailable",
      providerVersion: "Version",
      providerVersionPending: "Version pending",
      workloadTitle: "Live Workload",
      workloadDescription:
        "Track how many AI sessions, terminals, and previews are currently associated with the selected device.",
      managementDescription:
        "Use this secondary view for runtime inspection, deployment metadata, and governance context.",
      emptySelection:
        "Select a device from the inventory to inspect its runtime details, workload, and provider availability.",
    },
    sessions: {
      title: "AI Sessions",
      primaryBadge: "Primary Workflow",
      primaryDescription:
        "Keep the everyday path on one surface: connect to the relay, choose a device, launch an AI session, then review the resulting workspace and Git changes before dropping into advanced tools.",
      steps: {
        connect: "Connect Relay",
        chooseDevice: "Choose Device",
        start: "Start Session",
        review: "Review Result",
      },
      devicePickerTitle: "Choose A Device",
      devicePickerDescription:
        "Pick the machine that should run the next AI session. Runtime capability and provider readiness stay visible here.",
      recentTitle: "Recent Sessions",
      launchTitle: "Start A New Session",
      launchDescription:
        "Use the selected device context to launch the next AI run without leaving the primary workflow.",
      launchState: {
        needs_relay:
          "Connect to a relay first so devices and sessions can load.",
        needs_device: "Pick an online device to unlock the session composer.",
        device_offline:
          "The selected device is offline. Choose another device or wait for it to reconnect.",
        needs_provider:
          "This device has no available providers right now. Resolve provider readiness before starting a session.",
        ready:
          "Ready to launch a session on {device}. Fill in the prompt and start the run.",
      },
      providerIssuesTitle: "Provider Readiness Issues",
      visibleSummary: "{visible} visible of {total} total",
      empty:
        "No AI sessions match the current filters. Pick a device and start a new session first.",
      start: "Start AI Session",
      cancelSession: "Cancel Session",
      newOnDevice: "New AI Session on {name}",
      reviewTitle: "Current Session Review",
      reviewDescription:
        "Track the live run, confirm where it executed, and keep the prompt and outcome summary together.",
      resultReviewTitle: "Review Changed Files And Git Context",
      resultReviewDescription:
        "Judge what the AI changed before opening a terminal or handing the work off.",
      eventStreamTitle: "Session Event Stream",
      eventStreamDescription:
        "Use the event stream as the execution narrative while the result review stays focused on outcome.",
      readySelected:
        "The device is selected. Launch a session here, then review the result, event stream, and workspace without leaving the main workflow.",
      readyEmpty: "Pick an online device first, then start a new AI session.",
    },
    chat: {
      liveBadge: "Conversation First",
      setupTitle: "Connect Once, Then Stay In Chat",
      threadControl: "Long-Lived Thread Control",
      sidebarTitle: "Project And Threads",
      setupSummary:
        "Relay, device, provider, and thread target stay in one compact panel until the first session is ready.",
      sidebarSummary:
        "Keep relay setup, device choice, project folder, thread history, and file browsing in the sidebar so the main pane stays focused on chat.",
      threadSummary:
        "The chat stays primary. Relay switching and new-thread targeting are compact secondary controls.",
      contextControls: "Thread Context",
      hideContextControls: "Hide Context",
      relayHintLabel: "Relay",
      threadSwitcherTitle: "Thread Switcher",
      newConversation: "New Conversation",
      autoTitleHint: "Auto-generate from first prompt",
      cwd: "Working Directory",
      projectFolder: "Project Folder",
      projectFolderSummary:
        "New conversations use this folder by default after it is set.",
      devicesEntry: "Devices & Settings",
      switchRelay: "Switch relay or token",
      reconnect: "Reconnect",
      historyTitle: "Conversation History",
      historySummary: "Durable provider-native threads, newest first.",
      startBlank: "Start a fresh thread",
      startBlankSummary:
        "Clear the current selection and send the next prompt as a new long-lived conversation.",
      activeThread: "Active Thread",
      newThread: "Draft Thread",
      composeTitle: "Start a durable AI conversation",
      composeSummary: "Continuing on {device} with {provider}.",
      composeEmptySummary:
        "Write the first prompt here. After creation, later turns continue on the provider's native thread.",
      archive: "Archive",
      userTurn: "You",
      waitingInput:
        "The provider is waiting for your decision before it can continue.",
      generating: "The agent is still generating the next response.",
      traceOnlyCompleted:
        "This turn completed without a normal assistant message. Open Trace for runtime details.",
      traceOnlyFailed:
        "This turn failed before a normal assistant reply was produced. Open Trace for details.",
      traceOnlyCanceled:
        "This turn was canceled before a normal assistant reply was produced.",
      traceEntryTitle: "Runtime details moved to Trace",
      emptyBadge: "Chat Surface",
      emptyTitle: "Keep the main page focused on dialogue",
      emptySummary:
        "Start or select a conversation here. Workspace, Git, terminal, and preview stay available, but reduced to secondary inspection.",
      inputRequestBadge: "Action Required",
      inputRequestSummary:
        "Some provider tools pause for an explicit human choice. Choose an option or provide custom input inline.",
      customOption: "Other",
      customOptionSummary: "Use custom text instead of the preset options.",
      customInputPlaceholder:
        "Describe the choice, value, or path the agent should use.",
      submitChoice: "Submit Choice",
      replyPlaceholder: "Send the next instruction, correction, or follow-up…",
      startPlaceholder:
        "Describe what the remote AI should build, change, or inspect…",
      sendReply: "Send Reply",
      startConversation: "Start Conversation",
      inspectorTitle: "Compact Inspector",
      inspectorSummary:
        "Git, workspace, and task telemetry remain available without taking over the main surface.",
      latestTurn: "Latest Turn",
      toolEvents: "Tool Events",
      systemEvents: "System Events",
      traceSummary:
        "Review tool output, system notices, and provider stderr without crowding the transcript.",
      traceEmpty: "No trace events are available for this conversation yet.",
      gitSummary: "Review repository drift and changed files next to the chat.",
      gitEmpty: "No Git context is available for the current conversation yet.",
      workspaceBrowserSummary:
        "Browse the full project tree in the sidebar, including parent and root navigation.",
      workspaceSummary:
        "Browse the active workspace without leaving the conversation.",
      workspaceEmpty:
        "No workspace snapshot is available for the current conversation yet.",
      panels: {
        status: "Status",
        git: "Git",
        files: "Files",
        trace: "Trace",
      },
    },
    workspace: {
      title: "Session Workspace",
      newTitle: "New AI Session",
      newDescription:
        "Describe the goal, working directory, and optional model override for the selected device.",
      currentTitle: "Current Session",
      promptTitle: "Prompt",
      waitingEvents:
        "The session was created and is waiting for provider events.",
      workingDirectory: "Working Directory",
      sessionMetrics: "Session Metrics",
      device: "Device",
      workingRoot: "Working Root",
      deviceCapacity: "Device Capacity",
      providers: "{count} providers",
      capacitySummary:
        "{sessions} sessions · {terminals} terminals · {previews} previews",
      relativePathHint:
        "Use a relative path in session CWD to stay inside this workspace.",
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
            "Git supervision is not available for this workspace yet, so rely on the event stream and workspace preview.",
        },
        counts: {
          assistant: "Assistant Messages",
          tool: "Tool Activity",
          stderr: "Provider Errors",
          changed: "Changed Files",
        },
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
          file: "File",
        },
        sizeLabel: "File Size",
        size: "{size} bytes",
        lines: "Lines",
        previewTitle: "File Preview",
        previewLoading: "Loading file preview...",
        previewEmpty: "Select a file to preview it here.",
        binaryNotice:
          "This file is binary or non-text and cannot be previewed inline.",
        truncated: "Preview truncated to keep the control plane responsive.",
      },
      git: {
        title: "Git Inspect",
        description:
          "Inspect branch state, changed files, and recent commits without dropping into terminal.",
        unsupported:
          "The selected device does not advertise Git inspect capability yet.",
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
          not_repository:
            "The current workspace is not inside a Git repository.",
          git_unavailable: "Git is not available on the selected device.",
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
          unstagedLines: "+{additions} / -{deletions} unstaged",
        },
      },
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
      targetPort: "Service Port",
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
      targetPort: "3000",
    },
    composerNote:
      "OpenCode currently runs through standard ACP. Codex and Claude Code currently run through CLI. Terminal and Preview remain below as advanced tools.",
    advanced: {
      badge: "Advanced Tools",
      title: "Terminal And Preview",
      description:
        "These capabilities stay available for environment checks, manual fallback, and preview access, but they are no longer the homepage's primary workflow.",
      empty:
        "No advanced capabilities are enabled for the current deployment. Terminal and preview surfaces will appear here when the corresponding feature flags are available.",
    },
    terminal: {
      title: "Terminal",
      visibleSummary: "{visible} visible of {total} total · WS {state}",
      open: "Open Terminal",
      empty: "No terminal sessions match the current filters.",
      waiting:
        "The session was created and is waiting for the agent to start the terminal.",
      select:
        "Pick a terminal session or create a new one for the selected device.",
      noCapability:
        "The selected device does not advertise shell capability, so a terminal session cannot be created.",
      note: "Use this for environment checks, manual fallback, and advanced diagnosis. It is not the default product path.",
      detailTitle: "Terminal {id}",
      detailSummary: "{status} · started {time}",
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
      note: "Preview is the product-facing path for common local web services. Raw relay and target endpoint details remain below for advanced debugging.",
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
      detailTitle: "Preview {id}",
    },
  },
} as const;

export default en;
