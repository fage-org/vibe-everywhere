# 阶段1实施计划: 核心功能补全

> **生成日期**: 2026-04-13
> **目标时间**: 2-3 周
> **目的**: 完成 Happy vs Vibe-Remote 差距分析中阶段1的三大模块

## 模块A: 消息渲染增强

### 需求

当前 `vibe-remote` 的消息渲染器已经存在但功能有限，需要增强以下方面：

1. **工具调用渲染器完善** - 当前 ToolRenderer.tsx 已有基础结构，但缺少：
   - 内联工具显示选项 (viewInline 设置)
   - 更好的参数语法高亮
   - 工具特定结果渲染器 (如文件读取、Bash 命令等)
   - 折叠/展开状态持久化

2. **内联工具显示选项** - 参考 Happy 的设置：
   - `viewInline`: 是否在消息流中内联显示工具调用
   - `expandTodos`: 是否自动展开待办列表
   - 当前是独立的工具调用区块，需要添加内联模式

3. **代码块语法高亮优化** - 当前 syntax-code-block.tsx 存在：
   - 需要确认是否使用了高性能的语法高亮库
   - 添加行号显示选项 (showLineNumbers, showLineNumbersInToolViews)
   - 添加行换行选项 (wrapLinesInDiffs)

4. **Diff 渲染器改进** - DiffRenderer.tsx 已有基础：
   - 添加行号切换功能
   - 添加行换行功能
   - 改进大文件的性能

### 实施步骤

#### A1. 设置系统基础设施 (优先级: 高)

1. **创建本地设置存储系统**
   - 文件: `packages/vibe-app-tauri/src/local-settings.ts`
   - 使用 localStorage 存储客户端设置
   - 提供类型安全的设置读写 API

2. **扩展设置类型定义**
   - 文件: `packages/vibe-app-tauri/src/sources/shared/sync/settings.ts` (已存在，需扩展)
   - 添加渲染相关设置字段：
     ```typescript
     viewInline: boolean;
     expandTodos: boolean;
     showLineNumbers: boolean;
     showLineNumbersInToolViews: boolean;
     wrapLinesInDiffs: boolean;
     alwaysShowContextSize: boolean;
     ```

#### A2. 工具调用渲染器增强 (优先级: 高)

1. **添加内联工具显示模式**
   - 文件: `packages/vibe-app-tauri/src/components/renderers/ToolRenderer.tsx`
   - 添加 `inline` prop 控制显示模式
   - 内联模式：精简显示，只显示工具名和状态
   - 完整模式：当前实现

2. **创建工具特定结果渲染器**
   - 文件: `packages/vibe-app-tauri/src/components/renderers/tool-results/`
   - `BashToolResult.tsx` - Bash 命令结果渲染
   - `FileReadToolResult.tsx` - 文件读取结果渲染
   - `EditToolResult.tsx` - 文件编辑结果渲染 (包含 diff)
   - `SearchToolResult.tsx` - 搜索结果渲染

3. **添加参数语法高亮**
   - 使用现有的 `syntax-code-block.tsx`
   - JSON 参数使用 JSON 高亮
   - 支持语言自动检测

#### A3. 代码块优化 (优先级: 中)

1. **确认语法高亮库**
   - 检查当前实现是否使用 shiki 或 prism
   - 如需替换，选择轻量级的 highlight.js 或 shiki/wasm

2. **添加行号显示**
   - 文件: `packages/vibe-app-tauri/src/syntax-code-block.tsx`
   - 添加 `showLineNumbers` prop
   - 添加样式支持

3. **添加行换行选项**
   - 添加 `wrapLines` prop
   - 实现水平滚动 vs 换行的切换

#### A4. Diff 渲染器改进 (优先级: 中)

1. **添加设置联动**
   - 文件: `packages/vibe-app-tauri/src/components/renderers/DiffRenderer.tsx`
   - 从设置中读取 `showLineNumbers`, `wrapLinesInDiffs`

2. **性能优化**
   - 虚拟化长 diff 列表
   - 延迟渲染大型 diff hunk

### 文件清单

