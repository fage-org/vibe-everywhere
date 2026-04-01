# Module Plan: vibe-app-logs/log-server

## Purpose

Implement the sidecar log server required by imported app tooling.

## Happy Source Of Truth

- `packages/happy-app-logs/src/server.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-app-logs`
- files:
  - `src/server.rs`
  - `src/config.rs`
  - `src/main.rs`

## Responsibilities

- start log-sidecar server
- expose any ingestion endpoints required by the app
- format or forward logs as needed by parity flows

## Non-Goals

- general observability platform work

## Public Types And Interfaces

- sidecar server bootstrap
- config struct

## Data Flow

- app tooling sends log payloads to sidecar
- sidecar stores, prints, or forwards them per parity behavior

## Dependencies

- `axum`
- `tokio`

## Implementation Steps

1. Port Happy log server surface.
2. Implement typed config and startup.
3. Add startup and ingestion smoke tests.

## Edge Cases And Failure Modes

- port already in use
- invalid log payload format

## Tests

- startup test
- ingestion test

## Acceptance Criteria

- app tooling requirements for log-sidecar behavior are satisfied

## Open Questions

- None.

## Locked Decisions

- keep the sidecar minimal and app-driven
