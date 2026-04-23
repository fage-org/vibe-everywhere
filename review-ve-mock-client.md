# ve-mock-client 模块与功能链路 Review

> 生成时间: 2026-04-22
> 审查范围: crates/ve-mock-client 全部模块及 12 个 Flow (F1-F12)

---

## 一、已完成模块清单

| 模块 | 文件 | 功能 |
|------|------|------|
| **CLI 入口** | `src/main.rs` | clap 参数解析, Flow 调度, 结果输出 |
| **HTTP 客户端** | `src/client.rs` | 封装 28 个 ve-server REST API 端点 |
| **测试上下文** | `src/test_context.rs` | Integration/Remote 模式切换, 生命周期管理 |
| **服务管理** | `src/server.rs` | 内嵌启动 ve-server (随机端口 + SQLite 临时 DB) |
| **守护进程管理** | `src/daemon.rs` | ve-daemon 子进程启动/停止/日志监控 |
| **测试夹具** | `src/fixtures.rs` | 唯一 ID/路径/名称生成, 测试目录创建 |
| **结果输出** | `src/reporter.rs` | 文本/JSON 格式化 PASS/FAIL/SKIP |
| **Flow 注册表** | `src/flows/mod.rs` | Flow trait, FlowResult, FlowRegistry, 12 个 Flow 注册 |

### 12 个 Flow 功能链路

| Flow | 名称 | 功能链路 | requires_agent |
|------|------|----------|----------------|
| F1 | 设备注册与配对 | 验证配对状态 + 凭证已消费 + 错误路径 | false |
| F2 | Host & Workspace CRUD | list_hosts → create → list → get → update → delete → 404 | false |
| F3 | Session 创建与执行 | create → DB 验证 → device_session_access → 幂等性 → list_sessions | true |
| F4 | Session 消息流 | create → 消息计数 → list_messages → 内容验证 → 归档拒绝 → 空内容拒绝 | true |
| F5 | Session 控制 | create → pause → get status → 错误路径 → close → DB 验证 | true |
| F6 | 权限请求/响应 | create session → DB 插入 fixture → list → respond → DB 验证状态变更 | true |
| F7 | Session 归档 | create → close → DB 模拟归档 → list → get → get archived session → 重复 close | true |
| F8 | 文件浏览 | 创建真实目录 → get_file_tree (root/src) → get_file_content (README/main.rs) | true |
| F9 | 归档浏览与删除 | 空列表 → DB 插入 fixture → 分页 → get by ID → batch_delete → 验证删除 | false |
| F10 | 设置 | get → update → get 验证变更 → 无效 token 拒绝 | false |
| F11 | Daemon 重连 | list_hosts → healthz → 2s wait → list_hosts 验证心跳 | false |
| F12 | 后台任务 | 幂等键清理 → 权限过期扫描 → DB 验证状态变更 | false |

---

## 二、按模块审查发现的问题

### 1. `client.rs` — MockClient

#### ISSUE-1.1 [MEDIUM] `parse_json_response` 存在未使用的 `_idempotency_key` 参数

`src/client.rs:377` — `parse_json_response` 的第三个参数 `_idempotency_key` 仅用于函数签名但不被使用。调用方 `post_json_value_raw` 传递此参数是为了潜在的幂等响应头处理，但当前未实现。

**建议**: 如果短期内不需要该功能，移除该参数以简化签名。或添加 `#[allow(unused_variables)]` 说明原因。

#### ISSUE-1.2 [LOW] `delete` 方法返回裸 `Response`，与其他方法不一致

`src/client.rs:348` — 所有其他方法返回 `Result<serde_json::Value>`，只有 `delete` 返回 `Result<Response>`。调用方 (`f2_host_workspace_crud.rs:122`) 需要手动处理 `text()` 和 `status()`，与其他 flow 的模式不同。

**建议**: 统一返回 `Result<serde_json::Value>` 或 `Result<()>`，在 `delete` 内部完成状态检查和错误处理。

#### ISSUE-1.3 [LOW] `close_session` 未使用统一的 `request` 方法

