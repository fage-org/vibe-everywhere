# Vibe Everywhere 全量模块与功能链路 ECC Comprehensive Review

**Reviewed**: 2026-04-26
**Scope**: ve-server + ve-daemon + ve-shared 全部已完成服务模块
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
| M3 | ve-server Session Lifecycle | `api/sessions/*` | 会话创建/列表/获取/发消息/控制/关闭/归档/重跑 |
| M4 | ve-server Permissions | `api/permissions.rs` | 权限请求列表/获取/响应，CAS 并发更新保护 |
| M5 | ve-server Archives | `api/archives/*` | 归档列表/获取/批量删除 |
| M6 | ve-server Files | `api/files.rs` | 通过 send_and_wait 向 daemon 请求文件树/文件内容 |
| M7 | ve-server Hosts & Workspaces | `api/hosts.rs`, `api/workspaces/*` | 主机 CRUD、工作区 CRUD |
| M8 | ve-server Settings | `api/settings.rs` | 通知偏好设置 |
| M9 | ve-server WS Transport & Hub | `hub.rs`, `ws/client_ws.rs`, `ws/daemon_ws.rs` | WS 连接管理、会话订阅、请求/响应关联、事件广播 |
| M10 | ve-server DB & Migrations | `db/mod.rs`, `db/idempotency.rs`, `migrations/*` | DB 池、迁移运行、幂等键存储 |
| M11 | ve-server Background Tasks | `tasks/permission_expiry.rs`, `tasks/idempotency_cleanup.rs` | 定时任务：过期权限标记、幂等键清理 |
| D1 | ve-daemon Config & Credentials | `config.rs`, `credentials.rs` | 配置加载、本地凭据存储（0o600 权限） |
| D2 | ve-daemon Pairing | `pairing.rs`, `pairing_identity.rs` | 配对状态机、安装身份管理 |
| D3 | ve-daemon WS Client | `ws_client/*` | 自动重连（指数退避+抖动）、心跳、事件转发 |
| D4 | ve-daemon Session Registry | `session_registry.rs` | SessionRunner 句柄注册表、并发控制、最大并行会话限制 |
| D5 | ve-daemon Session Runner | `session_runner/*` | 状态机、审批缓存、超时追踪、命令处理 |
| D6 | ve-daemon File Ops | `file_ops/*` | 工作区边界验证、文件树收集、文件分类、文本读取 |
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

## 三、按模块发现的新问题

### M2: Auth & Pairing

#### CRITICAL-M2-01: `legacy_acl` 列完全无授权效果 — 死访问控制
- **文件**: `authz.rs:21-38` (`ensure_legacy_client_access`), `auth.rs:415-424` (写入点)
- **问题**: `ensure_legacy_client_access` 只做设备存在性检查 (`SELECT 1 FROM client_devices WHERE device_id = $1`)，完全忽略了 `legacy_acl` 列。所有 `client_devices` 中的设备无论 `legacy_acl` 值如何，都获得同等的 host/session 访问权限。
- **影响**: `legacy_acl` 列对授权零影响。如果设计意图是通过 `legacy_acl` 控制设备是否有配对后访问权限，这是彻底的访问控制失效。
- **建议**: 要么实现检查 (`WHERE legacy_acl = 0`)，要么删除 `legacy_acl` 列及其写入逻辑（如果是废弃列）。

#### CRITICAL-M2-02: 原子配对码声明使用 `CURRENT_TIMESTAMP` — PostgreSQL 类型不匹配
- **文件**: `auth.rs:384-389`
- **问题**: 原子声明 UPDATE 使用 `expires_at > CURRENT_TIMESTAMP` 做 WHERE 过滤。`expires_at` 是 `TEXT NOT NULL` 列，在 PostgreSQL 中 `CURRENT_TIMESTAMP` 返回 `TIMESTAMPTZ`。TEXT 与 TIMESTAMPTZ 比较触发隐式转换，可能导致解析失败或错误的比较结果。
- **影响**: 在 PostgreSQL 上，过期的配对码可能仍然可被声明（过期检查可能无声失败），造成配对绕过。
- **建议**: 将 timestamp 作为 Rust 参数化传递（已在 `daemon_hello` handler 中使用此模式），而非依赖 SQL `CURRENT_TIMESTAMP`。

#### CRITICAL-M2-03: WebSocket 路径跳过 `jti_matches_device` 检查 — 旧 Token 在 WS 上永远有效
- **文件**: `authz.rs:178-187` (缺少检查) vs `middleware/auth.rs:83-95` (有此检查)
- **问题**: HTTP 中间件同时检查 `jti_matches_device` 和 `is_revoked`，但 WebSocket 认证路径 (`decode_ws_claims`) 只检查 `is_revoked`。
- **影响**: 设备重新配对后（会轮换 `current_jti`），旧 Token 仍然可以建立 WebSocket 连接，因为 WS 路径从未调用 `jti_matches_device`。
- **建议**: 在 `decode_ws_claims` 中添加 `jti_matches_device` 检查，与 HTTP 中间件保持一致。

#### HIGH-M2-01: `daemon_hello` 中使用 `format!` 构建 SQL — 脆弱的模式
- **文件**: `auth.rs:172-183`
- **问题**: `format!()` 宏将 `{expires_at_expr}` 内联进 SQL 字符串。虽然当前 `ttl_secs` 是数字配置值（安全），但如果将来变为用户可控制的输入，则成为 SQL 注入向量。
- **建议**: 在 Rust 中计算 `expires_at` 值并通过 bind 参数 (`$7`) 传入 SQL，消除 `format!` 使用。

#### HIGH-M2-02: jti 更新在配对事务之外 — 失败时设备进入许可性认证状态
- **文件**: `auth.rs:476-492`
- **问题**: 配对事务在 line 469 提交后，撤销旧 token (line 488) 和设置新 jti (line 492) 分别执行。如果 jti 更新失败，设备 `current_jti` 变为 NULL，在此状态下 `jti_matches_device` 返回 `true`（匹配一切 token）。
- **影响**: 配对后部分失败使设备进入所有 Token 均有效的许可性认证状态。
- **建议**: 将 jti 轮换放入配对事务中。

