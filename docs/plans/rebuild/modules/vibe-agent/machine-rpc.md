# Module Plan: vibe-agent/machine-rpc

## Purpose

Implement machine-scoped RPC behavior for listing machines, targeting a machine, and requesting
remote session control actions beyond the plain session APIs.

## Happy Source Of Truth

- `packages/happy-agent/src/machineRpc.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-agent`
- file: `src/machine_rpc.rs`

## Responsibilities

- model machine RPC requests and responses
- send machine-targeted control commands
- bridge machine control flows to CLI surface

## Non-Goals

- local provider execution

## Public Types And Interfaces

- machine RPC client
- typed machine request/response enums

## Data Flow

- operator chooses a machine
- agent issues RPC request over the planned transport
- machine/runtime path executes action and server returns state/result

## Dependencies

- `http-api-client`
- `session-socket-client`
- `protocol-api-rpc`

## Implementation Steps

1. Port Happy RPC verbs and payloads.
2. Define typed request/response models.
3. Implement transport client and retries if Happy behavior requires them.
4. Add tests against a mocked RPC endpoint.

## Edge Cases And Failure Modes

- target machine offline
- stale machine identity
- RPC result timeout

## Tests

- request serialization tests
- machine offline error test
- success path test

## Acceptance Criteria

- remote machine control behaviors needed by Happy parity are callable from `vibe-agent`

## Open Questions

- None.

## Locked Decisions

- machine RPC surface is distinct from plain session REST APIs
- transport details must follow the shared RPC plan, not ad hoc client invention
