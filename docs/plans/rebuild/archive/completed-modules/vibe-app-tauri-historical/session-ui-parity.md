# Module Plan: vibe-app-tauri/session-ui-parity

## Archival Status

This file is historical Wave 8 desktop-only module planning material.

Do not use it as active execution authority for Wave 9. Active session rendering ownership now
belongs to `session-rendering-and-composer.md` and the current Wave 9 module set.

## Purpose

Port the core session-heavy UI surfaces with pixel-close parity where practical and maintainable
desktop-web fallbacks where exact matching is not reasonable.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `packages/vibe-app/sources/app/(app)/session/**`
- `packages/vibe-app/sources/components/**` related to session rendering and composer behavior

## Target Location

- `packages/vibe-app-tauri`
- session list/detail routes
- message renderer
- composer
- tool/diff/file/markdown UI

## Responsibilities

- recreate the session list and session detail experience
- preserve composer behavior and tool rendering semantics
- preserve active/resume indicators
- reproduce desktop-visible interaction density and panel relationships

## Non-Goals

- redesign
- secondary settings/artifacts/social surfaces

## Dependencies

- `desktop-shell-and-routing`
- `desktop-platform-adapters`
- `auth-and-session-state`

## Implementation Steps

1. Port session list shell.
2. Port session detail shell and message timeline.
3. Port composer and input interaction model.
4. Port tool/diff/file/markdown rendering.
5. Validate session parity against current desktop behavior.

## Edge Cases And Failure Modes

- message rendering drift
- keyboard/composer regressions
- information density loss on desktop
- tool/file rendering behavior diverging from current expectations

## Tests

- route-level session UI smoke tests
- rendering tests for message/tool/diff/markdown/file surfaces
- one end-to-end session interaction chain

## Acceptance Criteria

- a real desktop session flow works end-to-end
- session UI is parity-complete enough for internal dogfooding

## Locked Decisions

- pixel-close parity is the primary target
- maintainability overrides exact visual duplication only when device/layout constraints demand it
- session UI exceptions must be judged against `docs/plans/rebuild/shared/ui-visual-parity.md`
  rather than ad hoc desktop-web styling preferences
