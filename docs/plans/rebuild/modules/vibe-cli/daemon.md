# Module Plan: vibe-cli/daemon

## Purpose

Implement the local daemon control plane used by the CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/daemon/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/daemon.rs`

## Responsibilities

- start and stop daemon
- expose local control API
- manage install/uninstall and status flows
- coordinate long-running runtime processes

## Non-Goals

- provider-specific execution logic

## Public Types And Interfaces

- daemon service
- control client
- daemon command handlers

## Data Flow

- CLI command talks to local daemon client
- daemon starts or manages runtime processes
- daemon exposes status to CLI UI

## Dependencies

- `agent-core`

## Implementation Steps

1. Port daemon command surface and lifecycle model.
2. Implement local control channel using Unix socket or localhost HTTP.
3. Add start/stop/status/install flows.
4. Add daemon integration tests.

## Edge Cases And Failure Modes

- stale daemon PID/socket files
- mismatched daemon version and CLI binary
- crash recovery

## Tests

- daemon start/stop/status tests
- crash recovery test
- install/uninstall test

## Acceptance Criteria

- local daemon lifecycle is functional and test-covered

## Open Questions

- None.

## Locked Decisions

- daemon communicates over a local control channel, not stdout parsing
- installation helpers remain non-interactive by default
- sandbox and persistence integrations plug into this control plane after the daemon surface is
  stable; they are not prerequisites for defining it