| 文件路径 | 操作 | 描述 |
|----------|------|------|
| `packages/vibe-app-tauri/src/local-settings.ts` | 新建 | 本地设置存储 |
| `packages/vibe-app-tauri/src/components/renderers/ToolRenderer.tsx` | 修改 | 添加内联模式 |
| `packages/vibe-app-tauri/src/components/renderers/tool-results/BashToolResult.tsx` | 新建 | Bash 结果渲染 |
| `packages/vibe-app-tauri/src/components/renderers/tool-results/FileReadToolResult.tsx` | 新建 | 文件读取渲染 |
| `packages/vibe-app-tauri/src/components/renderers/tool-results/EditToolResult.tsx` | 新建 | 编辑结果渲染 |
| `packages/vibe-app-tauri/src/components/renderers/tool-results/index.ts` | 新建 | 导出入口 |
| `packages/vibe-app-tauri/src/syntax-code-block.tsx` | 修改 | 行号和换行 |
| `packages/vibe-app-tauri/src/components/renderers/DiffRenderer.tsx` | 修改 | 设置联动 |

### 依赖

- 无外部依赖，可独立开始

### 风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 语法高亮库性能 | 中 | 使用虚拟化或懒加载 |
| 设置同步复杂度 | 低 | 使用简单 localStorage |
| Diff 大文件性能 | 中 | 添加虚拟滚动 |

---

## 模块B: 设置系统完善

### 需求

当前 `vibe-remote` 设置页面 (SettingsRoute.tsx, SettingsSurface.tsx) 是基础实现，需要添加以下子页面：

1. **设置-外观页面** (`/settings/appearance`)
   - 主题切换 (adaptive/light/dark)
   - 内联工具调用显示
   - 展开待办列表
   - 行号显示设置
   - 头像样式选择
   - Flavor 图标显示

2. **设置-功能页面** (`/settings/features`)
   - 实验性功能开关
   - Web 专属功能
   - Enter 发送设置
   - Markdown 复制 v2
   - 隐藏非活跃会话

3. **设置-语言页面** (`/settings/language`)
   - 自动检测语言
   - 手动选择语言
   - 当前支持: en, zh-CN
   - 计划扩展: 参考 Happy 的 9 种语言

4. **设置-使用量页面** (`/settings/usage`)
   - Token 使用量统计
   - 费用统计
   - 按模型分组
   - 时间范围选择 (今天/7天/30天)

### 实施步骤

#### B1. 路由扩展 (优先级: 高)

1. **更新路由模型**
   - 文件: `packages/vibe-app-tauri/src/useAppV2RouteModel.ts`
   - 添加新视图类型：
     ```typescript
     | 'settings-appearance'
     | 'settings-features'
     | 'settings-language'
     | 'settings-usage'
     ```

2. **更新路由出口**
   - 文件: `packages/vibe-app-tauri/src/AppV2RouteOutlet.tsx`
   - 添加新路由的渲染分支

3. **创建路由组件**
   - 文件: `packages/vibe-app-tauri/src/routes/appv2/SettingsAppearanceRoute.tsx`
   - 文件: `packages/vibe-app-tauri/src/routes/appv2/SettingsFeaturesRoute.tsx`
   - 文件: `packages/vibe-app-tauri/src/routes/appv2/SettingsLanguageRoute.tsx`
   - 文件: `packages/vibe-app-tauri/src/routes/appv2/SettingsUsageRoute.tsx`

#### B2. 外观设置页面 (优先级: 高)

1. **主题切换组件**
   - 文件: `packages/vibe-app-tauri/src/components/settings/ThemeSelector.tsx`
   - 三种模式: adaptive, light, dark
   - 使用 CSS 变量切换主题

2. **显示设置组件**
   - 文件: `packages/vibe-app-tauri/src/components/settings/DisplaySettings.tsx`
   - 使用 `SettingsSurface.tsx` 作为容器
   - 复用 `Item`, `ItemGroup` 组件模式

#### B3. 功能设置页面 (优先级: 中)

1. **功能开关组件**
   - 文件: `packages/vibe-app-tauri/src/components/settings/FeatureToggles.tsx`
   - 实验性功能区域
   - Web 专属功能区域

2. **设置持久化**
   - 复用 `local-settings.ts`
   - 功能设置多为本地设置

