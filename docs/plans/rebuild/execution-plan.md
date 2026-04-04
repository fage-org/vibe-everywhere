# Rebuild Execution Plan

## Purpose

This file is the authoritative execution order for the Happy-aligned rebuild. Unlike
`migration-order.md`, which is stage-level only, this file defines the module-level sequencing that
AI implementation tasks should follow.

The AI-dispatch companion file is `execution-batches.md`.

Use this file when deciding:

- which module to implement next
- which modules may run in parallel
- which modules must wait for upstream gates
- which modules need multi-pass implementation instead of a one-shot rewrite

## Execution Rules

- Dispatch one implementation task against one module plan file.
- Do not start a downstream module until its listed prerequisites are actually implemented or
  explicitly stubbed with stable interfaces.
- If a module needs a shared contract that is still ambiguous, stop and update the shared plan
  first.
- Prefer one end-to-end usable slice over broad but incomplete surface area.
- Treat the order below as the default critical path. Parallel work is allowed only where this file
  explicitly marks it safe.
- When an ordered item is completed, mark it inline in this file as `[done]` before starting the
  next item. This completion-tracking rule applies to every subsequent wave as well.

## Critical Path Summary

1. freeze shared contracts
2. implement `vibe-wire`
3. implement the minimum `vibe-server` spine
4. implement `vibe-agent` against the real server
5. expand `vibe-server` to cover app and CLI support surfaces
6. implement `vibe-cli`
7. import and adapt `vibe-app`
8. implement `vibe-app-logs` only if still needed

## Wave 0: Shared Planning Freeze

### Goal

Freeze all shared contracts before any Rust or imported-app implementation starts.

### Order

1. `[done]` `shared/source-crosswalk.md`
2. `[done]` `shared/naming.md`
3. `[done]` `shared/data-model.md`
4. `[done]` `shared/protocol-session.md`
5. `[done]` `shared/protocol-auth-crypto.md`
6. `[done]` `shared/protocol-api-rpc.md`
7. `[done]` `shared/validation.md`
8. `[done]` `shared/migration-order.md`
9. `[done]` `execution-plan.md`
10. `[done]` `execution-batches.md`

### Output

- all major Happy source trees have explicit Vibe owners
- naming, data, crypto, session, and API/socket contracts are frozen
- the module-level execution order is frozen

### Gate To Next Wave

- `vibe-wire` can be implemented without inventing missing wire fields or transport semantics

## Wave 1: `vibe-wire`

### Goal

Create the canonical Rust contract crate before any downstream project defines public shapes.

### Order

1. `[done]` `modules/vibe-wire/message-meta.md`
   - unblocks legacy/session messages and provider/runtime metadata
2. `[done]` `modules/vibe-wire/legacy-protocol.md`
   - unblocks compatibility with imported app legacy rendering
3. `[done]` `modules/vibe-wire/session-protocol.md`
   - unblocks CLI/provider event mapping
4. `[done]` `modules/vibe-wire/messages.md`
   - unblocks server updates, session storage, and app update parsing
5. `[done]` `modules/vibe-wire/voice.md`
   - unblocks late server support APIs

### Output

- stable Rust wire types
- serde-compatible JSON contracts
- compatibility fixtures and vectors

### Gate To Next Wave

- downstream projects can depend on `vibe-wire` without defining duplicate public contracts

## Wave 2: `vibe-server` Minimum Spine

### Goal

Deliver the smallest real backend that supports account auth, sessions, machines, and live updates
for `vibe-agent`.

### Feature Inventory

1. typed bootstrap/config path plus version/build metadata
2. storage seams for accounts, auth requests, sessions, messages, machines, and monotonic sequence
   allocation
3. bearer-token auth plus challenge/account-link request-response flows
4. session HTTP surface:
   - `GET /v1/sessions`
   - `GET /v2/sessions/active`
   - `GET /v2/sessions`
   - `POST /v1/sessions`
   - `GET /v1/sessions/:sessionId/messages`
   - `DELETE /v1/sessions/:sessionId`
   - `GET /v3/sessions/:sessionId/messages`
   - `POST /v3/sessions/:sessionId/messages`
