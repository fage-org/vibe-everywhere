# Module Plan: vibe-app/desktop-tauri-adaptation

## Purpose

Adapt the imported app's desktop shell to the Vibe repository layout and product naming.

## Happy Source Of Truth

- `packages/happy-app/src-tauri/*`
- Happy app build scripts relevant to Tauri/Desktop

## Target Rust/Vibe Location

- package: `packages/vibe-app`
- expected edit areas:
  - `src-tauri/`
  - desktop build config
  - package scripts

## Responsibilities

- update desktop shell naming and identifiers
- align Tauri config with Vibe layout and binaries
- keep desktop builds runnable after import

## Non-Goals

- desktop feature expansion

## Public Types And Interfaces

- desktop package metadata
- Tauri bundle identifiers and config

## Data Flow

- desktop build reads Vibe app config
- Tauri shell starts imported app bundle against Vibe backend path

## Dependencies

- `import-and-build`
- `branding-and-naming-adaptation`

## Implementation Steps

1. Port Tauri config into the Vibe package structure.
2. Update bundle identifiers, names, and assets.
3. Verify script and asset paths after import.

## Edge Cases And Failure Modes

- stale Happy bundle ids
- asset path breakage after repo move

## Tests

- Tauri config validation
- desktop build smoke test

## Acceptance Criteria

- desktop shell builds with Vibe naming and package layout

## Open Questions

- None.

## Locked Decisions

- desktop shell follows imported app structure first; do not split it into a separate package
