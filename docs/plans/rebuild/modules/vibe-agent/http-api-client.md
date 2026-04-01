# Module Plan: vibe-agent/http-api-client

## Purpose

Implement the authenticated REST client for session and machine operations.

## Happy Source Of Truth

- `packages/happy-agent/src/api.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-agent`
- file: `src/api.rs`

## Responsibilities

- list sessions
- list active sessions
- list machines
- create sessions
- fetch history
- delete/archive sessions
- resolve record encryption variant

## Non-Goals

- live socket updates

## Public Types And Interfaces

- REST client
- decrypted session and machine models
- encryption-resolution helpers

## Data Flow

- credentials provide bearer token and content keys
- client fetches raw encrypted records
- client resolves encryption mode and decrypts metadata/state
- caller receives typed decrypted projections

## Dependencies

- `credentials-and-auth`
- `encryption`
- `reqwest`
- `vibe-wire`

## Implementation Steps

1. Port raw session and machine wire projections.
2. Implement encryption resolution rules for legacy and dataKey records.
3. Add typed REST calls with consistent error mapping.
4. Add mocked server tests for each endpoint and error family.

## Edge Cases And Failure Modes

- dataEncryptionKey present but undecryptable
- 401/403/404 mapping inconsistencies
- partial record decryption failures

## Tests

- list/create/history/delete tests
- machine listing tests
- encryption-resolution tests
- error mapping tests

## Acceptance Criteria

- agent can fetch and decrypt remote session and machine state over HTTP

## Open Questions

- None.

## Locked Decisions

- use `reqwest` for HTTP
- return typed decrypted projections rather than exposing raw JSON to callers
