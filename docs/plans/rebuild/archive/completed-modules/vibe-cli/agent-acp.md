# Module Plan: vibe-cli/agent-acp

## Purpose

Implement ACP-specific runtime support and session management used by the CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/agent/acp/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/agent/acp/mod.rs`
  - `src/agent/acp/session_manager.rs`
  - `src/agent/acp/backend.rs`

## Responsibilities

- ACP backend implementation
- ACP session lifecycle handling
- ACP config metadata extraction

## Non-Goals

- non-ACP provider execution

## Public Types And Interfaces

- ACP backend
- ACP session manager
- ACP config metadata helpers

## Data Flow

- CLI launches ACP backend
- ACP session manager tracks session state and emits internal runtime events
- mapper forwards compatible updates to server/app surfaces

## Dependencies

- `agent-core`
- `transport`
- `session-protocol-mapper`

## Implementation Steps

1. Port Happy ACP session manager behavior.
2. Implement ACP backend lifecycle and config metadata extraction.
3. Add tests for ACP run and state update handling.

## Edge Cases And Failure Modes

- ACP session recovery
- malformed ACP config metadata

## Tests

- session manager tests
- config metadata tests
- ACP run smoke test

## Acceptance Criteria

- ACP flows available in CLI without special-case hacks outside this module

## Open Questions

- None.

## Locked Decisions

- ACP remains a first-class provider path
- ACP state handling stays isolated from other provider runtimes
- Wave 5 may route ACP through a wrapper-backed runtime while preserving ACP-owned session
  management boundaries; deeper ACP-native parity is deferred until after the compatibility gate is
  green
