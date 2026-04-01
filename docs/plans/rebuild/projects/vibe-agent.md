# Project Plan: vibe-agent

## Purpose

`vibe-agent` is the Rust remote-control client replacing Happy agent-control CLI behavior. It does
not run local providers; it controls machines and sessions remotely.

## Happy Source

- primary source: `packages/happy-agent`
- supporting sources: `packages/happy-wire`, `packages/happy-server`

## Target Layout

- crate: `crates/vibe-agent`
- expected modules:
  - `config`
  - `credentials`
  - `auth`
  - `encryption`
  - `api`
  - `session`
  - `machine_rpc`
  - `output`
  - `main`

## Public Interfaces

- binary: `vibe-agent`
- subcommands for auth, list, status, create, send, history, stop, wait, and machine control
- local credential storage under `~/.vibe`

## Internal Module Map

- `config`: env and path resolution
- `credentials`: read/write/require credential flows
- `auth`: QR-based account linking and status/logout
- `encryption`: crypto helpers required by auth and record decryption
- `api`: REST client for sessions and machines
- `session`: socket update client and idle detection
- `machine_rpc`: remote machine control RPC
- `output`: human-readable and JSON formatting

## Implementation Order

1. config and credentials
2. encryption and auth
3. API client
4. session socket client
5. machine RPC
6. command output and CLI wiring

## Compatibility Requirements

- command behavior must mirror Happy agent semantics
- encrypted records and wait logic must be wire-compatible with Happy
- user-visible names use `vibe`, not `happy`

## Testing Strategy

- unit tests for config, credentials, encryption, and output
- mocked REST and socket tests
- end-to-end tests against a real `vibe-server`

## Acceptance Criteria

- real account auth works
- remote sessions can be listed, created, controlled, and monitored
- output is stable in both human and JSON modes

## Deferred Items

- any local provider runtime features
- speculative code sharing with `vibe-cli` beyond documented shared helpers
