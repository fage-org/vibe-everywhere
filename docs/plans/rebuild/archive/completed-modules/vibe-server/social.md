# Module Plan: vibe-server/social

## Purpose

Implement relationship, friend, and username social APIs expected by the imported app.

## Happy Source Of Truth

- `packages/happy-server/sources/app/social/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/api/social.rs`
  - `src/api/types.rs`

## Responsibilities

- list/add/remove friends
- get/set relationships
- update usernames
- emit any required notifications

## Non-Goals

- redesigning social features

## Public Types And Interfaces

- social DTOs
- social service
- notification hooks if required

## Data Flow

- app issues social requests
- server validates auth and relationship rules
- data is persisted and returned in Happy-compatible format

## Dependencies

- `storage-db`
- `app-api`

## Implementation Steps

1. Port social route set and DTOs.
2. Implement relationship persistence and lookups.
3. Add notification hooks only where Happy behavior requires them.
4. Add integration tests for friend and username flows.

## Edge Cases And Failure Modes

- duplicate friend relationships
- self-targeting operations
- username uniqueness conflicts

## Tests

- friend add/remove/list tests
- relationship set/get tests
- username update conflict test

## Acceptance Criteria

- imported app social flows run without shape drift

## Open Questions

- None.

## Locked Decisions

- preserve Happy endpoint semantics first
- keep notifications behind explicit service hooks
- phase-one implementation keeps social handlers and service logic inside the shared API tree rather
  than forcing a standalone `social/` module split