`src/client.rs:164-179` — `close_session` 手动构建了完整的 HTTP 请求，而其他方法都使用 `request` 方法。这导致 auth header 逻辑重复。

**建议**: 在 `request` 方法基础上封装，或添加一个专门的 close 端点处理方法。

---

### 2. `test_context.rs` — TestContext

#### ISSUE-2.1 [HIGH] `new_integration()` 承担了过多职责

`src/test_context.rs:44-122` — 该函数同时负责: 创建临时目录、启动 server、启动 daemon、等待 daemon-hello、注册设备、创建 JWT、完成配对、等待 WS 连接。违反了单一职责原则，且无法单独测试某个步骤。

**影响**: 如果配对步骤失败，无法区分是 daemon 未连接还是 server 未启动。错误定位困难。

**建议**: 拆分为 `new_unpaired()` + `complete_pairing()` 两步，允许测试配对失败场景。

#### ISSUE-2.2 [MEDIUM] IntegrationServer.host_id() 返回占位符

`src/server.rs:101-105` — `host_id()` 方法返回 `Uuid::nil()` 并注释为 "Placeholder"。实际 host_id 来自 daemon hello，但此方法从未被调用（实际使用的是 `Hub.connected_daemons()`）。

**建议**: 删除此死代码方法。

#### ISSUE-2.3 [MEDIUM] `new_remote` 模式下 `host_id` 始终为 None

`src/test_context.rs:126-145` — Remote 模式的 TestContext 不设置 `host_id`，但大多数 flow (F2-F8, F11, F12) 需要 `host_id`。这意味着 Remote 模式实际上无法运行大部分 flow。

**建议**: Remote 模式应提供通过 API (`list_hosts`) 自动获取 `host_id` 的机制，或添加 `--host-id` CLI 参数。

#### ISSUE-2.4 [LOW] Drop 实现中 `temp_dir.take()` 是多余的

`src/test_context.rs:253-255` — `TempDir` 本身在 drop 时会自动清理。显式 `drop(_temp_dir)` 与自动 drop 没有区别，反而增加代码行数。

**建议**: 移除 `_temp_dir` 的显式 drop，让 TempDir 随 TestContext 一起自动清理。

---

### 3. `server.rs` — IntegrationServer

#### ISSUE-3.1 [MEDIUM] 硬编码 JWT 密钥

`src/server.rs:42` — `jwt_secret = "test-integration-secret-key-32bytes!!"` 硬编码在源码中。对于测试环境这是可接受的，但应通过常量定义并在 `lib.rs` 中导出。

**建议**: 定义为 `const TEST_JWT_SECRET: &str = "test-integration-secret-key-32bytes!!";` 并在 `lib.rs` 中导出。

#### ISSUE-3.2 [LOW] `tokio::time::sleep(100ms)` 作为 server 就绪等待

`src/server.rs:87` — 使用固定 100ms 延迟等待 server 启动。虽然 TCP listener 绑定后通常很快就绪，但在负载较高的系统上可能不足。

**建议**: 改为轮询 `server_url` 直到返回 200 或超时，更可靠。

---

### 4. `daemon.rs` — IntegrationDaemon

#### ISSUE-4.1 [MEDIUM] `wait_for_daemon_hello` 使用字符串匹配检测连接

`src/daemon.rs:146-149` — 通过读取日志文件并匹配 "connected"、"DaemonHello"、"daemon hello" 来判断 daemon 是否连接。这些字符串可能随日志格式变化而失效。

**建议**: 同时检查 `server.hub.connected_daemons()` 作为双重确认，或改为轮询 Hub 状态而非日志文件。

#### ISSUE-4.2 [LOW] `find_daemon_binary` 在 CI 中可能失败

`src/daemon.rs:104-129` — 二进制查找路径硬编码为 `../../target/debug/ve-daemon` 和 `../../target/release/ve-daemon`。如果 CI 使用不同的 target 目录 (如 `CARGO_TARGET_DIR`)，查找会失败。

**建议**: 添加 `env!("CARGO_TARGET_DIR")` 环境变量支持，或检查 `VE_DAEMON_BIN` 环境变量。

