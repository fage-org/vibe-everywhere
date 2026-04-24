# Vibe-Remote 项目模块与功能链路 ECC Review

**Reviewed**: 2026-04-24  
**Scope**: 整个仓库（`ve-server`、`ve-daemon`、`ve-shared`、`ve-mock-client`、`client`、`docs`）  
**Method**:
- 按 ECC 思路先拆模块、再拆功能链路、最后逐项 review。
- 重点检查链路真实性、安全/鉴权、契约一致性、验证闭环。
- 本轮结论基于代码审查 + 实测验证：`cargo test`、`cargo fmt --check`、`cargo clippy -D warnings`、`ve-mock-client` 默认 flow、remote 失败路径、release real-agent smoke。

## 一、实际验证结果

### 1.1 当前命令执行结果

- `cargo test --workspace`：通过。
- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo run -q -p ve-mock-client -- --flows f1,f2,f3,f4,f5,f6,f7,f8,f9,f10,f11,f12 --output json`：通过，12/12 PASS。
- `cargo run -q -p ve-mock-client -- --remote --server-url http://127.0.0.1:1 --host-name x --client-token y --flows f1 --output json`：按预期 FAIL，不再 panic，且会输出完整错误链。
- `which claude && claude --version`：通过；本机可用 CLI 为 `Claude Code 2.1.118`。
- `cargo run -q -p ve-mock-client -- --profile release --real-agent --output json`：通过，13/13 PASS；当前 `release` profile 覆盖 `f1`-`f12` + `f13`，其中 `f13` 真实 Claude Code smoke 通过。
- `cargo run -q -p ve-mock-client -- --flows f14,f15,f16,f17,f18,f19 --real-agent --output json`：通过，6/6 PASS；其中 `f15` 在当前环境下未实际触发 permission prompt，但完整 session/API/error path 均通过。

### 1.2 当前验证结论

- 已实现的服务端 / 守护进程 / 共享协议 / mock-client harness 主链路当前稳定，ECC 基础质量门已完整收口。
- 与前一轮相比，以下问题已被消化：F2 契约漂移、F6 重复触发、F12 scheduler 假验证、remote panic、`ts-rs` 噪音、release gate 缺失。
- 本轮 **没有发现 active HIGH / MEDIUM / LOW 级实现缺陷**。
- 当前剩余的是 **测试覆盖边界** 与 **产品阶段边界**，不是已实现链路中的明显 correctness/security bug。

## 二、模块拆分与逐模块结论

| 编号 | 模块 | 关键路径 | 主要职责 | 本轮结论 |
|---|---|---|---|---|
| M0 | 产品/架构文档 | `docs/需求文档.md`、`docs/技术设计文档.md` | 定义产品目标、系统拓扑、客户端/服务端职责 | 文档已明显对齐现状；当前将“目标态”和“落地态”区分描述 |
| M1 | 共享领域模型与协议 | `crates/ve-shared/src/models.rs`、`crates/ve-shared/src/proto.rs`、`crates/ve-shared/src/types.rs` | Rust 共享模型、JWT、WS 契约、TS 类型导出 | 契约稳定；TS 绑定与 Rust 结构当前一致 |
| M2 | 服务端启动/配置/路由 | `crates/ve-server/src/main.rs`、`crates/ve-server/src/lib.rs`、`crates/ve-server/src/config.rs` | 服务启动、配置加载、中间件与路由挂载 | 稳定，质量门通过 |
| M3 | 服务端认证与访问控制 | `crates/ve-server/src/api/auth.rs`、`crates/ve-server/src/authz.rs`、`crates/ve-server/src/middleware/auth.rs` | 设备注册、配对、JWT、ACL 校验 | 主链路稳定，未发现新的权限绕过问题 |
| M4 | 服务端资源 API | `crates/ve-server/src/api/hosts.rs`、`crates/ve-server/src/api/workspaces.rs`、`crates/ve-server/src/api/files.rs`、`crates/ve-server/src/api/settings.rs` | Host / Workspace / 文件 / 设置 | 当前整体可用，F2/F8/F10 全绿 |
| M5 | 服务端会话域 | `crates/ve-server/src/api/sessions.rs`、`crates/ve-server/src/api/permissions.rs`、`crates/ve-server/src/api/archives.rs` | 会话、消息、控制、权限、归档 | 当前稳定，F3/F4/F5/F6/F7/F9 均通过 |
| M6 | 服务端实时层 / DB / 后台任务 | `crates/ve-server/src/ws/*`、`crates/ve-server/src/hub.rs`、`crates/ve-server/src/tasks/*` | WS 分发、Hub、迁移、后台任务 | 稳定；scheduler 已进入 integration 验证，建连噪音已清理 |
| M7 | 守护进程接入层 | `crates/ve-daemon/src/config.rs`、`crates/ve-daemon/src/pairing.rs`、`crates/ve-daemon/src/ws_client.rs` | daemon 配置、配对、WS 重连 | 主链路稳定；header 鉴权、重连、配对均正常 |
| M8 | 守护进程会话执行层 | `crates/ve-daemon/src/session_registry.rs`、`crates/ve-daemon/src/session_runner.rs`、`crates/ve-daemon/src/agent/*` | runner 生命周期、文件访问、Claude Code / Mock driver 驱动 | 稳定；mock permission 触发精度已修正 |
| M9 | 集成测试运行时 | `crates/ve-mock-client/src/main.rs`、`crates/ve-mock-client/src/test_context.rs`、`crates/ve-mock-client/src/server.rs` | 本地/remote 上下文、server/daemon 编排 | 可用性明显提升；默认验证与 release 验证入口都可用 |
| M10 | Flow 套件 | `crates/ve-mock-client/src/flows/*` | F1-F19 业务链路回归 | 默认 F1-F12 全绿；release gate 已覆盖 F13 real-agent smoke |
| M11 | 客户端运行时/类型产物 | `client/src/types/generated/*` | 提供未来客户端消费的 TS bindings | 当前仅有类型产物，没有运行时代码；文档已明确这一点 |

