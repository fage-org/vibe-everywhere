# Module Plan: vibe-server/session-lifecycle

## Purpose

Implement encrypted session CRUD, message history, metadata updates, and lifecycle state changes.

## Happy Source Of Truth

- `packages/happy-server/sources/app/session/sessionDelete.ts`
- session-related API usage in `packages/happy-agent/src/api.ts`
- update/session handling from `packages/happy-wire/src/messages.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/sessions/mod.rs`
  - `src/sessions/service.rs`
  - `src/sessions/http.rs`

## Responsibilities

- list sessions
- list active sessions
- create sessions
- delete/archive sessions
- append message history
- update encrypted metadata and agent state

## Non-Goals

- message content decryption
- provider runtime execution

## Public Types And Interfaces

- session service
- session repository interface
- HTTP handlers for session operations

## Data Flow

- client creates or updates session via HTTP or socket
- server stores encrypted records and increments sequence/version fields
- event router emits durable update containers whose body variants include `update-session` and
  `new-message`

## Dependencies

- `storage-db`
- `storage-redis`
- `event-router`
- `vibe-wire`

## Implementation Steps

1. Define session persistence model aligned with `shared/data-model.md`.
2. Implement list/create/delete/message-history operations.
3. Keep metadata and agent-state versioning explicit.
4. Trigger event-router updates on every state-changing write.
5. Add integration tests covering create, list, history, and deletion.

## Edge Cases And Failure Modes

- legacy vs dataKey session records
- race conditions around sequence increments
- archived session access

## Tests

- create/list/history/delete flow
- metadata and agent-state version increment tests
- archived-session rejection tests
- update emission tests

## Acceptance Criteria

- core session operations work and emit correct updates
- encrypted session fields remain opaque and versioned

## Open Questions

- None.

## Locked Decisions

- persistence is append/update with explicit sequence and version counters
- lifecycle state belongs in encrypted metadata, not as a parallel public schema
