# Project Plan: vibe-server

## Purpose

`vibe-server` is the Rust backend replacing Happy server responsibilities: auth, sessions,
machines, update fanout, storage, feed/social, and supporting APIs needed by the app and clients.

## Happy Source

- primary source: `packages/happy-server`
- supporting source: `packages/happy-wire` and server-focused Happy planning docs

## Target Layout

- crate: `crates/vibe-server`
- top-level module groups:
  - `api`
  - `auth`
  - `events`
  - `sessions`
  - `machines`
  - `presence`
  - `storage`
  - `monitoring`
  - `config`

## Public Interfaces

- HTTP API routes
- socket update namespace and events
- encrypted session and machine persistence
- machine registration, machine metadata, and daemon-state lifecycle
- service-level configuration and startup surface

## Wave 2 Minimum Spine Feature Inventory

1. typed config/bootstrap and version reporting
2. account bootstrap plus auth request lifecycle for challenge and account-link flows
3. bearer-auth middleware shared by HTTP and socket layers
4. session CRUD, history, v3 bulk message insert, and session-scoped update emission
5. machine create/list/detail plus metadata and daemon-state optimistic concurrency
6. presence validation cache, queued heartbeats, batched `activeAt` writes, and timeout sweeps
7. centralized event router for durable and ephemeral fanout
8. Socket.IO-compatible `/v1/updates` transport for session/machine updates and machine RPC
9. minimum app API mounting for auth, sessions, machines, and version metadata
10. route/service tests covering auth, sessions, machines, updates, and presence transitions

## Wave 2 And Wave 4 Remediation Audit

The earlier project-level remediation list has been closed and folded back into the owning modules
and tests. The current `vibe-server` tree now includes automated coverage for the previously
tracked gaps, including:

- Happy-compatible auth DTO parsing and self-verifiable bearer tokens
- session history, v3 message, and incremental session-list parity
- `/v1/updates` invalid-auth handling, `ping`, RPC payload compatibility, and auxiliary socket APIs
- machine/session presence flush and monotonicity rules
- durable update publish failure surfacing and `vibe-wire` fixture validation
- artifact optimistic concurrency, KV all-or-nothing mutation semantics, and socket ownership checks
- GitHub disconnect/takeover compatibility, artifact-create idempotency, and usage-report
  `createdAt` preservation
- local/S3-compatible file storage, persisted upload reuse metadata, image processing, and
  `/metrics` export coverage
- social/feed relationship semantics, notification repeat-key behavior, and durable update routing

No active project-level remediation blockers remain. New defects should be tracked in the owning
module plan first, then promoted back to this project plan only if they reopen a cross-module gate.

## Internal Module Map

- `api`: route registration and response shaping
- `auth`: auth guards, QR/account flows, token lifecycle
- `events`: update routing and fanout
- `sessions`: CRUD, history, lifecycle, session update generation
- `machines`: registration, list/detail, metadata, daemon state, and machine-scoped socket writes
- `presence`: session/machine heartbeat caching, DB flush, active state, and timeout handling
- `account/usage`: account profile, settings, and usage query APIs implemented under `api/`
- `integrations`: generic connect/vendor token routes plus GitHub-specific flows implemented under
  `api/connect.rs`
- `artifacts/access-keys`: opaque artifact bodies and session-machine access-key APIs implemented
  under `api/artifacts.rs` and `api/socket.rs`
- `utility-apis`: KV, push tokens, version, and voice routes implemented under `api/utility.rs`
- `storage`: db/files/redis/image subsystems
- `social` / `feed`: Happy feature parity handlers and services implemented under `api/social.rs`
  and `api/feed.rs`
- `monitoring`: metrics and service health

## Implementation Order

1. versions/config plus storage foundation
2. auth and event-router pass A
3. session lifecycle and machine lifecycle
4. presence plus minimum socket/API surface
5. files/images and support APIs needed by imported clients
6. connect, GitHub, social, and feed domains
7. event-router/socket/API finalization
8. monitoring

## Compatibility Requirements

- wire shapes must come from `vibe-wire`
- imported app paths must not need server-specific protocol forks
- endpoint and socket behavior should match Happy semantics unless a plan explicitly records a
  Vibe-only adaptation
- wave-2 bootstrap storage may be process-local internally, but all consumers must go through the
  same `storage-db` and `storage-redis` seams so external adapters can land without handler or
  protocol churn

## Testing Strategy

- route tests
- update stream tests
- encrypted record storage tests
- integration tests with `vibe-agent` and `vibe-cli`

## Acceptance Criteria

- supports minimum auth/session/machine/update flows used by downstream projects
- machine registration and daemon-state updates work with explicit version control
- session and machine presence transitions are deterministic and test-covered
- update sequencing and encrypted record handling are stable
- route and socket behavior are documented and test-covered
- a single `vibe-server` process can satisfy the minimum remote-control path end to end

## Deferred Items

- performance optimizations not needed for parity
- feature expansions beyond Happy parity