## 三、功能链路拆分与逐链路结论

| 链路 | 说明 | 涉及模块 | 本轮结论 |
|---|---|---|---|
| F1 | 设备注册与配对 | M1 → M3 → M7 → M9 → M10 | 通过，主链路真实可回归 |
| F2 | Host / Workspace 管理 | M3 → M4 → M9 → M10 | 通过，分页契约已对齐 |
| F3 | Session 创建 | M3 → M5 → M6 → M8 → M10 | 通过 |
| F4 | Session 消息发送/拉取 | M5 → M6 → M8 → M10 | 通过 |
| F5 | Session 控制（pause / restart / terminate / close） | M5 → M6 → M8 → M10 | 通过 |
| F6 | 权限请求 / 响应 | M5 → M6 → M8 → M10 | 通过，且现已覆盖 daemon → WS → server 写链路，并断言单次 trigger 只新增 1 条 permission_request |
| F7 | Session 归档 | M5 → M6 → M8 → M10 | 通过 |
| F8 | 文件浏览 | M4 → M6 → M8 → M10 | 通过 |
| F9 | Archive 浏览 / 删除 | M5 → M10 | 通过 |
| F10 | 通知设置 | M4 → M10 | 通过 |
| F11 | daemon 重连 | M6 → M7 → M10 | 通过 |
| F12 | 后台任务（permission expiry / idempotency cleanup） | M6 → M10 | 通过，且现已验证真实 scheduler tick |
| F13 | Real agent session | M8 → M9 → M10 | 通过，已纳入 `release` profile smoke gate |
| F14 | Real agent multi-turn | M8 → M9 → M10 | 未纳入默认 gate；保留为扩展 real-agent 回归项 |
| F15 | Real agent permission | M8 → M9 → M10 | 同上 |
| F16 | Real agent file browsing | M8 → M9 → M10 | 同上 |
| F17 | Real agent session control | M8 → M9 → M10 | 同上 |
| F18 | Real agent archival lifecycle | M8 → M9 → M10 | 同上 |
| F19 | Real agent error handling | M8 → M9 → M10 | 同上 |

## 四、历史 review 问题复核

