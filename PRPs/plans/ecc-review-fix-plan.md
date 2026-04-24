# Vibe-Remote ECC Review 8-Item Fix Plan

## Background

Review document identified 8 issues (3 HIGH / 3 MEDIUM / 2 LOW) covering mock client contract drift, remote mode unusability, weak flow assertions, TS binding drift, settings concurrency race, input validation gap, and workspace pagination gap.

---

## Phase 1: Fix ve-mock-client API Contract Drift (HIGH-01)

### 1.1 `MockClient::register_device` — align to `RegisterDeviceRequest`

**File**: `crates/ve-mock-client/src/client.rs`

Current sends `{name, server_url}`, but server `RegisterDeviceRequest` needs `{device_name, device_type, server_url}`.

- Signature: `register_device(&self, device_name: &str, device_type: DeviceType, server_url: &str, idempotency_key: &str)`
- Body fields: `device_name` / `device_type` / `server_url`
- Use `ve_shared::types::DeviceType` with serde `snake_case` rename

### 1.2 `MockClient::pairing_status` — align to query + header route

Current calls `/api/auth/pairing-status/{id}` (nonexistent). Real route: `GET /api/auth/pairing-status?host_id=X` + `x-pairing-secret` header.

- Signature: `pairing_status(&self, host_id: Uuid, pairing_secret: &str)`
- URL: `/api/auth/pairing-status?host_id={host_id}`
- Add header: `x-pairing-secret: {pairing_secret}`

### 1.3 `MockClient::pair` — align to POST body route

Current calls `/api/auth/pair/{device_id}`. Real route: `POST /api/auth/pair` + `{"pair_code": "..."}`.

- Signature: `pair(&self, pair_code: &str)`
- URL: `/api/auth/pair`
- Body: `{"pair_code": pair_code}`

### 1.4 `MockClient::create_workspace` — align to `CreateWorkspaceRequest`

Current sends `name/description`, real model is `display_name` + `path` + `host_id`.

- Signature: `create_workspace(&self, host_id: Uuid, path: &str, display_name: Option<&str>)`
- Body fields: `host_id` / `path` / `display_name`

### 1.5 `MockClient::update_notification_preferences` — align field names

Current sends `email_enabled/desktop_enabled/sound_enabled`, real fields: `enabled/permission_request_enabled/task_completed_enabled/task_failed_enabled/session_error_enabled`.

- Align body to `UpdateNotificationPreferencesRequest` fields

### 1.6 `RegisterDeviceResponse` — align server response fields

Current declares `device_secret: String`, but server `RegisterDeviceResponse` returns `token: String`.

- Change to `{device_id: Uuid, token: String}`

### 1.7 Update all callers

- `integration_env.rs` already does direct DB inserts — unaffected
- All flow files calling `create_workspace` need parameter updates

---

## Phase 2: Fix Remote Mode (HIGH-02)

### 2.1 CLI: `--daemon-token` → `--client-token`

**File**: `crates/ve-mock-client/src/main.rs`

- Add `--client-token` parameter (required with `--remote`)
- Keep `--daemon-token` as optional (only for daemon-side verification scenarios)

### 2.2 `TestContext::new_remote` — use client token

**File**: `crates/ve-mock-client/src/test_context.rs`

- `new_remote(server_url, host_name, client_token, host_id)` — third param becomes `client_token`
- `MockClient` uses `client_token` for Authorization header

### 2.3 Implement `host_id` auto-discovery

- In `new_remote`, call `list_hosts()` to auto-discover the first available host_id
- If discovery fails and `host_id` not provided, return error

---

## Phase 3: Tighten Flow Assertions (HIGH-03)

### 3.1 F5 Session Control

**File**: `crates/ve-mock-client/src/flows/f5_session_control.rs`

- Pause (lines 59-72): HTTP 4xx/5xx (non-network) should `bail!`, not mark "acceptable"
- Close (lines 110-122): Same

### 3.2 F7 Session Archival

**File**: `crates/ve-mock-client/src/flows/f7_session_archival.rs`

- Close (lines 64-79): HTTP errors should `bail!` (except network errors)

### 3.3 F17 Real Session Control

**File**: `crates/ve-mock-client/src/flows/f17_real_session_control.rs`

- Pause (lines 109-121): HTTP errors should `bail!`
- Close cleanup (lines 143-155): HTTP errors should `bail!`

