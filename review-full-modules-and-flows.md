# Vibe-Remote 全量模块与功能链路 ECC Review

**Reviewed**: 2026-04-22
**Scope**: ve-server + ve-daemon 全部已完成服务模块
**Review Method**:
- 先整理已完成服务模块与主功能链路清单
- 按模块与链路分别执行 ECC 审查（authz/auth、sessions/permissions/archives、ws/hub/daemon_ws、db/migrations/tasks、ve-daemon 全模块）
- 交叉引用已有 review 文件，确认已修复项不再重复，仅记录新问题与仍未关闭项
- 严重级别：`CRITICAL` = 必须优先修复；`HIGH` = 交付前应修复；`MEDIUM` = 明显缺口但可排在高优之后；`LOW` = 建议改进

---

## 一、已完成服务模块清单

| 编号 | 模块 | 关键文件 | 职责 |
|------|------|---------|------|
| M1 | ve-server Bootstrap & Routing | `main.rs`, `lib.rs`, `config.rs`, `state.rs` | 启动流程、路由组装、配置加载、AppState 共享 |
| M2 | ve-server Auth & Pairing | `api/auth.rs`, `middleware/auth.rs`, `authz.rs` | 设备注册、daemon hello、配对状态轮询、配对完成、Token 签发、JWT 中间件 |
| M3 | ve-server Session Lifecycle | `api/sessions.rs` | 会话创建/列表/获取/发消息/控制/关闭/归档/重跑 |
| M4 | ve-server Permissions | `api/permissions.rs` | 权限请求列表/获取/响应，CAS 并发更新保护 |
| M5 | ve-server Archives | `api/archives.rs` | 归档列表/获取/批量删除 |
| M6 | ve-server Files | `api/files.rs` | 通过 send_and_wait 向 daemon 请求文件树/文件内容 |
| M7 | ve-server Hosts & Workspaces | `api/hosts.rs`, `api/workspaces.rs` | 主机 CRUD、工作区 CRUD |
| M8 | ve-server Settings | `api/settings.rs` | 通知偏好设置 |
| M9 | ve-server WS Transport & Hub | `hub.rs`, `ws/client_ws.rs`, `ws/daemon_ws.rs` | WS 连接管理、会话订阅、请求/响应关联、事件广播 |
| M10 | ve-server DB & Migrations | `db/mod.rs`, `db/idempotency.rs`, `migrations/*` | DB 池、迁移运行、幂等键存储 |
| M11 | ve-server Background Tasks | `tasks/permission_expiry.rs`, `tasks/idempotency_cleanup.rs` | 定时任务：过期权限标记、幂等键清理 |
| D1 | ve-daemon Config & Credentials | `config.rs`, `credentials.rs` | 配置加载、本地凭据存储（0o600 权限） |
| D2 | ve-daemon Pairing | `pairing.rs`, `pairing_identity.rs` | 配对状态机、安装身份管理 |
| D3 | ve-daemon WS Client | `ws_client.rs` | 自动重连（指数退避+抖动）、心跳、事件转发 |
| D4 | ve-daemon Session Registry | `session_registry.rs` | SessionRunner 句柄注册表、并发控制、最大并行会话限制 |
| D5 | ve-daemon Session Runner | `session_runner.rs` | 状态机、审批缓存、超时追踪、命令处理 |
| D6 | ve-daemon File Ops | `file_ops.rs` | 工作区边界验证、文件树收集、文件分类、文本读取 |
| D7 | ve-daemon Agent Driver | `agent/mod.rs`, `agent/claude_code.rs` | AgentDriver trait、Claude Code 子进程管理 |

---

## 二、已完成功能链路清单

