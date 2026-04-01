# Module Plan: vibe-server/image-processing

## Purpose

Implement image processing helpers required for avatars, uploads, thumbnails, and thumbhash-style
metadata.

## Happy Source Of Truth

- `packages/happy-server/sources/storage/processImage.ts`
- `packages/happy-server/sources/storage/thumbhash.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/storage/process_image.rs`
  - `src/storage/thumbhash.rs`

## Responsibilities

- decode uploaded images
- normalize image size/format as required
- compute thumbhash or equivalent metadata for app placeholders

## Non-Goals

- generalized media pipeline

## Public Types And Interfaces

- processed image output struct
- thumbhash helper

## Data Flow

- upload enters file storage path
- image-processing normalizes image and returns metadata
- storage layer persists normalized asset and reference metadata

## Dependencies

- `storage-files`
- Rust image processing crates

## Implementation Steps

1. Port required output metadata fields.
2. Implement resize/normalize pipeline only for supported image types.
3. Compute thumbhash-compatible placeholder data.
4. Add deterministic tests using fixture images.

## Edge Cases And Failure Modes

- unsupported or corrupt image input
- oversized image memory usage
- thumbhash mismatch across languages

## Tests

- valid image processing test
- corrupt image rejection test
- thumbhash fixture comparison test

## Acceptance Criteria

- app receives usable image metadata and placeholders compatible with Happy expectations

## Open Questions

- None.

## Locked Decisions

- support only image formats needed by imported app flows initially
- keep image normalization deterministic for fixture testing
