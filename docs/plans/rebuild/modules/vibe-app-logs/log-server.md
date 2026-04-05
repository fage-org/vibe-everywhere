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
- preserve the imported app's existing `/logs` JSON payload contract used by remote console
  forwarding
- write log lines to a timestamped file under the Vibe home directory

## Non-Goals

- general observability platform work

## Public Types And Interfaces

- sidecar server bootstrap
- config struct
- `POST /logs`
- `OPTIONS /logs`

## Data Flow

- app tooling sends log payloads to sidecar
- sidecar stores, prints, or forwards them per parity behavior

Current app-owned proof that the sidecar is still required:

- `packages/vibe-app/sources/utils/consoleLogging.ts`
- `packages/vibe-app/sources/app/(app)/dev/index.tsx`

## Dependencies

- `axum`
- `tokio`
- `clap`

## Implementation Steps

1. Port Happy log server surface.
2. Implement typed config and startup.
3. Re-home storage/output paths under Vibe naming and add a root helper command for the app docs.
4. Add startup and ingestion smoke tests.

## Edge Cases And Failure Modes

- port already in use
- invalid log payload format
- missing or unwritable Vibe home directory path

## Tests

- startup test
- ingestion test
- root command smoke path if added

## Acceptance Criteria

- app tooling requirements for log-sidecar behavior are satisfied

## Open Questions

- None.

## Locked Decisions

- keep the sidecar minimal and app-driven
- keep the HTTP surface compatible with the imported app (`POST /logs` with permissive CORS)
- default logs path is `~/.vibe/app-logs/`, overridable via `VIBE_HOME_DIR`
- default port is `8787`; `VIBE_APP_LOGS_PORT` is the preferred override, while `PORT` is retained
  only as a Happy-compatible fallback and must be reflected in app-facing guidance