#### HIGH-M2-03: `ClientBootstrap` Token 从未在 HTTP 中间件中检查撤销
- **文件**: `middleware/auth.rs:82-97`
- **问题**: 撤销检查仅限 `TokenType::Client`，`ClientBootstrap` Token 无法被撤销。如果 bootstrap token 被盗，没有撤销机制。
- **建议**: 扩展撤销检查至 `TokenType::ClientBootstrap`。

#### HIGH-M2-04: `jti_matches_device` 和 `is_revoked` 在 DB 错误时 Fail-Open
- **文件**: `middleware/auth.rs:87-90`
- **问题**: `jti_matches_device` 的错误用 `unwrap_or(true)` 处理（DB 错误 = token 匹配）。`is_revoked` 的错误用 `unwrap_or(false)` 处理（DB 错误 = 未撤销）。
- **影响**: 如果数据库不可用或超时，所有 Token 通过中间件认证（fail-open）。
- **建议**: 在数据库错误时拒绝请求（fail-closed），改为真正的错误传播。

#### HIGH-M2-05: `daemon_token` 明文存储到数据库
- **文件**: `auth.rs:509-518`
- **问题**: Daemon 离线时，完整 daemon JWT 存储为 `pending_daemon_token`（明文）。
- **影响**: 数据库遭入侵时所有 daemon token 暴露。
- **建议**: 哈希后存储，或加密 token。

#### MEDIUM-M2-01: 配对成功后 `pairing_secret` 未清理
- **文件**: `auth.rs:260-275`
- **问题**: `pair()` 标记 `used = 1`，但 `pairing_status()` 不检查 `used`。配对成功后 `pairing_secret` 未被清空。
- **影响**: 在 TTL 窗口内获得 `pairing_secret` 的人在合法配对完成后仍可通过 `pairing-status` 轮询获取 daemon token。
- **建议**: 配对成功时清空 `pairing_secret`，`pairing_status()` 拒绝 `used` 记录。

#### MEDIUM-M2-02: `pair_code` 在日志中输出（调试模式）
- **文件**: `auth.rs:194`
- **影响**: 日志读者或集中日志系统可赎回有效配对码。
- **建议**: 从日志中移除或遮盖原始配对码。

#### MEDIUM-M2-03: `device_name` 未在验证后使用修剪值
- **文件**: `auth.rs:40, 71, 86`
- **问题**: `validate_device_name` 内部做了 trim，但返回原始值。JWT claim 和 DB 存储使用的是未修剪的 `req.device_name`。
- **影响**: `"  MyPhone  "` 可验证通过但以含前导/尾随空格的形式存储。

#### LOW-M2-01: `pair()` 中的设备存在检查有 TOCTOU
- **文件**: `auth.rs:322-336`
- **问题**: 检查 `device_exists` 后到实际配对操作之间设备可能被删除。

---

### M3: Session Lifecycle

#### CRITICAL-M3-01: PostgreSQL 下并发 Rerun 创建重复 Session
- **文件**: `api/sessions/control.rs:191-218` (检查在事务外) 和 `control.rs:219-295` (事务内 INSERT)
- **问题**: `has_dispatching_rerun` 检查在事务外执行。PostgreSQL MVCC 下，两个并发的 rerun 请求可以同时通过检查 → 各自在事务内 INSERT → 两个都成功提交 → 两个 dispatching session 指向同一个归档 session。
- **影响**: 同一归档 session 派生出两个独立运行的 rerun session，`find_reusable_rerun_session_id` 的设计假设每个归档至多一个活跃 rerun。
- **建议**: 添加部分唯一索引 `CREATE UNIQUE INDEX ON sessions (rerun_from_session_id) WHERE status = 'dispatching'`，或将 `has_dispatching_rerun` 移入事务内并使用 `SELECT ... FOR UPDATE`。

#### CRITICAL-M3-02: 服务崩溃留下孤儿 `dispatching` Session 永久阻塞 Rerun
- **文件**: `api/sessions/control.rs:345-354` (INSERT 事务已提交后，状态更新在事务外)
- **问题**: Rerun 流程：1) 事务中 INSERT status='dispatching' 并提交 → 2) 发送 daemon 命令等 ACK → 3) UPDATE status='pending'（无事务）。如果服务在步骤 2-3 之间崩溃，session 永远停在 dispatching。`has_dispatching_rerun` 将永久阻止所有后续 rerun。
- **影响**: 一次服务崩溃永久阻止该归档 session 的所有 future rerun，需人工 DB 干预。
- **建议**: 添加启动恢复逻辑，将超时的 orphaned dispatching session 转入 error 状态。

#### HIGH-M3-01: `list_sessions` 无分页 — 无界结果集
- **文件**: `api/sessions/crud.rs:29-85`
- **问题**: `list_sessions` 使用 `fetch_all` 无 `LIMIT`/`OFFSET`。拥有数千 session 的设备会导致无界内存和 DB 负载。
- **建议**: 添加 `page` / `limit` 参数。

#### HIGH-M3-02: Archived Rerun 授权存在潜在绕过风险
- **文件**: `api/sessions/control.rs:124-131` 和 `control.rs:100-102`
- **问题**: `handle_archived_rerun` 是 `pub(crate)` 且内部不做授权检查，完全信任调用方已验证。如果未来有其他调用方直接调用此函数，授权被绕过。
- **建议**: 将 `require_host_access` 检查移入 `handle_archived_rerun` 内部。同时将元组参数改为类型化的 struct。

#### MEDIUM-M3-01: `list_messages` 整数溢出
- **文件**: `api/sessions/messages.rs:165`
- **问题**: `let offset = (page - 1) * limit;` 在 `u32` 上操作。大 page 值导致 debug 模式 panic (DoS)，release 模式 wrap 到 small 值产生错误结果。
- **建议**: 添加 page 上限验证或使用 checked/saturating 算术。

#### MEDIUM-M3-02: `limit: 0` 未被拒绝
- **文件**: `api/sessions/messages.rs:164`
- **问题**: `.min(100)` 对 0 返回 0，`LIMIT 0` 查询浪费但不会 error。
- **建议**: 显式验证 `limit > 0`。

