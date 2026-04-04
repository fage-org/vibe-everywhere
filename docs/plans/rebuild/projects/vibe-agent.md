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

## Wave 3 Feature Checklist

- typed config loading from env and default `~/.vibe` paths
- account-link auth flow with QR/deep link rendering, polling, credential persistence, logout, and status
- shared crypto helpers for base64/base64url, key derivation, legacy secretbox, data-key AES bundles, and public-key box bundles
- authenticated HTTP client for session list/active/create/history/delete and machine list/detail access
- record decryption for legacy and `dataKey` session/machine payloads
- live session Socket.IO client for update handling, send-message, stop, wait-for-idle, and turn-completion behavior
- machine RPC client for spawn/resume flows over the shared socket RPC contract
- stable human-readable and `--json` CLI output for every planned command
- crate-level tests covering config, credentials, crypto, HTTP mapping, socket behavior, and CLI parsing/output

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

1. config
2. encryption
3. credentials and auth
4. HTTP API client
5. session socket client
6. machine RPC
7. command output and CLI wiring

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

## Status

- Wave 3 implementation is complete
- module set matches the planned target layout
- command surface covers auth, session list/status/create/send/history/stop/wait, and machine
  spawn/resume control

## Validation Status

- `cargo check --workspace`
- `cargo test -p vibe-agent`
- real `vibe-server` integration coverage exercises auth persistence, session CRUD/control, lossy
  decrypt handling, and idle waiting
- mocked socket/CLI coverage exercises `vibe-agent` machine RPC spawn/resume flows
- `vibe-server` socket-layer tests exercise RPC register/call/unregister transport behavior

## Deferred Items

- any local provider runtime features
- speculative code sharing with `vibe-cli` beyond documented shared helpers