| 历史问题 | 当前状态 | 说明 |
|---|---|---|
| remote 模式 nested runtime / panic | 已修复 | remote setup 失败现在返回 FAIL，不再崩进程 |
| remote 错误信息丢失底层原因 | 已修复 | 当前会输出完整错误链 |
| WS token 走 query string | 已修复 | 已改为 `Authorization: Bearer` header |
| F1 配对链路假绿 | 已修复 | 已走真实 `/api/auth/register-device` + `/api/auth/pair` |
| F2 workspace 分页契约漂移 | 已修复 | mock client 已解析 `Paginated<Workspace>` |
| F6 非真实写链路 | 已修复 | 已覆盖 daemon → WS → server 写链路 |
| F6 mock trigger 重复发射 | 已修复 | 现已断言单次 trigger 仅新增 1 条 permission_request |
| F12 仅函数级验证 | 已修复 | 现在 integration server 启动真实 scheduler，并等待 tick 生效 |
| TS optional 字段漂移 | 已修复 | Rust 与 TS bindings 当前一致 |
| `ts-rs` serde warning 噪音 | 已修复 | 已通过 `no-serde-warnings` 特性静默 |
| daemon_hello warning 噪音 | 已修复 | 服务端现静默忽略建连后的 `daemon_hello` |
| release real-agent gate 缺失 | 已修复 | 当前已有 `--profile release --real-agent`，且在本机实跑通过 |
| 文档/客户端结构严重漂移 | 已修复主体 | 文档已明确“当前仅有类型产物，客户端运行时代码尚未落地” |

## 五、当前 Findings

### 无 active findings

本轮基于当前代码和已实现模块的 review，没有发现需要立即修复的 active correctness / security / contract / regression 级缺陷。

## 六、Residual Risks / Gaps

以下不是当前已实现链路中的缺陷，但仍是后续值得跟进的边界与测试缺口：

- `F14`-`F19` 真实 Claude Code 链路已在本轮手动实测通过，但仍未纳入 release smoke gate；当前 release gate 只覆盖最小 real-agent smoke `F13`。
- 当前仓库仍没有客户端运行时代码，只有 `client/src/types/generated/*`；因此移动端 / 桌面端真实 UI 链路无法在本仓库做运行态 review。
- `ve-mock-client` 首次执行在源码有更新时仍会自动重建 `ve-daemon`；这已从“总是重建”优化为“源码变更后重建”，但冷启动时间仍会受本地构建速度影响。
- real-agent smoke 依赖本机 `claude` CLI、网络和可用凭证；虽然本机当前已通过，但 CI / 其他开发机仍需要相同前置条件。

## 七、整体结论

- 当前已落地的后端 / daemon / shared / harness 代码在 ECC 标准下处于 **稳定、可回归、质量门闭合** 的状态。
- 默认回归链路 `F1`-`F12` 全绿，release real-agent smoke `F13` 也已在当前环境中实跑通过；相关结果已记录到本节。
- 对当前实现而言，本轮 review 结论是：
  - **No active findings**
  - 存在少量 **residual risks / testing gaps**，但它们不构成当前代码中的直接缺陷。

## 八、2026-04-24 补充复核（最新）

> 本节覆盖第五节和第七节中的 “No active findings” 结论，作为当前最新 review 结果。

### 8.1 本轮补充验证

- `cargo fmt --all -- --check`：通过。
- `cargo clippy --workspace --all-targets -- -D warnings`：通过。
- `cargo test --workspace`：通过。
- 复核仍沿用本文第二节 / 第三节的模块与功能链路切分；新增问题集中在 `M0` / `M1` / `M4` / `M9` / `M10`，对应 `F1` / `F2` / `F5` / `F8` / `F9` / `F12`。

### 8.2 Active Findings

#### HIGH-M4/F2/F8-01: `create_workspace` 目前只是“写一条元数据”，并未创建或验证远端目录，却直接返回 `exists_on_host = true`

- **模块 / 链路**: `M4 服务端资源 API`；`F2 Host / Workspace 管理`；连带影响 `F3 Session 创建` 与 `F8 文件浏览`
- **位置**:
  - `docs/需求文档.md:146`
  - `docs/需求文档.md:148`
  - `docs/页面字段清单与状态机.md:61`
  - `crates/ve-server/src/api/workspaces.rs:172`
  - `crates/ve-server/src/api/workspaces.rs:219`
  - `crates/ve-daemon/src/ws_client.rs:1017`
  - `crates/ve-daemon/src/ws_client.rs:1028`
  - `crates/ve-daemon/src/ws_client.rs:1117`
- **问题**:
  - PRD 明确写了“创建新目录作为 workspace”“如果目录不存在，可直接创建”。
  - 当前 `create_workspace` 只做 DB `INSERT`，没有发 daemon RPC 去创建目录，也没有校验目录是否真实存在。
  - handler 还会直接把 `exists_on_host` 回给客户端为 `true`，而仓库里也没有后续同步这个字段真实性的写路径。
