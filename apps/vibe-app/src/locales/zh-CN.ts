const zhCN = {
  app: {
    title: "Vibe Everywhere"
  },
  nav: {
    home: "首页",
    projects: "项目",
    notifications: "通知",
    settings: "我的"
  },
  common: {
    refresh: "刷新",
    openProject: "打开项目",
    viewDetails: "查看详情",
    all: "全部",
    online: "在线",
    offline: "离线",
    itemsCount: "{count} 项",
    projectsCount: "{count} 个项目",
    refreshedAt: "更新于 {value}"
  },
  shell: {
    badge: "AI Worktree",
    title: "跨主机与项目的远程 AI 开发工作台",
    serverError: "服务器错误",
    hostsOnline: "{count} 台主机在线",
    noHostOnline: "当前没有在线主机",
    emptyServer: "请先在“我的”中配置 relay，然后开始浏览主机与项目会话。",
    runningTasks: "运行中任务：{count}",
    attention: "待处理事项：{count}"
  },
  projectCard: {
    failed: "失败",
    waiting: "等待中",
    running: "运行中",
    ready: "就绪",
    changedFiles: "变更 {count} 个文件",
    inventoryOnly: "新发现",
    availability: {
      available: "可用",
      offline: "主机离线",
      unreachable: "待重新确认",
      history_only: "仅历史记录"
    },
    discovery: {
      working_root: "主机库存",
      git_worktree: "Git 工作树",
      known_project: "已知项目"
    },
    topics: "话题",
    updated: "更新时间"
  },
  home: {
    stats: {
      onlineHosts: "在线主机",
      runningTasks: "运行中任务",
      needsAttention: "待处理"
    },
    continueWorkBadge: "继续工作",
    continueWorkEmptyTitle: "先连接服务器并打开一个项目。",
    continueWorkEmptySummary: "一旦项目会话开始，最近项目会出现在这里，方便在手机上快速续接。",
    runningNow: "当前运行",
    noRunningTasks: "当前没有活跃任务，但最近项目和待审查工作仍会显示在下方。",
    needsReview: "待审查",
    noReviewProjects: "当前没有失败任务或待确认事项。",
    recentProjects: "最近项目"
  },
  projects: {
    title: "按主机与项目浏览",
    summary: "从主机状态直接进入具体项目上下文，不丢失当前路径。",
    searchPlaceholder: "搜索主机、项目或路径",
    recentFilter: "最近",
    hostEmptyOffline: "这台主机当前离线，暂时无法读取它的项目库存。",
    hostEmptyNoWorkspace: "这台主机还没有配置工作目录根路径，暂时无法开始项目发现。",
    hostEmptyNoProjects: "这台主机下暂时没有发现 Git 项目，请检查工作目录根路径和仓库布局。",
    empty: "当前筛选条件下没有匹配项目。"
  },
  notifications: {
    badge: "通知",
    title: "快速回到需要决策的工作",
    summary: "失败任务、Provider 提问和已完成任务都会在这里出现，方便手机端快速回到正确项目。",
    unreadCount: "未读 {count}",
    visibleCount: "当前显示 {count}",
    preferencesTitle: "项目通知偏好",
    preferencesSummary: "为每个项目选择接收全部完成通知，还是仅接收失败与等待输入事件。",
    defaultPreferenceTitle: "默认偏好",
    defaultPreferenceSummary: "没有单独覆盖的项目会继承这里的通知策略。",
    preferenceInherited: "当前继承默认值：{value}",
    preferenceOverride: "当前项目覆盖值：{value}",
    preferenceReset: "恢复默认",
    preferencesEmpty: "先打开一些项目，这里才会显示它们的通知偏好。",
    preferenceImportant: "仅失败与等待",
    preferenceAll: "全部活动",
    unread: "未读",
    unreadSummary: "优先处理这些仍然需要你决策的工作。",
    recent: "最近",
    recentSummary: "已经看过的事项仍保留在这里，方便快速重新进入。",
    newBadge: "新",
    actions: {
      conversation: "打开会话",
      changes: "打开变更",
      logs: "打开日志"
    },
    empty: "暂时没有新通知。运行中、失败或等待输入的工作会显示在这里。",
    completed: "已完成"
  },
  settings: {
    badge: "我的",
    title: "服务器、外观与客户端偏好",
    summary: "主流程留在项目里，relay 设置、语言和主题等次级控制放在这里。",
    serverTitle: "服务器设置",
    serverSummary: "配置当前客户端连接 relay 所需的地址和控制令牌。",
    relayUrl: "Relay 地址",
    accessToken: "访问令牌",
    accessTokenPlaceholder: "可选 Bearer Token",
    save: "保存并刷新",
    currentServer: "当前服务器：{value}",
    notConfigured: "未配置",
    language: "语言",
    theme: "主题",
    policy: {
      badge: "策略",
      title: "执行策略中心",
      summary: "在发起任务前，先统一查看当前有哪些 Provider 可用，以及每种执行模式已经真实生效了哪些限制。",
      providerAvailability: "{online}/{available} 在线",
      empty: "当前主机上还没有检测到可汇总的 Provider 能力。",
      manageBadge: "默认值",
      manageTitle: "策略默认值",
      manageSummary: "这些默认值由当前客户端先应用，之后才会被项目级或任务级覆盖。",
      defaultExecutionMode: "默认执行模式",
      defaultNotifications: "默认通知偏好",
      sensitiveConfirm: "高风险确认",
      confirmEnabled: "开启",
      confirmDisabled: "关闭"
    },
    audit: {
      badge: "审计",
      title: "审计覆盖",
      summary: "在更完整的管理面完成前，这里先提供当前策略与审计覆盖范围的只读汇总。",
      coverageTitle: "当前覆盖范围",
      manageBadge: "记录",
      manageTitle: "全局审计轨迹",
      manageSummary: "在一个全局次级入口里查看最近的任务、Shell 和预览审计记录。",
      empty: "当前筛选条件下没有匹配的审计记录。",
      filters: {
        task: "任务",
        shellPreview: "Shell 与预览"
      },
      facts: {
        projectLogs: "项目日志会先展示当前会话相关的审计记录，再展示原始运行输出。",
        taskLifecycle: "任务创建和取消动作已经进入审计轨迹。",
        shellPreview: "使用 Shell 和预览等次级工具时，其生命周期动作也会进入审计轨迹。",
        secondarySurface: "这里目前只提供可见性汇总；真正的策略编辑仍主要依赖运行时默认值和次级视图。"
      }
    }
  },
  workspace: {
    badge: "项目",
    title: "项目工作台",
    loadingPath: "正在加载项目路径...",
    currentState: "当前状态",
    tabs: {
      conversation: "会话",
      changes: "变更",
      files: "文件",
      logs: "日志"
    },
    desktop: {
      projects: "主机项目",
      hostTree: "主机与项目树",
      hostEmpty: "这台主机当前没有可见项目。",
      worktreeTitle: "新建工作树",
      worktreeSummary: "在当前项目旁边创建一个新的工作树和分支，不打断现有会话。",
      worktreeBranch: "分支名",
      worktreeBranchPlaceholder: "feature/mobile-review",
      worktreeDirectory: "目录名",
      worktreeDestinationHint: "会在当前仓库旁边创建 ../{value}。",
      worktreeSubmit: "创建工作树",
      worktreeCreating: "正在创建工作树...",
      worktreeCreated: "工作树 ../{value} 已创建。",
      worktreeRemove: "移除",
      worktreeRemoving: "正在移除...",
      worktreeRemoved: "工作树 {value} 已移除。",
      worktreeRemoveConfirm: "确定移除工作树 {value} 吗？这会删除对应的 sibling 工作树目录。",
      worktreeBranchRequired: "创建工作树前请先填写分支名。",
      worktreeDirectoryRequired: "请填写目标目录名。",
      worktreeList: "工作树列表",
      worktreeCurrent: "当前",
      worktreeDetached: "游离",
      worktreeStates: {
        current: "当前",
        detached: "游离",
        inventory_missing: "未进入项目库存",
        offline: "主机离线",
        unreachable: "待重新确认",
        available: "可用",
        remove_failed: "移除失败"
      }
    },
    metrics: {
      topics: "话题 {count}",
      running: "运行中 {count}",
      waiting: "等待中 {count}",
      branch: "分支 {value}",
      worktrees: "工作树 {count}",
      changedFiles: "变更文件 {count}",
      updated: "更新于 {value}"
    },
    state: {
      failed: "有失败结果待审查",
      waiting: "正在等待你的输入",
      running: "AI 正在执行",
      ready: "可以开始新一轮任务"
    }
  },
  conversation: {
    topics: "话题",
    emptyTitle: "还没有会话",
    emptySummary: "这个项目已经可以开始工作。发送第一条请求后，就会在 {project} 下创建持续会话。",
    firstTurnTitle: "开始第一轮任务",
    firstTurnSummary: "使用下方输入框为 {project} 创建第一条 AI 任务，随后会话记录会持续挂在这个项目下。",
    executionMode: {
      title: "执行模式",
      readOnly: "只读",
      readOnlySummary: "仅查看、解释和提出建议，不改动文件。",
      workspaceWrite: "可改文件",
      workspaceWriteSummary: "允许修改当前工作区文件，但默认避免直接跑测试。",
      workspaceWriteAndTest: "可改并测试",
      workspaceWriteAndTestSummary: "允许修改当前工作区文件，并运行聚焦的验证或测试命令。"
    },
    executionModeMeta: {
      read_only: "只读",
      workspace_write: "可改文件",
      workspace_write_and_test: "可改并测试"
    },
    policyTitle: "有效约束",
    policySummary: {
      acp: {
        readOnly: "ACP 运行时会直接拦截写文件和终端会话。",
        workspaceWrite: "ACP 运行时允许修改工作区文件，但会拦截终端测试命令。",
        workspaceWriteAndTest: "ACP 运行时允许修改工作区文件，并允许聚焦的终端验证。"
      },
      codex: {
        readOnly: "Codex 会使用原生只读沙箱运行，并且不再请求额外审批。",
        workspaceWrite: "Codex 会使用 workspace-write 沙箱运行，遇到不受信动作时仍会要求审批。",
        workspaceWriteAndTest: "Codex 会使用 workspace-write 沙箱运行，并默认放开聚焦验证。"
      },
      claude: {
        readOnly: "Claude 会使用原生 plan 模式运行，并默认禁用写入和终端工具。",
        workspaceWrite: "Claude 可以修改工作区，但默认会拦截常见测试命令。",
        workspaceWriteAndTest: "Claude 可以修改工作区，并放开当前任务的测试命令限制。"
      },
      generic: {
        readOnly: "当前模式要求只读，但最终约束强度仍取决于实际 Provider。",
        workspaceWrite: "当前模式允许修改工作区，是否存在更强限制取决于实际 Provider。",
        workspaceWriteAndTest: "当前模式允许修改和验证，是否存在更强限制取决于实际 Provider。"
      }
    },
    statusLabel: {
      pending: "排队中",
      assigned: "已分配",
      running: "执行中",
      waiting_input: "等待输入",
      cancel_requested: "已请求取消",
      succeeded: "已完成",
      failed: "失败",
      canceled: "已取消"
    },
    statusSummary: {
      pending: "任务已进入队列，但还没有开始执行。",
      assigned: "任务已分配，正在等待启动。",
      running: "任务正在执行中。",
      waiting_input: "任务正在等待你的回复后继续。",
      cancel_requested: "已经发出取消请求，运行时正在收尾。",
      succeeded: "任务已成功完成。",
      failed: "任务在返回 AI 文本前失败了。",
      canceled: "任务在完成前被取消。"
    },
    eventSummary: {
      idle: "这一轮还没有记录到机器事件。",
      status: "{count} 条状态更新",
      toolCalls: "{count} 次工具调用",
      toolOutputs: "{count} 条工具结果",
      errors: "{count} 条 stderr 事件"
    },
    eventStreamTitle: "最近执行事件",
    rawEventsTitle: "详细事件输出",
    waitingInput: "等待输入",
    stopTask: "停止任务",
    retryTask: "重试",
    explainResult: "解释结果",
    viewChanges: "查看变更",
    viewLogs: "查看日志",
    replyPlaceholder: "输入回复内容",
    sendCustomReply: "发送自定义回复",
    optionalModel: "可选模型",
    promptPlaceholder: "告诉 AI 在这个项目里做什么...",
    send: "发送",
    sensitiveConfirmTitle: "敏感操作",
    sensitiveConfirmSummary: "这条提示词看起来可能触发破坏性或高风险命令。",
    sensitiveConfirmDetail: "请再次确认提示词和执行模式，仍要继续时再发送。",
    confirmSend: "确认并发送",
    cancelConfirm: "取消"
  },
  changes: {
    branch: "分支",
    changedFiles: "变更文件",
    aheadBehind: "领先 / 落后",
    untracked: "未跟踪",
    staged: "已暂存",
    unstaged: "未暂存",
    reviewTitle: "审查摘要",
    reviewSummary: "先确认范围和风险，再进入原始 diff。",
    reviewNeedsAttention: "需要审查",
    reviewClean: "干净",
    reviewUnavailableTitle: "Git 上下文不可用",
    reviewUnavailableSummary: "当前工作区还没有返回可审查的 Git 状态。",
    reviewCleanTitle: "当前没有本地变更",
    reviewCleanSummary: "这个工作区当前是干净的，暂时没有可审查内容。",
    reviewScopeTitle: "变更范围",
    reviewScopeSummary: "当前分支 {branch} 上共有 {count} 个变更文件。",
    reviewConflictTitle: "存在冲突",
    reviewConflictSummary: "有 {count} 个冲突文件需要在通过前明确处理。",
    reviewDeleteTitle: "删除风险",
    reviewDeleteSummary: "当前包含删除文件，请先确认删除意图再继续。",
    reviewNewFilesTitle: "新增文件检查",
    reviewNewFilesSummary: "有 {count} 个新增或未跟踪文件，需要先检查结构和位置。",
    reviewMixedTitle: "暂存与未暂存混合",
    reviewMixedSummary: "当前项目同时存在已暂存和未暂存修改，审查时需要同时确认两个区域。",
    riskConflict: "冲突",
    riskDelete: "删除",
    riskNewFile: "新文件",
    riskTypeChange: "类型变更",
    riskStandard: "常规",
    diffTitle: "文件 diff",
    diffEmpty: "选择一个文件后即可查看它的补丁。",
    diffUnavailable: "当前选中文件还没有可展示的 diff 输出。",
    diffStaged: "已暂存 diff",
    diffUnstaged: "未暂存 diff",
    diffTruncated: "为了保证手机端审查流畅度，diff 输出已被截断。",
    binary: "二进制补丁",
    noChanges: "当前没有本地 Git 变更。",
    recentCommits: "最近提交",
    noCommits: "当前没有可用的最近提交记录。"
  },
  files: {
    title: "文件",
    truncated: "预览已截断",
    empty: "选择一个文件后会在这里预览。"
  },
  logs: {
    empty: "当前还没有运行日志。",
    errorSummaryTitle: "错误摘要",
    errorSummaryBody: "最近失败或 stderr 较多的运行会优先汇总在这里。",
    errorFallback: "这个任务失败了，但没有结构化错误消息。",
    noFilteredEvents: "当前筛选条件下，这个任务没有匹配的日志事件。",
    audit: {
      title: "审计轨迹",
      summary: "当前会话及其任务相关的控制动作会优先展示在这里。",
      outcomes: {
        succeeded: "成功",
        rejected: "已拒绝",
        failed: "失败"
      },
      actions: {
        device_registered: "设备注册",
        task_created: "创建任务",
        task_canceled: "取消任务",
        shell_session_created: "创建 Shell 会话",
        shell_session_closed: "关闭 Shell 会话",
        preview_created: "创建预览",
        preview_closed: "关闭预览"
      }
    },
    filters: {
      all: "全部",
      errors: "错误",
      tools: "工具",
      provider: "Provider"
    }
  }
};

export default zhCN;