#### MEDIUM-M3-03: `archive_session_with_metadata` 在事务外读取元数据，事务内只写 UPDATE
- **文件**: `api/sessions/control.rs:492-545` (事务外读取) vs `control.rs:552` (事务开始)
- **问题**: message_count/permission_count 在事务外读取，事务内 UPDATE session。期间消息/权限可能新增，使归档元数据不准确。
- **建议**: 将计数查询移入事务内。

#### MEDIUM-M3-04: `validate_content` 使用修剪后长度但存储未修剪内容
- **文件**: `api/sessions/validation.rs:102-113` vs `messages.rs:102`
- **问题**: 验证用 trimmed length 检查 ≤100KB，但未修剪的完整字符串存入 DB。99KB 内容 + 1KB 空格可绕过限制。
- **建议**: 要么验证未修剪长度，要么存储前 trim。

#### LOW-M3-01: Rerun Session 硬编码 `can_resume_cross_device = TRUE`
- **文件**: `api/sessions/control.rs:228-229`
- **问题**: 不继承原归档 session 的 `can_resume_cross_device` 设置，始终设为 TRUE。
- **建议**: 复制原 session 的值。

#### LOW-M3-02: `close_session` 不更新 Session 状态或建归档
- **文件**: `api/sessions/close.rs:42-103`
- **问题**: `close_session_for_id` 只发送命令给 daemon 并立即返回。实际归档由 daemon 回调异步完成。客户端可能困惑于 session 状态未变。

---

### M4: Permissions

#### HIGH-M4-01: `get_permission_by_id` 死代码标注误导 — 潜在授权绕过
- **文件**: `permissions.rs:265-284`
- **问题**: 函数无任何授权检查但标注 `#[allow(dead_code)]`。实际在 `respond_permission_existing` (line 369) 中被调用（调用方已验证授权）。标注掩盖了这是一个需要授权上下文包裹的 live 代码路径。
- **建议**: 移除 `#[allow(dead_code)]`，并考虑重命名为 `unsafe_get_permission_by_id` 以警示缺少内置授权检查。

#### MEDIUM-M4-01: 权限过期未传播给 daemon 的错误被静默吞掉
- **文件**: `tasks/permission_expiry.rs:128-137`
- **问题**: `let _ = hub.send_to_daemon(...).await;` — daemon 通知失败被静默忽略。
- **影响**: Daemon 断开时，过期权限在 DB 中正确标记但 daemon 永远不知道，可能继续执行本应停止的操作。
- **建议**: 至少记录 warning 日志。

#### LOW-M4-01: `PermissionListQuery` 是死代码
- **文件**: `permissions.rs:109-112`
- **问题**: `pub` struct 从未被使用。`list_permissions` 使用 `PermissionCollectionAccess` extractor。

#### LOW-M4-02: `InvalidChars` 变体上的 `#[allow(dead_code)]` 误导
- **文件**: `validation.rs:31`
- **问题**: `InvalidChars` 实际被 `validate_pair_code` 和 `validate_device_id_format` 使用，其上的 `#[allow(dead_code)]` 不正确。

---

### M5: Archives

#### MEDIUM-M5-01: 批量归档删除的 Session 访问权限清理可能不完整
- **文件**: `api/archives/delete.rs:96, 104-113`
- **问题**: `deleted_session_ids` 仅在 `rows_affected() > 0` 时收集。如果并发请求已删除归档，rows_affected 为 0，但 session 访问权限清理被跳过。
- **影响**: 留下孤立 `device_session_access` 行。

---

### M6: Files

#### MEDIUM-M6-01: `validate_relative_path` 不拒绝绝对路径
- **文件**: `files.rs:59-69`
- **问题**: 只检查 `..` 组件，绝对路径（如 `/etc/passwd`）可通过。虽然 daemon 侧有 canonicalize + starts_with 验证，但绝对路径仍可能与其验证逻辑交互。
- **建议**: 添加 `Component::RootDir` 检查，拒绝绝对路径。

---

### M7: Hosts & Workspaces

#### CRITICAL-M7-01: `unbind_host_by_id` 多步 DELETE 无事务 — 数据一致性风险
- **文件**: `hosts.rs:258-298`
- **问题**: 四个顺序 DELETE（device_session_access → sessions → session_archives → hosts）无事务边界。操作之间崩溃导致不可恢复的数据不一致。计数检查 (lines 217-243) 与实际删除之间还存在 TOCTOU 竞态。
- **影响**: 部分解绑 — host 已删除但 sessions/archives 保持孤立，或反之。
- **建议**: 将所有检查和 DELETE 包裹在单个事务中。

#### MEDIUM-M7-01: Workspace 端点使用手动 auth 而非 `WorkspaceAccess` extractor
- **文件**: `workspaces/get.rs:18-47`, `update.rs:27-55`, `delete.rs:15-49`
- **问题**: get/update/delete 手动调用 `require_client_device_id` + `require_host_access`（两个独立 DB 查询），而非使用单查询的 `WorkspaceAccess` extractor。
- **建议**: 复用 `WorkspaceAccess` extractor 减少 DB 往返。

#### LOW-M7-01: Workspace update 响应中 `updated_at` 与 DB 不匹配
- **文件**: `workspaces/update.rs:79, 84`
- **问题**: DB 用 `CURRENT_TIMESTAMP` 更新，响应用 `chrono::Utc::now()` 构造。两个时间戳可能差几毫秒。

#### LOW-M7-02: UNIQUE 约束检测使用字符串匹配
- **文件**: `workspaces/create.rs:105-111`
- **问题**: `e.to_string().contains("UNIQUE constraint")` — 错误消息因 SQLite 版本、PostgreSQL 方言、locale 而异。

---

### M8: Settings

#### LOW-M8-01: 冗余的 `validate_device_id_format` 调用
- **文件**: `settings.rs:24-25, 98`
- **问题**: `require_client_device_id` 已返回有效 `Uuid`，再调用 `validate_device_id_format` 永不失败。

