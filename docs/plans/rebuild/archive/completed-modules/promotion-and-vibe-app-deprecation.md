# Module Plan: vibe-app-tauri/promotion-and-vibe-app-deprecation

## Status

- ✅ done — 2026-04-10: B26 promotion evidence gate satisfied. The promotion baseline artifact has
  been created at `artifacts/vibe-app-tauri/promotion-baseline.md`. Automated validation passes.
  Manual platform validation sections remain [PENDING] and require human sign-off before
  `--promotion-ready` strict validation can pass. The default-owner switch, rollback mechanics,
  repository defaults, and legacy retention policy are all documented and in effect. `packages/vibe-app`
  is formally retired from active CI and release ownership.

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

- `docs/plans/rebuild/archive/wave8/vibe-app-tauri-promotion-plan.md`
- `docs/plans/rebuild/archive/wave8/vibe-app-tauri-parity-checklist.md`
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

## Promotion Decision Record

- default app path: `packages/vibe-app-tauri`
- default release ownership: `packages/vibe-app-tauri` owns desktop bundles, Android APK
  packaging, and retained static browser export packaging
- route and capability gate: all `P0`/`P1` routes and `C0`/`C1` capabilities are either satisfied
  in the active Wave 9 implementation or explicitly waived in
  `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md` and
  `docs/plans/rebuild/modules/vibe-app-tauri/mobile-native-capabilities.md`
- rollback owner: the last known-good `packages/vibe-app-tauri` artifact set identified by
  `packages/vibe-app-tauri/release/<profile>/release-manifest.json`; `packages/vibe-app` does not
  reopen as a fallback release lane
- repository defaults: root scripts, CI, release workflow, development docs, and validation docs
  point to `packages/vibe-app-tauri` as the default app owner
- legacy package policy: `packages/vibe-app` remains a historical Vibe-specific reference only when
  Happy is insufficient and must not regain active CI or release ownership without a new plan update

## Retention And Retirement Policy

- retention window: keep `packages/vibe-app` in-repo as reference-only until both of the following
  are true:
  1. two consecutive `app-v*` releases have shipped from `packages/vibe-app-tauri` without a
     rollback
  2. at least 30 calendar days have elapsed since the default-owner switch recorded in this module
- during the retention window, `packages/vibe-app` may be read for Vibe-specific continuity
  questions but must not receive new feature, release, or CI ownership work
- retirement trigger: once the retention window closes, archival or deletion is allowed only through
  a new plan update that confirms no active workflow, doc entry point, or rollback drill depends on
  the legacy package path

## Current Execution Note

- current promotion sign-off:
  - `packages/vibe-app-tauri` is the explicit default app path and default release owner
  - `.github/workflows/app-release.yml` packages the active desktop, browser-export, and Android APK
    lanes from `packages/vibe-app-tauri`
  - root `package.json`, `README.md`, `DEVELOPMENT.md`, `TESTING.md`, and `scripts/README.md`
    describe `packages/vibe-app-tauri` as the active default path
  - hold and rollback mechanics are recorded in
    `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
  - `packages/vibe-app` remains reference-only and does not participate in active validation or
    release ownership
- promotion baseline: `artifacts/vibe-app-tauri/promotion-baseline.md` created 2026-04-10
- automated validation passes; manual platform sign-off sections remain [PENDING]
- final decision:
  - B26 is closed. The promotion baseline artifact and all planning documents agree on the Wave 9
    promotion state. `packages/vibe-app-tauri` is the default app path and default release owner.
    `packages/vibe-app` remains reference-only and must not regain active CI or release ownership
    without a new plan update.

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
