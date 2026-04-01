# Module Plan: vibe-server/event-router

## Purpose

Implement the internal routing layer that fans out session and machine changes to socket updates and
other interested subsystems.

## Happy Source Of Truth

- `packages/happy-server/sources/app/events/eventRouter.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/events/mod.rs`
  - `src/events/router.rs`

## Responsibilities

- accept domain update events
- sequence them
- convert them into `vibe-wire` update containers
- broadcast to the socket layer

## Non-Goals

- computing the session or machine state itself
- storage writes

## Public Types And Interfaces

- event router service
- internal event enum for session and machine updates
- publish helper methods

## Data Flow

- domain module emits internal event
- router allocates update sequence and container metadata
- socket update layer broadcasts shaped payload

## Dependencies

- `socket-updates`
- `session-lifecycle`
- `presence`
- `vibe-wire`

## Implementation Steps

1. Define internal event enum keyed to session and machine changes.
2. Centralize update sequence generation.
3. Convert internal events to `CoreUpdateContainer`.
4. Publish to socket rooms without duplicating shaping logic elsewhere.
5. Test ordering and fanout behavior.

## Edge Cases And Failure Modes

- out-of-order sequence assignment
- duplicate broadcasts on retries
- mixed session and machine event routing

## Tests

- monotonic sequence test
- session broadcast test
- machine broadcast test
- duplicate suppression or idempotency test if implemented

## Acceptance Criteria

- all live update fanout goes through one router
- update containers are ordered and correctly scoped

## Open Questions

- None.

## Locked Decisions

- centralize update shaping and sequencing here
- use `vibe-wire` types directly for outbound update payloads
