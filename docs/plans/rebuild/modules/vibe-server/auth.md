# Module Plan: vibe-server/auth

## Purpose

Implement account, token, and request authentication behavior required by app, agent, and CLI
clients.

## Happy Source Of Truth

- `packages/happy-server/sources/app/auth/auth.ts`
- auth-related flows referenced by `packages/happy-agent/src/auth.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/auth/mod.rs`
  - `src/auth/account.rs`
  - `src/auth/token.rs`
  - `src/auth/middleware.rs`

## Responsibilities

- issue and validate tokens
- support QR/account request-response flow
- expose auth middleware for HTTP and socket layers
- normalize auth failures into stable API errors

## Non-Goals

- client-side credential storage
- provider-level runtime authentication

## Public Types And Interfaces

- auth service
- account request DTOs
- bearer auth middleware
- socket auth validator

## Data Flow

- account request creates pending auth challenge
- app approves and encrypts secret response
- client polls and receives auth material
- bearer token gates subsequent API and socket access

## Dependencies

- `storage-db`
- `storage-redis`
- `protocol-auth-crypto`
- `axum`

## Implementation Steps

1. Implement pending account-link request lifecycle.
2. Validate and mint bearer tokens.
3. Add reusable request guards for HTTP and socket layers.
4. Persist or cache auth state using DB/Redis as planned.
5. Add end-to-end tests with simulated account approval.

## Edge Cases And Failure Modes

- expired or replayed account-link requests
- malformed encrypted auth response
- token expiry and refresh edge cases

## Tests

- account request lifecycle test
- bearer auth middleware test
- expired token rejection test
- invalid auth response test

## Acceptance Criteria

- app and agent auth flows can complete against the server
- protected routes and sockets enforce consistent auth behavior

## Open Questions

- None.

## Locked Decisions

- HTTP auth is bearer-token based
- account-linking flow must stay compatible with Happy semantics
- auth context is injected centrally, not re-parsed in handlers