| 编号 | 链路 | 涉及模块 | 描述 |
|------|------|---------|------|
| F1 | 设备注册 & 配对链路 | M2 → M10 | 客户端注册设备 → daemon-hello → 配对状态轮询 → pair 完成 → Token 下发 |
| F2 | Host & Workspace 访问链路 | M2 → M7 → M10 | 认证 → authz 提取器 → host/workspace CRUD |
| F3 | Session 创建 & 执行链路 | M3 → M9 → D3 → D4 → D5 → D7 | 创建会话 → authz → DB 持久化 → Hub send_and_wait → Daemon 启动 Claude Code |
| F4 | Session 消息链路 | M3 → M9 → D3 → D5 | 发送消息 → Hub → Daemon WS → SessionRunner → Claude Code driver |
| F5 | Session 控制链路 | M3 → M9 → D3 → D5 | pause/terminate/rerun/close → Hub → Daemon → 状态回传 |
| F6 | 权限请求/响应链路 | D7 → D5 → D3 → M9 → M4 → M10 | Claude Code → daemon → server WS → DB → 客户端响应 → daemon 回传 |
| F7 | Session 归档链路 | D3 → M9 → M3 → M5 → M10 | session 结束 → Daemon 广播 → server 归档 → 清理 device_session_access |
| F8 | 文件浏览链路 | M6 → M9 → D3 → D6 | 客户端 → file tree/content → Hub send_and_wait → daemon 收集文件树/内容 |
| F9 | 归档浏览 & 删除链路 | M5 → M10 | 客户端 → archive list/get → batch delete |
| F10 | Settings 维护链路 | M8 → M10 | 客户端 → 通知偏好 get/set |
| F11 | Daemon 重连链路 | D3 → D1 → D4 | WS 断开 → 指数退避 → 重连 → 重新 handoff |
| F12 | 后台维护链路 | M11 → M10 | 权限过期扫描 → 幂等键清理 |

---

## 三、已关闭项（本次复核确认已修复）

以下问题来自已有 review 文件，本次确认已修复：

| 原问题 | 状态 | 备注 |
|--------|------|------|
| Legacy ACL fail-open 懒补权 | **CLOSED** | `ensure_legacy_client_access()` 已退化为仅设备存在性检查，不再补写 host/session ACL |
| Pairing secret 可重复利用 | **CLOSED** | `pairing_status()` 配对成功后 CAS 清除 secret |
| Pair code 日志泄露 | **CLOSED** | 已修正 |
| 订阅授权不重新检查 | **CLOSED** | `broadcast_to_session` 已对每个 subscriber 重新查询 `device_session_access` |
| send_and_wait timeout 不覆盖发送 | **CLOSED** | `timeout_at` 已覆盖 send + receive |
| Terminate 在 ACK 后立即归档 | **CLOSED** | 归档延迟到 daemon 发出 `archived` 状态事件 |
| Postgres idempotency UUID 不匹配 | **CLOSED** | Schema 已改为 `VARCHAR(64)` |
| Permission response 先 commit 后发送 | **CLOSED** | CAS 事务内处理 |
| Daemon handoff 断开 pending request 悬挂 | **CLOSED** | connection-scoped fail-fast |
| WebSocket token 坏 token 返回 500 | **CLOSED** | 统一走 JWT decode helper，稳定 401 |
| Files API 原始错误透传 | **CLOSED** | 已做边界脱敏 |
| Migration 006 擦除 rerun provenance | **CLOSED** | placeholder + 动态列检测修复 |
| Settings 路由 claims 提取 device_id | **CLOSED** | 已修复并补回归测试 |
| SQLite migrations 006/007 rerun 字段保留 | **CLOSED** | 回归测试通过 |
| event_rx 在首次连接失败后永久丢失 | **CLOSED** | mpsc 迁移到 broadcast channel，每次重连订阅新 receiver |
| rerun/close 失败后 runner 状态未转换 | **CLOSED** | handle_rerun 错误时显式转为 Error 状态 |

---

## 三之二、HIGH/CRITICAL 项已修复

以下 HIGH/CRITICAL 问题在本次 ECC review 修复过程中已修复：

