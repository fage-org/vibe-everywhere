# Module Plan: vibe-agent/config

## Purpose

Implement typed configuration loading for the remote-control client.

## Happy Source Of Truth

- `packages/happy-agent/src/config.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-agent`
- file: `src/config.rs`

## Responsibilities

- resolve server URL
- resolve home/config directory paths
- derive credential file paths

## Non-Goals

- command parsing
- credential file I/O

## Public Types And Interfaces

- `Config`
- path helper methods for agent credentials and related files

## Data Flow

- binary startup loads config from env/defaults
- auth, credentials, API, and session modules consume typed config

## Dependencies

- `directories`
- `serde` if config serialization is needed

## Implementation Steps

1. Port Happy config fields with Vibe names.
2. Use `~/.vibe` as the default home directory.
3. Define typed helpers for agent credential file path.
4. Add env override tests.

## Edge Cases And Failure Modes

- missing home directory resolution
- invalid server URL format

## Tests

- default config test
- env override test
- path derivation test

## Acceptance Criteria

- all downstream agent modules load config from one typed source

## Open Questions

- None.

## Locked Decisions

- env-first config loading
- no module-local path conventions outside this file