#### B4. 语言设置页面 (优先级: 中)

1. **语言选择组件**
   - 文件: `packages/vibe-app-tauri/src/components/settings/LanguageSelector.tsx`
   - 自动检测 + 手动选择
   - 语言切换需要重新加载

2. **扩展 i18n 支持**
   - 复用现有 `src/i18n/` 结构
   - 确保所有 UI 文本已国际化

#### B5. 使用量页面 (优先级: 低)

1. **使用量面板组件**
   - 文件: `packages/vibe-app-tauri/src/components/settings/UsagePanel.tsx`
   - 文件: `packages/vibe-app-tauri/src/components/settings/UsageChart.tsx`
   - 文件: `packages/vibe-app-tauri/src/components/settings/UsageBar.tsx`

2. **API 客户端**
   - 文件: `packages/vibe-app-tauri/src/desktop-client.ts`
   - **TODO: 检查后端 `/v1/usage` API 是否已实现**
   - 添加使用量查询方法 (如后端已支持)

### 文件清单

| 文件路径 | 操作 | 描述 |
|----------|------|------|
| `packages/vibe-app-tauri/src/useAppV2RouteModel.ts` | 修改 | 添加新视图类型 |
| `packages/vibe-app-tauri/src/AppV2RouteOutlet.tsx` | 修改 | 添加新路由分支 |
| `packages/vibe-app-tauri/src/routes/appv2/SettingsAppearanceRoute.tsx` | 新建 | 外观设置页面 |
| `packages/vibe-app-tauri/src/routes/appv2/SettingsFeaturesRoute.tsx` | 新建 | 功能设置页面 |
| `packages/vibe-app-tauri/src/routes/appv2/SettingsLanguageRoute.tsx` | 新建 | 语言设置页面 |
| `packages/vibe-app-tauri/src/routes/appv2/SettingsUsageRoute.tsx` | 新建 | 使用量页面 |
| `packages/vibe-app-tauri/src/components/settings/ThemeSelector.tsx` | 新建 | 主题选择器 |
| `packages/vibe-app-tauri/src/components/settings/DisplaySettings.tsx` | 新建 | 显示设置 |
| `packages/vibe-app-tauri/src/components/settings/FeatureToggles.tsx` | 新建 | 功能开关 |
| `packages/vibe-app-tauri/src/components/settings/LanguageSelector.tsx` | 新建 | 语言选择器 |
| `packages/vibe-app-tauri/src/components/settings/UsagePanel.tsx` | 新建 | 使用量面板 |
| `packages/vibe-app-tauri/src/components/settings/UsageChart.tsx` | 新建 | 使用量图表 |
| `packages/vibe-app-tauri/src/components/settings/UsageBar.tsx` | 新建 | 使用量条形图 |

### 依赖

- 依赖模块A的本地设置系统 (`local-settings.ts`)
- 使用量页面依赖后端 API (`/v1/usage/query`)

### 风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 使用量 API 可能未实现 | 高 | **TODO: 检查后端 API** - 如未实现则跳过此页面 |
| 图表库选择 | 低 | 使用简单的 CSS/SVG 图表 |
| 主题切换兼容性 | 中 | 确保 Tauri 支持 CSS 变量 |

---

## 模块C: 机器管理

### 需求

当前 `vibe-remote` 缺少机器详情页面，需要实现：

1. **机器详情页面** (`/machine/:id`)
   - 机器基本信息 (host, username, platform, arch)
   - 在线状态显示
   - 守护进程状态
   - CLI 可用性 (claude, codex, gemini, openclaw)
   - 最近会话列表
   - 最近使用路径

2. **机器状态显示**
   - 在线/离线状态指示器
   - 守护进程状态 (likely alive/stopped)
   - 最后在线时间

3. **机器在线/离线管理**
   - 重命名机器
   - 停止守护进程
   - 在机器上启动新会话

### 实施步骤

#### C1. 路由扩展 (优先级: 高)

1. **添加机器路由**
   - 文件: `packages/vibe-app-tauri/src/useAppV2RouteModel.ts`
   - 添加视图类型：
     ```typescript
     | 'machine-detail'
     ```

