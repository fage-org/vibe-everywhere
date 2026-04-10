# Module Plan: vibe-cli/claude-runtime

## Purpose

Implement the Claude runtime path for the local CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/claude/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/providers/claude.rs`

## Responsibilities

- launch and control Claude runtime
- parse Claude output/events
- feed normalized runtime events into transport

## Non-Goals

- Codex/Gemini/OpenClaw behavior

## Public Types And Interfaces

- Claude runtime backend
- Claude-specific config and helpers

## Data Flow

- CLI selects Claude provider
- runtime launches Claude process or SDK path
- events are normalized and mapped to transport/session outputs

## Dependencies

- `agent-core`
- `agent-adapters`
- `transport`
- `session-protocol-mapper`

## Implementation Steps

1. Port Claude startup and stream handling flows.
2. Port Claude-specific helpers needed for settings, permissions, and session discovery.
3. Add tests for message conversion and local session handling.

## Edge Cases And Failure Modes

- local session reuse mismatch
- permission mode handling drift
- Claude process startup failures

## Tests

- runtime stream mapping tests
- permission handling tests
- session reuse tests

## Acceptance Criteria

- Claude runtime can produce compatible server/app-visible outputs

## Open Questions

- None.

## Locked Decisions

- keep Claude-specific helpers inside the provider tree
- no shared provider hacks in core transport
