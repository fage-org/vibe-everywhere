# Module Plan: vibe-server/storage-redis

## Purpose

Implement Redis-backed ephemeral state used for updates, presence, queues, and short-lived auth
flows.

## Happy Source Of Truth

- `packages/happy-server/sources/storage/redis.ts`
- presence-related Happy modules

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/storage/redis.rs`
  - `src/presence/cache.rs`

## Responsibilities

- manage Redis connection
- store ephemeral auth and presence state
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
- `auth`
- `presence`

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

## Acceptance Criteria

- ephemeral auth and presence flows are backed by Redis predictably

## Open Questions

- None.

## Locked Decisions

- Redis is mandatory for ephemeral distributed state
- key naming must stay namespaced under Vibe-specific prefixes