#### LOW-M8-02: 设备存在检查与 settings upsert 之间的 TOCTOU
- **文件**: `settings.rs:27-39, 100-134`
- **问题**: 检查设备存在和后续 UPSERT 不在事务中。实际影响极小。

---

### M9: WS Transport & Hub

#### HIGH-M9-01: Daemon 配对 token 可能在连接丢失时永久丢失
- **文件**: `daemon_ws.rs:147-178` (`deliver_pending_daemon_token`)
- **问题**: 将 token 推入 bounded mpsc channel 后立即从数据库清除 (`pending_daemon_token = NULL`)。如果 send_task 未能实际送达 daemon（序列化失败、WS 发送失败）、token 永远丢失 — DB 中已清除但 daemon 从未收到。
- **影响**: 需要人工重新配对的运营中断。
- **建议**: 仅在收到 daemon 确认回执后再清除 DB 中的 token。

#### HIGH-M9-02: `WsEnvelope::new` 负载序列化失败时无声产生 `Value::Null`
- **文件**: `proto.rs:28-34` (或对应源路径)，在 `hub.rs` 和其他地方普遍使用
- **问题**: `serde_json::to_value(&payload).unwrap_or(serde_json::Value::Null)` — 如果 payload 类型不可序列化（编程错误），所有地方无声产生 null payload。
- **影响**: 客户端/daemon 收到结构有效但语义为空的消息，无错误信号。服务器和客户端之间状态漂移。
- **建议**: 传播错误或使用可区分的 `__serialization_error__` sentinel 值。

#### MEDIUM-M9-01: `daemon_ws.rs` 中对 "error" envelope 的冗余 `complete_with_error` + `handle_response` 双路径
- **文件**: `daemon_ws.rs:345-353`
- **问题**: "error" arm 先调用 `complete_with_error`（移除 pending entry + 发送 Error），再调用 `handle_response`（尝试再次移除同一 request_id → 因已被移除而 no-op）。无害但控制流混乱。
- **建议**: `complete_with_error` 后直接 `return Ok(true)`，与 "ack" arm 保持一致。

#### MEDIUM-M9-02: `send_and_wait` 成功路径隐式依赖外部清理
- **文件**: `hub.rs:476-477`
- **问题**: 成功路径 `Ok(Ok(Ok(response))) => Ok(response)` 不显式移除 pending request。所有其他结果路径显式清理。如新增不清理的 response 类型，pending entry 泄漏。
- **建议**: 在成功路径添加防御性清理作为安全网。

#### MEDIUM-M9-03: `unsubscribe_session` 缺乏授权检查
- **文件**: `client_ws.rs:125-134`
- **问题**: `subscribe_session` 有授权检查，但 `unsubscribe_session` 没有。当前无害但违反对称授权原则。

#### LOW-M9-01: `broadcast_to_session` 每次广播时查询 DB
- **文件**: `hub.rs:310-331`
- **问题**: 每次广播验证所有订阅设备的 `device_session_access`。高吞吐流式输出下可能耗尽 DB 连接池。
- **建议**: 考虑短期 TTL 缓存。

#### LOW-M9-02: `send_task.abort()` 与自然任务退出竞态
- **文件**: `client_ws.rs:99`, `daemon_ws.rs:141`
- **问题**: `abort()` 在 sender 已被移除后调用，任务自然退出。无害但多余。

#### LOW-M9-03: JWT 过期只在连接时检查，不在会话中检查
- **文件**: `authz.rs:173-175`
- **问题**: 长寿命 WebSocket 连接（数天）即使 JWT 已过期仍继续操作。标准 WS 模式但意味着 token 撤销仅在下次连接时生效。

---

### M10: DB & Migrations

#### CRITICAL-M10-01: SQLite 上幂等键永不过期 — 无限表增长
- **文件**: `idempotency.rs:127-139`, `sqlite/002_supplemental_fields.sql:10`
- **问题**: `store()` 的 INSERT 省略 `expires_at` 列。SQLite migration 002 添加的 `expires_at` 没有 DEFAULT，所以每个新键 `expires_at = NULL`。`delete_expired()` 过滤 `expires_at IS NOT NULL`，在 SQLite 上永不清理任何键。
- **影响**: SQLite 部署上 `idempotency_keys` 表无限增长，最终影响性能。`delete_expired()` 在 SQLite 上是死代码。
- **建议**: 在 `store()` INSERT 中添加 `expires_at` 绑定值。

#### CRITICAL-M10-02: SQLite vs PostgreSQL `expires_at` 可空性不匹配
- **文件**: `sqlite/001_initial.sql:139-143` vs `postgres/001_initial.sql:139-146`
- **问题**: SQLite: `TEXT` (nullable, no default)；PostgreSQL: `TIMESTAMPTZ NOT NULL DEFAULT ...`。列在两种后端上有根本不同的 nullability 和默认行为。
- **影响**: 任何假设 `expires_at IS NOT NULL` 的逻辑在 SQLite 上无声失败。
- **建议**: 为 SQLite 添加 `DEFAULT (datetime('now', '+24 hours'))` 并回填现有 NULL 行。

#### CRITICAL-M10-03: `store()` 返回值不验证 `request_hash` — 幂等性绕过
- **文件**: `idempotency.rs:160-169`
- **问题**: 当两个并发请求使用相同 idempotency key 但不同 body 时，`store()` 遇到 UNIQUE 冲突后 fallback 到 `self.get(key)`，返回已存在记录但**不比较 `request_hash`**。`verify_hash()` 方法存在但从未在 `store()` 中调用。
- **影响**: 客户端可复用 idempotency key 搭配完全不同的请求，无声收到旧结果。
- **建议**: 在 duplicate-key handler 中比较 hash，不匹配时返回 Conflict 错误。

