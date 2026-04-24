# Server/Daemon ECC Review Open Issues

**Updated**: 2026-04-25 — 历史 open 项复核完成

## 2026-04-25 追加结论

截至 2026-04-25，本文此前残留的 legacy ACL 兼容问题也已失效：

- `ensure_legacy_client_access()` 已退化为“仅校验 device 是否存在”，不再执行 host/session ACL 懒补权。
- 现有回归 `list_hosts_does_not_backfill_older_paired_hosts_even_after_partial_legacy_acl_exists` 已覆盖“不回填旧 paired host”的行为。

当前状态：除独立 rerun/resume 专项历史记录外，server/daemon 主体 review 已无 active open issue；下文保留为历史审查上下文。

**Reviewed**: 2026-04-21  
**Scope**: 当前 `ve-server` / `ve-daemon` 代码，按模块与功能链路做 ECC 风格 review  
**Review Method**:
- 先逐项列出模块与功能链路清单。
- 再按“一代理只审一项”的方式分批审查，每批不超过 3 个 subagent。
- 本文只保留当前确认仍存在的开放问题；本轮已验证关闭的问题不再继续列为 open。
- `session rerun/resume` 专项仍单独记录在 `review-session-rerun-open-issues.md`，本文不重复展开同类问题。
- 严重级别含义：`CRITICAL` = 必须优先修复；`HIGH` = 合并/交付前应修复；`MEDIUM` = 明显缺口但可排在高优之后。

## 当前状态

本轮已核对并确认以下修复已落到主树，原先对应 open 项已关闭：
- daemon 主入口已接入 registry-backed `WsClient`：`crates/ve-daemon/src/main.rs`、`crates/ve-daemon/src/session_registry.rs`
- `Hub::send_and_wait()` 已能接收 ack / error / file response：`crates/ve-server/src/hub.rs`、`crates/ve-server/src/ws/daemon_ws.rs`
- `control_session()` / `close_session()` 已在 daemon ACK 后再推进 DB 状态：`crates/ve-server/src/api/sessions.rs`
- `send_message()` 已改为 daemon ACK 后再落库，避免 phantom message：`crates/ve-server/src/api/sessions.rs`、`crates/ve-server/tests/session_message_ack_test.rs`
- `respond_permission()` 已改为事务内 CAS 更新，同一 permission 不再出现并发双重决策/双重通知：`crates/ve-server/src/api/permissions.rs`
- permission 并发回归已补到 SQLite + PostgreSQL 双后端：`crates/ve-server/tests/permission_response_race_test.rs`
- daemon 连接 handoff / active disconnect 已会按 connection 维度主动失败 pending request，不再把旧连接上的 in-flight request 一律挂到 timeout：`crates/ve-server/src/hub.rs`、`crates/ve-server/src/ws/daemon_ws.rs`
- daemon 启动前已校验凭据里的 `server_url` 与当前配置是否一致，不匹配时会要求重新配对：`crates/ve-daemon/src/main.rs`、`crates/ve-daemon/src/credentials.rs`
- files API 已对 daemon 文件错误做边界脱敏，不再把宿主机内部路径等原始错误文案直接透传给 HTTP 调用方：`crates/ve-server/src/api/files.rs`、`crates/ve-server/tests/access_scope_test.rs`
- settings notifications 已恢复从认证 claims 提取 `device_id`，并补了路由回归：`crates/ve-server/src/api/settings.rs`、`crates/ve-server/tests/settings_notifications_test.rs`
- client/daemon WebSocket 入口已统一走共享 JWT decode + subject helper，并补了真实握手回归；坏 token 现在稳定返回 `401` / `Invalid token`，不再落成 `JWT error` / `500`：`crates/ve-server/src/ws/client_ws.rs`、`crates/ve-server/src/ws/daemon_ws.rs`、`crates/ve-server/tests/access_scope_test.rs`
- 已通过验证：`cargo test -p ve-server` 相关聚焦回归（`session_completion_ack_test`、`session_message_ack_test`、`permission_response_race_test`、`settings_notifications_test`、`access_scope_test`）、`cargo test -p ve-daemon`
- 当前仍保留 1 个未关闭问题：legacy ACL 兼容目前采用 fail-open 懒补权语义，正式 client 只要自身 ACL 为空，就会被补成对所有已 `paired` host/session 可见；这能兼容旧数据，但会打破严格的 per-device isolation，仍需进一步收口。
## 审查清单

