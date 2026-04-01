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
- emit `update` containers, `ephemeral` activity payloads, and RPC lifecycle events
- handle inbound session events such as `message`, `update-metadata`, `update-state`,
  `session-alive`, and `session-end`
- handle inbound machine events such as `machine-alive`, `machine-update-metadata`, and
  `machine-update-state`
- expose auxiliary socket APIs such as `usage-report`, `artifact-read`, `artifact-update`, and
  `access-key-get`
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
- event router broadcasts `update` containers and ephemeral activity payloads to matching
  subscribers
- clients maintain local caches from update stream

## Dependencies

### Pass A

- `auth`
- `event-router` pass A
- `session-lifecycle`
- `machine-lifecycle`
- `presence`
- `socketioxide`

### Pass B additions

- `account-and-usage`
- `artifacts-and-access-keys`
- `event-router` pass B
- `utility-apis`

## Pass Boundaries

- pass A:
  - stand up the `/v1/updates` transport
  - implement auth handshake, session/machine event handlers, and machine RPC forwarding
  - emit core `update` and `ephemeral` payloads for remote-control flows
- pass B:
  - add auxiliary socket APIs such as artifacts, access keys, and usage
  - keep the handshake and core room/subscription model unchanged

## Implementation Steps

1. Pass A: stand up Socket.IO-compatible endpoint under `/v1/updates`.
2. Parse auth payload fields used by Happy clients.
3. Assign rooms keyed by session id, machine id, and client role.
4. Pass A: wire inbound session, machine, and RPC event handlers.
5. Pass B: wire inbound artifact, access-key, and usage handlers.
6. Emit `vibe-wire` update containers unchanged on the `update` event.
7. Add reconnect and invalid-auth tests.

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
- inbound ack/callback shapes match the shared protocol document

## Open Questions

- None.

## Locked Decisions

- use `socketioxide` for server-side Socket.IO compatibility
- keep update payloads opaque once shaped by `vibe-wire`
