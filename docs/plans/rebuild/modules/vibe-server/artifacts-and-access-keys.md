# Module Plan: vibe-server/artifacts-and-access-keys

## Purpose

Implement artifact storage/update APIs and session-machine access-key APIs required by imported app
and runtime flows.

## Happy Source Of Truth

- `packages/happy-server/sources/app/api/routes/artifactsRoutes.ts`
- `packages/happy-server/sources/app/api/routes/accessKeysRoutes.ts`
- `packages/happy-server/sources/app/api/socket/artifactUpdateHandler.ts`
- `packages/happy-server/sources/app/api/socket/accessKeyHandler.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/api/artifacts.rs`
  - `src/api/access_keys.rs`
  - `src/api/socket_artifacts.rs`
  - `src/api/socket_access_keys.rs`

## Responsibilities

- serve artifact HTTP routes:
  - list
  - get
  - create
  - update
  - delete
- serve access-key HTTP routes:
  - get
  - create
  - update
- handle socket artifact read/update/create/delete flows
- handle socket `access-key-get` flow
- preserve optimistic-concurrency semantics where Happy uses per-part version checks

## Non-Goals

- generic file/blob storage backends owned by `storage-files`
- session lifecycle logic owned by `session-lifecycle`

## Public Types And Interfaces

- artifact DTOs
- access-key DTOs
- artifact service
- access-key service
- socket ack/result DTOs

## Data Flow

- authenticated client reads or writes artifact/access-key data over HTTP or socket
- service validates account ownership and session/machine relationships
- storage persists encrypted or opaque blobs plus version counters
- event router emits updates where Happy behavior requires it

## Dependencies

- `auth`
- `storage-db`
- `storage-files`
- `event-router`
- `session-lifecycle`
- `machine-lifecycle`

## Implementation Steps

1. Lock HTTP and socket route/event shapes from `shared/protocol-api-rpc.md`.
2. Implement artifact read/write/delete semantics with version-aware header/body updates.
3. Implement access-key CRUD semantics for `(accountId, sessionId, machineId)`.
4. Return exact Happy-style success/error/version-mismatch ack shapes for socket flows.
5. Add integration tests covering both HTTP and socket entrypoints.

## Edge Cases And Failure Modes

- header version matches but body version does not
- artifact or access key exists but belongs to another account
- socket and HTTP updates racing on the same artifact
- missing session or machine on access-key lookup

## Tests

- artifact list/get/create/update/delete tests
- artifact socket read/update tests
- access-key HTTP get/create/update tests
- access-key socket get test
- version-mismatch tests for artifact header/body updates

## Acceptance Criteria

- artifact and access-key surfaces have an explicit owning module plan
- HTTP and socket result shapes stay aligned for imported client behavior

## Open Questions

- None.

## Locked Decisions

- artifact and access-key APIs stay grouped because both are session/machine-adjacent opaque data
  surfaces used by remote control and app sync
