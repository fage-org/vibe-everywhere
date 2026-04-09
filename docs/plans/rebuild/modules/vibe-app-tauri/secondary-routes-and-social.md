# Module Plan: vibe-app-tauri/secondary-routes-and-social

## Status

- completed on 2026-04-09 after all promotion-critical `P1` route families were wired in
  `packages/vibe-app-tauri`, route smoke coverage was added for the migrated surfaces, and the
  remaining social/developer routes were kept as explicit `P2` deferrals instead of implicit gaps

## Purpose

Port the remaining user-visible `P1` routes and surface families needed for the active Wave 9
replacement package.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `/root/happy/packages/happy-app/sources/app/(app)/artifacts/**`
- `/root/happy/packages/happy-app/sources/app/(app)/friends/**`
- `/root/happy/packages/happy-app/sources/app/(app)/user/**`
- `/root/happy/packages/happy-app/sources/app/(app)/machine/**`
- `/root/happy/packages/happy-app/sources/app/(app)/settings/**`
- `/root/happy/packages/happy-app/sources/app/(app)/terminal/**`
- `/root/happy/packages/happy-app/sources/app/(app)/changelog.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/server.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/text-selection.tsx`
- `/root/happy/packages/happy-app/sources/components/InboxView.tsx`
- `/root/happy/packages/happy-app/sources/components/SettingsView.tsx`
- `/root/happy/packages/happy-app/sources/components/usage/**`

## Target Location

- mobile and desktop secondary route surfaces inside `packages/vibe-app-tauri`

## Responsibilities

- settings detail pages
- artifacts flows
- friends/social surfaces
- user and machine detail pages
- terminal/connect flows
- changelog, server, and text utility surfaces

## Non-Goals

- developer-only route parity unless promoted explicitly
- release-owner switch

## Dependencies

- `session-runtime-and-storage`
- `session-rendering-and-composer`
- `mobile-native-capabilities` where routes depend on it

## Implementation Steps

1. Port `P1` routes in priority order from the matrix.
2. Keep route names and entry semantics aligned to Happy.
3. Port supporting shared state and API wiring only as needed for those surfaces.
4. Record any remaining `P2` deferrals explicitly.

## Edge Cases And Failure Modes

- parity scope creep into low-value dev routes too early
- settings routes rendering without the state they actually depend on
- artifact or social surfaces regressing differently on desktop and mobile

## Tests

- route smoke tests for every migrated `P1` route family
- targeted integration tests for artifacts, settings detail, and friend flows

## Acceptance Criteria

- all promotion-critical user-visible route families exist and are wired
- remaining deferrals are explicit and approved in planning docs

## Locked Decisions

- `P1` routes come before `P2` or cosmetic extras
- route semantics follow Happy first, even if the internal implementation differs
- secondary-route visuals and information density must remain governed by
  `docs/plans/rebuild/shared/ui-visual-parity.md` unless a narrower exception is recorded first

## Explicit Wave 9 Deferrals

- `/(app)/friends/index` and `/(app)/friends/search` stay `P2` until desktop value is confirmed;
  they do not block the current promotion gate
- `/(app)/dev/**` stays `P2` and does not block promotion unless a route is explicitly promoted
