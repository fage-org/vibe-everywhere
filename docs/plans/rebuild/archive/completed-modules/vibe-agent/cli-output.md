# Module Plan: vibe-agent/cli-output

## Purpose

Implement the human-readable and JSON output layer for `vibe-agent`.

## Happy Source Of Truth

- `packages/happy-agent/src/output.ts`
- `packages/happy-agent/src/index.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-agent`
- files:
  - `src/output.rs`
  - `src/main.rs`

## Responsibilities

- parse CLI commands
- render human-readable output
- render `--json` output
- keep command UX stable

## Non-Goals

- auth/session business logic

## Public Types And Interfaces

- CLI command enum
- output renderer helpers

## Data Flow

- command parser resolves subcommand
- handler returns typed result
- output layer formats result according to flags

## Dependencies

- `clap`
- all agent service modules

## Implementation Steps

1. Define CLI command tree matching Happy agent behavior.
2. Implement shared output formatting helpers.
3. Keep JSON mode machine-readable and stable.
4. Add CLI parsing and rendering tests.

## Edge Cases And Failure Modes

- mismatched exit codes
- human and JSON output diverging semantically

## Tests

- CLI parsing tests
- human output snapshot tests
- JSON output tests

## Acceptance Criteria

- every planned `vibe-agent` command has stable output and parsing behavior

## Open Questions

- None.

## Locked Decisions

- use `clap` for CLI parsing
- JSON output is a first-class compatibility surface, not debug-only output
