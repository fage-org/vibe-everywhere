# Module Plan: vibe-server/versions-and-config

## Purpose

Implement server configuration, version reporting, and bootstrap-time environment handling.

## Happy Source Of Truth

- `packages/happy-server/sources/versions.ts`
- `packages/happy-server/sources/context.ts`
- server startup files `main.ts` and `standalone.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/config.rs`
  - `src/version.rs`
  - `src/context.rs`
  - `src/main.rs`

## Responsibilities

- load environment configuration
- expose version/build metadata
- wire application state for the rest of the service

## Non-Goals

- domain-specific config buried in unrelated modules

## Public Types And Interfaces

- `Config`
- `AppContext`
- version metadata helpers

## Data Flow

- process starts
- configuration is read from env and flags
- shared app context is constructed and injected into routers/services

## Dependencies

- `clap`
- `serde`
- all core server subsystems via bootstrap wiring

## Implementation Steps

1. Define strongly typed config struct with Vibe env names.
2. Expose version/build metadata helpers.
3. Build shared `AppContext`.
4. Add startup validation for required config.
5. Add config parsing tests.

## Edge Cases And Failure Modes

- missing required env vars
- inconsistent URLs/ports
- partial startup where some subsystems initialize and others fail

## Tests

- env parsing tests
- version helper test
- startup validation failure test

## Follow-Up Status

- Wave 2 validation hardening completed for config parsing, including automated coverage for
  required-secret and invalid-host failures.

## Acceptance Criteria

- server can start from a single typed config path
- version metadata is consistently available

## Open Questions

- None.

## Locked Decisions

- config is env-first with typed parsing
- no ad hoc module-local env parsing is allowed
