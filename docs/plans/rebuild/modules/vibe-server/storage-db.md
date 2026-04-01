# Module Plan: vibe-server/storage-db

## Purpose

Implement relational persistence for accounts, sessions, machines, feed, social, and related
server state.

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

## Open Questions

- None.

## Locked Decisions

- use PostgreSQL with `sqlx`
- use checked SQL migrations committed to the repo
