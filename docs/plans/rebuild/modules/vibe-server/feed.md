# Module Plan: vibe-server/feed

## Purpose

Implement Happy-parity feed endpoints used by the imported app.

## Happy Source Of Truth

- `packages/happy-server/sources/app/feed/feedGet.ts`
- `packages/happy-server/sources/app/feed/feedPost.ts`
- `packages/happy-server/sources/app/feed/types.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/api/feed.rs`
  - `src/api/types.rs`

## Responsibilities

- expose feed read and write APIs
- keep response typing aligned with app needs

## Non-Goals

- redesigning feed product behavior

## Public Types And Interfaces

- feed DTOs
- feed service interface
- route handlers

## Data Flow

- app calls feed endpoints
- server loads or persists feed entries
- response uses Happy-compatible JSON shape

## Dependencies

- `storage-db`
- `app-api`

## Implementation Steps

1. Port Happy feed DTOs and endpoints.
2. Implement persistence queries.
3. Add app-facing integration tests using imported client expectations.

## Edge Cases And Failure Modes

- auth gating on feed mutations
- pagination or ordering mismatches with app expectations

## Tests

- feed get test
- feed post test
- auth failure test

## Acceptance Criteria

- imported app feed features can operate without API shape changes

## Open Questions

- None for planning baseline.

## Locked Decisions

- do not expand feed semantics during parity phase
- phase-one implementation keeps feed handlers and service logic inside the shared API tree rather
  than forcing a standalone `feed/` module split