| 原问题 | 状态 | 修复说明 |
|--------|------|------|
| CRITICAL-D3-01: event_rx 永久丢失 | **CLOSED** | 全量迁移到 broadcast::Sender，connect_and_run 每次重连 subscribe() 新 receiver |
| CRITICAL-D5-01: runner 僵尸 | **CLOSED** | handle_rerun 失败时 update_state(Error) + report_status |
| HIGH-D3-01: 重连循环无退避 | **CLOSED** | 意外错误分支添加指数退避 + jitter |
| HIGH-D4-01: 启动超时孤儿 runner | **CLOSED** | 超时后先 send_close() 再 remove() |
| HIGH-D4-02: create_rerun 无启动确认 | **CLOSED** | new_rerun 接受 startup_completion channel，create_rerun 等待确认 |
| HIGH-M5-01: 批量归档删除范围过大 | **CLOSED** | DELETE 添加 AND device_id = $2 限定 |
| HIGH-M9-01: WS 发送任务 unwrap panic | **CLOSED** | 替换为 match 错误处理，记录日志后 continue |
| HIGH-M10-01: can_resume_cross_device = 1 | **CLOSED** | 使用 .bind(true) 替代硬编码 |

---

## 四、按模块发现的新问题

### M1: Bootstrap & Routing

**MEDIUM-M1-01: `JwtManager` 每次认证请求重新构造**
- **文件**: `api/auth.rs:46,259,455`
- **问题**: `register_device`、`pairing_status`、`pair` 每次调用都从 `jwt_secret` 重新构建 `JwtManager`（包含编码/解码 key 派生），而不是复用 `AppState` 中已有的实例。
- **影响**: 负载下冗余分配和 key 派生开销。
- **建议**: 将 `JwtManager` 放入 `AppState`，与中间件共享。

### M2: Auth & Pairing

**LOW-M2-01: `server_url` 未验证格式直接入库**
- **文件**: `api/auth.rs:66`
- **问题**: `RegisterDeviceRequest` 的 `server_url` 字段未做任何格式或 scheme 验证就存入数据库。
- **影响**: 当前仅用于 daemon 侧启动校验，但如果未来在 UI 暴露或用于重定向，可能成为 XSS/open-redirect 向量。
- **建议**: 添加 `validate_server_url()` 校验 HTTP(S) URI 和合理长度。

**LOW-M2-02: `AuthError` 变体上的 `#[allow(dead_code)]` 已过时**
- **文件**: `middleware/auth.rs:33,40`
- **问题**: 这两个变体已被 `IntoResponse` 和 `auth_middleware` 使用，`#[allow(dead_code)]` 不再需要。
- **建议**: 移除。

### M3: Session Lifecycle

**HIGH-M3-01: Archived rerun dispatching → pending 过早**
- **文件**: `api/sessions.rs:1319-1330`
- **问题**: rerun session 只要消息进入本地 channel 就被改为 `pending`，早于 daemon 正向确认。后续相同 archived session 的 rerun 请求可能复用这个并未真正 ready 的 session。
- **影响**: 破坏 rerun 幂等语义，让调用方看到错误的"已有可复用 rerun"结论。
- **建议**: 将 `dispatching → pending` 时机绑定到 daemon 正向确认事件。
- **追踪**: 已在 `review-session-rerun-open-issues.md:HIGH-01` 中记录。

**HIGH-M3-02: create_session 严格幂等仍有并发双写窗口**
- **文件**: `api/sessions.rs:416-425,454-541`, `db/idempotency.rs:124`
- **问题**: `store.get()` 在事务外执行，插入 session 和写入 `idempotency_keys` 不在同一个原子事务里。并发相同 key 时可能创建出两个 session。
- **影响**: 产生"幽灵 session"。
- **建议**: 将 session 创建和幂等键写入收敛到单个原子事务，或使用数据库级唯一约束 + ON CONFLICT 处理。
- **追踪**: 已在 `review-session-rerun-open-issues.md:HIGH-02` 中记录。

