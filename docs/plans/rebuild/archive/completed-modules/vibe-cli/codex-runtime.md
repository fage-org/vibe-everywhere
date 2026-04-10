# Module Plan: vibe-cli/codex-runtime

## Purpose

Implement the Codex runtime path for the local CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/codex/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/providers/codex/mod.rs`
  - `src/providers/codex/runtime.rs`
  - provider-specific helpers under `src/providers/codex/`

## Responsibilities

- launch Codex runtime
- parse Codex MCP/output events
- handle permission and execution policy behavior
- emit compatible session or legacy message flows

## Non-Goals

- non-Codex provider behavior

## Public Types And Interfaces

- Codex runtime backend
- execution policy helpers
- resume helpers specific to Codex if needed

## Data Flow

- Codex produces MCP or structured events
- provider utilities normalize tool calls, reasoning, and lifecycle
- mapper emits session-protocol or legacy-compatible messages

## Dependencies

- `agent-core`
- `agent-adapters`
- `session-protocol-mapper`
- `sandbox`

## Implementation Steps

1. Port Codex launch and output stream handling.
2. Port permission and execution policy helpers.
3. Port reasoning and diff processing required for compatible UI output.
4. Add tests for event-to-wire mapping.

## Edge Cases And Failure Modes

- incomplete tool-call pairing
- reasoning delta ordering
- resume-existing-thread mismatch

## Tests

- execution policy tests
- session protocol mapper tests
- resume behavior tests

## Acceptance Criteria

- Codex runtime emits compatible session output and can resume correctly

## Open Questions

- None.

## Locked Decisions

- Codex tool/reasoning handling must map through the dedicated mapper module
- permission handling stays provider-local with shared base traits only where justified
