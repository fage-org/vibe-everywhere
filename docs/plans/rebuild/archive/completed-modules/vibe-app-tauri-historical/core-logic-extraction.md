# Module Plan: vibe-app-tauri/core-logic-extraction

## Archival Status

This file is historical Wave 8 desktop-only module planning material.

Do not use it as active execution authority for Wave 9. Active shared-core ownership now belongs to
`shared-core-from-happy.md` and the current Wave 9 module set.

## Purpose

Extract the minimum reusable non-visual logic required by the new desktop app while keeping that
logic package-local to `packages/vibe-app-tauri` during early phases.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/vibe-app-tauri-extraction-inventory.md`
- `packages/vibe-app/sources/sync/**`
- `packages/vibe-app/sources/auth/**`
- `packages/vibe-app/sources/encryption/**`
- `packages/vibe-app/sources/utils/**`
- `packages/vibe-app/sources/text/**`

## Target Location

- `packages/vibe-app-tauri`
- package-local domain/state/adapter modules

## Responsibilities

- identify logic that can be reused without React Native primitives
- port or adapt that logic into `vibe-app-tauri`
- define platform-adapter seams where Expo/RN APIs are still assumed

## Non-Goals

- creating a shared `vibe-app-core` package
- mass refactors inside `packages/vibe-app`
- screen/UI migration

## Dependencies

- `bootstrap-and-package`
- `projects/vibe-app-tauri.md`
- `shared/source-crosswalk.md`

## Implementation Steps

1. Fill in `docs/plans/rebuild/vibe-app-tauri-extraction-inventory.md` for the candidate modules.
2. Sort candidates into:
   - package-local reusable as-is
   - reusable after adapter seams
   - desktop-only rewrites
3. Port the minimum logic needed for auth/session/account/sync bootstrap.
4. Add platform abstraction seams instead of importing React Native UI primitives.
5. Record any future shared-package candidates as deferred follow-up only after parity proof.

## Edge Cases And Failure Modes

- hidden Expo/RN assumptions inside “utility” code
- creating leaky abstractions too early
- modifying `packages/vibe-app` for convenience
- extracting too broadly before a working desktop slice exists

## Tests

- package-local unit tests for extracted modules
- protocol/parser compatibility tests where applicable
- compile-time guard that extracted logic does not import React Native UI modules

## Acceptance Criteria

- the new package can consume extracted auth/session/sync logic
- extracted logic lives inside `packages/vibe-app-tauri` unless explicitly deferred for later sharing

## Locked Decisions

- default extraction destination is `packages/vibe-app-tauri`
- no shared-core package in early phases
- shared-core extraction must not redefine UI tokens, branding assets, or presentation defaults in a
  way that conflicts with `docs/plans/rebuild/shared/ui-visual-parity.md`
