# Module Plan: vibe-app-tauri/desktop-shell-and-platform-parity

## Purpose

Own the active Wave 9 desktop shell, route chrome, interaction semantics, and desktop-specific
platform adapters inside `packages/vibe-app-tauri`.

This module replaces the old Wave 8 desktop-only execution ownership for desktop shell and adapter
work. Historical Wave 8 module plans remain continuity references only.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `/root/happy/packages/happy-app/src-tauri/**`
- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx`
- `/root/happy/packages/happy-app/sources/components/MainView.tsx`
- `/root/happy/packages/happy-app/sources/components/SidebarNavigator.tsx`
- `/root/happy/packages/happy-app/sources/components/navigation/Header.tsx`
- `/root/happy/packages/happy-app/sources/modal/**`
- `/root/happy/packages/happy-app/sources/utils/copySessionMetadataToClipboard.ts`
- `/root/happy/packages/happy-app/sources/app/(app)/text-selection.tsx`

## Historical Continuity References

- `docs/plans/rebuild/modules/vibe-app-tauri/desktop-shell-and-routing.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/desktop-platform-adapters.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/session-ui-parity.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/secondary-surfaces.md`

These files no longer define active execution ownership for Wave 9. Use them only when a desktop
continuity question is not already answered by Happy or the active Wave 9 docs.

## Target Location

- `packages/vibe-app-tauri/sources/desktop/**`
- desktop route shell and navigation surfaces
- desktop adapter seams for clipboard, file dialogs, notifications, keyboard/focus, and modal
  behavior

## Responsibilities

- desktop route shell, header/sidebar chrome, and top-level navigation behavior
- desktop modal, overlay, focus, and keyboard interaction semantics
- desktop clipboard and copy/export handoff
- desktop file open/save dialog ownership where required for parity
- desktop notification handoff where required for parity
- desktop host-component ownership for the routes that remain desktop-specific in behavior even when
  their shared/runtime state lives elsewhere

## Non-Goals

- mobile shell ownership
- browser static export ownership
- release-owner switch
- redefining route semantics away from Happy

## Dependencies

- `universal-bootstrap-and-runtime`
- `shared-core-from-happy`
- `session-runtime-and-storage`
- `auth-and-identity-flows` for desktop callback semantics

## Implementation Steps

1. Recreate the desktop shell structure and top-level route chrome from Happy/Tauri references.
2. Keep keyboard, focus, modal, and overlay behavior explicit rather than implied by framework defaults.
3. Recreate the required desktop adapter seams for clipboard, file dialogs, and notifications.
4. Hand off route-specific shared state concerns back to the active Wave 9 state/rendering modules while
   keeping desktop host behavior owned here.
5. Record any remaining desktop-only gaps as explicit deferrals in the Wave 9 matrix before promotion.

## Edge Cases And Failure Modes

- desktop shell parity drifting while mobile/browser work advances
- keyboard or focus regressions that do not show up in touch-first testing
- clipboard or save/export flows working on one desktop OS but not others
- route ownership becoming split ambiguously between this module and the session/secondary modules

## Tests

- desktop shell rendering smoke test
- keyboard/focus navigation checks for primary shell flows
- modal and overlay behavior checks
- clipboard smoke checks for primary copy/export flows
- file dialog and notification smoke checks where required by the Wave 9 matrix

## Acceptance Criteria

- desktop UI ownership for active Wave 9 routes is explicit in the current module set
- primary desktop shell interactions remain Happy-aligned enough for side-by-side review
- required desktop adapter behavior no longer depends on deprecated Wave 8 execution ownership

## Locked Decisions

- Wave 9 desktop shell work must land in the active module set, not by reopening Wave 8 as an
  execution owner
- maintainable desktop-web/Tauri implementation is acceptable only when route and interaction
  semantics stay Happy-aligned
