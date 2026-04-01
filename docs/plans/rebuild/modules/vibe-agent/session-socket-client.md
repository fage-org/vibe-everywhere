# Module Plan: vibe-agent/session-socket-client

## Purpose

Implement the live session socket client used for updates, message send, and idle/wait behavior.

## Happy Source Of Truth

- `packages/happy-agent/src/session.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-agent`
- file: `src/session.rs`

## Responsibilities

- connect to `/v1/updates`
- authenticate with token and session scope
- decrypt `new-message` and `update-session` payloads
- expose wait-for-idle behavior
- send user messages and stop requests

## Non-Goals

- HTTP session CRUD

## Public Types And Interfaces

- `SessionClient`
- live update events
- idle-state evaluation helpers

## Data Flow

- caller creates client with session encryption material
- socket receives update containers
- client decrypts content and emits typed events
- user input is encrypted and emitted back to server

## Dependencies

- `http-api-client`
- `encryption`
- Socket.IO client crate

## Implementation Steps

1. Port Happy idle-state logic and ready-event handling rules.
2. Implement authenticated socket connect and reconnect behavior.
3. Implement decrypt-and-emit path for message and state updates.
4. Add send-message and stop helpers.
5. Add mocked socket tests for wait behavior.

## Edge Cases And Failure Modes

- connect timeout
- archived session detected during wait
- ready or turn-end semantics drifting from Happy behavior

## Tests

- connect test
- decrypt update test
- wait-for-idle success and timeout tests
- send-message emit test

## Acceptance Criteria

- agent can monitor and control a live remote session over the socket path

## Open Questions

- None.

## Locked Decisions

- use a dedicated Socket.IO client crate; default choice is `rust_socketio`
- idle detection must mirror Happy semantics before any simplification