5. machine HTTP surface:
   - `POST /v1/machines`
   - `GET /v1/machines`
   - `GET /v1/machines/:id`
6. presence subsystem:
   - validation cache
   - heartbeat queueing
   - batched `activeAt` persistence
   - offline timeout sweeps
7. internal event router for durable `new-message` / `update-session` / `update-machine` fanout and
   ephemeral activity fanout
8. Socket.IO-compatible `/v1/updates` transport with auth handshake plus session, machine, and RPC
   core handlers
9. minimum HTTP router/middleware shell that mounts auth, session, machine, and version routes
10. validation coverage for config parsing, route behavior, optimistic concurrency, update shaping,
    and presence timing rules

### Wave 2 Bootstrap Decision

- deliver a single-instance backend first
- keep `storage-db` and `storage-redis` as the only storage seams used by the rest of the crate
- allow the initial implementation to use process-local typed stores behind those seams so the
  protocol-compatible server path can land before external persistence adapters
- treat PostgreSQL/Redis adapter hardening as follow-up work behind the same storage interfaces,
  without changing HTTP/socket contracts

### Order

1. `[done]` `modules/vibe-server/versions-and-config.md`
   - stand up service config, startup, and version basics
2. `[done]` `modules/vibe-server/storage-db.md`
   - define the primary persistence model first
3. `[done]` `modules/vibe-server/storage-redis.md`
   - required before session/presence/event fanout paths that assume cache/queue support
4. `[done]` `modules/vibe-server/auth.md`
   - required before any protected HTTP or socket surface
5. `[done]` `modules/vibe-server/event-router.md` pass A
   - define internal event contracts and sequencing interfaces
6. `[done]` `modules/vibe-server/session-lifecycle.md`
   - implement session CRUD, history, metadata, and agent-state writes
7. `[done]` `modules/vibe-server/machine-lifecycle.md`
   - implement machine registration, metadata, and daemon-state writes
8. `[done]` `modules/vibe-server/presence.md`
   - implement session/machine validation cache, heartbeats, and timeout rules
9. `[done]` `modules/vibe-server/socket-updates.md` pass A
   - implement `/v1/updates`, auth handshake, session/machine events, and RPC core
10. `[done]` `modules/vibe-server/app-api.md` pass A
   - stand up router, shared middleware, and register the minimum auth/session/machine routes

### Output

- account-linking auth works
- session and machine records exist with live updates
- socket updates and machine RPC core work
- the minimum remote-control path is usable against one running `vibe-server` instance without
  module-local protocol forks

### Gate To Next Wave

- `vibe-agent` can authenticate and exercise the minimum remote-control path against a real server

## Wave 3: `vibe-agent`

### Goal

Ship the remote-control client against the real server before tackling local runtime complexity.

### Order

1. `[done]` `modules/vibe-agent/config.md`
2. `[done]` `modules/vibe-agent/encryption.md`
3. `[done]` `modules/vibe-agent/credentials-and-auth.md`
4. `[done]` `modules/vibe-agent/http-api-client.md`
5. `[done]` `modules/vibe-agent/session-socket-client.md`
6. `[done]` `modules/vibe-agent/machine-rpc.md`
7. `[done]` `modules/vibe-agent/cli-output.md`

### Why This Order

- config and crypto are foundational
- auth depends on config and crypto
- API client depends on auth state
- session socket client depends on both server API and wire contracts
- machine RPC depends on socket transport and machine-scoped server behavior
- CLI output comes last so command UX is shaped around real service behavior, not guesswork

### Output

- `vibe-agent` can log in, list, create, send, history, stop, wait, and issue machine RPC calls
- validation coverage now includes unit tests, CLI smoke tests, mocked RPC tests, server-side RPC
  socket transport tests, and real `vibe-server` end-to-end flows for auth, session control, and
  wait behavior

### Gate To Next Wave

- `[done]` one real end-to-end remote-control flow works through `vibe-agent -> vibe-server`

