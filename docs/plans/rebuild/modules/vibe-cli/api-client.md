# Module Plan: vibe-cli/api-client

## Purpose

Implement server communication used by the local CLI runtime.

## Happy Source Of Truth

- `packages/happy-cli/src/api/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/api/mod.rs`
  - `src/api/session.rs`
  - `src/api/machine.rs`
  - `src/api/rpc.rs`

## Responsibilities

- authenticate to Vibe server
- create/open sessions
- send messages and updates
- support machine and RPC interactions required by runtime flows

## Non-Goals

- local credential storage format definition outside auth module

## Public Types And Interfaces

- API client
- session client helpers
- RPC request/response helpers

## Data Flow

- CLI auth/session commands call API client
- runtime transport writes server-side session state through this client

## Dependencies

- `auth`
- `transport`
- `vibe-wire`
- `reqwest`

## Implementation Steps

1. Port Happy API client surface needed by runtime and daemon flows.
2. Reuse `vibe-wire` containers directly for payload shape.
3. Add error mapping aligned with agent and server plans.
4. Add mocked HTTP tests.

## Edge Cases And Failure Modes

- expired auth while runtime is active
- reconnect and retry decisions drifting across commands

## Tests

- session create/send tests
- auth error mapping tests
- machine RPC request tests

## Acceptance Criteria

- CLI runtime can talk to Vibe server without ad hoc HTTP code elsewhere

## Open Questions

- None.

## Locked Decisions

- use `reqwest`
- keep API code centralized under one module tree
