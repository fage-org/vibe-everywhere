const zhCN = {
  app: {
    name: "Vibe Everywhere",
    title: "Vibe Everywhere"
  },
  locale: {
    label: "语言",
    zhCN: "中文",
    en: "English"
  },
  theme: {
    label: "主题",
    system: "跟随系统",
    light: "浅色",
    dark: "深色"
  },
  common: {
    connect: "连接",
    refresh: "刷新",
    cancel: "取消",
    close: "关闭",
    online: "在线",
    offline: "离线",
    status: "状态",
    scope: "范围",
    provider: "Provider",
    title: "标题",
    model: "模型",
    prompt: "提示词",
    protocol: "协议",
    created: "创建时间",
    started: "开始时间",
    finished: "结束时间",
    pending: "等待中",
    allDevices: "所有设备",
    selectedDevice: "当前设备",
    allStatus: "全部状态",
    sendCommand: "发送命令",
    optionalAccessToken: "可选访问令牌",
    useAgentWorkingRoot: "使用 agent 工作根目录",
    defaultModel: "默认模型",
    waiting: "等待中"
  },
  connectionState: {
    disconnected: "未连接",
    connecting: "连接中",
    connected: "已连接"
  },
  taskStatus: {
    pending: "等待中",
    assigned: "已分配",
    running: "运行中",
    cancel_requested: "已请求取消",
    succeeded: "成功",
    failed: "失败",
    canceled: "已取消"
  },
  shellStatus: {
    pending: "等待中",
    active: "活动中",
    close_requested: "已请求关闭",
    succeeded: "成功",
    failed: "失败",
    closed: "已关闭"
  },
  portForwardStatus: {
    pending: "等待中",
    active: "活动中",
    close_requested: "已请求关闭",
    closed: "已关闭",
    failed: "失败"
  },
  eventKind: {
    system: "系统",
    status: "状态",
    provider_stdout: "Provider 标准输出",
    provider_stderr: "Provider 错误输出",
    assistant_delta: "助手增量输出",
    tool_call: "工具调用",
    tool_output: "工具输出"
  },
  stream: {
    stdin: "标准输入",
    stdout: "标准输出",
    stderr: "错误输出",
    system: "系统"
  },
  transport: {
    relay_tunnel: "Relay 隧道",
    overlay_proxy: "Overlay 代理"
  },
  gitFileStatus: {
    modified: "已修改",
    added: "已新增",
    deleted: "已删除",
    renamed: "已重命名",
    copied: "已复制",
    updated_but_unmerged: "有冲突",
    untracked: "未跟踪",
    type_changed: "类型变更",
    unknown: "未知"
  },
  auth: {
    required: "需要认证",
    optional: "认证可选"
  },
  role: {
    owner: "所有者",
    admin: "管理员",
    member: "成员",
    viewer: "只读",
    agent: "Agent"
  },
  authMode: {
    disabled: "关闭认证",
    access_token: "访问令牌"
  },
  storageKind: {
    file: "文件存储",
    memory: "内存存储",
    external: "外部存储"
  },
  deploymentMode: {
    self_hosted: "自建部署",
    hosted_compatible: "托管兼容"
  },
  notificationChannel: {
    in_app: "应用内",
    system: "系统通知"
  },
  auditAction: {
    device_registered: "设备注册",
    task_created: "任务创建",
    task_canceled: "任务取消",
    shell_session_created: "Shell 会话创建",
    shell_session_closed: "Shell 会话关闭",
    preview_created: "Preview 创建",
    preview_closed: "Preview 关闭"
  },
  auditOutcome: {
    succeeded: "成功",
    rejected: "拒绝",
    failed: "失败"
  },
  platform: {
    client: {
      web: "Web",
      tauri_desktop: "桌面端",
      android: "Android"
    }
  },
  error: {
    targetPortInvalid: "目标端口必须是 1 到 65535 之间的整数。"
  },
  dashboard: {
    heroBadge: "Vue + Tauri 2 + Rust Relay",
    heroDescription:
      "以 AI Sessions 为中心的远程开发控制台。选择设备、发起 AI 会话、监督输出，再按需进入 terminal 或 preview tunnel 处理高级诊断。",
    relayTitle: "Relay",
    relayBaseUrl: "Relay 地址",
    mobileLoopbackWarning:
      "移动端不能连接 localhost / 127.0.0.1，请改成 relay 所在机器的局域网 IP 或 HTTPS 公网地址。",
    mobileRemoteHint: "移动端请填写 relay 所在机器的局域网 IP 或 HTTPS 公网地址。",
    activity: {
      title: "活动中心",
      summary: "未读 {unread} 条 / 共 {total} 条",
      markAllRead: "全部标记已读",
      empty: "任务完成、失败和 Preview 就绪等活动会显示在这里。",
      unread: "未读",
      categories: {
        taskSucceeded: "任务成功",
        taskFailed: "任务失败",
        taskCanceled: "任务取消",
        previewReady: "Preview 就绪",
        previewFailed: "Preview 失败"
      },
      messages: {
        taskSucceeded: "“{title}” 已在设备 {deviceId} 上完成。",
        taskFailed: "“{title}” 已在设备 {deviceId} 上失败。",
        taskCanceled: "“{title}” 已在设备 {deviceId} 上取消。",
        previewReady: "设备 {deviceId} 上的 Preview {host}:{port} 已就绪。",
        previewFailed: "设备 {deviceId} 上的 Preview {host}:{port} 创建失败。"
      }
    },
    deployment: {
      title: "部署与连接面",
      mode: "部署模式",
      authMode: "认证模式",
      storageKind: "存储",
      currentClient: "当前客户端",
      relayOrigin: "Relay 对外地址",
      documentation: "自建部署文档",
      guidance: {
        selfHosted:
          "当前控制面以自建模式运行。relay 地址、访问令牌和转发 host 都应按环境配置，不能假设单机 localhost。",
        hostedCompatible:
          "当前 relay 暴露的是托管兼容表面。客户端不要依赖 loopback 或单机网络假设。",
        explicitRemote:
          "当前客户端应使用显式远程 relay 地址，不要使用 localhost，应优先填写 relay 机器的局域网或公网地址。"
      }
    },
    platform: {
      title: "客户端能力矩阵",
      current: "当前",
      available: "可用",
      mobileOptimized: "移动端优化",
      desktopOptimized: "桌面端优化",
      systemNotifications: "系统通知",
      inAppOnly: "仅应用内通知",
      persistedConfig: "运行时配置持久化",
      explicitRelay: "显式远程 Relay",
      loopbackFriendly: "支持本地 Loopback"
    },
    governance: {
      title: "治理与审计",
      description: "在控制台内直接展示当前 actor、tenant 边界、通知通道与最近审计事件。",
      tenant: "租户",
      user: "用户",
      role: "角色",
      notificationChannels: "通知通道",
      auditTitle: "审计事件",
      auditEmpty: "当前租户暂时还没有审计事件。"
    },
    stats: {
      onlineDevices: "在线设备",
      devices: "设备总数",
      aiSessions: "AI Sessions",
      advancedTools: "高级工具"
    },
    devices: {
      title: "设备",
      registered: "已注册设备 {count} 台",
      sessions: "Sessions {count}",
      terminals: "Terminal {count}"
    },
    sessions: {
      title: "AI Sessions",
      visibleSummary: "当前显示 {visible} / 总计 {total}",
      empty: "当前筛选条件下还没有 AI Session。先选择设备并发起一个新的会话。",
      start: "启动 AI Session",
      cancelSession: "取消会话",
      newOnDevice: "{name} 上的新 AI Session",
      readySelected:
        "当前设备已选中。填写 prompt 后即可创建 AI Session，并在这里监督事件流、工作目录和后续工具链。",
      readyEmpty: "先选择一台在线设备，再启动新的 AI Session。"
    },
    workspace: {
      title: "Session 工作台",
      newTitle: "新的 AI Session",
      newDescription: "为当前设备描述目标、工作目录和可选的模型覆盖。",
      currentTitle: "当前会话",
      promptTitle: "提示词",
      waitingEvents: "Session 已创建，等待 provider 输出事件流。",
      workingDirectory: "工作目录",
      sessionMetrics: "会话指标",
      device: "设备",
      workingRoot: "工作根目录",
      deviceCapacity: "设备能力",
      providers: "{count} 个 providers",
      capacitySummary: "{sessions} 个 sessions · {terminals} 个 terminals · {previews} 个 previews",
      relativePathHint: "推荐在 Session CWD 中使用相对路径，保持在当前工作区内。",
      exitCode: "退出码 {code}",
      eventsSummary: "{count} 条事件 · 设备 {deviceId}",
      supervision: {
        title: "会话监督",
        description: "将 provider 输出信号与 Git 上下文结合起来，再决定是否进入 Terminal 或 Preview。",
        summary: {
          running: "AI Session 仍在运行中。当前工作区范围内已检测到 {changed} 个变更文件。",
          succeededWithChanges:
            "AI Session 已完成，当前分支 {branch} 上留下了 {changed} 个待审查变更文件。",
          succeededClean: "AI Session 已完成，但在当前工作区范围内没有看到 Git 变更。",
          failedWithChanges:
            "AI Session 已失败，但当前工作区范围内仍保留 {changed} 个变更文件。",
          failedClean: "AI Session 已失败，且当前工作区范围内没有可见 Git 变更。",
          noGit: "当前工作区暂时无法提供 Git 监督，请结合事件流和工作区预览继续判断结果。"
        },
        counts: {
          assistant: "助手消息",
          tool: "工具活动",
          stderr: "Provider 错误",
          changed: "变更文件"
        }
      },
      browser: {
        title: "工作区浏览器",
        description: "直接浏览当前 Session 工作区，不必先进入 Terminal。",
        unsupported: "当前设备还没有声明工作区浏览能力。",
        loading: "正在加载工作区...",
        empty: "当前目录为空。",
        up: "返回上级",
        root: "工作区根目录",
        path: "当前路径",
        entries: "{count} 个条目",
        kind: {
          directory: "目录",
          file: "文件"
        },
        sizeLabel: "文件大小",
        size: "{size} 字节",
        lines: "行数",
        previewTitle: "文件预览",
        previewLoading: "正在加载文件预览...",
        previewEmpty: "选择一个文件后，可在这里查看内容。",
        binaryNotice: "该文件是二进制或非文本文件，无法直接内联预览。",
        truncated: "为保持控制台响应速度，预览内容已截断。"
      },
      git: {
        title: "Git 检查",
        description: "直接查看分支状态、变更文件和最近提交，不必先进入 Terminal。",
        unsupported: "当前设备还没有声明 Git 检查能力。",
        loading: "正在加载 Git 上下文...",
        empty: "选择一台设备后即可查看仓库状态。",
        workspaceRoot: "工作区范围",
        repoRoot: "仓库根目录",
        scopePath: "范围路径",
        branch: "分支",
        upstream: "上游",
        drift: "上游差异",
        noUpstream: "没有上游分支",
        driftSummary: "ahead {ahead} · behind {behind}",
        state: {
          not_repository: "当前工作区不在 Git 仓库中。",
          git_unavailable: "当前设备上不可用 Git。"
        },
        clean: "当前工作区范围内没有可见的变更文件。",
        noCommits: "当前仓库还没有任何提交。",
        changedFilesTitle: "变更文件",
        recentCommitsTitle: "最近提交",
        preview: "预览",
        deletedPreviewUnavailable: "已删除文件无法预览。",
        stats: {
          changedFiles: "{count} 个变更文件",
          stagedFiles: "{count} 个 staged",
          unstagedFiles: "{count} 个 unstaged",
          untrackedFiles: "{count} 个 untracked",
          conflictedFiles: "{count} 个冲突文件",
          stagedLines: "staged +{additions} / -{deletions}",
          unstagedLines: "unstaged +{additions} / -{deletions}"
        }
      }
    },
    fields: {
      accessToken: "访问令牌",
      provider: "Provider",
      title: "标题",
      sessionCwd: "Session CWD",
      model: "模型",
      prompt: "提示词",
      terminalCwd: "Terminal CWD",
      targetHost: "服务主机",
      targetPort: "服务端口"
    },
    placeholders: {
      selectProvider: "选择 provider",
      sessionTitle: "临时 AI 会话",
      sessionCwd: "repo/path 或 /absolute/path",
      model: "可选模型覆盖",
      prompt: "告诉 Codex / Claude Code / OpenCode 在当前设备上做什么。",
      terminalCwd: "可选工作目录",
      terminalInput: "pwd",
      targetHost: "127.0.0.1",
      targetPort: "3000"
    },
    composerNote:
      "Codex / OpenCode 优先走 ACP；Claude Code 当前仍走 CLI。Terminal 和 Preview 保留在下方高级工具区。",
    advanced: {
      badge: "高级工具",
      title: "Terminal 与 Preview",
      description:
        "这些能力保留给环境检查、人工兜底和预览访问，不再作为首页主工作流。"
    },
    terminal: {
      title: "Terminal",
      visibleSummary: "当前显示 {visible} / 总计 {total} · WS {state}",
      open: "打开 Terminal",
      empty: "当前筛选条件下还没有 terminal sessions。",
      waiting: "会话已创建，等待 agent 启动 terminal 并回传输出。",
      select: "选择一个 terminal session，或先为当前设备创建新的 terminal。",
      noCapability: "当前选中的设备没有声明 shell capability，无法创建 terminal session。",
      note: "用于环境检查、人工兜底和高级诊断，不作为默认主路径。",
      detailTitle: "Terminal {id}",
      detailSummary: "{status} · 开始于 {time}"
    },
    preview: {
      title: "Preview",
      description: "把当前设备上的本地 Web 应用或 HTTP 服务暴露出来，而不是先面对原始隧道细节。",
      visibleSummary: "当前显示 {visible} / 总计 {total}",
      open: "创建 Preview",
      launchTitle: "从当前工作台发起 Preview",
      launchDescription:
        "直接复用当前设备和 Session 上下文暴露本地 HTTP 服务。更完整的历史记录和原始连接细节保留在下方高级面板。",
      empty:
        "当前还没有匹配的 Preview。选择设备并填写本地服务主机与端口后即可创建。",
      select: "选择一个 Preview 在这里查看，或先创建新的 Preview。",
      note:
        "Preview 是面向常见本地 Web 服务的产品路径。原始 relay / target 端点细节仍保留在下方，用于高级排障。",
      serviceHint:
        "优先使用常见场景：当前设备上的本地 HTTP 服务。Host 仍然可以编辑，以支持非 localhost 服务。",
      serviceTarget: "服务 {host}:{port}",
      previewUrl: "预览地址",
      openLink: "浏览器打开",
      ready: "Preview relay 已激活。可以直接在浏览器中打开，或查看下方高级连接细节。",
      waiting: "Preview 仍在准备中，请等待其进入激活状态后再打开。",
      mobileWarning:
        "当前 Preview 指向 loopback 地址，移动端无法访问。请改为正确配置 relay 的 forward host。",
      advancedTitle: "高级连接细节",
      advancedDescription: "原始 relay / target 端点保留在这里，用于自建网络排查和低层诊断。",
      transportLabel: "传输方式",
      relayEndpoint: "Relay 入口",
      targetEndpoint: "目标地址",
      detailTitle: "Preview {id}"
    }
  }
} as const

export default zhCN
