# Module Plan: vibe-cli/session-protocol-mapper

## Purpose

Implement the mapping layer from internal runtime events to session-protocol and legacy wire output.

## Happy Source Of Truth

- `packages/happy-cli/src/codex/utils/sessionProtocolMapper.ts`
- `packages/happy-cli/src/claude/utils/sessionProtocolMapper.ts`
- related provider reasoning/diff processors

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- file: `src/session_protocol_mapper.rs`

## Responsibilities

- map normalized runtime events into `vibe-wire` envelopes or legacy messages
- preserve turn lifecycle, tool call lifecycle, and service messages
- centralize provider-to-wire translation rules

## Non-Goals

- provider-native parsing
- transport buffering

## Public Types And Interfaces

- mapping functions or service
- mapper input event enum

## Data Flow

- provider runtime emits normalized event
- mapper decides legacy vs session-protocol representation
- transport publishes `vibe-wire` payloads

## Dependencies

- `agent-adapters`
- `vibe-wire`

## Implementation Steps

1. Define mapper input model shared by provider runtimes.
2. Port Happy mapping rules for tool calls, thinking, file events, and turn lifecycle.
3. Add fixture-based tests for each mapped wire variant.

## Edge Cases And Failure Modes

- turn start/end ordering drift
- unmatched tool-call end
- provider emits data that belongs in service vs text message

## Tests

- per-event mapping tests
- invalid sequence tests
- provider-specific fixture comparisons

## Acceptance Criteria

- all runtime outputs reach the wire layer through one mapping module

## Open Questions

- None.

## Locked Decisions

- this module is the only place that chooses session protocol vs legacy output shape
- output types come directly from `vibe-wire`