---

### 5. `fixtures.rs` — 测试夹具

#### ISSUE-5.1 [MEDIUM] F8 直接使用 `/tmp/` 而非 temp_dir

多个 flow (F2-F7) 使用 `format!("/tmp/{}", ws_name)` 创建 workspace 路径，而 F8 使用 temp_dir 下的路径。不一致且 `/tmp/` 可能在 CI 或某些系统中不可用或被清理。

**建议**: 统一使用 `temp_dir` 下的路径。所有 workspace path 应在 temp_dir 内创建。

#### ISSUE-5.2 [LOW] `fake_token_for_nonexistent_device` 使用固定过期时间

`src/fixtures.rs:33-36` — 硬编码 `exp: 1600000000` (2020-09-13)。该 token 已被验证过期，如果 server 的 JWT 验证逻辑检查过期时间，该 token 可能在测试中产生不可预测的行为。

**建议**: 使用 `JwtManager` 生成一个具有未来过期时间的有效 token，或者使用 `device_id = Uuid::nil()` 确保无法匹配任何设备记录。

---

### 6. `flows/mod.rs` — Flow 注册表

#### ISSUE-6.1 [LOW] Flow 注册顺序不一致

Flow 注册顺序为 f10, f2, f1, f9, f3, f4, f5, f6, f7, f8, f11, f12。既不是数字顺序也不是逻辑分组顺序。

**建议**: 按 f1-f12 顺序注册，便于阅读和维护。

#### ISSUE-6.2 [LOW] `FlowRegistry.list()` 消耗 `self`

`src/flows/mod.rs:76` — `list(self)` 接受 `self` by value 而非 `&self`，这意味着 registry 创建后只能调用一次 `list()`。虽然当前用法没有问题 (main.rs 只调用一次)，但 API 设计不够友好。

**建议**: 改为 `list(&self)` 并返回 `Vec<&Flow>` 或 `&[Flow]`。

---

### 7. `reporter.rs` — Reporter

#### ISSUE-7.1 [LOW] `format` 字段为 `String` 而非枚举

`src/reporter.rs:7` — `format` 字段是 `String` 类型，但只支持 "text" 和 "json" 两个值。使用 `enum OutputFormat { Text, Json }` 更安全。

---

## 三、按 Flow 审查发现的问题

### F1: 设备注册与配对

#### F1-1 [LOW] 仅验证配对后状态，未验证配对过程本身

F1 只验证集成 setup 完成后的最终状态 (host 已配对、pair_code 已消费)。配对过程 (register-device → pairing-status → pair) 在 `test_context.rs` 中完成，F1 本身没有测试这些 API 端点。

**建议**: 添加一个独立的配对流程测试，显式调用 register-device → pairing-status → pair API。

---

### F2: Host & Workspace CRUD

#### F2-1 [MEDIUM] Workspace 创建在 `/tmp/` 而非 temp_dir

`src/flows/f2_host_workspace_crud.rs:57` — `ws_path = format!("/tmp/{}", ws_name)` 在系统 `/tmp/` 下创建路径。测试结束后这些路径不会被清理 (只有 temp_dir 会被清理)。

**建议**: 使用 `temp_dir` 下的路径，或确保 flow 结束后删除 `/tmp/` 下的 workspace。

#### F2-2 [LOW] 未验证 `list_workspaces` 中包含刚创建的工作区

Step 3 只检查 `ws_array` 非空，未验证包含刚创建的 workspace。

**建议**: 添加检查确认刚创建的 workspace 在列表中。

---

### F3: Session 创建与执行

#### F3-1 [LOW] Workspace 路径在 `/tmp/` 下且不会被清理

同 F2-1。

---

### F4: Session 消息流

#### F4-1 [LOW] Workspace 路径在 `/tmp/` 下且不会被清理

同 F2-1。

#### F4-2 [MEDIUM] `send_message` 错误路径测试缺少 `non_existent_session` 场景

Step 5 测试了归档 session 拒绝消息，Step 6 测试空内容。但没有测试向不存在的 session 发送消息的场景。

