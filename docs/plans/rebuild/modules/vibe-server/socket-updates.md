# Module Plan: vibe-server/socket-updates

## Purpose

Implement the Socket.IO-compatible update transport used for session and machine live updates.

## Happy Source Of Truth

- `packages/happy-server/sources/app/api/socket.ts`
- `packages/happy-server/sources/app/events/eventRouter.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/api/socket.rs`
  - `src/events/socket_updates.rs`

## Responsibilities

- accept authenticated socket connections on `/v1/updates`
- scope connections by session, machine, or generic client type
- emit `new-message`, `update-session`, and `update-machine` payloads
- manage reconnect-safe subscription state

## Non-Goals

- computing domain state changes
- decrypting payload data

## Public Types And Interfaces

- socket bootstrap function
- connection auth context
- room/subscription helpers

## Data Flow

- client connects with auth payload
- socket auth resolves identity and scope
- event router broadcasts update containers to matching subscribers
- clients maintain local caches from update stream

## Dependencies

- `auth`
- `event-router`
- `session-lifecycle`
- `presence`
- `socketioxide`

## Implementation Steps

1. Stand up Socket.IO-compatible endpoint under `/v1/updates`.
2. Parse auth payload fields used by Happy clients.
3. Assign rooms keyed by session id, machine id, and client role.
4. Emit `vibe-wire` update containers unchanged.
5. Add reconnect and invalid-auth tests.

## Edge Cases And Failure Modes

- client connects without required scope fields
- unauthorized reconnect loops
- stale room memberships after disconnect

## Tests

- authenticated connect test
- invalid auth rejection test
- session-scoped update broadcast test
- machine-scoped update broadcast test

## Acceptance Criteria

- Happy-style socket clients can connect and receive scoped updates
- update payloads are emitted without shape drift

## Open Questions

- None.

## Locked Decisions

- use `socketioxide` for server-side Socket.IO compatibility
- keep update payloads opaque once shaped by `vibe-wire`
