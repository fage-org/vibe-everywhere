# Module Plan: vibe-app-tauri/release-ota-and-store-migration

## Status

- completed on 2026-04-09 after package-local release profiles, desktop preview/candidate bundle
  configs, Android release-property overrides, browser/desktop/APK artifact packaging, workflow
  packaging updates, analytics/tracking defer decisions, and rollback-safe release manifests all
  landed under `packages/vibe-app-tauri`

## Purpose

Finalize desktop, Android, retained browser build/export, updater, and store-release ownership
inside `packages/vibe-app-tauri`.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `/root/happy/packages/happy-app/package.json`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/eas.json`
- `/root/happy/packages/happy-app/release.cjs`
- `/root/happy/packages/happy-app/release-dev.sh`
- `/root/happy/packages/happy-app/release-production.sh`
- `.github/workflows/app-release.yml`
- root `package.json`

## Wave 9 Canonical Inputs

- package-local release scripts and artifact naming rules
- package-local `release.config.json`
- package-local desktop preview and production-candidate Tauri config overrides
- repository-owned Android native project, signing inputs, and APK packaging config
- GitHub Release workflow inputs and artifact publication rules
- package-local retained static browser export config

## Target Location

- `packages/vibe-app-tauri`
- repo release workflows that package app artifacts

## Responsibilities

- native build and release config ownership
- native project and release-profile ownership
- desktop bundle scripts
- Android build/signing/submission ownership
- retained static browser export ownership
- updater/update channel ownership
- release artifact naming and continuity
- analytics / tracking continuity decision ownership
- explicit statement that legacy `packages/vibe-app` upgrade validation is not required

## Non-Goals

- promotion approval itself
- deleting legacy release paths before rollback is documented

## Dependencies

- `universal-bootstrap-and-runtime`
- `mobile-native-capabilities`
- `secondary-routes-and-social`

## Implementation Steps

1. Recreate Wave 9 release/bootstrap files in `packages/vibe-app-tauri` using Happy only as a
   continuity reference.
2. Port Vibe-specific identifiers, envs, Android signing inputs, APK naming, and release metadata
   carefully.
3. Update repo workflows so preview lanes can ship from the new package first.
4. Validate desktop, Android APK, and retained static browser export artifact generation from the new
   package.
5. Record the Wave 9 keep/defer decision for analytics and tracking continuity explicitly:
   - root provider bootstrap ownership
   - screen tracking ownership
   - analytics opt-in/out continuity
   - review-prompt telemetry implications where still enabled
6. Record explicitly that no legacy `packages/vibe-app` upgrade-validation lane remains in scope.
7. Record exact production-switch mechanics and rollback paths in the migration plan.

## Edge Cases And Failure Modes

- preview and production identifiers colliding
- desktop updater policy drifting from the Android APK release path
- Android signing or GitHub Release inputs tied to stale package paths or secrets
- release scripts still reading from `packages/vibe-app`
- browser build/export ownership remaining implicit instead of package-local

## Tests

- release-script dry runs
- preview artifact generation for desktop, Android APK, and retained static browser export
- workflow-level packaging checks
- analytics / tracking bootstrap and opt-out continuity review
- rollback drill documentation review

## Acceptance Criteria

- `packages/vibe-app-tauri` can generate the desktop, Android APK, and retained static export
  artifacts required for the default app path
- default release ownership can move without undefined identifier, desktop updater policy, retained
  static export behavior, or rollback gaps
- analytics / tracking continuity is either preserved or explicitly deferred in writing before
  promotion

## Locked Decisions

- preview lanes move before production ownership moves
- no legacy `packages/vibe-app` upgrade-validation lane is required or implied
- release migration must remain rollback-safe at every stage
- release-owned assets, metadata, screenshots, and packaged presentation must not drift from
  `docs/plans/rebuild/shared/ui-visual-parity.md` without an approved plan exception
