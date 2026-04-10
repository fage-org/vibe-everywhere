# Module Plan: vibe-wire/message-meta

## Purpose

Implement the shared metadata object attached to legacy and session-protocol messages.

## Happy Source Of Truth

- `packages/happy-wire/src/messageMeta.ts`
- `packages/happy-app/sources/sync/typesMessageMeta.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-wire`
- file: `src/message_meta.rs`

## Responsibilities

- define shared message metadata fields
- preserve permission-mode string keys compatibly
- preserve optional/nullable behavior required by Happy clients

## Non-Goals

- validating business rules for tool permissions
- mapping metadata to provider-specific runtime options

## Public Types And Interfaces

- `MessageMeta`

## Data Flow

- user messages carry metadata for permission mode and model hints
- session protocol wrappers may carry the same metadata
- app and clients read metadata for rendering and control decisions

## Dependencies

- `serde`

## Implementation Steps

1. Port every metadata field using Happy wire names.
2. Represent `permissionMode` as a string key so current Happy clients can send arbitrary values.
3. Use `Option` for nullable or optional fields exactly where Happy does.
4. Add serialization tests for known and custom permission-mode values plus optional fields.

## Edge Cases And Failure Modes

- some fields are nullable and optional; the implementation must distinguish both when needed
- permission-mode values containing hyphens must serialize exactly
- unknown permission-mode keys from current Happy clients must round-trip unchanged
- optional-only fields such as `sentFrom`, `permissionMode`, and `displayText` must reject explicit
  JSON `null`

## Tests

- full object round-trip
- sparse object round-trip
- known permission-mode round-trip tests
- custom permission-mode round-trip tests
- invalid non-string permission-mode parsing tests
- invalid explicit `null` tests for optional-only fields

## Acceptance Criteria

- all metadata fields are present and wire-compatible
- permission-mode values round-trip exactly, including custom client-defined keys

## Open Questions

- None.

## Locked Decisions

- keep the field set identical to current Happy metadata
- do not fold metadata into broader session models
