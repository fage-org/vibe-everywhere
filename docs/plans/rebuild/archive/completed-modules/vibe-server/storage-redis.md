# Module Plan: vibe-server/storage-redis

## Purpose

Implement Redis-backed ephemeral state used for updates, presence, queues, and short-lived auth
flows.

## Wave 2 Bootstrap Note

Wave 2 may start with a process-local ephemeral store behind this module while the server remains a
single-instance backend. The Redis adapter remains the long-term target, but the initial minimum
spine may ship if the same typed helper surface is preserved for auth/presence consumers.

## Happy Source Of Truth

- `packages/happy-server/sources/storage/redis.ts`
- presence-related Happy modules

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/storage/redis.rs`

## Responsibilities

- manage Redis connection
- store ephemeral auth and coordination state
- support update fanout helpers if required

## Non-Goals

- durable record persistence

## Public Types And Interfaces

- Redis bootstrap
- typed helper methods for auth/presence/cache use cases

## Data Flow

- auth stores pending link requests
- presence stores active session or machine state
- event and API layers query/update ephemeral state

## Dependencies

- `redis`

## Implementation Steps

1. Define Redis key layout.
2. Implement typed accessors for auth and presence.
3. Add TTL policies matching Happy semantics where needed.
4. Add integration tests against real Redis.

## Edge Cases And Failure Modes

- stale keys causing false active presence
- missing TTL on short-lived auth state
- Redis outage fallback behavior

## Tests

- key round-trip tests
- TTL behavior test
- Redis outage handling test

## Follow-Up Status

- Wave 2 bootstrap hardening now requires auth and presence consumers to use typed helpers from
  this module even while the backing store remains process-local, so the later Redis adapter does
  not require higher-level service refactors.

## Acceptance Criteria

- ephemeral auth and presence flows are backed by Redis predictably
- wave-2 bootstrap may satisfy the same interface with a process-local TTL store until the Redis
  adapter lands

## Open Questions

- None.

## Locked Decisions

- Redis is mandatory for ephemeral distributed state
- key naming must stay namespaced under Vibe-specific prefixes
- presence-specific cache and timeout policy stay owned by `presence`; this module only supplies the
  Redis client and typed storage helpers that those higher-level services consume
- no higher-level module may parse or manage raw ephemeral keys outside this module