- **影响**:
  - 用户可以成功创建一个实际上并不存在的 workspace。
  - 后续文件浏览链路会在 daemon 侧因为 `workspace_path` 不存在而返回 `WorkspaceInvalid`。
  - UI 如果依赖 `exists_on_host` 字段，会被“乐观真值”误导，属于链路真实性问题。
- **建议**:
  - 要么在 `create_workspace` 时通过 daemon 显式创建 / 校验目录；
  - 要么把接口语义降级成“仅保存候选路径”，并把 `exists_on_host` 改成真实探测值而不是硬编码。

#### MEDIUM-M1/M9/F1-01: `ve-mock-client` 的配对响应模型又开始偏离真实 API 契约

- **模块 / 链路**: `M1 共享领域模型与协议`；`M9 集成测试运行时`；`F1 设备注册与配对`
- **位置**:
  - `crates/ve-mock-client/src/client.rs:503`
  - `crates/ve-mock-client/src/client.rs:510`
  - `crates/ve-server/src/api/auth.rs:117`
  - `crates/ve-shared/src/models.rs:46`
- **问题**:
  - mock client 本地定义的 `PairingStatusResponse` 只保留了 `status` + `pair_code`，但真实服务端响应是 `status` + `daemon_token`。
  - mock client 本地定义的 `PairResponse` 只保留了 `token`，但共享模型里还有 `host_id` 和 `host_name`。
- **影响**:
  - 当前 serde 会静默忽略多余字段，所以 flow 仍会 PASS。
  - 这会让 `F1` 无法捕捉配对链路响应字段被删改的 regression，属于“假绿式契约漂移”。
- **建议**:
  - 直接复用 `ve_shared` 中的共享响应模型；
  - 若必须使用本地轻量 struct，也要和服务端响应字段保持 1:1 对齐，并补一个契约级解析测试。

#### MEDIUM-M9/F11-01: `--concurrency` 已暴露给 CLI，但并发 integration flow 仍共享同一个 `/tmp/ve-mock-daemon.log`

- **模块 / 链路**: `M9 集成测试运行时`；连带影响所有并发 flow，尤其是 `F11 daemon 重连`
- **位置**:
  - `crates/ve-mock-client/src/main.rs:69`
  - `crates/ve-mock-client/src/daemon.rs:24`
  - `crates/ve-mock-client/src/daemon.rs:41`
  - `crates/ve-mock-client/src/daemon.rs:84`
- **问题**:
  - CLI 已支持 `--concurrency`。
  - 但每个 daemon 子进程都写固定文件 `/tmp/ve-mock-daemon.log`，启动前还会先删旧文件。
  - `wait_for_daemon_hello()` 又依赖这个共享日志来判断 daemon 是否进入 pairing / connected 状态。
- **影响**:
  - 两个并发 flow 会互相覆盖日志、互删日志、互相读取对方的启动信号。
  - 这使并发集成回归变得不确定，属于 harness 自身的并发污染。
- **建议**:
  - 日志文件改成落在各自 `temp_dir` 下的唯一文件名；
  - `wait_for_daemon_hello()` 最好以 hub / HTTP 信号为主、日志为辅。

#### MEDIUM-M10/F5-01: `F5` 只验证 control endpoint “有回应”，没有断言 pause/restart 真正改变会话状态

- **模块 / 链路**: `M10 Flow 套件`；`F5 Session 控制`
- **位置**:
  - `crates/ve-mock-client/src/flows/f5_session_control.rs:58`
  - `crates/ve-mock-client/src/flows/f5_session_control.rs:75`
  - `crates/ve-mock-client/src/flows/f5_session_control.rs:81`
- **问题**:
  - flow 对 `pause` 的 happy path 只要求“接口返回了成功或可识别错误”。
  - 后续虽然拉取了一次 session，但只打印日志，不断言状态必须进入 `paused` 或其他预期状态。
- **影响**:
  - 如果服务端未来退化成“返回 200 但不做任何状态迁移”，`F5` 仍可能通过。
  - 会话控制是核心链路，这个回归盲区会放大 control 语义漂移风险。
- **建议**:
  - 对 `pause` / `restart` / `close` 分别断言最终 session 状态；
  - 最好再补 daemon ACK / status event 级别的链路断言，避免只测 REST 表面成功。

#### MEDIUM-M9/M10/F9/F12-01: 仓库声明支持 PostgreSQL，但 flow 级验证仍然基本锁死在 SQLite 路径

