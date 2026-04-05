# Module Plan: vibe-cli/sandbox

## Purpose

Implement local runtime sandboxing and permission policy handling.

## Happy Source Of Truth

- `packages/happy-cli/src/sandbox/*`
- provider permission helpers where they interact with sandbox policy

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/sandbox.rs`

## Responsibilities

- resolve sandbox mode and policy
- enforce file/network/runtime boundaries where supported
- expose sandbox manager API to provider runtimes and daemon

## Non-Goals

- provider-native permission prompts

## Public Types And Interfaces

- sandbox config
- sandbox manager
- sandbox policy enum

## Data Flow

- CLI or daemon selects sandbox mode
- runtime is launched under sandbox constraints
- permission handlers consult sandbox policy

## Dependencies

- `daemon`
- `agent-core`
- OS-specific process isolation crates if needed

## Implementation Steps

1. Port sandbox flags and config model.
2. Implement manager abstraction with a no-op and enforced mode.
3. Wire provider runtimes to request sandbox execution through the manager.
4. Add unit and integration tests for config and manager behavior.

## Edge Cases And Failure Modes

- unsupported sandbox mode on host OS
- network isolation drift from expected behavior

## Tests

- config parsing tests
- manager policy tests
- network/isolation smoke test where supported

## Acceptance Criteria

- sandbox policy is explicit, testable, and reusable across providers

## Open Questions

- None.

## Locked Decisions

- sandbox behavior is mediated by one manager abstraction
- unsupported modes must fail clearly rather than silently degrade
- during Wave 5 parity, workspace sandbox is Claude-only until equivalent enforcement exists for
  the remaining provider paths
