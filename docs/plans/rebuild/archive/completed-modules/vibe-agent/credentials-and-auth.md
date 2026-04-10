# Module Plan: vibe-agent/credentials-and-auth

## Purpose

Implement local credential storage and the QR/account authentication flow for `vibe-agent`.

## Happy Source Of Truth

- `packages/happy-agent/src/credentials.ts`
- `packages/happy-agent/src/auth.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-agent`
- files:
  - `src/credentials.rs`
  - `src/auth.rs`

## Responsibilities

- read/write/clear agent credentials
- perform account-linking request flow
- display QR code
- persist token and secret locally

## Non-Goals

- session control
- machine RPC

## Public Types And Interfaces

- `Credentials`
- credential repository helpers
- auth service and CLI-facing commands

## Data Flow

- config resolves credential path
- auth creates pending account-link request
- user approves via app
- encrypted secret is decrypted and stored

## Dependencies

- `config`
- `encryption`
- `reqwest`
- `qrcode`

## Implementation Steps

1. Implement credential file model under `~/.vibe/agent.key`.
2. Implement account-link request, poll, and success handling.
3. Expose auth subcommands for login, logout, and status.
4. Add end-to-end tests with mocked server responses.

## Edge Cases And Failure Modes

- missing credential file
- auth timeout
- malformed auth response bundle

## Tests

- credential round-trip test
- login success test
- login timeout test
- logout/status tests

## Acceptance Criteria

- `vibe-agent` can authenticate and persist usable credentials

## Open Questions

- None.

## Locked Decisions

- QR auth flow mirrors Happy semantics
- credential file format stores `token` and base64 `secret`