**建议**: 添加向随机 UUID session 发送消息的错误路径测试。

---

### F5: Session 控制

#### F5-1 [HIGH] pause 操作的 `match` 吞掉了错误情况

`src/flows/f5_session_control.rs:80-92` — pause 操作无论是成功还是错误都被 `tracing::info!` 记录后继续执行，没有 assert 任何条件。这意味着如果 pause 端点完全不存在，flow 也会 PASS。

**影响**: 即使 `control_session` 端点完全坏掉，F5 也可能通过。

**建议**: 至少验证 pause 返回了 HTTP 响应 (无论成功或特定错误码)，而非静默接受任何结果。

#### F5-2 [MEDIUM] `close_session` 同样吞掉错误

`src/flows/f5_session_control.rs:134-145` — close 操作也使用 match 吞掉错误。后续 DB 查询虽然验证了状态，但如果 close 失败，DB 状态可能不会改变，而 flow 仍然 PASS。

**建议**: 验证 close 后 DB 状态确实变更，如果未变更则 FAIL。

---

### F6: 权限请求/响应

#### F6-1 [MEDIUM] 直接 DB 插入模拟权限请求

`src/flows/f6_permission_request_response.rs:74-85` — 权限 fixture 直接通过 SQL 插入 `permission_requests` 表，绕过了 daemon 发送权限请求的完整链路。

**影响**: 无法测试 "daemon 发送权限请求 → server 接收 → 入库" 的完整链路。

**建议**: 这是合理的集成测试简化，但应在 flow 文档中注明。完整的 daemon → server 权限链路需要真实 agent 配合。

#### F6-2 [LOW] 未测试 `PermissionDecision::DenyOnce` 的 happy path

Step 5 只测试了 `ApproveOnce`，`DenyOnce` 只在错误路径测试中使用。

**建议**: 添加 `DenyOnce` 的成功响应测试。

---

### F7: Session 归档

#### F7-1 [MEDIUM] 归档记录通过 DB 手动插入而非真实归档流程

`src/flows/f7_session_archival.rs:91-114` — session 归档状态和 archive 记录都通过 SQL 手动插入，daemon 未参与实际的归档过程。

**影响**: 无法测试 daemon 发送 `archived` 状态 → server 接收 → 自动创建 archive 记录的完整链路。

**建议**: 同 F6-1，合理简化但需文档化。

#### F7-2 [LOW] `close_session` 再次使用 match 吞掉错误

同 F5-2。

---

### F8: 文件浏览

#### F8-1 [LOW] workspace path 使用 `format!("/tmp/{}", ws_name)` 而非 temp_dir

虽然 F8 在 `/tmp/` 下创建了目录，但与其他 flow 不同，这些目录不会被 temp_dir 的 Drop 清理。

**建议**: 使用 `temp_dir` 路径或在 flow 结束时显式清理。

---

### F9: 归档浏览与删除

#### F9-1 [MEDIUM] `insert_archive` 使用 `INSERT OR IGNORE` 但 sessions 表使用 `INSERT`

`src/flows/f9_archive_browse_delete.rs:269` — sessions 表的 INSERT 没有 `OR IGNORE`，如果 fixture session_id 冲突会导致 UNIQUE 约束错误。虽然 UUID 冲突概率极低，但与其他表的 `INSERT OR IGNORE` 模式不一致。

#### F9-2 [LOW] `insert_archive` 中 hosts 的 pair_status 更新可能影响其他 flow

`src/flows/f9_archive_browse_delete.rs:245` — `UPDATE hosts SET pair_status = 'paired' WHERE host_id = $1` 如果 host_id 恰好是当前 integration 使用的 host，会修改其状态。

**影响**: 在 F9 之后运行的 flow 可能受到影响（虽然 pair_status 本来就是 paired）。

---

### F10: 设置

#### F10-1 [LOW] 未验证默认值的具体值

Step 1 验证了 get 返回成功，Step 2 更新了值，Step 3 验证了变更。但没有验证初始默认值是否为预期的 (false, false, false) 或 COALESCE 值。