#### CRITICAL-M10-04: SQLite Migration 007 每次启动无条件删除重建 Sessions 表
- **文件**: `mod.rs:463-469`, `sqlite/007_session_rerun_idempotency.sql:1-49`
- **问题**: Migration 007 每次启动都执行（无 guard 条件），执行 `DROP TABLE sessions; ALTER TABLE sessions_new RENAME TO sessions;`。加上 migration 006 也总是执行同样操作，sessions 表每次启动被重建两次。
- **影响**: 每次重启的启动性能退化、不必要的写入 I/O、事务失败时可能丢数据。
- **建议**: 检查 partial unique index 是否已存在，已存在则跳过。

#### HIGH-M10-01: SQLite 全新安装缺少 Host 状态索引
- **文件**: `mod.rs:432-443`, `sqlite/004_windows_platform.sql:50-52`
- **问题**: `idx_hosts_online_status`、`idx_hosts_daemon_status`、`idx_hosts_pair_status` 在 migration 004 中创建，但 migration 004 仅在 `sqlite_hosts_supports_windows()` 返回 false 时执行。由于 001_initial 已包含 'windows'，此检查对新安装返回 true，迁移被跳过 — 三个 host 状态索引永不创建。
- **影响**: 全新 SQLite 安装上所有按 online/daemon/pair 状态过滤的查询全部全表扫描。
- **建议**: 将索引创建移出 migration 004 到独立迁移或 001_initial。

#### HIGH-M10-02: 跨 DB CASCADE 行为不一致 (sessions.host_id)
- **文件**: `postgres/010_host_fk_cascades.sql:11-13`, `sqlite/001_initial.sql:52`
- **问题**: PostgreSQL migration 010 添加 `ON DELETE CASCADE` 到 `sessions.host_id` FK。SQLite 在同 FK 上**没有 CASCADE**。同样，`session_archives.host_id` 在 PostgreSQL 有 CASCADE FK，SQLite 无 FK 约束。
- **影响**: PostgreSQL 上删 host 级联删除 sessions/archives；SQLite 上因 FK violation 失败。任何依赖级联删除的逻辑在 SQLite 上崩溃。
- **建议**: 为 SQLite 添加等效 CASCADE 或文档化需要手动清理。

#### HIGH-M10-03: SQLite Migration 006 无条件重建 Sessions（非幂等）
- **文件**: `mod.rs:293-310`, `sqlite/006_session_pending_status.sql`
- **问题**: Migration 006 每次启动无条件执行，执行 `DROP TABLE sessions` + rename 重建。
- **影响**: 与 C4 相同 — 不必要的表重建、写 I/O、数据风险。
- **建议**: 检查 `rerun_from_session_id` 列和索引是否已存在。

#### HIGH-M10-04: Migration 004 中 Foreign Key 开关错误处理缺口
- **文件**: `mod.rs:201-268`
- **问题**: `PRAGMA foreign_keys = OFF` ... 迁移操作 ... `PRAGMA foreign_keys = ON`。如果重新启用 FK 失败（因迁移中引入孤立引用），连接返回给连接池时 FK 仍为 OFF。
- **影响**: 后续使用此连接的查询无声 bypass FK 约束，可能造成数据损坏。
- **建议**: FK 启用失败时从池中移除连接。

#### MEDIUM-M10-01: `store()` 中重复键 fallback 的竞态窗口
- **文件**: `idempotency.rs:142-169`
- **问题**: INSERT 失败和 `self.get(key)` fallback 之间，并发 `delete_expired()` 可能删除记录。代码返回 `Internal("key disappeared")` 而非重试或返回合适的 NotFound。
- **影响**: 偶发 500 错误。

#### MEDIUM-M10-02: SQLite `session_archives` 缺少多个 FK
- **文件**: `sqlite/001_initial.sql:98-107`
- **问题**: `session_archives.host_id` 无 FK，`session_id` 和 `workspace_id` 在两个后端都无 FK。记录可引用不存在的 sessions 或 workspaces。

#### MEDIUM-M10-03: Migration 编号在两种后端之间不匹配
- **问题**: SQLite "supplemental fields" (002) 的内容在 PostgreSQL 中 baked into 001_initial。SQLite 003 (device access) 对应 PostgreSQL 002。不同后端的迁移号无法直接比对。

#### LOW-M10-01: PostgreSQL Migration 005 是空操作
- **问题**: 删除并重建的 CHECK 约束与 `001_initial.sql` 中的完全相同（都包含 `'pending'`）。此迁移在当前初始 Schema 上不做任何事。

#### LOW-M10-02: SQLite 布尔值使用 INTEGER，PostgreSQL 使用 BOOLEAN
- **问题**: SQLite 用 `INTEGER NOT NULL DEFAULT 1`，PostgreSQL 用 `BOOLEAN NOT NULL DEFAULT TRUE`。any driver 处理映射但直接 DB 访问会看到差异。

---

### M11: Background Tasks

#### CRITICAL-M11-01: `expire_stale_permissions_in_transaction` 实际上不在事务中
- **文件**: `permission_expiry.rs:153-228`
- **问题**: 函数名为 "in_transaction" 但代码中没有任何 `state.db.begin()` 调用。标记 permissions 为 expired (line 158) 和重算 `pending_permission_count` (line 178) 独立执行。第二个 UPDATE 失败时，permissions 标记为 expired 但 session counter 永不被重算。
- **影响**: Session `pending_permission_count` 与实际情况永远偏离，导致 UI 状态错误、session 可能卡死。
- **建议**: 添加 `let mut tx = db.begin().await?;` 包裹两个操作。

#### MEDIUM-M11-01: 后台任务首次执行延迟一个完整周期
- **文件**: `permission_expiry.rs:34`, `idempotency_cleanup.rs:33`
- **问题**: `ticker.tick().await` 先等待完整间隔才首次执行。权限过期在重启后 60 秒内不扫描，幂等键清理 1 小时内不执行。
- **建议**: 先立即执行一次再进入 ticker 循环。

---

### D3: ve-daemon WS Client

#### CRITICAL-D3-01: 重连时 `event_rx` 为 None 导致 panic
- **文件**: `ws_client/connection.rs:211` 和 `connection.rs:117-119`
- **问题**: `connect_and_run()` 首次调用通过 `self.event_rx.take()` 消费了 `event_rx`。重连时 `self.event_rx` 为 None，但 line 211 仍有 `event_rx.as_mut().unwrap().recv()` — 直接 panic。
- **影响**: 每次重连都 crash daemon，彻底破坏重连机制。
- **建议**: 改用 `tokio::sync::broadcast` channel 以支持每次重连 subscribe 新 receiver。