2. **创建路由组件**
   - 文件: `packages/vibe-app-tauri/src/routes/appv2/MachineDetailRoute.tsx`

3. **更新路由出口**
   - 文件: `packages/vibe-app-tauri/src/AppV2RouteOutlet.tsx`

#### C2. 机器状态组件 (优先级: 高)

1. **状态指示器组件**
   - 文件: `packages/vibe-app-tauri/src/components/machine/MachineStatusIndicator.tsx`
   - 在线: 绿色圆点 + "Online"
   - 离线: 灰色圆点 + "Offline"

2. **守护进程状态组件**
   - 文件: `packages/vibe-app-tauri/src/components/machine/DaemonStatusView.tsx`
   - 显示 PID, HTTP 端口, 启动时间, CLI 版本
   - 停止守护进程按钮

#### C3. 机器详情页面 (优先级: 高)

1. **页面布局**
   - 文件: `packages/vibe-app-tauri/src/routes/appv2/MachineDetailRoute.tsx`
   - 参考 Happy 的 `machine/[id].tsx`
   - 使用类似设置页面的 Item/ItemGroup 布局

2. **启动会话功能**
   - 路径输入框 (支持最近路径选择)
   - 启动按钮
   - 错误处理

3. **机器信息展示**
   - 主机信息组
   - CLI 可用性组
   - 守护进程状态组
   - 最近会话组

#### C4. 机器列表入口 (优先级: 中)

1. **更新首页**
   - 在首页添加机器列表入口
   - 显示在线机器数量

2. **更新导航**
   - 如果有侧边栏，添加机器入口

### 文件清单

| 文件路径 | 操作 | 描述 |
|----------|------|------|
| `packages/vibe-app-tauri/src/useAppV2RouteModel.ts` | 修改 | 添加机器视图类型 |
| `packages/vibe-app-tauri/src/AppV2RouteOutlet.tsx` | 修改 | 添加机器路由分支 |
| `packages/vibe-app-tauri/src/routes/appv2/MachineDetailRoute.tsx` | 新建 | 机器详情页面 |
| `packages/vibe-app-tauri/src/components/machine/MachineStatusIndicator.tsx` | 新建 | 状态指示器 |
| `packages/vibe-app-tauri/src/components/machine/DaemonStatusView.tsx` | 新建 | 守护进程状态 |
| `packages/vibe-app-tauri/src/components/machine/MachineLauncher.tsx` | 新建 | 会话启动器 |
| `packages/vibe-remote/packages/vibe-app-tauri/src/components/routes/index.ts` | 修改 | 导出新组件 |

### 依赖

- 依赖后端 API (已在 `desktop-client.ts` 中定义)
  - `listMachines()`
  - `getMachine(machineId)`
  - `machineStopDaemon(machineId)`
  - `machineUpdateMetadata(machineId, metadata, version)`
  - `machineSpawnNewSession(options)`

### 风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 后端 API 不完整 | 高 | **TODO: 检查 API 可用性** - 缺失的 API 需先实现 |
| 守护进程状态不准确 | 中 | 使用心跳或最后活动时间 |
| 启动会话错误处理 | 中 | 完善错误提示和重试机制 |

---

## 实施顺序

基于依赖关系和优先级，建议按以下顺序实施：

### 第 1 周: 基础设施 + 设置入口

1. **Day 1-2**: 本地设置系统 (模块 A1)
   - 创建 `local-settings.ts`
   - 扩展设置类型定义

2. **Day 3-4**: 设置路由框架 (模块 B1)
   - 更新路由模型
   - 创建路由组件骨架

3. **Day 5**: 外观设置页面基础 (模块 B2)
   - 主题切换组件
   - 显示设置组件

### 第 2 周: 设置完善 + 机器管理

4. **Day 1-2**: 功能设置页面 (模块 B3)
   - 功能开关组件

5. **Day 3-4**: 语言设置页面 (模块 B4)
   - 语言选择器
   - i18n 完善

6. **Day 5**: 机器管理路由 (模块 C1)
   - 路由扩展
   - 页面骨架

### 第 3 周: 消息渲染 + 使用量

