# Module Plan: vibe-cli/persistence-resume

## Purpose

Implement local persistence and resume/attach behavior for CLI sessions and daemon-managed state.

## Happy Source Of Truth

- `packages/happy-cli/src/persistence.ts`
- `packages/happy-cli/src/resume/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/persistence.rs`
  - `src/resume.rs`

## Responsibilities

- store local CLI session state
- resolve resumable sessions
- attach runtime flows to existing local or remote sessions

## Non-Goals

- remote session history storage, which belongs to the server

## Public Types And Interfaces

- persistence store
- resume resolver
- persisted session metadata models

## Data Flow

- runtime or daemon writes local session state
- resume command queries persisted state
- resolver attaches to the correct runtime or server session

## Dependencies

- `daemon`
- `api-client`
- `agent-core`

## Implementation Steps

1. Port persistence schema and storage layout.
2. Port resume lookup and selection behavior.
3. Add tests for stale, active, and missing session states.

## Edge Cases And Failure Modes

- stale local state after crash
- multiple candidate sessions for resume
- remote session no longer exists

## Tests

- persistence round-trip tests
- resume resolution tests
- stale-state cleanup tests

## Acceptance Criteria

- CLI can resume or reattach sessions predictably using persisted state

## Open Questions

- None.

## Locked Decisions

- keep persistence and resume coupled in planning because they share the same local state model
- store local state under `~/.vibe`
