# Module Plan: vibe-app-tauri/bootstrap-and-package

## Archival Status

This file is historical Wave 8 desktop-only module planning material.

Do not use it as active execution authority for Wave 9. For active replacement work, use the Wave 9
planning set and current module ownership instead.

## Purpose

Create `packages/vibe-app-tauri` as a new desktop-only package with a runnable Tauri 2 shell and a
web-native frontend bootstrap, without destabilizing `packages/vibe-app`.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-coexistence-matrix.md`
- `packages/vibe-app`
- `packages/vibe-app/src-tauri/*`
- `/root/happy/packages/happy-app` as desktop behavior reference only

## Target Location

- package: `packages/vibe-app-tauri`

## Responsibilities

- scaffold a separate package
- bootstrap Tauri 2 app structure
- choose and wire the desktop web frontend stack
- apply the coexistence rules for package IDs, release artifacts, and local app-owned state
- define local build, typecheck, test, and release entrypoints
- ensure desktop packaging is independent from `packages/vibe-app`

## Non-Goals

- feature parity
- shared logic extraction
- screen migration

## Dependencies

- `shared/source-crosswalk.md`
- `projects/vibe-app-tauri.md`

## Implementation Steps

1. Create `packages/vibe-app-tauri` with package metadata and package-manager integration.
2. Apply the coexistence matrix to bundle identifiers, updater/release channel names, artifact
   names, and package-local state directories before bootstrapping the shell.
3. Add a minimal Tauri 2 shell and runnable desktop frontend.
4. Define package scripts for dev, build, typecheck, test, and release packaging.
5. Ensure the new package does not require edits to `packages/vibe-app` to boot.
6. Add CI/release hooks only for the new package.

## Edge Cases And Failure Modes

- accidentally sharing build outputs with `packages/vibe-app`
- naming collisions with the existing desktop package
- Tauri config coupling to Expo/mobile assumptions
- package bootstrapping that silently depends on root app scripts

## Tests

- package install verification
- local dev boot smoke test
- Tauri bundle smoke test
- package-local typecheck/test command smoke test

## Acceptance Criteria

- `packages/vibe-app-tauri` exists as a separate runnable package
- Tauri desktop shell boots
- package-local scripts do not mutate `packages/vibe-app`

## Locked Decisions

- create a new package; do not split or rename `packages/vibe-app`
- optimize for isolated bootstrap first, not shared abstractions