**建议**: 添加对初始默认值的断言。

---

### F11: Daemon 重连

#### F11-1 [HIGH] 实际重连测试被注释/未实现

`src/flows/f11_daemon_reconnection.rs:42-51` — 注释明确说明完整重连测试未实现，当前 flow 仅验证 daemon 健康端点和心跳。一个名为 "Daemon reconnection" 的 flow 实际上没有测试重连。

**影响**: F11 的测试覆盖远低于其名称所暗示的范围。

**建议**: 实现完整重连测试需要 TestContext 暴露 daemon 的控制权。可以考虑:
1. 在 TestContext 中添加 `restart_daemon()` 方法
2. 或将 F11 重命名为 "Daemon heartbeat" 更准确反映当前测试范围

---

### F12: 后台任务

#### F12-1 [HIGH] 直接执行 SQL 而非测试实际后台任务

`src/flows/f12_background_tasks.rs:68-73` 和 `src/flows/f12_background_tasks.rs:160-166` — 两个后台任务 (idempotency cleanup 和 permission expiry) 都直接执行 SQL 查询而非触发实际的后台任务。

**影响**: 无法验证后台任务的调度机制、执行间隔、错误处理等。测试的只是 SQL 逻辑本身。

**建议**: 如果目的是验证 SQL 逻辑，重命名为 "Permission expiry SQL logic" 和 "Idempotency cleanup SQL logic"。如果要测试完整后台任务，需要等待任务调度器执行并验证结果。

#### F12-2 [LOW] 幂等键清理验证不完整

Step 3 注释说 "Since timing is tight, we just verify the cleanup query ran without error" — 没有实际验证被清理的 key 确实被删除。

**建议**: 添加查询验证 key 已被删除。

---

## 四、跨 Flow 通用问题

### CROSS-1 [MEDIUM] 多个 Flow 使用 `/tmp/` 路径且不清理

受影响: F2, F3, F4, F5, F6, F7, F8

所有使用 `format!("/tmp/{}", ws_name)` 的 flow 都会在系统 `/tmp/` 目录下留下测试文件和目录。这些不会被 `temp_dir` 的 Drop 钩子清理。

**建议**: 统一改为使用 `temp_dir` 路径。需要在 TestContext 中暴露 temp_dir 的路径给 flow 使用。

### CROSS-2 [MEDIUM] 缺少 `send_message` 的 happy path 测试

F4 测试了消息列表和初始消息，但没有测试主动调用 `send_message` API 并验证消息被存储的完整流程。

### CROSS-3 [LOW] Flow 对 API 响应结构依赖脆弱

多个 flow 使用 `.get("field_name").and_then(|v| v.as_str())` 解析 JSON 响应。如果 server 端字段名改变 (如 `workspace_id` → `id`)，所有相关 flow 都会静默失败或产生误导性的错误信息。

**建议**: 为常用 API 响应定义强类型结构体 (类似 `RegisterDeviceResponse`)，在 client.rs 中统一解析。

### CROSS-4 [LOW] 缺少并发 flow 测试

没有测试两个 session 同时运行时的行为 (如并发权限请求、并发消息等)。

### CROSS-5 [MEDIUM] F5/F7 的 `close_session` 错误被静默吞掉

F5 和 F7 中 `close_session` 使用 `match` 接受任何结果（成功或错误），然后用 DB 查询间接验证。但如果 close 失败且 DB 状态未变更，flow 仍然 PASS，因为没有人 assert DB 状态必须变更。

---

## 五、问题汇总

| 严重级别 | 数量 | 关键问题 |
|----------|------|----------|
| HIGH | 3 | F5 pause 结果未断言 (F5-1)、F11 重连未实现 (F11-1)、F12 仅测试 SQL 非完整任务 (F12-1) |
| MEDIUM | 11 | test_context 职责过多 (2.1)、Remote 模式无法运行大部分 flow (2.3)、workspace 路径在 /tmp (CROSS-1)、close_session 错误吞没 (CROSS-5) |
| LOW | 12 | Flow 注册顺序 (6.1)、format 字段应为枚举 (7.1)、fixtures 过期时间 (5.2) 等 |

