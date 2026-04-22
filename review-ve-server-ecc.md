# VE Server ECC Review

## Review scope

Target: `crates/ve-server`

Review method:
- First整理当前已完成的服务模块与主功能链路
- Then按模块与链路执行 ECC 风格审查
- Focus on correctness, authorization, concurrency, transport reliability, state consistency, and test coverage gaps

---

## Completed service modules

### 1. App bootstrap and routing
- Entry: `crates/ve-server/src/main.rs`
- Router assembly: `crates/ve-server/src/lib.rs`
- Responsibility:
  - Load config and tracing
  - Initialize DB and run migrations
  - Create `Hub`, JWT manager, shared `AppState`
  - Start background tasks
  - Expose HTTP API and WebSocket routes

### 2. Authentication and pairing
- Main files:
  - `crates/ve-server/src/api/auth.rs`
  - `crates/ve-server/src/middleware/auth.rs`
  - `crates/ve-server/src/state.rs`
- Responsibility:
  - Device registration
  - Daemon hello and pair-code generation
  - Pairing status polling with pairing secret
  - Pair completion and token issuance
  - In-memory auth throttling

### 3. Authorization and scoped access control
- Main file: `crates/ve-server/src/authz.rs`
- Responsibility:
  - Extract device/host/session identity from claims
  - Enforce host/session/workspace/archive/permission scoped access
  - Legacy ACL backfill path for old devices

### 4. Host management
- Main file: `crates/ve-server/src/api/hosts.rs`
- Responsibility:
  - List paired hosts visible to current device
  - Read host details
  - Unbind host after dependency checks

### 5. Workspace management
- Main file: `crates/ve-server/src/api/workspaces.rs`
- Responsibility:
  - List/create/get/update/delete workspaces
  - Bind workspaces to authorized hosts

### 6. Session lifecycle management
- Main file: `crates/ve-server/src/api/sessions.rs`
- Responsibility:
  - List/create/get sessions
  - List/send messages
  - Pause/restart/interrupt/terminate/close session
  - Archive closed sessions
  - Rerun archived sessions
  - Idempotent create-session path

### 7. Permission request flow
- Main file: `crates/ve-server/src/api/permissions.rs`
- Responsibility:
  - List/get pending and historical permission requests
  - Respond to permission requests
  - Synchronize pending count onto session state

### 8. Archive management
- Main file: `crates/ve-server/src/api/archives.rs`
- Responsibility:
  - List archives
  - Get archive details
  - Batch delete archive records

### 9. Remote file access
- Main file: `crates/ve-server/src/api/files.rs`
- Responsibility:
  - Read-only file tree and file content requests via daemon RPC

### 10. WebSocket transport and connection hub
- Main files:
  - `crates/ve-server/src/hub.rs`
  - `crates/ve-server/src/ws/client_ws.rs`
  - `crates/ve-server/src/ws/daemon_ws.rs`
- Responsibility:
  - Client/daemon WS connection lifecycle
  - Session subscription fanout
  - Request/response correlation to daemon
  - Daemon event ingestion and client broadcast

### 11. Database and migrations
- Main files:
  - `crates/ve-server/src/db/mod.rs`
  - `crates/ve-server/src/db/idempotency.rs`
  - `crates/ve-server/src/db/migrations/postgres/001_initial.sql`
  - `crates/ve-server/src/db/migrations/sqlite/*.sql`
- Responsibility:
  - Runtime DB backend selection
  - SQLite/Postgres migration orchestration
  - Idempotency key storage and cleanup helpers

### 12. Background tasks
- Main files:
  - `crates/ve-server/src/tasks/permission_expiry.rs`
  - `crates/ve-server/src/tasks/idempotency_cleanup.rs`
- Responsibility:
  - Expire stale permission requests
  - Clean expired idempotency keys

---

## Completed functional flows

### A. Device registration and pairing flow
1. `POST /api/auth/register-device`
2. `POST /api/auth/daemon-hello`
3. `GET /api/auth/pairing-status`
4. `POST /api/auth/pair`
5. Server grants access and issues client/daemon tokens
6. Server optionally notifies daemon via `DaemonMessage::Paired`

Cross-module path:
- `api/auth.rs` -> `state.rs` throttle -> DB -> `authz.rs` access materialization -> `hub.rs`

### B. Host and workspace access flow
1. Authenticated client lists hosts
2. Client lists or creates workspaces on authorized hosts
3. Workspace reads and mutations are checked against host access

