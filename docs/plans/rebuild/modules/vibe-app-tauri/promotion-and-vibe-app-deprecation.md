# Module Plan: vibe-app-tauri/promotion-and-vibe-app-deprecation

## Purpose

Define the final switch that makes `packages/vibe-app-tauri` the default app path and active Wave 9
replacement package while retiring `packages/vibe-app` from active ownership.

## Source Of Truth

### Active Wave 9 planning inputs

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`

### Historical desktop-only references

- `docs/plans/rebuild/vibe-app-tauri-promotion-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-parity-checklist.md`
- current `packages/vibe-app` release and fallback behavior

Historical Wave 8 desktop-only documents may inform continuity review, but they do not override the
active Wave 9 cross-platform gate or expand it implicitly.

## Target Location

- planning docs
- repo release ownership notes
- deprecation and fallback notes for `packages/vibe-app`

## Responsibilities

- define the production switch criteria
- record the rollback path
- record the fallback retention window for `packages/vibe-app`
- update docs and helper scripts when the owner switches
- define when `packages/vibe-app` can stop being the default path

## Non-Goals

- broad code migration work already owned by earlier modules
- deleting `packages/vibe-app` immediately after promotion

## Dependencies

- `release-ota-and-store-migration`
- all promotion-critical route and capability modules

## Implementation Steps

1. Confirm the route and capability matrix is satisfied or explicitly waived.
2. Confirm release ownership is runnable and rollback-safe.
3. Record the exact production switch steps and fallback owner.
4. Update docs and workflow defaults only after sign-off.
5. Define the retention and eventual retirement policy for `packages/vibe-app`.

## Edge Cases And Failure Modes

- switching the default owner while one platform family still depends on `packages/vibe-app`
- ambiguous rollback ownership
- helper docs and scripts still pointing to the old default path
- deprecating the legacy package before the retention window is honored

## Tests

- promotion checklist review
- release-owner switch review
- rollback drill review
- documentation and workflow default-path review

## Acceptance Criteria

- `packages/vibe-app-tauri` is approved as the default app owner
- `packages/vibe-app` has an explicit fallback and retirement policy
- the switch is documented, reversible, and not dependent on tribal knowledge

## Locked Decisions

- production promotion is an explicit act, not an accidental drift
- `packages/vibe-app` remains a documented fallback until this module signs off the switch
- promotion sign-off must treat violations of `docs/plans/rebuild/shared/ui-visual-parity.md` as
  parity gaps unless an approved exception is already recorded
