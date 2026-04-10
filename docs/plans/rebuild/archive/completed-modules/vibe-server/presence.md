# Module Plan: vibe-server/presence

## Purpose

Implement the shared session and machine presence subsystem: validation cache, heartbeat handling,
batched `activeAt` persistence, timeout sweeps, and ephemeral activity fanout.

## Happy Source Of Truth

- `packages/happy-server/sources/app/presence/sessionCache.ts`
- `packages/happy-server/sources/app/presence/timeout.ts`
- machine/session activity emission usage in `packages/happy-server/sources/app/api/socket.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/presence/mod.rs`
  - `src/presence/cache.rs`
  - `src/presence/timeout.rs`

## Responsibilities

- validate session and machine ids against account ownership with a short-lived cache
- queue session and machine `activeAt` updates instead of writing on every heartbeat
- flush queued `activeAt` updates in small batches
- maintain `active` state for sessions and machines
- mark sessions and machines inactive after timeout
- emit ephemeral activity changes for user-scoped observers

## Non-Goals

- session or machine CRUD
- decrypting metadata, daemon state, or message content
- socket transport framing itself

## Public Types And Interfaces

- presence service
- session/machine validation helpers
- heartbeat queue interface
- timeout worker bootstrap

## Data Flow

- socket or HTTP domain logic asks presence to validate a session or machine
- cache hit returns immediately; cache miss loads from storage and seeds the cache
- heartbeat updates queue a new `activeAt` timestamp when the threshold is exceeded
- batch flusher persists queued timestamps and sets `active = true`
- timeout worker scans for stale active sessions and machines, flips them inactive, and emits
  ephemeral activity updates

## Dependencies

- `storage-db`
- `event-router`
- `tokio`

## Implementation Steps

1. Implement a unified activity cache for sessions and machines keyed by `(accountId, entityId)`.
2. Lock the validation cache TTL to `30_000 ms`.
3. Lock the DB-write threshold to `30_000 ms` so repeated heartbeats inside the window do not write.
4. Lock the batch flush interval to `5_000 ms`.
5. Persist queued session updates as `activeAt = timestamp, active = true`.
6. Persist queued machine updates as `activeAt = timestamp, active = true`.
7. Run a timeout sweep every `60_000 ms` and mark sessions or machines inactive when
   `activeAt <= now - 600_000 ms`.
8. Emit ephemeral offline activity updates only when the active flag actually transitions.
9. Add tests for cache hit/miss, flush batching, timeout transitions, and duplicate suppression.

## Edge Cases And Failure Modes

- cache entry survives after session/machine deletion
- out-of-order heartbeat timestamps
- duplicate offline transitions during concurrent timeout sweeps
- very frequent heartbeats causing unbounded queue growth
- disconnect events racing with timeout processing

## Tests

- session validation cache hit/miss test
- machine validation cache hit/miss test
- heartbeat threshold skip test
- batch flush persistence test
- session timeout transition test
- machine timeout transition test
- ephemeral activity emission deduplication test

## Follow-Up Status

- Wave 2 presence hardening completed for offline-state consistency: pending heartbeat writes are
  now cleared on explicit and timeout-driven offline transitions, with regression coverage for
  session-end and timeout sweep paths.
- Wave 2 monotonic-heartbeat hardening is now completed as well: stale heartbeats no longer regress
  emitted `activeAt`, and explicit offline-to-online heartbeat recovery restores persisted
  `active` state immediately before broadcasting the reactivation.
- machine socket lifecycle transitions remain presence-owned so user-scoped activity updates stay
  aligned with persisted liveness.
- Wave 2 seam hardening now also requires the bootstrap cache state to live behind
  `storage-redis` typed helpers instead of a presence-owned raw in-memory map, so the later Redis
  adapter can replace the bootstrap store without changing presence call sites.

## Acceptance Criteria

- session and machine heartbeats no longer write the database on every event
- session and machine offline transitions happen deterministically after the locked timeout window
- ephemeral activity emissions stay consistent with persisted `active` state

## Open Questions

- None.

## Locked Decisions

- validation cache TTL: `30 seconds`
- DB update threshold: `30 seconds`
- batch flush interval: `5 seconds`
- inactivity timeout window: `10 minutes`
- timeout sweep interval: `1 minute`
- presence owns `active` / `activeAt` transitions; domain modules consume that state but do not
  duplicate timeout logic