Cross-module path:
- `middleware/auth.rs` -> `authz.rs` -> `api/hosts.rs` / `api/workspaces.rs` -> DB

### C. Session creation and live execution flow
1. `POST /api/sessions`
2. Host/workspace authorization
3. Session row + initial message + idempotency row persisted
4. Device/session access granted
5. Daemon command dispatched through `hub.send_and_wait`
6. Daemon later pushes status/events over WS
7. Server updates DB and broadcasts to subscribed clients

Cross-module path:
- `middleware/auth.rs` -> `authz.rs` -> `api/sessions.rs` -> `db/idempotency.rs` / DB -> `hub.rs` -> `ws/daemon_ws.rs` -> `hub.rs`

### D. Session message flow
1. `POST /api/sessions/{id}/messages`
2. Session access check
3. Daemon command send-and-wait
4. Persist user message and later stream daemon-side events

Cross-module path:
- `api/sessions.rs` -> `authz.rs` -> `hub.rs` -> `ws/daemon_ws.rs`

### E. Session control and close flow
1. `POST /api/sessions/{id}/control`
2. `POST /api/sessions/{id}/close`
3. Command dispatched to daemon
4. Server mutates local session state after ACK or later daemon status event
5. Archived sessions produce archive records

Cross-module path:
- `api/sessions.rs` -> `hub.rs` -> `ws/daemon_ws.rs` -> archive persistence in `api/sessions.rs`

### F. Permission request/response flow
1. Daemon sends permission request over WS
2. Server inserts permission row and increments session pending count
3. Server broadcasts permission event to subscribed clients
4. Client responds through `POST /api/permissions/{id}/respond`
5. Server updates DB and attempts daemon reply delivery

Cross-module path:
- `ws/daemon_ws.rs` -> DB -> `hub.rs` -> `api/permissions.rs` -> `hub.rs`

### G. Archive browsing and deletion flow
1. Client lists archives, optionally by host/workspace filter
2. Client fetches archive detail
3. Client batch deletes archive records

Cross-module path:
- `api/archives.rs` -> `authz.rs` -> DB

### H. Remote file browsing flow
1. Client calls file tree/content API on a host
2. Host access + workspace authorization enforced
3. Server dispatches file request to daemon over WS
4. Daemon replies with file tree/content payload
5. Server sanitizes transport and operation errors

Cross-module path:
- `api/files.rs` -> `authz.rs` -> `hub.rs` -> `ws/daemon_ws.rs`

---

## ECC review findings

Severity legend:
- HIGH: should fix before merge/use in production path
- MEDIUM: correctness or maintainability issue with meaningful runtime impact

---

## Findings by module

### Module: Authentication and pairing

#### HIGH 1. Legacy ACL path still grants newly paired hosts and sessions to every legacy device
- Files:
  - `crates/ve-server/src/api/auth.rs:401`
  - `crates/ve-server/src/api/auth.rs:417`
  - `crates/ve-server/src/authz.rs:20`
  - `crates/ve-server/src/authz.rs:41`
  - `crates/ve-server/src/authz.rs:58`
- Problem:
  - Pairing a host inserts access not only for the current device, but also for every `client_devices.legacy_acl = 1` device.
  - `ensure_legacy_client_access` also continues to auto-expand host/session access for legacy devices.
- Impact:
  - A stale or forgotten old device can continue to inherit access to all paired hosts and their sessions.
  - This weakens the intended device-scoped authorization model.
- ECC judgment:
  - This is a cross-device authorization expansion bug, not just backward compatibility behavior.
- Suggested fix direction:
  - Remove global legacy auto-grant behavior, or scope migration to an explicit one-time upgrade for the current device only.

#### MEDIUM 2. Pairing secret remains sufficient to mint daemon token after pair success within TTL window
- Files:
  - `crates/ve-server/src/api/auth.rs:211`
  - `crates/ve-server/src/api/auth.rs:242`
  - `crates/ve-server/src/api/auth.rs:346`
- Problem:
  - `pair()` marks `used = 1`, but `pairing_status()` does not check `used`; it only checks host, secret, and expiration.
  - `pairing_secret` is not cleared on successful pair.
- Impact:
  - Anyone who obtains the pairing secret during the TTL window can still poll `pairing-status` after the legitimate pair completes and receive a daemon token.
- Suggested fix direction:
  - Invalidate or clear the pairing secret on successful pair, and make `pairing_status()` reject used records.

