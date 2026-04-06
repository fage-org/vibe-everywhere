# Module Plan: vibe-app-tauri/desktop-shell-and-routing

## Purpose

Recreate the current desktop route tree, app chrome, and layout structure in the new web-native
desktop app.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-route-inventory.md`
- `packages/vibe-app/sources/app/**`
- `packages/vibe-app/sources/components/navigation/**`
- `packages/vibe-app/sources/components/*` that define desktop shell structure

## Target Location

- `packages/vibe-app-tauri`
- route tree
- desktop shell layout
- modal/overlay and navigation primitives

## Responsibilities

- map current desktop-visible routes
- recreate desktop header/sidebar/main panel structure
- preserve navigation semantics and route entry points
- preserve keyboard/focus behavior where required for desktop parity

## Non-Goals

- business logic extraction
- full feature parity
- stylistic redesign

## Dependencies

- `bootstrap-and-package`
- `projects/vibe-app-tauri.md`

## Implementation Steps

1. Fill in `docs/plans/rebuild/vibe-app-tauri-route-inventory.md` against `packages/vibe-app`.
2. Implement routing/layout primitives in `vibe-app-tauri`.
3. Recreate shell-level chrome and panel structure.
4. Port modal/overlay/focus handling for desktop semantics.
5. Validate side-by-side parity on navigation and layout hierarchy.

## Edge Cases And Failure Modes

- route drift from the current desktop app
- shell simplification that changes information density
- keyboard navigation regressions
- modal and overlay stacking inconsistencies

## Tests

- route smoke tests
- desktop shell rendering smoke test
- keyboard/focus navigation checks for primary shell flows

## Acceptance Criteria

- desktop shell and route tree are navigable
- main layout is pixel-close where practical and semantically equivalent elsewhere

## Locked Decisions

- preserve route and shell semantics before redesign
- favor maintainable desktop-web layout only where exact visual parity is impractical