### 3.4 F18 Real Archival — already strict (lines 90-97 have `.map_err`), no changes needed

### 3.5 F19 Real Error Handling

**File**: `crates/ve-mock-client/src/flows/f19_real_error_handling.rs`

- Tighten close cleanup error swallowing to bail

---

## Phase 4: Mark Fixture-Driven Flows (MEDIUM-01)

### 4.1 Add `integration-read-path` comments

**Files**: F6, F7, F9, F12

Add top-level doc comments explicitly marking these as fixture-driven (read-path only, not full E2E):

```rust
//! F6: Permission request/response
//!
//! NOTE: This flow is marked as `integration-read-path`. It creates test fixtures
//! directly in the database to verify the read-side API path (list/respond/query).
//! It does NOT exercise the full daemon → WS → server → DB write chain.
```

---

## Phase 5: Fix Settings Concurrency Race (MEDIUM-03)

### 5.1 settings.rs: check-then-insert → atomic upsert

**File**: `crates/ve-server/src/api/settings.rs` (lines 110-158)

Replace SELECT-then-INSERT/UPDATE with a single `INSERT ... ON CONFLICT(device_id) DO UPDATE`:

```sql
INSERT INTO notification_preferences (device_id, enabled, permission_request_enabled, ...)
VALUES ($1, $2, $3, $4, $5, $6)
ON CONFLICT(device_id) DO UPDATE SET
    enabled = COALESCE(EXCLUDED.enabled, notification_preferences.enabled),
    permission_request_enabled = COALESCE(EXCLUDED.permission_request_enabled, notification_preferences.permission_request_enabled),
    ...
```

SQLite and PostgreSQL both support `ON CONFLICT DO UPDATE` syntax.

---

## Phase 6: Fix TS Binding Drift (MEDIUM-02)

### 6.1 Address ts-rs `skip_serializing_if` parsing warnings

**File**: `crates/ve-shared/src/models.rs` (lines 157-170)

The `ArchiveMetadata` struct uses `#[serde(skip_serializing_if = "Option::is_none")]` on 5 fields, but ts-rs generates them as required keys with nullable values (`field: string | null` instead of `field?: string | null`).

Approach:
1. Check if upgrading `ts-rs` to a version that properly parses `skip_serializing_if` resolves this
2. If not, manually patch the generated TS files to make these fields optional

### 6.2 Regenerate bindings

Run `cargo test` to trigger `#[ts(export)]` regeneration. Verify `crates/ve-shared/bindings/` output.

### 6.3 Sync client directory

Copy updated bindings to `client/src/types/generated/`.

---

## Phase 7: Add register_device URL Validation (LOW-01)

### 7.1 auth.rs: validate server_url format

**File**: `crates/ve-server/src/api/auth.rs` (around line 58)

Before INSERT in `register_device`, add URL validation:
- Must start with `http://` or `https://`
- Max length 2048 characters

---

## Phase 8: Add Workspace List Pagination (LOW-02)

### 8.1 workspaces.rs: add page/limit parameters

**File**: `crates/ve-server/src/api/workspaces.rs` (line 68)

- Add `Query<Pagination>` parameter (reuse `ve_shared::types::Pagination`)
- SQL adds `LIMIT / OFFSET`
- Return type changes from `Vec<Workspace>` to `Paginated<Workspace>`

---

## Processing Order & Dependencies

```
Phase 1 (HIGH-01: mock client contract)
    ↓ After updating, verify all flow callers compile
Phase 2 (HIGH-02: remote mode)
    ↓ Depends on Phase 1 (MockClient must be correct first)
Phase 3 (HIGH-03: flow assertion tightening)
    ↓ Depends on Phase 1 (assertions only make sense with correct requests)
Phase 4 (MEDIUM-01: fixture markers)
    ↓ Independent, can run in parallel with Phases 1-3
Phase 5 (MEDIUM-03: settings upsert)
    ↓ Independent
Phase 6 (MEDIUM-02: TS bindings)
    ↓ Independent
Phase 7 (LOW-01: URL validation)
    ↓ Independent
Phase 8 (LOW-02: workspace pagination)
    ↓ Independent
```

## Verification

After all changes:
1. `cargo test --workspace` — all pass
2. `cargo clippy --workspace --all-targets -- -W clippy::all` — no new warnings
3. `cargo fmt` — formatted
4. Run mock-client flows to verify regression signals are correct