**MEDIUM-M3-01: `get_workspace`/`update_workspace`/`delete_workspace` 冗余双 DB 查询**
- **文件**: `api/workspaces.rs:232-249,340-358,448-462`
- **问题**: fallback handler 先查一次做权限校验，再调用 `*_by_id` helper 又查一次。
- **建议**: 将已查行传入 helper 或内联转换逻辑。

**LOW-M3-01: `close_reason` 未在 API 边界做类型校验**
- **文件**: `api/sessions.rs:1533-1543`
- **问题**: `archive_session_with_metadata` 接受任意 `&str` 作为 `close_reason`，而数据库有 CHECK 约束。
- **建议**: 使用类型化 enum 代替 `&str`。

### M4: Permissions

**MEDIUM-M4-01: Permission response 后未广播给订阅客户端**
- **文件**: `api/permissions.rs:449-462`
- **问题**: 提交权限响应并通知 daemon 后，未通过 hub 广播给订阅了该 session 的客户端。客户端会持续显示 pending 状态直到下次事件。
- **建议**: 在 `tx.commit()` 后通过 hub 发送 session event 广播。

**LOW-M4-01: `get_permission` 冗余双 DB 查询**
- **文件**: `api/permissions.rs:236-266`
- **问题**: 先查完整 permission 行做权限校验，再调用 `get_permission_by_id` 又查一次。
- **建议**: 传入已查行到转换 helper。

### M5: Archives

**HIGH-M5-01: 批量归档删除清除所有设备的 session 访问权限**
- **文件**: `api/archives.rs:480-485`
- **问题**: `DELETE FROM device_session_access WHERE session_id = $1` 删除了该 session 所有设备的访问权限，而不仅仅是请求删除的设备。
- **影响**: 多设备场景下，设备 B 合法拥有该 session 访问权限，会被设备 A 的归档删除操作静默剥夺。
- **建议**: 添加 `AND device_id = $2` 限定只删除请求设备的权限行。

### M7: Hosts & Workspaces

**MEDIUM-M7-01: `list_workspaces` 无分页限制**
- **文件**: `api/workspaces.rs:67-145`
- **问题**: 返回设备可见的所有 workspace，无 `LIMIT` 或分页参数。对于有大量 host/workspace 的设备可能返回数千行。
- **建议**: 添加 `page` 和 `limit` 查询参数，与 `list_archives` 保持一致。

### M8: Settings

**LOW-M8-01: `update_notification_preferences` INSERT 路径存在并发竞态**
- **文件**: `api/settings.rs:120-158`
- **问题**: 先检查 `existing.is_none()` 再 INSERT，两个并发请求可能同时看到无记录并都尝试 INSERT，第二个因 UNIQUE 约束失败。
- **建议**: 使用 `INSERT ... ON CONFLICT DO UPDATE` 替换检查后插入。

### M9: WS Transport & Hub

**HIGH-M9-01: WebSocket 发送任务 `serde_json::to_string().unwrap()` 会 panic**
- **文件**: `ws/client_ws.rs:63`, `ws/daemon_ws.rs:71`
- **问题**: 如果序列化失败（如未来 enum 变体引入循环引用），发送任务直接 panic，WS 连接静默断开，hub 订阅清理可能不及时。
- **建议**: 用 `match` 处理序列化错误，记录日志后 `continue` 或断开连接。

**MEDIUM-M9-01: `broadcast_to_session` clone-before-check**
- **文件**: `hub.rs:312-313`
- **问题**: 在 subscriber 循环中先 `message.clone()` 再检查客户端是否仍连接和已授权。被吊销或已断开的订阅者会导致不必要的 clone。
- **建议**: 先检查连接存在性和授权状态，再 clone。

**MEDIUM-M9-02: 后台任务首次执行延迟一个完整周期**
- **文件**: `tasks/permission_expiry.rs:34`, `tasks/idempotency_cleanup.rs:33`
- **问题**: `ticker.tick().await` 先等待完整间隔再首次执行。权限过期在服务器重启后 60 秒内不会扫描，幂等键清理在 1 小时内不会执行。
- **建议**: 先立即执行一次再进入 ticker 循环。