7. **Day 1-2**: 工具调用渲染器 (模块 A2)
   - 内联模式
   - 工具特定渲染器

8. **Day 3-4**: 机器详情页面 (模块 C2, C3)
   - 状态组件
   - 详情页面完整实现

9. **Day 5**: 使用量页面 (模块 B5)
   - 使用量面板
   - 图表组件

---

## 总体风险

| 风险类别 | 描述 | 缓解措施 |
|----------|------|----------|
| 后端 API 兼容性 | 部分功能可能依赖未实现的后端 API | **TODO: 优先检查 API 可用性** - 未实现的 API 需先完成后再实现对应功能 |
| 组件复用 | Happy 使用 React Native，Vibe-Remote 使用 React DOM | 适配组件样式和行为，而非直接移植 |
| 国际化 | 当前只有 en, zh-CN，可能需要扩展 | 先完成核心功能，后续添加更多语言 |
| 性能 | 大型 diff 或长会话列表可能影响性能 | 使用虚拟化、懒加载等优化技术 |

---

## 预估工作量

| 模块 | 任务数 | 预估时间 | 复杂度 |
|------|--------|----------|--------|
| 模块A: 消息渲染增强 | 8 | 5-6 天 | 中 |
| 模块B: 设置系统完善 | 12 | 6-7 天 | 低-中 |
| 模块C: 机器管理 | 6 | 3-4 天 | 中 |
| **总计** | **26** | **14-17 天** | **中** |

---

## 验收标准

### 模块A: 消息渲染增强
- [x] ✅ 工具调用支持内联显示模式 (viewInline 设置已实现)
- [x] ✅ Bash、文件读取、编辑、搜索工具有专用渲染器 (tool-results/ 目录)
- [x] ✅ 代码块支持行号显示 (SyntaxCodeBlock + showLineNumbersInToolViews 设置联动)
- [x] ✅ Diff 渲染器支持行号和换行设置 (DiffRenderer + showLineNumbers/wrapLines 设置联动)
- [x] ✅ 本地设置上下文提供器 (LocalSettingsContext.tsx)
- [x] ✅ 工具结果渲染器注册表 (toolResultRegistry)

### 模块B: 设置系统完善
- [x] ✅ 外观设置页面可用，支持主题切换 (SettingsAppearanceRoute.tsx)
- [x] ✅ 功能设置页面可用，支持实验性功能开关 (SettingsFeaturesRoute.tsx)
- [x] ✅ 语言设置页面可用，支持 en/zh-CN 切换 (SettingsLanguageRoute.tsx)
- [x] ✅ 使用量页面可用，显示 token 和费用统计 (SettingsUsageRoute.tsx)

### 模块C: 机器管理
- [x] ✅ 机器详情页面 UI 框架可用 (MachineDetailRoute.tsx)
- [x] ✅ 在线/离线状态正确显示
- [x] ✅ 后端 RPC `spawn-happy-session` 已实现 (vibe-cli/src/daemon.rs)
- [x] ✅ 后端 RPC `stop-daemon` 已实现 (vibe-cli/src/daemon.rs)
- [x] ✅ 客户端 `spawnSessionOnMachine()` 方法已添加 (useDesktopState.ts)
- [x] ✅ 客户端 `stopMachineDaemon()` 方法已添加 (useDesktopState.ts)
- [x] ✅ 机器详情页面 UI 交互完成 (启动会话/停止守护进程按钮、路径输入、错误处理)

---

## 实施进度 (2026-04-13)