#### MEDIUM 3. Pair code is logged in debug output
- File:
  - `crates/ve-server/src/api/auth.rs:194`
- Problem:
  - Pair code is emitted to logs while still valid.
- Impact:
  - Log readers or centralized log systems can redeem the live code.
- Suggested fix direction:
  - Remove the raw code from logs or redact it.

### Module: WebSocket transport and hub

#### HIGH 4. Session subscription authorization is only checked on subscribe, not on later broadcasts
- Files:
  - `crates/ve-server/src/ws/client_ws.rs:114`
  - `crates/ve-server/src/ws/client_ws.rs:115`
  - `crates/ve-server/src/hub.rs:251`
  - `crates/ve-server/src/hub.rs:255`
  - `crates/ve-server/src/ws/daemon_ws.rs:387`
  - `crates/ve-server/src/ws/daemon_ws.rs:490`
- Problem:
  - Access is verified once when a client subscribes to a session.
  - Later broadcasts fan out purely from cached subscriber sets without re-checking DB access.
- Impact:
  - If access is revoked after subscribe, that client can keep receiving session events and permission prompts until disconnect.
- Suggested fix direction:
  - Remove subscriptions on revocation, or re-check authorization before fanout to each subscriber.

#### HIGH 5. `send_and_wait` timeout does not cover queue backpressure before send
- Files:
  - `crates/ve-server/src/hub.rs:343`
  - `crates/ve-server/src/hub.rs:375`
  - `crates/ve-server/src/hub.rs:380`
- Problem:
  - The function awaits `sender.send(...)` before starting `tokio::time::timeout(timeout, rx)`.
  - If the bounded daemon channel is full, request paths can hang waiting for queue space longer than the declared timeout.
- Impact:
  - File and session RPC calls may block far longer than API contract suggests under congestion.
- Suggested fix direction:
  - Bound enqueue time too, or use non-blocking send with explicit backpressure handling.

#### MEDIUM 6. WebSocket bearer tokens are passed in URL query parameters
- Files:
  - `crates/ve-server/src/ws/client_ws.rs:26`
  - `crates/ve-server/src/ws/client_ws.rs:30`
  - `crates/ve-server/src/ws/daemon_ws.rs:29`
  - `crates/ve-server/src/ws/daemon_ws.rs:33`
- Problem:
  - Both WS endpoints accept `?token=<jwt>`.
- Impact:
  - Tokens can leak via browser history, reverse-proxy logs, access logs, crash reports, and tracing.
- Suggested fix direction:
  - Move WS auth to headers, subprotocol, or a short-lived one-time upgrade token.

### Module: Session lifecycle

#### HIGH 7. `terminate` archives locally on ACK, even though ACK only proves delivery/queueing
- Files:
  - `crates/ve-server/src/api/sessions.rs:960`
  - `crates/ve-server/src/api/sessions.rs:1001`
  - `crates/ve-server/src/ws/daemon_ws.rs:351`
  - `crates/ve-server/src/ws/daemon_ws.rs:533`
- Problem:
  - `ensure_command_acked()` accepts daemon ACK as success.
  - `persist_control_success()` archives immediately for terminate.
  - Later daemon events/status for that session are ignored once the session is archived.
- Impact:
  - A daemon that ACKs but does not actually terminate can leave server and daemon state diverged, with later events dropped by the server.
- Suggested fix direction:
  - Treat terminate ACK as delivery only; archive only after daemon emits final archived/closed status.

#### HIGH 8. Postgres idempotency schema mismatches the runtime string-based session_id writes
- Files:
  - `crates/ve-server/src/db/migrations/postgres/001_initial.sql:141`
  - `crates/ve-server/src/api/sessions.rs:504`
  - `crates/ve-server/src/db/idempotency.rs:128`
- Problem:
  - Postgres migration defines `idempotency_keys.session_id UUID`.
  - Runtime inserts `session_id` as string values and the idempotency store models it as `String`.
- Impact:
  - The advertised dual-backend path is likely broken on Postgres for create-session idempotency.
- Suggested fix direction:
  - Make schema and runtime representation consistent, and add a Postgres integration test for idempotent session creation.

#### MEDIUM 9. Pagination underflows when `page=0`
- Files:
  - `crates/ve-server/src/api/sessions.rs:843`
  - `crates/ve-server/src/api/sessions.rs:845`
  - `crates/ve-server/src/api/archives.rs:111`
  - `crates/ve-server/src/api/archives.rs:113`
