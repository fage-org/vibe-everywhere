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
- finalize auxiliary/account/social/feed/artifact update shaping once those domain services exist

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

### Pass A

- `vibe-wire`

### Pass B additions

- `account-and-usage`
- `artifacts-and-access-keys`
- `feed`
- `github`
- `social`
- `utility-apis`

## Pass Boundaries

- pass A:
  - define internal event contracts, sequencing, and the core session/machine update builders
  - expose a stable publish interface that downstream services can target
- pass B:
  - add late support-domain update builders once account, artifact, feed, social, and related
    services are real
  - keep the sequencing spine unchanged while broadening payload coverage

## Implementation Steps

1. Pass A: define the internal event enum keyed to session and machine changes.
2. Centralize update sequence generation.
3. Convert pass-A internal events to `CoreUpdateContainer`.
4. Pass B: add late auxiliary update builders without reworking the sequencing spine.
5. Publish to socket rooms without duplicating shaping logic elsewhere.
6. Test ordering and fanout behavior.

## Edge Cases And Failure Modes

- out-of-order sequence assignment
- duplicate broadcasts on retries
- mixed session and machine event routing

## Tests

- monotonic sequence test
- session broadcast test
- machine broadcast test
- duplicate suppression or idempotency test if implemented

## Follow-Up Status

- Wave 2 hardening now requires router-owned publish helpers for session and machine durable
  updates so account update-sequence allocation and compatibility payload shaping stop leaking into
  service modules.
- Wave 2 hardening also requires session and machine services to surface publish failures instead
  of silently acknowledging successful state writes when durable fanout could not be sequenced.

## Acceptance Criteria

- all live update fanout goes through one router
- update containers are ordered and correctly scoped

## Open Questions

- None.

## Locked Decisions

- centralize update shaping and sequencing here
- use `vibe-wire` types directly for outbound update payloads
