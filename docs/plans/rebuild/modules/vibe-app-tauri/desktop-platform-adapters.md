# Module Plan: vibe-app-tauri/desktop-platform-adapters

## Archival Status

This file is historical Wave 8 desktop-only module planning material.

Do not use it as active execution authority for Wave 9. Active desktop shell and adapter ownership
now belongs to `desktop-shell-and-platform-parity.md` and the current Wave 9 module set.

## Purpose

Replace Expo/mobile platform assumptions with Tauri/web-native desktop adapters required for the
desktop-first app.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-coexistence-matrix.md`
- platform capability usage in `packages/vibe-app`
- `packages/vibe-app/src-tauri/**`

## Target Location

- `packages/vibe-app-tauri`
- Tauri commands/plugins
- platform adapter layer

## Responsibilities

- secure storage
- clipboard
- external browser / OAuth callback handling
- file dialogs
- notifications where required for parity

## Non-Goals

- mobile-only device capability support
- broad native-platform experimentation before parity

## Dependencies

- `bootstrap-and-package`
- `core-logic-extraction`
- `desktop-shell-and-routing`

## Implementation Steps

1. Fill in `docs/plans/rebuild/vibe-app-tauri-capability-matrix.md` from current app usage.
2. Implement the auth-critical adapter layer first:
   - secure storage
   - external browser / OAuth callback handling
   - any callback route ownership required to avoid hijacking the shipping desktop path
3. Implement the remaining required Tauri/web-native adapters for the first usable desktop slice.
4. Continue hardening later adapters such as file dialogs and notifications once session flows prove
   their exact parity requirements.
5. Remove desktop dependencies on Expo/mobile-specific modules for supported flows.
6. Explicitly defer unsupported/mobile-only capabilities.

## Edge Cases And Failure Modes

- OAuth callback handling mismatch
- insecure credential storage fallback
- file-dialog and clipboard regressions
- hidden assumptions about mobile notifications or sensors

## Tests

- adapter unit tests where practical
- desktop capability smoke tests
- auth/connect/file-dialog flow checks

## Acceptance Criteria

- auth-critical desktop flows no longer block on missing mobile-only Expo behavior
- supported desktop flows no longer depend on mobile-only Expo platform behavior

## Locked Decisions

- capability replacement is driven by parity needs, not by completeness of all Expo APIs
- unsupported mobile-only features must be explicitly deferred rather than silently dropped
