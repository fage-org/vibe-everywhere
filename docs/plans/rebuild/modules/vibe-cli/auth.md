# Module Plan: vibe-cli/auth

## Purpose

Implement CLI-side authentication and connect flows for local runtime usage.

## Happy Source Of Truth

- `packages/happy-cli/src/api/auth.ts`
- `packages/happy-cli/src/api/webAuth.ts`
- `packages/happy-cli/src/commands/connect/*`
- `packages/happy-cli/src/ui/auth.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/auth/mod.rs`
  - `src/auth/connect.rs`
  - `src/auth/storage.rs`

## Responsibilities

- authenticate CLI user/device
- store CLI credentials under `~/.vibe`
- implement connect flows for supported providers if required by Happy parity

## Non-Goals

- remote agent account-linking flow, which belongs to `vibe-agent`

## Public Types And Interfaces

- auth service
- auth storage helpers
- connect command handlers

## Data Flow

- user authenticates or connects provider
- credentials are stored locally
- API client uses stored auth material

## Dependencies

- `ui-terminal`
- `directories`

## Implementation Steps

1. Port Happy CLI auth and connect entrypoints.
2. Separate local credential storage from remote agent credential storage.
3. Add tests for auth status, storage, and connect argument handling.

## Edge Cases And Failure Modes

- concurrent auth attempts
- stale provider connection state

## Tests

- credential storage tests
- auth command tests
- connect flow parsing tests

## Acceptance Criteria

- CLI auth path is functional and isolated from `vibe-agent`

## Open Questions

- None.

## Locked Decisions

- CLI auth state uses `~/.vibe/access.key`
- provider-connect flows stay under the CLI auth tree
- the shared API client consumes auth state after this module lands; auth does not depend on the API
  client to define its storage and connect semantics