## Wave 4: `vibe-server` Support Surface Expansion

### Goal

Finish the server APIs needed by imported app flows and by the local CLI runtime.

### Order

1. `[done]` `modules/vibe-server/storage-files.md`
2. `[done]` `modules/vibe-server/image-processing.md`
3. `[done]` `modules/vibe-server/account-and-usage.md`
4. `[done]` `modules/vibe-server/utility-apis.md`
5. `[done]` `modules/vibe-server/artifacts-and-access-keys.md`
6. `[done]` `modules/vibe-server/connect-vendors.md`
7. `[done]` `modules/vibe-server/github.md`
8. `[done]` `modules/vibe-server/social.md`
9. `[done]` `modules/vibe-server/feed.md`
10. `[done]` `modules/vibe-server/event-router.md` pass B
    - broaden update shaping to late support domains without changing the sequencing spine
11. `[done]` `modules/vibe-server/socket-updates.md` pass B
    - wire artifact/access-key/usage auxiliary socket APIs now that their services exist
12. `[done]` `modules/vibe-server/app-api.md` pass B
    - register and finalize the remaining route groups
13. `[done]` `modules/vibe-server/monitoring.md`

### Why This Order

- file/blob infrastructure must exist before artifact-heavy routes
- account/settings/usage and support APIs unblock app startup flows
- artifacts/access-keys and connect/github are deeper feature surfaces that depend on the service
  skeleton already existing
- event routing, socket transport, and router modules are finalized only after their domain services
  exist
- monitoring is last because it should instrument stable behavior, not half-built paths

### Output

- imported app and future CLI runtime have all required server route groups and socket extensions

### Gate To Next Wave

- no route group in `shared/protocol-api-rpc.md` remains without a real owning server
  implementation path

## Wave 5: `vibe-cli`

### Goal

Implement the local runtime and daemon only after the server and wire surfaces are stable enough to
consume.

### Order

1. `modules/vibe-cli/utils-and-parsers.md`
2. `modules/vibe-cli/ui-terminal.md`
3. `modules/vibe-cli/bootstrap-and-commands.md` pass A
   - define config/bootstrap types and the top-level command skeleton
4. `modules/vibe-cli/agent-core.md`
5. `modules/vibe-cli/agent-adapters.md`
6. `modules/vibe-cli/session-protocol-mapper.md`
7. `modules/vibe-cli/transport.md`
8. `modules/vibe-cli/auth.md`
9. `modules/vibe-cli/api-client.md`
10. `modules/vibe-cli/daemon.md`
11. `modules/vibe-cli/sandbox.md`
12. `modules/vibe-cli/persistence-resume.md`
13. `modules/vibe-cli/builtin-modules.md`
14. `modules/vibe-cli/claude-runtime.md`
15. `modules/vibe-cli/testing-fixtures.md` pass A
    - establish first provider/runtime harness around the first implemented provider
16. `modules/vibe-cli/codex-runtime.md`
17. `modules/vibe-cli/gemini-runtime.md`
18. `modules/vibe-cli/openclaw-runtime.md`
19. `modules/vibe-cli/agent-acp.md`
20. `modules/vibe-cli/bootstrap-and-commands.md` pass B
    - finish command wiring once the underlying services are real
21. `modules/vibe-cli/testing-fixtures.md` pass B
    - broaden the fixture matrix across providers

### Why This Order

- utils, parsing, and terminal helpers are shared prerequisites for many CLI paths
- bootstrap needs an early pass for config ownership and a late pass for final command wiring
- core abstractions, adapters, mapper, and transport must exist before any provider runtime lands
- auth must exist before the shared API client hardens around real credential and token behavior
- daemon, sandbox, and persistence must exist before resume and local long-running workflows are
  reliable
- first provider should land before the full provider matrix, so end-to-end behavior is verified
  early

### Output

- at least one provider path works end-to-end
- daemon, sandbox, persistence, and resume are real
- remaining providers are added onto a stable transport core

### Gate To Next Wave

- one local provider can run, persist state, and stream compatible updates through `vibe-server`

