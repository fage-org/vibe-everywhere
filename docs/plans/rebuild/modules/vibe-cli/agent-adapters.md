# Module Plan: vibe-cli/agent-adapters

## Purpose

Implement adapter logic that translates provider-native output into the CLI's internal message model.

## Happy Source Of Truth

- `packages/happy-cli/src/agent/adapters/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/agent/adapters/mod.rs`
  - `src/agent/adapters/mobile_message_format.rs`

## Responsibilities

- normalize provider-native messages
- preserve fields needed for app/mobile compatibility

## Non-Goals

- final wire serialization
- provider process launching

## Public Types And Interfaces

- adapter traits or functions
- mobile message format projection

## Data Flow

- provider runtime emits native event
- adapter normalizes to internal message
- downstream mapper decides session-protocol or legacy wire output

## Dependencies

- `agent-core`
- `session-protocol-mapper`

## Implementation Steps

1. Port Happy adapter surfaces.
2. Keep provider-specific normalization local to this module tree.
3. Add tests that compare normalized output against Happy fixtures.

## Edge Cases And Failure Modes

- missing provider fields needed by app rendering
- adapter drift across providers

## Tests

- normalized message snapshot tests
- provider edge-case tests

## Acceptance Criteria

- provider-native outputs can be normalized consistently before wire mapping

## Open Questions

- None.

## Locked Decisions

- adapters normalize to one internal message model before transport mapping