- **模块 / 链路**: `M9 集成测试运行时`、`M10 Flow 套件`；直接影响 `F9` / `F12`，间接影响其他 DB 敏感链路
- **位置**:
  - `.env.example:7`
  - `.env.example:9`
  - `crates/ve-mock-client/src/server.rs:37`
  - `crates/ve-mock-client/src/server.rs:45`
  - `crates/ve-mock-client/src/flows/f9_archive_browse_delete.rs:206`
  - `crates/ve-mock-client/src/flows/f12_background_tasks.rs:84`
- **问题**:
  - 项目对外明确暴露了 PostgreSQL 作为可选数据库。
  - 但 integration server 固定起 `sqlite:`，多个 flow 还直接写了 `INSERT OR IGNORE` 这类 SQLite 专用 SQL。
- **影响**:
  - 当前的 flow 绿灯并不能证明 PostgreSQL 路径同样稳定。
  - 一旦 Postgres 在 archive / background task / idempotency 等链路上出现行为偏差，现有 flow 套件不会第一时间报警。
- **建议**:
  - 把 harness 的 DB backend 参数化；
  - 至少补一条 PostgreSQL CI lane，覆盖 `F3/F6/F9/F12` 这几条最容易受数据库语义影响的链路。

### 8.3 结论修正

- 当前最新结论不再是 “No active findings”。
- 截至这轮补充复核，仓库存在：
  - `1` 个 `HIGH`
  - `4` 个 `MEDIUM`
- 这些问题多数不是“编译不过 / 测试红”的显性故障，而是：
  - 核心链路语义与 PRD 不一致；
  - harness / flow 的验证闭环仍有假绿窗口；
  - 并发和跨数据库路径还没有被稳定兜住。

### 8.4 修复状态（2026-04-24）

- `HIGH-M4/F2/F8-01`：**CLOSED**
  - `create_workspace` 现在会先走 daemon `ensure_workspace` 请求，确保远端目录存在；不存在时会创建，失败时不会落库。
  - 对应实现：`crates/ve-server/src/api/workspaces.rs`、`crates/ve-daemon/src/ws_client.rs`、`crates/ve-shared/src/proto.rs`
- `MEDIUM-M1/M9/F1-01`：**CLOSED**
  - `PairResponse` / `PairingStatusResponse` 已收敛到 `ve-shared` 共享模型，mock-client 不再维护漂移的本地响应 struct。
  - 对应实现：`crates/ve-shared/src/models.rs`、`crates/ve-server/src/api/auth.rs`、`crates/ve-mock-client/src/client.rs`
- `MEDIUM-M9/F11-01`：**CLOSED**
  - integration daemon 日志已改为落在各自 `temp_dir` 内，不再共享固定 `/tmp/ve-mock-daemon.log`。
  - 并发 smoke：`ve-mock-client --flows f2,f9 --concurrency 2` 已通过。
  - 对应实现：`crates/ve-mock-client/src/daemon.rs`
- `MEDIUM-M10/F5-01`：**CLOSED**
  - `F5` 现在会断言 `pause` 后状态为 `paused`；`restart` 会根据实际返回断言“恢复为 running”或“显式拒绝且仍保持 paused”；`close` 会等待归档状态收敛。
  - 对应实现：`crates/ve-mock-client/src/flows/f5_session_control.rs`
- `MEDIUM-M9/M10/F9/F12-01`：**CLOSED（代码路径）**
  - mock-client harness 已支持通过 `--database-url` 或 `VE_MOCK_CLIENT_DATABASE_URL` 指定 integration DB。
  - PostgreSQL 路径改为为每次 integration run 创建独立 schema；`F9` / `F12` 中 SQLite 专有 `INSERT OR IGNORE` 已移除。
  - 对应实现：`crates/ve-mock-client/src/server.rs`、`crates/ve-mock-client/src/main.rs`、`crates/ve-mock-client/src/integration_env.rs`、`crates/ve-mock-client/src/test_context.rs`、`crates/ve-mock-client/src/flows/f9_archive_browse_delete.rs`、`crates/ve-mock-client/src/flows/f12_background_tasks.rs`
  - 说明：当前环境未提供 PostgreSQL DSN，因此本轮已完成字符串/配置级单测与 SQLite smoke；真实 PostgreSQL smoke 需在提供 DSN 后补跑。
