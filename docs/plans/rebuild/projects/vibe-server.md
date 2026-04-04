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
  - `social`
  - `feed`
  - `github`
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

## Current Remediation Focus

- harden Wave 2 auth compatibility by restoring Happy-compatible request DTOs and self-verifiable
  bearer tokens
- restore session HTTP DTO parity for history and v3 message surfaces
- restore session list freshness so message writes advance the same session change signal that
  drives `/v1/sessions` ordering and `/v2/sessions` incremental sync
- close `/v1/updates` transport gaps around invalid-auth behavior, `ping`, and RPC payload
  compatibility
- align machine-registration compatibility fanout with Happy semantics so the bootstrap
  `update-machine` payload only mirrors the expected metadata backfill
- tighten presence flush behavior so queued heartbeats are not dropped at cache-expiry boundaries
- harden presence monotonicity so out-of-order heartbeats and socket lifecycle transitions cannot
  roll session or machine `activeAt` backward or drift from persisted `active` state
- finish Wave 2 event-router ownership by moving durable session/machine update sequence allocation
  and payload shaping behind router publish helpers instead of duplicating that logic in services
- stop swallowing durable update publish failures after successful session/machine CAS writes so
  socket ack paths cannot report success while fanout failed
- backfill route/socket tests for the above compatibility and regression cases
- close the remaining Wave 2 validation set with server-owned `vibe-wire` fixture compatibility
  checks and storage-focused integration coverage for encrypted records and sequence behavior
- move presence bootstrap cache state behind the `storage-redis` typed seam so the later Redis
  adapter can replace the process-local bootstrap without reopening presence/service boundaries
- close Wave 4 correctness gaps where HTTP/socket route presence exists but Happy-compatible write
  semantics are still missing:
  - restore atomic optimistic-concurrency behavior for artifact HTTP updates
  - restore all-or-nothing batch semantics for KV mutations
  - close auxiliary socket authorization gaps, especially artifact delete ownership checks
- restore Wave 4 support-domain payload compatibility:
  - artifact, KV, and related opaque payloads must preserve the Happy-compatible base64/string
    transport shape
  - access-key create/update bodies and account/connect error responses must match the shared API
    contract exactly
- replace Wave 4 placeholder integration behavior with real compatibility-locked flows:
  - GitHub params/callback/webhook/disconnect must follow Happy-compatible success and failure
    behavior
  - voice token issuance must perform the same entitlement and ElevenLabs checks, or fail with the
    same configured-service semantics
- finish the omitted Wave 4 platform surfaces:
  - `storage-files` needs a real local-filesystem adapter plus public file serving
  - `storage-files` remaining closure includes persisted uploaded-file reuse metadata and an
    environment-selected S3-compatible backend seam so Happy's restart-stable avatar reuse and
    object-storage deployment model are both preserved
  - `image-processing` needs deterministic resize and real thumbhash output rather than hash-based
    placeholders
  - `monitoring` needs route/socket instrumentation and a `/metrics` export surface
- restore Happy-compatible social/feed semantics, including relationship-state transitions,
  feed-item body typing, cursor rules, and durable update ownership
- expand Wave 4 validation beyond router smoke coverage to include auxiliary socket APIs,
  compatibility error cases, and the missing module surfaces above
- finish the remaining Wave 4 parity gaps found in the final review:
  - when GitHub account linkage migrates from one account to another, the displaced account still
    needs the same `update-account` durable disconnect fanout that Happy emits through
    `githubDisconnect`
  - socket `artifact-create` must mirror Happy's idempotent same-account success behavior and the
    cross-account conflict message used by the HTTP route
  - socket `usage-report` upserts must preserve the original `createdAt` timestamp when overwriting
    the existing `(accountId, sessionId, key)` report

## Internal Module Map

- `api`: route registration and response shaping
- `auth`: auth guards, QR/account flows, token lifecycle
- `events`: update routing and fanout
- `sessions`: CRUD, history, lifecycle, session update generation
- `machines`: registration, list/detail, metadata, daemon state, and machine-scoped socket writes
- `presence`: session/machine heartbeat caching, DB flush, active state, and timeout handling
- `account/usage`: account profile, settings, and usage query APIs
- `integrations`: generic connect/vendor token routes plus GitHub-specific flows
- `artifacts/access-keys`: opaque artifact bodies and session-machine access-key APIs
- `utility-apis`: KV, push tokens, version, and voice routes
- `storage`: db/files/redis/image subsystems
- `social` / `feed` / `github`: Happy feature parity modules
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