## Wave 6: `vibe-app`

### Goal

Bring in the Happy app baseline, then adapt it only after upstream contracts are stable enough to
avoid churn.

### Order

1. `modules/vibe-app/import-and-build.md`
2. `modules/vibe-app/protocol-parser-compat.md`
3. `modules/vibe-app/api-endpoint-adaptation.md`
4. `modules/vibe-app/branding-and-naming-adaptation.md`
5. `modules/vibe-app/desktop-tauri-adaptation.md`
6. `modules/vibe-app/release-and-env.md`

### Why This Order

- import/build must happen first
- protocol parser compatibility should be checked against the finished wire contracts before
  endpoint adaptation
- endpoint adaptation should wait until the server route surface is real
- branding comes after functional adaptation to avoid creating broad merge noise too early
- desktop/Tauri adaptation is last because it depends on the imported package and environment model
- release/env cleanup is the final app-surface normalization pass after endpoints, branding, and
  desktop packaging are stable

### Output

- imported app builds in this repo
- app talks to the Rust backend path
- public Vibe naming replaces Happy naming on user-visible surfaces

### Gate To Next Wave

- app works against Vibe services without protocol forks or Happy-branded public surfaces

## Wave 7: `vibe-app-logs`

### Goal

Only implement the sidecar after the app proves it is still required.

### Order

1. `modules/vibe-app-logs/log-server.md`

### Output

- minimal sidecar runtime if imported app tooling still depends on it

### Gate To Finish

- sidecar behavior required by app tooling is satisfied, or the need is explicitly retired in plans

## Safe Parallel Windows

### Parallel Window A: after Wave 0

Safe parallel candidates:

- none; keep the post-freeze critical path focused on locking and implementing `vibe-wire`

Rules:

- do not start downstream implementation before the wire contracts are real

### Parallel Window B: after Wave 1

Safe parallel candidates:

- `modules/vibe-server/versions-and-config.md`
- `modules/vibe-server/storage-db.md`
- `modules/vibe-server/storage-redis.md`

Rules:

- keep write scopes disjoint when parallelizing server storage/config work
- defer app module execution until the imported app baseline exists under `packages/vibe-app`

### Parallel Window C: after Wave 4

Safe parallel candidates:

- `modules/vibe-cli/utils-and-parsers.md`
- `modules/vibe-cli/ui-terminal.md`

Rules:

- only after server route shapes are stable
- keep the parallel window limited to disjoint CLI foundation work

## Modules That Should Be Implemented In Multiple Passes

- `modules/vibe-server/event-router.md`
  - pass A: internal event contracts and sequencing spine
  - pass B: any late integration refinements discovered by socket/app support modules
- `modules/vibe-server/socket-updates.md`
  - pass A: core handshake, session/machine events, and RPC
  - pass B: auxiliary socket APIs such as artifacts, access keys, and usage
- `modules/vibe-server/app-api.md`
  - pass A: router skeleton plus minimum auth/session/machine routes
  - pass B: remaining route groups after their services exist
- `modules/vibe-cli/bootstrap-and-commands.md`
  - pass A: config/bootstrap ownership
  - pass B: final command wiring
- `modules/vibe-cli/testing-fixtures.md`
  - pass A: first provider/runtime harness
  - pass B: full fixture matrix

## Dispatch Format For AI

When assigning work from this file, always provide:

1. the module plan path
2. the owning project plan path
3. the shared spec paths it depends on
4. the immediately previous module in the execution order
5. the gate that must remain true after the change

Example:

> Implement `docs/plans/rebuild/modules/vibe-server/session-lifecycle.md`.
> Follow `docs/plans/rebuild/projects/vibe-server.md`,
> `docs/plans/rebuild/shared/data-model.md`,
> `docs/plans/rebuild/shared/protocol-session.md`, and
> `docs/plans/rebuild/shared/protocol-api-rpc.md`.
> Treat `docs/plans/rebuild/modules/vibe-server/event-router.md` pass A as already completed and do
> not change shared contracts unless you update the shared plan first.
