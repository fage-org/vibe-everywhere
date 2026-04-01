# Module Plan: vibe-wire/message-meta

## Purpose

Implement the shared metadata object attached to legacy and session-protocol messages.

## Happy Source Of Truth

- `packages/happy-wire/src/messageMeta.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-wire`
- file: `src/message_meta.rs`

## Responsibilities

- define shared message metadata fields
- encode permission-mode enum values compatibly
- preserve optional/nullable behavior required by Happy clients

## Non-Goals

- validating business rules for tool permissions
- mapping metadata to provider-specific runtime options

## Public Types And Interfaces

- `MessageMeta`
- `PermissionMode`

## Data Flow

- user messages carry metadata for permission mode and model hints
- session protocol wrappers may carry the same metadata
- app and clients read metadata for rendering and control decisions

## Dependencies

- `serde`

## Implementation Steps

1. Port every metadata field using Happy wire names.
2. Implement `PermissionMode` with the current Happy value set.
3. Use `Option` for nullable or optional fields exactly where Happy does.
4. Add exhaustive serialization tests for the enum values and optional fields.

## Edge Cases And Failure Modes

- some fields are nullable and optional; the implementation must distinguish both when needed
- permission-mode values containing hyphens must serialize exactly

## Tests

- full object round-trip
- sparse object round-trip
- enum value serialization tests
- invalid permission-mode parsing tests

## Acceptance Criteria

- all metadata fields are present and wire-compatible
- permission-mode enum values match Happy strings

## Open Questions

- None.

## Locked Decisions

- keep the field set identical to current Happy metadata
- do not fold metadata into broader session models
