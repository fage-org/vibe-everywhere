# Module Plan: vibe-cli/agent-core

## Purpose

Implement provider-agnostic runtime abstractions for the Vibe CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/agent/core/*`
- `packages/happy-cli/src/agent/index.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/agent/core.rs`
  - `src/agent/registry.rs`

## Responsibilities

- define provider backend traits
- define runtime message abstractions
- register supported providers

## Non-Goals

- provider-specific process handling

## Public Types And Interfaces

- backend trait
- agent message enum
- provider registry

## Data Flow

- CLI selects provider
- registry resolves backend
- backend emits provider-agnostic runtime events

## Dependencies

- none beyond crate-local core types and runtime primitives

## Implementation Steps

1. Port Happy core abstractions into Rust traits and enums.
2. Keep provider registry explicit and compile-time discoverable.
3. Add tests for registry resolution and core trait contracts.

## Edge Cases And Failure Modes

- backend selection drift from Happy naming
- runtime event model too narrow for provider adapters

## Tests

- registry lookup tests
- trait contract tests with fake backend

## Acceptance Criteria

- provider runtimes depend on one stable core abstraction layer

## Open Questions

- None.

## Locked Decisions

- use trait-based backend abstraction
- avoid dynamic plugin loading during parity phase
- keep the core backend model transport-agnostic so mapper and transport layers can evolve after the
  registry shape is stable