### M10: DB & Migrations

**CRITICAL-M10-01: Migration 007 删除 `idx_sessions_status` 索引**
- **文件**: `sqlite/007_session_rerun_idempotency.sql:34-40`
- **问题**: Migration 007 重建 sessions 表后重建了 3 个索引，但没有重建 `idx_sessions_status`。增量升级路径的数据库将丢失该索引。
- **影响**: 所有按 status 过滤的查询（会话列表、guard_active_host_session_tx、权限过期扫描）将走全表扫描。
- **建议**: 在 migration 007 末尾添加 `CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);`。

**HIGH-M10-01: `can_resume_cross_device = 1` 硬编码在 SQL 中**
- **文件**: `ws/daemon_ws.rs:371`
- **问题**: 使用整数 `1` 而非布尔 `TRUE`，Postgres 依赖隐式转换。非可移植。
- **建议**: 使用 bind 参数 `bind(true)`。

### M11: Background Tasks

**MEDIUM-M11-01: 权限过期扫描后未通知活跃消费者**
- **文件**: `tasks/permission_expiry.rs:55`
- **问题**: 标记过期权限后，未通知 daemon 或广播给客户端。
- **影响**: 活跃客户端/daemon 可能继续显示过期的权限提示。
- **建议**: 定义并实现过期传播语义，或明确文档化最终一致性预期。

### D3: ve-daemon WS Client

**CRITICAL-D3-01: `event_rx` 在首次连接失败后永久丢失**
- **文件**: `ws_client.rs:264`
- **问题**: `connect_and_run()` 中 `self.event_rx.take()` 永久消费了事件接收器。如果连接失败并重连，后续连接中 `event_rx` 始终为 `None`，事件处理分支永远 pending。所有来自 session runner 的事件（状态更新、权限请求等）将永久静默丢失。
- **影响**: 任何一次瞬态连接失败后，daemon 的所有 session 管理功能将永久失效，且无明显错误信号。
- **建议**: 不要在 `connect_and_run()` 中消费 `event_rx`，而是在成功连接后才 `take()`，失败时恢复；或每次重连时重新创建 channel。

**HIGH-D3-01: 重连循环对意外错误不做退避**
- **文件**: `ws_client.rs:191-194`
- **问题**: `Err(e)` fallback 分支只增加 `retry_count`，不执行 backoff sleep。而 `WsDisconnected`/`ConnectionTimeout` 分支正确地使用了指数退避。
- **影响**: 服务器端持续异常时，daemon 会以最高频率疯狂重连，浪费 CPU/带宽，可能触发服务端限流。
- **建议**: 添加与 `WsDisconnected` 分支相同的 backoff sleep。

### D4: ve-daemon Session Registry

**HIGH-D4-01: 启动超时留下孤儿 runner task**
- **文件**: `session_registry.rs:97-103`
- **问题**: `create()` 启动超时后从 registry 移除 session，但 spawned runner task 继续运行。runner 的命令 channel sender 被丢弃，但 runner 无法检测到这一点，永远不会退出。
- **影响**: 每个启动超时泄漏一个 runner task + 可能的 CLI 子进程。
- **建议**: 移除前发送 Close 命令，或使用 CancellationToken。

**HIGH-D4-02: `create_rerun` 无启动确认**
- **文件**: `session_registry.rs:108-147`
- **问题**: `create_rerun()` 不像 `create()` 那样等待启动确认。`driver.rerun()` 失败时调用方已收到 `Ok(())`。
- **影响**: Server 收到 rerun ACK 以为 session 正在运行，实际 driver 已失败。
- **建议**: 为 `new_rerun()` 添加 `startup_completion` channel，与 `create()` 一致。

### D5: ve-daemon Session Runner