### 已完成
- ✅ 后端 API 可用性检查
- ✅ 本地设置系统 (`local-settings.ts`)
- ✅ 本地设置上下文 (`LocalSettingsContext.tsx`) - 提供全局设置访问
- ✅ 设置路由框架更新 (`useAppV2RouteModel.ts`, `AppV2RouteOutlet.tsx`)
- ✅ 外观设置页面 (`SettingsAppearanceRoute.tsx`)
- ✅ 功能设置页面 (`SettingsFeaturesRoute.tsx`)
- ✅ 语言设置页面 (`SettingsLanguageRoute.tsx`)
- ✅ 使用量页面 (`SettingsUsageRoute.tsx`)
- ✅ i18n 翻译 (英语/中文)
- ✅ 机器详情页面 UI 框架 (`MachineDetailRoute.tsx`)
- ✅ 后端 RPC `spawn-happy-session` 实现 (`vibe-cli/src/daemon.rs`)
- ✅ 后端 RPC `stop-daemon` 实现 (`vibe-cli/src/daemon.rs`)
- ✅ 客户端机器 RPC 类型 (`desktop-wire.ts`)
- ✅ 客户端机器加密方法 (`desktop-client.ts`)
- ✅ 客户端 `spawnSessionOnMachine()` 方法 (`useDesktopState.ts`)
- ✅ 客户端 `stopMachineDaemon()` 方法 (`useDesktopState.ts`)
- ✅ 机器详情页面 UI 交互 (启动会话/停止守护进程按钮、路径输入、目录创建审批流程)
- ✅ 代码块行号显示设置联动 (`SyntaxCodeBlock` + `useRichRenderOptions`)
- ✅ Diff 渲染器设置联动 (`DiffRenderer` + `wrapLines` 支持 + `DiffRendererWithSettings`)
- ✅ 工具特定渲染器 (`tool-results/` 目录)
  - `BashToolResult.tsx` - Bash 命令结果渲染
  - `FileReadToolResult.tsx` - 文件读取结果渲染
  - `EditToolResult.tsx` - 文件编辑结果渲染 (Diff)
  - `SearchToolResult.tsx` - 搜索结果渲染
  - `ToolResultSection.tsx` - 通用区块容器
  - `index.tsx` - 导出入口和注册表

### 待完成
- 无 (Phase 1 全部完成)

---

## 前置检查清单 (实施前必须完成)

### 后端 API 检查结果 (已完成 2026-04-13)

| API | 状态 | 说明 |
|-----|------|------|
| `/v1/usage/query` | ✅ 已实现 | 后端 `api/account.rs` + 客户端 `queryUsage()` |
| `machineUpdateMetadata` | ✅ 已实现 | 后端 Socket `machine-update-metadata` |
| `machineSpawnNewSession` | ✅ 已实现 | 后端 RPC `spawn-happy-session` + 客户端 `spawnSessionOnMachine()` |
| `machineStopDaemon` | ✅ 已实现 | 后端 RPC `stop-daemon` + 客户端 `stopMachineDaemon()` |

### 可独立实施的任务
以下任务不依赖未实现的后端 API，可立即开始：
- ✅ **模块A**: 消息渲染增强 (全部)
- ✅ **模块B**: 外观设置页面、功能设置页面、语言设置页面、使用量页面
- ✅ **模块C**: 机器详情页面完整功能 (包括启动会话和停止守护进程)

### 后端待实现项
~~以下功能需要后端先实现 RPC 方法后再完成客户端~~
- ~~**TODO (后端)**: 实现 `spawn-happy-session` RPC 方法~~ ✅ 已完成
- ~~**TODO (后端)**: 实现 `stop-daemon` RPC 方法~~ ✅ 已完成

---

## 附录: Happy 参考文件

### 设置相关
- `/root/happy/packages/happy-app/sources/app/(app)/settings/appearance.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/settings/features.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/settings/language.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/settings/usage.tsx`
- `/root/happy/packages/happy-app/sources/sync/settings.ts`
- `/root/happy/packages/happy-app/sources/sync/localSettings.ts`

### 机器相关
- `/root/happy/packages/happy-app/sources/app/(app)/machine/[id].tsx`
- `/root/happy/packages/happy-app/sources/utils/machineUtils.ts`
- `/root/happy/packages/happy-app/sources/sync/ops.ts` (machineSpawnNewSession 等)

### 使用量相关
- `/root/happy/packages/happy-app/sources/components/usage/UsagePanel.tsx`
- `/root/happy/packages/happy-app/sources/components/usage/UsageChart.tsx`
- `/root/happy/packages/happy-app/sources/components/usage/UsageBar.tsx`
- `/root/happy/packages/happy-app/sources/sync/apiUsage.ts`

### 国际化
- `/root/happy/packages/happy-app/sources/text/index.ts`
- `/root/happy/packages/happy-app/sources/text/_all.ts`