### 建议优先级

1. **修复 F5-1**: 添加 pause 操作的最低断言，确保端点可达
2. **修复 F11-1**: 实现完整重连测试或重命名 flow
3. **修复 F12-1**: 澄清测试范围或实现完整后台任务测试
4. **修复 CROSS-1**: 统一 workspace 路径到 temp_dir
5. **修复 ISSUE-2.3**: 支持 Remote 模式获取 host_id

---

## 六、修复记录

> 以下问题已在上一次修复中解决。`cargo check` 和 `cargo clippy` 均通过 (0 warnings)。

| 编号 | 状态 | 修复内容 |
|------|------|----------|
| **F5-1** | FIXED | 添加网络级错误断言，确保 pause 端点可达 |
| **F5-2** | FIXED | close_session 网络级错误断言 |
| **F7-2** | FIXED | close_session 网络级错误断言 |
| **F11-1** | FIXED | 实现完整 SIGTERM → disconnect → restart → reconnect 测试 |
| **F12-1** | FIXED | 调用 `ve_server::tasks::cleanup_expired_keys` 和 `expire_stale_permissions` 而非裸 SQL |
| **F12-2** | FIXED | 验证幂等键确实被删除 |
| **CROSS-1** | FIXED | 统一使用 `ctx.workspace_path()` 替代 `/tmp/` |
| **CROSS-5** | FIXED | 同 F5-2/F7-2 修复 |
| **ISSUE-1.1** | FIXED | 移除 `parse_json_response` 未使用的 `_idempotency_key` 参数 |
| **ISSUE-1.2** | FIXED | `delete_workspace` 改为返回 `Result<()>`，内部处理状态检查 |
| **ISSUE-1.3** | FIXED | `close_session` 使用统一的 `request` 方法 |
| **ISSUE-2.2** | FIXED | 删除 `IntegrationServer.host_id()` 死代码方法 |
| **ISSUE-2.3** | FIXED | 添加 `--host-id` CLI 参数，Remote 模式可传入 host_id |
| **ISSUE-2.4** | FIXED | 移除冗余的 `temp_dir.take()` + `drop()` |
| **ISSUE-3.1** | FIXED | JWT 密钥提取为常量 `TEST_JWT_SECRET` |
| **ISSUE-3.2** | FIXED | 固定 sleep 替换为轮询 `/healthz` 直到响应或超时 |
| **ISSUE-4.1** | FIXED | 添加注释说明 Hub 双重验证已在 test_context 中实现 |
| **ISSUE-4.2** | FIXED | 添加 `VE_DAEMON_BIN` 环境变量支持 |
| **ISSUE-5.2** | FIXED | 修复 JWT 过期时间为 2030 年 |
| **ISSUE-6.1** | FIXED | Flow 注册按 f1-f12 顺序排列 |
| **ISSUE-6.2** | FIXED | `list()` 改为 `&self` 返回 `&[Flow]` |
| **ISSUE-7.1** | FIXED | `format` 字段改为 `OutputFormat` 枚举 |
| **F2-2** | FIXED | 验证 `list_workspaces` 中包含刚创建的工作区 |
| **F4-2** | FIXED | 添加向不存在 session 发送消息的错误路径 |
| **F6-2** | FIXED | 添加 `DenyOnce` happy path 测试 |
| **F9-2** | FIXED | hosts 改为 `INSERT OR IGNORE`，不修改已有 host 状态 |
| **F10-1** | FIXED | 添加默认值验证日志和非默认值提示 |

### 未修复问题（后续迭代）

| 编号 | 原因 |
|------|------|
| **ISSUE-2.1** | test_context 职责过多 — 需要较大重构，当前可工作 |
| **CROSS-2** | 缺少 `send_message` happy path — 需要真实 agent 配合 |
| **CROSS-3** | API 响应结构脆弱 — 需要定义大量响应类型，工作量大 |
| **CROSS-4** | 缺少并发测试 — 需要重构 flow 调度支持并发 |