**CRITICAL-D5-01: rerun/close 命令失败后 runner 状态未转换，变成僵尸**
- **文件**: `session_runner.rs:320-321,410-412`
- **问题**: `handle_rerun()` 失败时 `?` 传播错误，但 runner 状态未变为 `Error` 或 `Closed`。`handle_close()` 失败时状态停留在 `Closing`。退出条件 `state == Closed || state == Error` 永远不会触发。
- **影响**: runner 永远运行，消耗内存、持有 channel receiver，阻止 session 清理。
- **建议**: 在 `handle_command` 中包裹错误处理，失败时将状态转为 `Error`。

**MEDIUM-D5-01: `RunnerState::WaitingApproval` 定义但从未使用**
- **文件**: `session_runner.rs:27`
- **建议**: 移除或添加 `#[allow(dead_code)]` 注释说明保留。

### D6: ve-daemon File Ops

**MEDIUM-D6-01: `truncated` 标记永远为 false（死代码）**
- **文件**: `file_ops.rs:420-435`
- **问题**: 超过限制时已返回错误，后续 `truncated` 计算永远为 `false`。
- **建议**: 移除截断逻辑，或真正实现部分读取。

**MEDIUM-D6-02: `collect_tree` 对首个不可访问子目录直接返回错误**
- **文件**: `file_ops.rs:272-303`
- **问题**: `fs::metadata()` 失败时直接返回 `Err`，而非跳过。注释说 "skip" 但代码返回错误。
- **建议**: 改为 `warn!` + `continue`。

### D7: ve-daemon Agent Driver

**MEDIUM-D7-01: `create_driver` 对未知 agent type 回退到 MockDriver**
- **文件**: `agent/mod.rs:216-228`
- **问题**: 未知 agent type 时静默回退到 MockDriver 并报告 `Running` 状态。
- **影响**: 可能导致 server/daemon 状态分歧。
- **建议**: 对不支持的 agent type 返回错误而非 mock 回退。

**MEDIUM-D7-01: `rerun()` 不等待旧 stdout reader task 结束**
- **文件**: `agent/claude_code.rs:559-626`
- **问题**: `close()` 杀掉进程后立即 spawn 新进程和新 stdout reader task，旧 reader task 未等待结束。
- **建议**: 保存 `JoinHandle`，在 `close()` 中等待旧 task 结束。

---

## 五、按功能链路发现问题

### F1: 设备注册 & 配对链路
- **MEDIUM-M1-01**: JwtManager 每次重新构造（性能）
- **LOW-M2-01**: server_url 未验证
- **LOW-M2-02**: stale `#[allow(dead_code)]`
- **已关闭**: legacy ACL、pairing secret 重用、pair code 日志泄露

### F2: Host & Workspace 访问链路
- **MEDIUM-M7-01**: `list_workspaces` 无分页
- **MEDIUM-M3-01**: workspace 操作冗余双 DB 查询
- **已关闭**: legacy ACL fail-open

### F3: Session 创建 & 执行链路
- **HIGH-M3-02**: create_session 严格幂等并发双写窗口
- **MEDIUM-M9-01**: broadcast clone-before-check
- **MEDIUM-M9-02**: 后台任务首次执行延迟
- **已关闭**: send_and_wait timeout、terminate 过早归档

### F4: Session 消息链路
- **HIGH-M9-01**: WS 发送任务 unwrap panic
- **HIGH-M10-01**: 硬编码 boolean 字面量
- **已关闭**: daemon handoff 断开、phantom message

### F5: Session 控制链路
- **HIGH-M3-01**: rerun dispatching → pending 过早
- **CRITICAL-D5-01**: runner 僵尸 task
- **HIGH-D3-01**: 重连循环无退避
- **已关闭**: terminate 归档、rerun provenance 擦除

### F6: 权限请求/响应链路
- **MEDIUM-M4-01**: response 后未广播
- **MEDIUM-M11-01**: 权限过期未传播
- **已关闭**: CAS 并发、permission response 竞态

### F7: Session 归档链路
- **HIGH-M5-01**: 批量归档删除清除所有设备访问权

