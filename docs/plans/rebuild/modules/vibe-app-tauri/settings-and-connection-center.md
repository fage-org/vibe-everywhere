# Module Plan: vibe-app-tauri/settings-and-connection-center

## Status

- planned for Wave 10

## Purpose

Turn the current settings area from a mixture of real preferences, retained helper pages, and
handoff-only flows into one coherent product surface.

## Source Of Truth

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `/root/happy/packages/happy-app/sources/app/(app)/settings/**`
- `/root/happy/packages/happy-app/sources/components/SettingsView.tsx`

## Target Location

- settings routes and supporting state inside `packages/vibe-app-tauri`

## Responsibilities

- settings index and detail route classification
- account, appearance, language, usage, voice, and server settings behavior
- connection-center lifecycle for vendor/service connection states
- handoff-only versus fully integrated connect routes

## Non-Goals

- adding new third-party integrations purely for scope expansion
- reviving deferred mobile-native flows that are outside the active Wave 10 contract

## Dependencies

- `validation-and-customer-capability-contract`

## Implementation Steps

1. Audit every settings route against the Wave 10 capability classes.
2. Split settings surfaces into fully supported, limited, and handoff-only sections.
3. Design the connection-center lifecycle and user feedback model.
4. Update route copy, state handling, and visibility rules to match the chosen contract.

## Edge Cases And Failure Modes

- vendor routes that appear interactive but only copy commands
- persisted preferences mixed with preview-only state
- mobile and desktop settings pages diverging without explicit platform notes

## Tests

- route coverage for all visible settings surfaces
- state persistence tests for settings that claim local or remote save behavior
- documentation review for any handoff-only vendor surface

## Acceptance Criteria

- each visible settings route has a clear support class
- connection-center routes no longer read like unfinished integrations
- settings copy accurately describes what saves locally, remotely, or requires an external step

## Locked Decisions

- handoff-only routes are allowed only when explicitly labeled as such
- settings hierarchy must prioritize user clarity over raw route completeness
