# Module Plan: vibe-server/storage-db

## Purpose

Implement relational persistence for accounts, sessions, machines, feed, social, and related
server state.

## Wave 2 Bootstrap Note

Wave 2 only needs one running server instance to unblock `vibe-agent`. The initial implementation
may therefore keep a process-local typed store behind this module's public interfaces, as long as:

- all higher-level modules depend only on `storage-db` interfaces
- monotonic sequence allocation semantics stay stable
- record shapes stay aligned with `shared/data-model.md`
- later PostgreSQL work can replace the backing store without changing handler or socket contracts

## Happy Source Of Truth

- `packages/happy-server/prisma/*`
- `packages/happy-server/sources/storage/db.ts`
- `packages/happy-server/sources/storage/inTx.ts`
- `packages/happy-server/sources/storage/seq.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/storage/db.rs`
  - `src/storage/tx.rs`
  - `src/storage/seq.rs`
  - `migrations/`

## Responsibilities

- own relational schema and migrations
- provide transaction helpers
- allocate sequence numbers for sessions, messages, and updates

## Non-Goals

- file/blob storage
- Redis caching

## Public Types And Interfaces

- database pool bootstrap
- transaction wrapper
- repository traits or services for higher-level modules

## Data Flow

- API/domain module opens transaction
- repositories persist encrypted or plain metadata rows
- sequence allocator returns monotonic values

## Dependencies

- `sqlx`
- PostgreSQL

## Implementation Steps

1. Model schema from Happy persistence concepts, not from old Vibe schema.
2. Create SQLx migrations for accounts, sessions, machines, feed, social, and support tables.
3. Add transaction helpers and repository primitives.
4. Add sequence allocation helpers used by session and event modules.
5. Add integration tests against a real Postgres instance.

## Edge Cases And Failure Modes

- concurrent sequence allocation
- transaction rollback around multi-step session updates
- migration drift between docs and code

## Tests

- migration smoke test
- sequence monotonicity test
- transaction rollback test

## Acceptance Criteria

- server modules can persist all parity-critical state using one relational backend
- wave-2 bootstrap may satisfy the same interface with a process-local store until the PostgreSQL
  adapter lands

## Open Questions

- None.

## Locked Decisions

- use PostgreSQL with `sqlx`
- use checked SQL migrations committed to the repo
- no higher-level module may bypass this storage seam even during the process-local bootstrap phase