- Problem:
  - Offset is computed as `(page - 1) * limit` on unsigned values without validating `page >= 1`.
- Impact:
  - In debug/test builds this can panic; in release it wraps to a huge offset and returns nonsense.
- Suggested fix direction:
  - Reject `page == 0` at the request boundary and add request-level tests.

### Module: Permission request flow

#### HIGH 10. Permission response commits DB state before daemon delivery is confirmed
- Files:
  - `crates/ve-server/src/api/permissions.rs:449`
  - `crates/ve-server/src/api/permissions.rs:451`
- Problem:
  - DB transaction commits first.
  - `send_to_daemon()` result is ignored.
- Impact:
  - If the daemon is disconnected or the WS queue is full, the server still returns success and decrements `pending_permission_count`, but the daemon never receives the decision and may remain blocked.
- Suggested fix direction:
  - Treat delivery failure as operation failure or persist a retryable outbound command.

### Module: Archive management

#### HIGH 11. Batch archive deletion breaks archived-session invariant used elsewhere
- Files:
  - `crates/ve-server/src/api/archives.rs:459`
  - `crates/ve-server/src/api/sessions.rs:1439`
  - `crates/ve-server/src/api/sessions.rs:1652`
- Problem:
  - Batch delete removes only `session_archives` rows.
  - Session rows can remain in `status = 'archived'`.
  - Session code explicitly treats "archived session without archive record" as conflict/inconsistent state.
- Impact:
  - Archive deletion can manufacture a state that breaks later close/rerun/archive-related APIs.
- Suggested fix direction:
  - Delete or normalize the corresponding session state together with archive deletion, or stop assuming archive row existence for archived sessions.

### Module: Background tasks

#### MEDIUM 12. Permission expiry task updates DB state only and does not propagate expiry to live consumers
- File:
  - `crates/ve-server/src/tasks/permission_expiry.rs:55`
- Problem:
  - Stale permissions are marked expired and session counters are recomputed.
  - No daemon notification and no client broadcast are emitted from this path.
- Impact:
  - Live clients or daemons may continue showing stale permission prompts until another refresh path runs.
- Suggested fix direction:
  - Define and implement expiry propagation semantics for live sessions, or clearly document eventual-consistency expectations.

---

## Findings by functional flow

### Flow A. Device registration and pairing
- Inherits Findings 1, 2, 3
- Main concern:
  - Pair completion mixes current-device authorization with global legacy-device auto-grants.

### Flow B. Host and workspace access
- No standalone ECC blocker found in `hosts.rs` / `workspaces.rs` during this pass.
- Residual risk:
  - They depend on `authz.rs` legacy ACL behavior, so access correctness is only as strong as that layer.

### Flow C. Session creation and live execution
- Inherits Findings 5 and 8
- Main concern:
  - Under load or on Postgres, create-session can violate timing and compatibility expectations.

### Flow D. Session message flow
- Inherits Finding 5
- Main concern:
  - Daemon queue saturation can turn nominally bounded RPC into long-hanging request latency.

### Flow E. Session control and close
- Inherits Finding 7
- Main concern:
  - Terminate path closes the server-side state machine too early.

### Flow F. Permission request/response
- Inherits Findings 4, 10, 12
- Main concern:
  - Permission state can diverge across DB, daemon, and subscribers.

### Flow G. Archive browsing and deletion
- Inherits Findings 9 and 11
- Main concern:
  - Pagination input validation is incomplete, and delete behavior can corrupt archive/session invariants.

### Flow H. Remote file browsing
- Inherits Findings 5 and 6
- Main concern:
  - File APIs depend on WS query-token auth and unbounded pre-timeout queue wait.

---

## Test coverage gaps worth adding next

1. Postgres integration test for idempotent `create_session`
2. Congested daemon-channel test proving `send_and_wait` fails within bounded time
3. Revoked-session-access WS broadcast test
4. Pairing-status test that rejects used pairing records or cleared pairing secrets
5. Permission-response test covering daemon send failure
6. Archive-delete test that asserts session/archive invariants remain valid afterward
7. Pagination boundary tests for `page=0`

---

## Overall assessment

Current backend surfaces are already substantial and cover the main service chain from pairing to live session control, archives, permissions, and file reads.

The most important ECC issues are:
- authorization drift caused by legacy ACL expansion
- state divergence between server and daemon on terminate/permission flows
- transport timeout mismatch under WS backpressure
- archived-session invariant breakage after archive deletion
- probable Postgres incompatibility in the idempotency path
