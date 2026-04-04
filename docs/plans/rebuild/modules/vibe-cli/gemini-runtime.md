# Module Plan: vibe-cli/gemini-runtime

## Purpose

Implement the Gemini runtime path for the local CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/gemini/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/providers/gemini/mod.rs`
  - `src/providers/gemini/runtime.rs`

## Responsibilities

- launch Gemini runtime
- parse Gemini output and reasoning
- apply permission and options parsing rules

## Non-Goals

- Claude or Codex behavior

## Public Types And Interfaces

- Gemini runtime backend
- Gemini config/options helpers

## Data Flow

- Gemini runtime emits provider events
- adapters normalize events
- transport and mapper publish compatible updates

## Dependencies

- `agent-core`
- `agent-adapters`
- `transport`

## Implementation Steps

1. Port Gemini startup and option parsing.
2. Port reasoning and diff helpers.
3. Add tests for event normalization and permission rules.

## Edge Cases And Failure Modes

- option parsing drift
- reasoning stream fragmentation

## Tests

- options parsing tests
- reasoning mapping tests
- runtime smoke test

## Acceptance Criteria

- Gemini runtime behaves compatibly with Happy on core flows

## Open Questions

- None.

## Locked Decisions

- keep Gemini-specific config parsing local to the provider module
- Wave 5 may use a wrapper-backed runtime adapter while preserving the Gemini-owned module
  boundary; deeper provider-native parity is deferred until after the compatibility gate is green