#### HIGH-D3-01: 重连期间事件丢失（即使修复 panic）
- **文件**: `ws_client/connection.rs:118-119`
- **问题**: 即使修复 panic，`event_rx` local 变量在 connection 断开时 drop，session runners 持有的 `event_tx` 发送的所有事件在重连窗口期间无声丢失。
- **建议**: 使用 broadcast channel 或实现重连期间的事件缓存队列。

---

### D4: ve-daemon Session Registry

现有 Review 已覆盖此模块的问题且大部分已修复。当前无新增 CRITICAL/HIGH。

---

### D5: ve-daemon Session Runner

#### CRITICAL-D5-01: `update_state` 验证失败仍应用状态转换
- **文件**: `session_runner/state_manager.rs:70-86`
- **问题**: `validate_transition` 正确检查了允许的状态转换，但 `update_state` 只记录 warning — 仍然应用转换 `self.state = new_state`。
- **影响**: 任何调用 `update_state` 使用无效转换的代码路径都会无声损坏状态机。所有基于 `self.state` 的状态 guard 变得不可靠。
- **建议**: 改为 `update_state` 返回 `Result<()>` 并拒绝无效转换，或 panic（防御性编程）。

---

### D6: ve-daemon File Ops

#### HIGH-D6-01: File Ops 使用消息中的 workspace_path 无 Session 关联验证
- **文件**: `ws_client/file_handlers.rs:88-107, 180-196`
- **问题**: `handle_file_tree_request` 和 `handle_file_content_request` 从 WS 消息中取 `workspace_path`（用户控制输入）直接用作 `FileOps` 的 workspace root。虽然 `validate_path` 用 `canonicalize` 防穿越，但没有验证请求者是否有权访问该 workspace path。每次请求创建新 `FileOps` 实例而非使用 daemon 的已注册 session workspace。
- **影响**: 如果攻击者能发送 crafted WS 消息，可读取 host 上 daemon 进程有权访问的任意文件。
- **建议**: 将请求中的 `workspace_path` 与 daemon 已注册 sessions 的 workspace 交叉验证。

#### MEDIUM-D6-01: `read_text_file` 使用 `from_utf8_lossy` — 静默损坏非 UTF-8 编码
- **文件**: `file_ops/handlers.rs:336`
- **问题**: 无效 UTF-8 字节被替换为 U+FFFD，非 UTF-8 编码的文本文件（ISO-8859-1、Shift-JIS 等）内容静默损坏。
- **建议**: 至少检测并报告有损转换发生。考虑添加 `content_encoding` 字段。

---

### D7: ve-daemon Agent Driver

#### CRITICAL-D7-01: stderr pipe 永不消费 — 子进程死锁
- **文件**: `agent/claude_code.rs:461-463, 694-696`
- **问题**: Claude CLI 子进程 spawn 时使用 `stderr(Stdio::piped())`，但永不 spawn stderr reader task。CLI 写足够多 stderr 输出（详细日志、警告、错误）时，stderr pipe buffer（通常 64KB）填满 → 子进程阻塞在 stderr write() → 整个 CLI 进程死锁卡死。
- **影响**: 所有长运行的 Claude CLI session 最终死锁。这是确定性 bug，不是竞态。
- **建议**: 添加 stderr reader task；或改用 `stderr(Stdio::inherit())`。

#### CRITICAL-D7-02: Daemon 关闭时无资源清理 — 子进程变为孤儿/僵尸
- **文件**: `main.rs:96-108`, `main.rs:41-47`
- **问题**: 关闭信号时 WS client 的 `run()` 退出，`main.rs` 打印 "shutdown complete" 返回 — 但 `registry.shutdown_all()` 永不调用。此方法存在但从未被调用。task destructors 和子进程 reaping 在 shutdown path 中被跳过。
- **影响**: Claude CLI 子进程变为僵尸（Unix）、file descriptors 泄漏、workspace 状态可能不一致。
- **建议**: 在返回前显式调用 `registry.shutdown_all().await`，添加 graceful shutdown timeout。

#### HIGH-D7-01: 无声事件丢弃 — `try_send` 在 channel 满时丢事件
- **文件**: `agent/claude_code.rs:26-29`
- **问题**: `emit()` 使用 `try_send` 在容量 256 的 mpsc channel 上。channel 满时（如 WS 重连中），事件无声丢弃只记录 warn。
- **影响**: `FatalError`、`StatusUpdate(Error)`、`PermissionRequest` 等关键事件可被无声丢弃。session 永远卡死而 server 不知晓。
- **建议**: 对关键事件使用 `send().await` 或显著增大 capacity 并添加背压。

#### HIGH-D7-02: Unbounded 权限桥处理中 Task Spawning
- **文件**: `ws_client/mod.rs:334-365`
- **问题**: 每个权限桥请求 spawn 新 `tokio::spawn` task 没有并发限制。恶意或异常 MCP server 可写多个 bridge 请求文件导致资源耗尽。
- **建议**: 添加并发限流如 `tokio::sync::Semaphore`。

#### MEDIUM-D7-01: `rerun` 缺少 `--verbose` flag
- **文件**: `agent/claude_code.rs:441` vs `agent/claude_code.rs:673-692`
- **问题**: `start()` 传 `--verbose` 给 CLI，`rerun()` 不传。Rerun session 可能错过 verbose-level 事件。
- **建议**: 为 rerun 添加 `--verbose`。

#### MEDIUM-D7-02: 死代码 `ClaudeCodeDriver::pending_permissions`
- **文件**: `agent/claude_code.rs:60`
- **问题**: 字段初始化空 HashMap 且永不填充或读取。实际权限跟踪在 `SessionRunner::pending_permissions`。
- **建议**: 移除。

---

## 四、按功能链路发现问题

