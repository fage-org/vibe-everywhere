# Module Plan: vibe-server/github

## Purpose

Implement GitHub connect, disconnect, and profile retrieval behavior used by the app.

## Happy Source Of Truth

- `packages/happy-server/sources/app/github/githubConnect.ts`
- `packages/happy-server/sources/app/github/githubDisconnect.ts`
- `packages/happy-server/sources/app/api/types.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/github/mod.rs`
  - `src/github/http.rs`
  - `src/github/types.rs`

## Responsibilities

- connect GitHub account
- disconnect GitHub account
- expose GitHub profile data in Happy-compatible shape

## Non-Goals

- generalized OAuth provider framework beyond GitHub parity

## Public Types And Interfaces

- GitHub profile DTO
- connect/disconnect handlers
- GitHub integration service

## Data Flow

- app initiates GitHub connect/disconnect
- server performs GitHub API interaction and stores profile linkage
- app receives updated account profile data

## Dependencies

- `auth`
- `storage-db`
- GitHub API client crate

## Implementation Steps

1. Port GitHub profile type shape.
2. Implement connect/disconnect service methods.
3. Store linked account state in DB.
4. Add tests with mocked GitHub responses.

## Edge Cases And Failure Modes

- token exchange failure
- revoked GitHub access
- partial profile data

## Tests

- connect success test
- disconnect success test
- provider error mapping test

## Acceptance Criteria

- app GitHub connect/disconnect flows operate with matching payload shapes

## Open Questions

- None.

## Locked Decisions

- only GitHub integration is in scope for parity phase
