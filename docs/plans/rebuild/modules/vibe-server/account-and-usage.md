# Module Plan: vibe-server/account-and-usage

## Purpose

Implement account profile, account settings, and usage-query APIs required by the imported app.

## Happy Source Of Truth

- `packages/happy-server/sources/app/api/routes/accountRoutes.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/api/account.rs`
  - `src/api/usage.rs`

## Responsibilities

- serve `GET /v1/account/profile`
- serve `GET /v1/account/settings`
- serve `POST /v1/account/settings`
- serve `POST /v1/usage/query`
- keep account/profile/settings payloads compatible with imported app expectations

## Non-Goals

- authentication flows owned by `auth`
- social/friend behavior owned by `social`

## Public Types And Interfaces

- account profile DTOs
- account settings DTOs
- usage query/request-response DTOs
- account and usage service methods

## Data Flow

- authenticated app or client requests account/profile/settings/usage data
- handler validates auth and delegates to account or usage service
- response returns Happy-compatible JSON shapes

## Dependencies

- `auth`
- `storage-db`

## Implementation Steps

1. Lock the exact route set from `shared/protocol-api-rpc.md`.
2. Port account profile and settings DTOs.
3. Implement settings read/write persistence.
4. Implement usage query surface and aggregation rules required by imported clients.
5. Add route and service tests.

## Edge Cases And Failure Modes

- missing account settings rows for newly linked users
- partial/default settings behavior drifting from Happy
- usage query filters returning inconsistent date windows

## Tests

- account profile route test
- account settings get/post tests
- usage query test
- auth gating test

## Acceptance Criteria

- imported app account settings and profile flows work without API shape changes
- usage query responses match the planned route contract

## Open Questions

- None.

## Locked Decisions

- account/profile/settings and usage-query stay grouped because they share account-scoped request
  context and are implemented from the same Happy route module