### F1: 设备注册 & 配对链路
- **CRITICAL-M2-01**: `legacy_acl` 无授权效果
- **CRITICAL-M2-02**: PostgreSQL 类型不匹配在原子配对码声明中
- **CRITICAL-M2-03**: WS 路径跳过 `jti_matches_device`
- **HIGH-M2-01**: SQL format! 脆弱模式
- **HIGH-M2-02**: jti 更新不在事务中
- **HIGH-M2-03**: ClientBootstrap 无法撤销
- **HIGH-M2-04**: 中间件 DB 错误时 fail-open
- **HIGH-M2-05**: daemon token 明文存储
- **MEDIUM-M2-01**: pairing_secret 配对后未清理
- **MEDIUM-M2-02**: pair_code 日志泄露

### F2: Host & Workspace 访问链路
- **CRITICAL-M7-01**: unbind_host_by_id 无事务
- **MEDIUM-M7-01**: Workspace 端点手动 auth 而非 extractor

### F3: Session 创建 & 执行链路
- **CRITICAL-M3-01**: PostgreSQL 并发 rerun 创建重复 session
- **CRITICAL-M3-02**: 服务崩溃留下孤儿 dispatching session
- **HIGH-M3-01**: list_sessions 无分页
- **HIGH-M3-02**: handle_archived_rerun 潜在授权绕过
- **CRITICAL-D7-01**: stderr pipe 永不消费

### F4: Session 消息链路
- **MEDIUM-M3-01**: list_messages 整数溢出
- **MEDIUM-M3-02**: limit:0 未拒绝
- **MEDIUM-M3-04**: 内容验证用修剪后长度

### F5: Session 控制链路
- **CRITICAL-M3-01**: 并发 rerun 问题
- **CRITICAL-M3-02**: 孤儿 dispatching session
- **CRITICAL-D5-01**: state machine 无效转换仍应用
- **LOW-M3-01**: rerun 硬编码 can_resume_cross_device = TRUE

### F6: 权限请求/响应链路
- **CRITICAL-M11-01**: expire_stale_permissions 不在事务中
- **HIGH-M4-01**: get_permission_by_id 死代码标注
- **MEDIUM-M4-01**: 权限过期未传播给 daemon
- **HIGH-D7-01**: 事件通道 try_send 丢关键事件
- **HIGH-D7-02**: Unbounded 权限桥 task spawning

### F7: Session 归档链路
- **MEDIUM-M3-03**: archive metadata 在事务外读取
- **MEDIUM-M5-01**: 批量删除访问权限清理不完整
- **CRITICAL-D7-02**: Daemon shutdown 无资源清理

### F8: 文件浏览链路
- **HIGH-D6-01**: File Ops 无 session 关联验证
- **MEDIUM-M6-01**: validate_relative_path 不拒绝绝对路径
- **MEDIUM-D6-01**: from_utf8_lossy 静默损坏非 UTF-8 编码

### F9: 归档浏览 & 删除链路
- **MEDIUM-M5-01**: 同上

### F10: Settings 维护链路
- **LOW-M8-01**: 冗余验证调用

### F11: Daemon 重连链路
- **CRITICAL-D3-01**: 重连时 event_rx panic
- **HIGH-D3-01**: 重连窗口事件丢失

### F12: 后台维护链路
- **CRITICAL-M11-01**: expire_stale_permissions 不在事务中
- **CRITICAL-M10-01**: SQLite 上幂等键永不过期
- **MEDIUM-M11-01**: 后台任务首次执行延迟一个周期

---

## 五、按严重级别汇总

### CRITICAL (必须优先修复) — 共 13 项

| ID | 模块 | 问题 |
|----|------|------|
| CRITICAL-M2-01 | M2 | `legacy_acl` 完全无授权效果 |
| CRITICAL-M2-02 | M2 | PostgreSQL 上原子配对声明类型不匹配 |
| CRITICAL-M2-03 | M2 | WS 路径跳过 `jti_matches_device` |
| CRITICAL-M3-01 | M3 | PostgreSQL 并发 Rerun 创建重复 Session |
| CRITICAL-M3-02 | M3 | 服务崩溃留孤儿 dispatching Session |
| CRITICAL-M7-01 | M7 | unbind_host_by_id 多步 DELETE 无事务 |
| CRITICAL-M10-01 | M10 | SQLite 上幂等键永不过期 |
| CRITICAL-M10-02 | M10 | SQL/Postgres `expires_at` 可空性不匹配 |
| CRITICAL-M10-03 | M10 | `store()` 不验证 request_hash 导致幂等性绕过 |
| CRITICAL-M10-04 | M10 | SQLite Migration 007 每次启动重建 Sessions |
| CRITICAL-M11-01 | M11 | `expire_stale_permissions` 声称 in_transaction 但无事务 |
| CRITICAL-D3-01 | D3 | 重连时 `event_rx` panic |
| CRITICAL-D5-01 | D5 | `update_state` 无效转换仍应用 |
| CRITICAL-D7-01 | D7 | stderr pipe 永不消费导致子进程死锁 |
| CRITICAL-D7-02 | D7 | Daemon 关闭无资源清理 |

### HIGH (交付前应修复) — 共 16 项

| ID | 模块 | 问题 |
|----|------|------|
| HIGH-M2-01 | M2 | SQL format! 脆弱模式 |
| HIGH-M2-02 | M2 | jti 更新在事务外 |
| HIGH-M2-03 | M2 | ClientBootstrap 无法撤销 |
| HIGH-M2-04 | M2 | 中间件 DB 错误 fail-open |
| HIGH-M2-05 | M2 | daemon_token 明文存储 |
| HIGH-M3-01 | M3 | list_sessions 无分页 |
| HIGH-M3-02 | M3 | handle_archived_rerun 潜在授权绕过 |
| HIGH-M4-01 | M4 | get_permission_by_id 死代码标注误导 |
| HIGH-M9-01 | M9 | Daemon 配对 token 可能在连接丢失时永久丢失 |
| HIGH-M9-02 | M9 | WsEnvelope 负载序列化失败无声产生 Null |
| HIGH-M10-01 | M10 | 全新 SQLite 安装缺 Host 状态索引 |
| HIGH-M10-02 | M10 | 跨 DB CASCADE 行为不一致 |
| HIGH-M10-03 | M10 | SQLite Migration 006 无条件重建 Sessions |
| HIGH-M10-04 | M10 | Migration 004 FK 开关错误处理 |
| HIGH-D3-01 | D3 | 重连期间事件丢失 |
| HIGH-D6-01 | D6 | File Ops 无 session 关联验证 |
| HIGH-D7-01 | D7 | try_send 无声丢事件 |
| HIGH-D7-02 | D7 | Unbounded 权限桥 task spawning |