### 模块项

- [x] M1. `ve-server` Authentication / Authorization
- [x] M2. `ve-server` WebSocket / Hub / Files
- [x] M3. `ve-daemon` Runtime Wiring / Session Lifecycle
- [x] M4. `ve-server` Settings / Permissions / Archives

### 功能链路项

- [x] F1. 设备注册 / 配对 / 凭据发放链路
- [x] F2. Host / Workspace / Files 访问链路
- [x] F3. Live Session 控制链路
- [x] F4. Settings / Permissions / Archive 维护链路

## 逐项审查结果

### M1. `ve-server` Authentication / Authorization

> 结论：当前仍有 1 个未关闭问题，集中在 legacy ACL 兼容语义。

#### HIGH-M1-LEGACY-ACL: legacy ACL 兼容当前是 fail-open，会把空 ACL 的正式 client 补权到所有 paired host/session

- **模块**: `ve-server` authz / route extractor
- **位置**: `crates/ve-server/src/authz.rs`
- **现状**:
  - 为了兼容引入 ACL 之前的旧 client，当前在 `ensure_legacy_client_access()` 中做了懒初始化。
  - 只要一个正式 `Client` token 对应的 `device_id` 已存在于 `client_devices`，但自身 `device_host_access` 为空，就会自动补写该设备到当前所有 `pair_status = 'paired'` 的 host，并进一步补写这些 host 下的 session ACL。
- **风险**:
  - 这属于 fail-open 兼容路径，而不是显式迁移标记或设备级历史绑定。
  - 一旦某个正式 client 的 ACL 因迁移、清理、异常或数据缺失而为空，该设备会被恢复为“可见所有已配对 host/session”，破坏严格的 per-device isolation。
- **为什么仍未关闭**:
  - 当前实现优先保证旧数据可用与回归测试通过，尚未引入更精确的 legacy 标记或一次性迁移收口机制。
  - 因此该问题仍应保留为 `HIGH` open issue，而不是判定完全关闭。

### M2. `ve-server` WebSocket / Hub / Files

> 结论：本轮已确认关闭此前记录的 open 项；当前无未关闭问题。

- files 错误透传问题已通过边界脱敏关闭：`crates/ve-server/src/api/files.rs`、`crates/ve-server/tests/access_scope_test.rs`
- daemon handoff / disconnect 时旧连接 pending request 悬挂到 timeout 的问题已通过 connection-scoped fail-fast 收敛关闭：`crates/ve-server/src/hub.rs`、`crates/ve-server/src/ws/daemon_ws.rs`

### M3. `ve-daemon` Runtime Wiring / Session Lifecycle

> 结论：本轮已确认此前 open 项已关闭；当前无未关闭问题。

- `send_message` 已进入 runner completion 级 ACK 闭环，不再是 admission 语义：`crates/ve-server/src/api/sessions.rs`、`crates/ve-daemon/src/ws_client.rs`、`crates/ve-daemon/src/session_runner.rs`

### M4. `ve-server` Settings / Permissions / Archives

> 结论：本轮已确认此前 open 项已关闭；当前无未关闭问题。

- permission 并发响应主逻辑已修复，且 PostgreSQL 定向并发回归已补齐：`crates/ve-server/src/api/permissions.rs`、`crates/ve-server/tests/permission_response_race_test.rs`

### F1. 设备注册 / 配对 / 凭据发放链路

> 结论：本轮已确认此前 open 项已关闭；当前无未关闭问题。

