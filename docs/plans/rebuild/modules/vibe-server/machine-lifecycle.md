# Module Plan: vibe-server/machine-lifecycle

## Purpose

Implement machine registration, machine record reads, machine metadata updates, daemon-state
updates, and machine-scoped coordination required by remote control.

## Happy Source Of Truth

- `packages/happy-server/sources/app/api/routes/machinesRoutes.ts`
- `packages/happy-server/sources/app/api/socket/machineUpdateHandler.ts`
- machine-scoped connection behavior in `packages/happy-server/sources/app/api/socket.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/machines/mod.rs`
  - `src/machines/service.rs`
  - `src/machines/http.rs`
  - `src/machines/socket.rs`

## Responsibilities

- create or register machine records
- list machines and fetch machine detail
- store encrypted machine metadata
- store encrypted daemon state independently from metadata
- enforce optimistic concurrency on machine metadata and daemon-state updates
- handle `machine-alive`, `machine-update-metadata`, and `machine-update-state`
- publish machine record and ephemeral activity updates via the event router

## Non-Goals

- executing local daemon actions
- decrypting machine payloads on the server
- timeout scanning logic owned by `presence`

## Public Types And Interfaces

- machine service
- machine repository interface
- HTTP handlers for `/v1/machines`
- socket handlers for machine-scoped updates

## Data Flow

- machine/daemon authenticates and creates or loads a machine record
- initial metadata is stored encrypted with its own version counter
- machine-scoped socket heartbeats update presence
- machine metadata and daemon-state writes use optimistic concurrency and return
  `success/version-mismatch/error`
- successful writes emit durable machine updates plus ephemeral activity when relevant

## Dependencies

- `auth`
- `event-router`
- `storage-db`
- `vibe-wire`

## Implementation Steps

1. Define the machine persistence model from `shared/data-model.md`.
2. Implement `POST /v1/machines`, `GET /v1/machines`, and `GET /v1/machines/:id`.
3. Keep `metadataVersion` and `daemonStateVersion` as independent optimistic-concurrency counters.
4. Implement `machine-update-metadata` ack handling:
   - on success, increment version and return the newly stored encrypted value
   - on mismatch, return current encrypted value and current version
5. Implement `machine-update-state` with the same optimistic-concurrency pattern.
6. Define the stable seam that later routes `machine-alive` heartbeats through `presence` instead of
   writing storage directly.
7. Emit machine update containers through `event-router` after successful metadata or daemon-state
   writes.
8. Add tests for registration, list/detail, metadata version mismatch, daemon-state version
   mismatch, and presence integration.

## Edge Cases And Failure Modes

- same machine id re-registers after reinstall or auth reset
- metadata and daemon-state updates race on different sockets
- machine reconnects after timeout and must flip back to active
- `legacy` vs `dataKey` machine encryption variants
- daemon state contains forward-compatible enum values not yet known by the app

## Tests

- machine registration test
- machine list/detail ownership test
- metadata optimistic-concurrency success test
- metadata version-mismatch test
- daemon-state optimistic-concurrency success test
- daemon-state version-mismatch test
- machine-alive to presence integration test
- machine update emission test

## Acceptance Criteria

- machine registration and lookup work for imported app and CLI flows
- metadata and daemon-state updates are versioned, deterministic, and wire-compatible
- machine heartbeat handling is delegated to the presence subsystem instead of ad hoc writes

## Open Questions

- None.

## Locked Decisions

- machine metadata and daemon state are separate encrypted documents with separate version counters
- canonical machine liveness fields are public record fields `active` and `activeAt`, not encrypted
  metadata fields
- machine-scoped socket writes must use the same `success/version-mismatch/error` ack pattern as
  session metadata/state updates