### MEDIUM (明显缺口) — 共 17 项

| ID | 模块 | 问题 |
|----|------|------|
| MEDIUM-M2-01 | M2 | pairing_secret 配对后未清理 |
| MEDIUM-M2-02 | M2 | pair_code 日志泄露 |
| MEDIUM-M2-03 | M2 | device_name 未使用修剪值 |
| MEDIUM-M3-01 | M3 | list_messages 整数溢出 |
| MEDIUM-M3-02 | M3 | limit:0 未拒绝 |
| MEDIUM-M3-03 | M3 | archive metadata 事务外读取 |
| MEDIUM-M3-04 | M3 | validate_content 修剪长度检查 |
| MEDIUM-M4-01 | M4/M11 | 权限过期 daemon 通知被静默吞掉 |
| MEDIUM-M5-01 | M5 | 批量删除访问权限清理不完整 |
| MEDIUM-M6-01 | M6 | validate_relative_path 不拒绝绝对路径 |
| MEDIUM-M7-01 | M7 | Workspace 端点用手动 auth 非 extractor |
| MEDIUM-M9-01 | M9 | error 响应双路径控制流混乱 |
| MEDIUM-M9-02 | M9 | send_and_wait 成功路径隐式依赖外部清理 |
| MEDIUM-M9-03 | M9 | unsubscribe_session 无授权检查 |
| MEDIUM-M10-01 | M10 | store duplicate-key fallback 竞态 |
| MEDIUM-M10-02 | M10 | session_archives 缺少 FK |
| MEDIUM-M11-01 | M11 | 后台任务首次执行延迟 |
| MEDIUM-D6-01 | D6 | from_utf8_lossy 静默损坏 |
| MEDIUM-D7-01 | D7 | rerun 缺 --verbose |
| MEDIUM-D7-02 | D7 | pending_permissions 死代码 |

### LOW (建议改进) — 共 14 项

| ID | 模块 | 问题 |
|----|------|------|
| LOW-M2-01 | M2 | pair 设备存在检查 TOCTOU |
| LOW-M3-01 | M3 | rerun 硬编码 can_resume_cross_device |
| LOW-M3-02 | M3 | close_session 不更新状态 |
| LOW-M4-01 | M4 | PermissionListQuery 死代码 |
| LOW-M4-02 | M4 | InvalidChars 上的 allow(dead_code) |
| LOW-M7-01 | M7 | workspace 响应 updated_at 与 DB 不匹配 |
| LOW-M7-02 | M7 | UNIQUE 约束检测用字符串匹配 |
| LOW-M8-01 | M8 | 冗余 validate_device_id_format |
| LOW-M8-02 | M8 | Settings upsert TOCTOU |
| LOW-M9-01 | M9 | broadcast 每次查 DB (性能) |
| LOW-M9-02 | M9 | send_task.abort 与自然退出竞态 |
| LOW-M9-03 | M9 | JWT 过期只在连接时检查 |
| LOW-M10-01 | M10 | PostgreSQL Migration 005 空操作 |
| LOW-M10-02 | M10 | SQL Integer vs Postgres Boolean |

---

## 六、测试覆盖缺口建议新增

1. PostgreSQL 集成测试：idempotent `create_session`
2. PostgreSQL 集成测试：并发 rerun 不产生重复 sessions
3. 孤儿 dispatching session 恢复测试
4. WS 重连后 `event_rx` 正确重建测试
5. Daemon 优雅关闭清理测试（子进程 reaping）
6. Claude CLI stderr 满 buffer 死锁测试
7. `unbind_host_by_id` 事务测试
8. `expire_stale_permissions` 事务测试
9. 跨 DB CASCADE 行为一致性测试
10. Migration 007 幂等性测试（已应用时不重复执行）

---

## 七、整体评估

本轮 review 覆盖全部 18 个模块（11 server + 7 daemon）和 12 条功能链路。

### 累计发现：
- **CRITICAL**: 15 项（含 2 项 daemon 稳定性和 2 项数据完整性阻塞项）
- **HIGH**: 17 项
- **MEDIUM**: 20 项  
- **LOW**: 14 项

### 最关键的三类问题：

1. **Daemon 稳定性问题**（CRITICAL-D3-01 重连 panic、CRITICAL-D7-01 stderr 死锁、CRITICAL-D7-02 关闭未清理）→ daemon 当前不适合生产部署

2. **数据库跨后端兼容性问题**（CRITICAL-M10-01 SQLite 幂等键永不清理、CRITICAL-M10-02 expires_at 可空性不匹配、HIGH-M10-02 CASCADE 不一致）→ SQLite 路径有根本性缺陷，PostgreSQL 路径相对干净

3. **事务缺失导致的数据一致性风险**（CRITICAL-M7-01 unbind 无事务、CRITICAL-M11-01 expire 函数声称有事务但代码无事务、CRITICAL-M3-01/+02 rerun 并发和崩溃窗口）

### 建议优先修复顺序：
1. 修复 daemon 三大稳定性问题（CRITICAL-D3/+D7）
2. 修复 SQLite 幂等键存储和过期清理（CRITICAL-M10-01/+02/+03）
3. 为 unbind 和 permission expiry 添加真正的事务（CRITICAL-M7-01、M11-01）
4. 修复 rerun 并发和崩溃恢复（CRITICAL-M3-01/+02）
5. 对齐跨 DB CASCADE 行为和索引（HIGH-M10-01/+02/+03）
6. 修复 WS 路径授权偏移和 token 管理（CRITICAL-M2-01/+02/+03）
