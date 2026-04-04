# Module Plan: vibe-server/storage-files

## Purpose

Implement file and object storage for uploads, avatars, and other blob references used by the app.

## Happy Source Of Truth

- `packages/happy-server/sources/storage/files.ts`
- `packages/happy-server/sources/storage/types.ts`
- `packages/happy-server/sources/storage/uploadImage.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/storage/files.rs`
  - `src/storage/types.rs`

## Responsibilities

- define file/image reference types
- upload and retrieve object blobs
- support S3-compatible storage behavior needed by parity flows

## Non-Goals

- image transformation logic beyond storage handoff

## Public Types And Interfaces

- `ImageRef` and related storage references
- object storage service

## Data Flow

- app uploads or references file/blob
- server stores object and returns storage reference
- reference is embedded in account or session payloads

## Dependencies

- S3-compatible object storage client

## Implementation Steps

1. Port storage reference types.
2. Implement a backend seam that can switch between local filesystem and S3-compatible object
   storage based on environment configuration.
3. Persist uploaded-file reuse metadata so repeated avatar/image fetches survive process restarts
   and match Happy's `reuseKey` behavior.
4. Add optional local filesystem adapter only for development/testing.
5. Add upload/retrieve tests against both the local adapter and an S3-compatible mock/test target,
   keeping the test contract aligned with the later MinIO integration surface.

## Edge Cases And Failure Modes

- missing object after DB reference persists
- unsupported mime type
- large upload failure and partial object cleanup

## Tests

- object upload test
- reference serialization test
- local adapter smoke test
- persisted reuse-key test
- S3-compatible adapter contract test

## Acceptance Criteria

- app-facing blob references and uploads work with stable reference types
- repeated `uploadImage`-style requests with the same reuse key return the same stored reference
- production configuration can select an S3-compatible backend without changing caller code

## Open Questions

- None.

## Locked Decisions

- primary backend is S3-compatible object storage
- local filesystem backend is dev-only
- base file/object storage comes first; image normalization and upload composition are layered on by
  `image-processing`
