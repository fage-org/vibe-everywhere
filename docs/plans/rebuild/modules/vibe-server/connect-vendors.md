# Module Plan: vibe-server/connect-vendors

## Purpose

Implement non-session integration connect flows exposed under `/v1/connect/*`, including generic
vendor token/register/delete routes and any shared OAuth plumbing not owned by the GitHub module.

## Happy Source Of Truth

- `packages/happy-server/sources/app/api/routes/connectRoutes.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/api/connect.rs`
  - shared integration helpers under `src/api/connect_helpers.rs` if needed

## Responsibilities

- serve generic vendor connect endpoints:
  - `POST /v1/connect/:vendor/register`
  - `GET /v1/connect/:vendor/token`
  - `DELETE /v1/connect/:vendor`
  - `GET /v1/connect/tokens`
- own route-level validation and vendor token storage behavior
- share OAuth/token plumbing with `github` where appropriate without making GitHub-specific logic
  the only implementation path

## Non-Goals

- GitHub-specific callback/webhook/profile behavior owned by `github`
- provider runtime auth owned by `vibe-cli`

## Public Types And Interfaces

- connect route DTOs
- vendor token persistence interface
- connect service methods

## Data Flow

- authenticated client requests vendor registration or token access
- handler validates the vendor and delegates to connect service
- tokens are persisted or revoked and returned in Happy-compatible shapes

## Dependencies

- `auth`
- `storage-db`

## Implementation Steps

1. Lock the `/v1/connect/*` route set from the shared API spec.
2. Separate GitHub-specific routes from generic vendor routes in the Rust implementation.
3. Implement token list/register/delete semantics.
4. Add tests for vendor registration, token retrieval, and revocation.

## Edge Cases And Failure Modes

- unsupported vendor name
- revoked or expired third-party token
- multiple active tokens for the same vendor/account

## Tests

- vendor register test
- vendor token fetch test
- vendor disconnect test
- connect tokens list test

## Acceptance Criteria

- all generic `/v1/connect/*` routes have an owning module plan
- GitHub-specific logic and generic vendor logic are not conflated in implementation

## Open Questions

- None.

## Locked Decisions

- generic vendor connect routes are a separate module boundary from GitHub-specific callback/webhook
  logic
- generic vendor token/register/disconnect flows must be implementable before GitHub-specific OAuth
  callback and webhook behavior is finalized
