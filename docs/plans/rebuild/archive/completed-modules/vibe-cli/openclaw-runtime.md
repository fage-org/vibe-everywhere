# Module Plan: vibe-cli/openclaw-runtime

## Purpose

Implement the OpenClaw runtime path for the local CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/openclaw/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/providers/openclaw/mod.rs`
  - `src/providers/openclaw/runtime.rs`

## Responsibilities

- launch OpenClaw runtime
- manage OpenClaw socket/auth path
- normalize runtime output

## Non-Goals

- other provider behavior

## Public Types And Interfaces

- OpenClaw runtime backend
- OpenClaw auth/session helpers

## Data Flow

- runtime authenticates with OpenClaw
- socket/events are normalized
- transport forwards updates to the server session

## Dependencies

- `agent-core`
- `agent-adapters`
- `transport`

## Implementation Steps

1. Port OpenClaw auth and socket handling.
2. Port output normalization.
3. Add provider-specific tests.

## Edge Cases And Failure Modes

- auth handshake failure
- socket reconnect issues

## Tests

- auth tests
- output normalization tests
- integration smoke test

## Acceptance Criteria

- OpenClaw runtime is available behind the same core transport path

## Open Questions

- None.

## Locked Decisions

- provider-specific socket/auth logic remains isolated from shared transport
- Wave 5 may use a wrapper-backed runtime adapter while preserving the OpenClaw-owned module
  boundary; deeper provider-native parity is deferred until after the compatibility gate is green