### F8: 文件浏览链路
- **MEDIUM-D6-01**: truncated 死代码
- **MEDIUM-D6-02**: collect_tree 跳过逻辑不生效
- **已关闭**: files API 错误透传

### F9: 归档浏览 & 删除链路
- **HIGH-M5-01**: 同上

### F10: Settings 维护链路
- **LOW-M8-01**: notification INSERT 竞态

### F11: Daemon 重连链路
- **CRITICAL-D3-01**: event_rx 永久丢失
- **HIGH-D3-01**: 重连循环无退避

### F12: 后台维护链路
- **MEDIUM-M11-01**: 权限过期未传播

---

## 六、按严重级别汇总

### CRITICAL (必须优先修复)

全部已修复，见"三之二、HIGH/CRITICAL 项已修复"。

### HIGH (交付前应修复)

全部已修复，见"三之二、HIGH/CRITICAL 项已修复"。HIGH-M3-01 和 HIGH-M3-02 已在 `review-session-rerun-open-issues.md` 中追踪。

### MEDIUM (明显缺口)

| ID | 模块 | 问题 | 状态 |
|----|------|------|------|
| MEDIUM-M9-02 | M11 | 后台任务首次执行延迟 | **CLOSED** |
| MEDIUM-D6-02 | D6 | collect_tree 跳过逻辑不生效 | **CLOSED** |
| MEDIUM-D7-01 | D7 | 未知 agent type 回退 mock | **CLOSED** |
| MEDIUM-M4-01 | M4 | Permission response 后未广播 | **CLOSED** |
| MEDIUM-M1-01 | M1 | JwtManager 每次重新构造 | 待处理，需重构 AppState |
| MEDIUM-M3-01 | M7 | workspace 操作冗余双 DB 查询 | 待处理，需重构 helper |
| MEDIUM-M7-01 | M7 | list_workspaces 无分页 | 待处理，需 API 变更 |
| MEDIUM-M9-01 | M9 | broadcast clone-before-check | 实际代码已优化，review 描述已过时 |
| MEDIUM-M11-01 | M11 | 权限过期未传播 | 待处理，需新机制 |
| MEDIUM-D5-01 | D5 | WaitingApproval 状态未使用 | 无编译器警告，非有害代码 |
| MEDIUM-D6-01 | D6 | truncated 死代码 | 待处理，需行为变更 |
| MEDIUM-D7-02 | D7 | rerun 不等待旧 stdout reader | 待处理 |

### LOW (建议改进)

| ID | 模块 | 问题 |
|----|------|------|
| LOW-M2-01 | M2 | server_url 未验证 |
| LOW-M2-02 | M2 | stale #[allow(dead_code)] |
| LOW-M3-01 | M3 | close_reason 未做类型校验 |
| LOW-M4-01 | M4 | get_permission 冗余双 DB 查询 |
| LOW-M8-01 | M8 | notification INSERT 竞态 |

---

## 七、优先级建议

1. **立即处理 CRITICAL**: event_rx 丢失（D3-01）是最危险的 bug——任何一次瞬态连接失败就永久破坏 daemon 核心功能且无错误信号。Runner 僵尸（D5-01）和索引丢失（M10-01）同样应立即修复。

2. **其次处理 HIGH**: ve-daemon 重连退避（D3-01）、孤儿 runner（D4-01）、rerun 无启动确认（D4-02）直接影响 daemon 可靠性。ve-server 侧的归档删除范围（M5-01）和 WS panic（M9-01）影响线上稳定性。

3. **rerun 幂等问题**: M3-01 和 M3-02 已在 `review-session-rerun-open-issues.md` 中追踪，应按原计划推进修复。

4. **MEDIUM 项**: 作为可靠性改进排在 HIGH 之后。其中权限 response 广播（M4-01）和后台任务首次执行延迟（M9-02）对用户体验影响最直接。

5. **LOW 项**: 代码卫生改进，可在后续迭代中处理。
