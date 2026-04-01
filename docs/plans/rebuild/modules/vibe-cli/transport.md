# Module Plan: vibe-cli/transport

## Purpose

Implement the transport layer between local provider runtimes, server sessions, and internal event
handlers.

## Happy Source Of Truth

- `packages/happy-cli/src/agent/transport/*`
- related API session usage in `packages/happy-cli/src/api/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/transport/mod.rs`
  - `src/transport/default_transport.rs`

## Responsibilities

- define outbound message transport contract
- connect runtime events to server session updates
- centralize buffering/backpressure behavior

## Non-Goals

- provider-specific parsing

## Public Types And Interfaces

- transport trait
- default transport implementation

## Data Flow

- runtime emits internal message
- transport applies buffering and mapping
- transport forwards to API/session clients

## Dependencies

- `agent-core`
- `api-client`
- `session-protocol-mapper`

## Implementation Steps

1. Port Happy transport handler abstractions.
2. Implement default transport with async channel buffering.
3. Add tests for message ordering and shutdown behavior.

## Edge Cases And Failure Modes

- backpressure under fast provider output
- message loss on shutdown

## Tests

- ordering test
- graceful shutdown test
- backpressure test

## Acceptance Criteria

- provider outputs reach the server in correct order with controlled buffering

## Open Questions

- None.

## Locked Decisions

- transport is async and channel-backed
- ordering guarantees are explicit and test-covered