- daemon 启动前已校验 `creds.server_url` 与当前 `config.server_url` 是否一致，不匹配时直接 fail fast：`crates/ve-daemon/src/main.rs`、`crates/ve-daemon/src/credentials.rs`

### F2. Host / Workspace / Files 访问链路

> 结论：主链路已恢复可用，但 legacy ACL 兼容仍保留 1 个未关闭高优问题。

#### HIGH-F2-LEGACY-ACL: 旧 client 兼容依赖运行时懒补权，当前语义仍是 fail-open

- **模块**: `ve-server` authz / host-session 可见性链路
- **位置**: `crates/ve-server/src/authz.rs`, `crates/ve-server/tests/access_scope_test.rs`
- **问题摘要**:
  - 当前 `access_scope_test` 已验证：旧正式 client 在 ACL 为空时，访问 host / archive 等受保护资源会触发懒补权并恢复可见性。
  - 但这条兼容路径的授权粒度仍然过宽：它不是基于设备历史绑定做恢复，而是直接扩展到所有当前已 paired host/session。
- **为什么仍未关闭**:
  - 该实现是为了先保住升级后旧 client 不立刻失能，但还没有做到“兼容旧数据”与“维持严格设备隔离”同时成立。
  - 因此 F2 当前仍保留一个 `HIGH` open issue。

### F3. Live Session 控制链路

> 结论：用户选择的 **B3 完整闭环** 已在 create / control / close / `send_message` 四条主路径完成主树落地与聚焦回归验证；F3 当前不再有未关闭的高优一致性缺口。

#### NOTE-F3-01: `send_message` 当前达到的是 runner completion 级 ACK，不是 agent end-to-end 结果 ACK

- **模块**: `ve-daemon` ws client / session runtime
- **位置**: `crates/ve-daemon/src/ws_client.rs`, `crates/ve-daemon/src/session_runner.rs`, `crates/ve-server/src/config.rs`
- **当前状态**:
  - server 侧 `send_message()` 已改为等待 `send_daemon_command_and_wait()` + `ensure_command_acked()` 成功后再落库，避免 failed send 产生 phantom message。
  - daemon 侧 `handle_send_message()` 已改为 `send_message_and_wait()`，ACK 以 runner 命令完成为边界，而不是 channel admission。
  - server 默认 `ack_timeout_ms` 已对齐到 30s，与 daemon `ack_timeout_secs` 默认预算一致，避免两端默认超时窗口漂移。
  - 已补聚焦回归：`crates/ve-server/tests/session_message_ack_test.rs`、`crates/ve-daemon/src/session_runner.rs` timeout test、`crates/ve-server/src/config.rs` default timeout test。
- **边界说明**:
  - 当前 ACK 保证的是“daemon runner 已接收并完成 `driver.send_message(...)` 调用”。
  - 若未来需要更强语义（例如 agent 真正产生日志/event 后才确认），还需要 driver 层提供 message-level completion signal。

### F4. Settings / Permissions / Archive 维护链路

> 结论：本轮已确认此前 open 项已关闭；当前无未关闭问题。

- permission 并发响应的 PostgreSQL 定向并发回归已补齐，因此该链路当前不再保留独立 open issue：`crates/ve-server/tests/permission_response_race_test.rs`

## 与现有专项 review 的关系

- `review-session-rerun-open-issues.md`：继续追踪 rerun/resume 专项未关闭项。
- 本文：补充当前 server/daemon 主体代码中，除 rerun/resume 专项外仍存在的开放问题。

## 当前优先级建议

1. 除 rerun/resume 专项外，本轮 server/daemon 主体链路当前没有其他已确认未关闭的访问控制缺口。
2. 若后续继续做 authz 收口，可把剩余零散 `require_*` helper 进一步内聚到 extractor / policy，作为可独立排期的重构项，而非当前 open issue。
3. 下一步若继续 review，建议把精力转回 `review-session-rerun-open-issues.md` 中仍开放的问题。
