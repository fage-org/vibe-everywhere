# Module Plan: vibe-app-tauri/secondary-surfaces

## Archival Status

This file is historical Wave 8 desktop-only module planning material.

Do not use it as active execution authority for Wave 9. Active secondary-route ownership now
belongs to `secondary-routes-and-social.md` and the current Wave 9 module set.

## Purpose

Port the non-core but still user-visible desktop surfaces after the main session flows are stable.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-route-inventory.md`
- `docs/plans/rebuild/vibe-app-tauri-parity-checklist.md`
- `packages/vibe-app/sources/app/**`
- `packages/vibe-app/sources/components/**`

## Target Location

- `packages/vibe-app-tauri`
- artifacts/settings/profile/connect/feed/social/diagnostics routes

## Responsibilities

- artifacts
- account/profile/settings detail screens
- connect/vendor flows
- feed/social/friends if retained for desktop
- changelog and diagnostics

## Non-Goals

- unrelated redesign
- mobile-only screens without desktop value

## Dependencies

- `session-ui-parity`
- `desktop-platform-adapters`

## Implementation Steps

1. Port P1 parity surfaces from the desktop parity inventory.
2. Reassess P2 surfaces for desktop value before implementing them.
3. Mark any explicit deferrals in planning files rather than silently omitting routes.

## Edge Cases And Failure Modes

- parity scope creep
- implementing low-value surfaces before required P1 flows are complete
- hidden desktop-only assumptions in secondary flows

## Tests

- route smoke tests for migrated secondary surfaces
- targeted integration tests for artifacts/settings/connect flows

## Acceptance Criteria

- all required P1 desktop surfaces are present
- any deferred P2 surfaces are explicitly recorded

## Locked Decisions

- P1 surfaces come before nice-to-have desktop extras
