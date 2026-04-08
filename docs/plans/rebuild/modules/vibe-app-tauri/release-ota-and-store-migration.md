# Module Plan: vibe-app-tauri/release-ota-and-store-migration

## Purpose

Finalize desktop, iOS, Android, retained browser web/export, OTA, and store-release ownership
inside `packages/vibe-app-tauri`.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `/root/happy/packages/happy-app/package.json`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/eas.json`
- `/root/happy/packages/happy-app/release.cjs`
- `/root/happy/packages/happy-app/release-dev.sh`
- `/root/happy/packages/happy-app/release-production.sh`
- `.github/workflows/app-release.yml`
- root `package.json`

## Target Location

- `packages/vibe-app-tauri`
- repo release workflows that package app artifacts

## Responsibilities

- app config ownership
- EAS profile ownership
- desktop bundle scripts
- Android and iOS build/submission ownership
- retained browser web/export ownership
- OTA/update channel ownership
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

1. Recreate Happy-aligned release/bootstrap files in `packages/vibe-app-tauri`.
2. Port Vibe-specific identifiers, envs, and store metadata carefully.
3. Update repo workflows so preview lanes can ship from the new package first.
4. Validate desktop/mobile/web artifact generation from the new package.
5. Record the Wave 9 keep/defer decision for analytics and tracking continuity explicitly:
   - root provider bootstrap ownership
   - screen tracking ownership
   - analytics opt-in/out continuity
   - review-prompt telemetry implications where still enabled
6. Record explicitly that no legacy `packages/vibe-app` upgrade-validation lane remains in scope.
7. Record exact production-switch mechanics and rollback paths in the migration plan.

## Edge Cases And Failure Modes

- preview and production identifiers colliding
- OTA channels switching before route or capability parity is ready
- store submissions tied to stale package paths or secrets
- release scripts still reading from `packages/vibe-app`
- browser export ownership remaining implicit instead of package-local

## Tests

- release-script dry runs
- preview artifact generation for desktop, Android, iOS, and retained browser web/export
- workflow-level packaging checks
- analytics / tracking bootstrap and opt-out continuity review
- rollback drill documentation review

## Acceptance Criteria

- `packages/vibe-app-tauri` can generate the app artifacts required for the default app path
- default release ownership can move without undefined identifier, OTA, web/export, or rollback behavior
- analytics / tracking continuity is either preserved or explicitly deferred in writing before
  promotion

## Locked Decisions

- preview lanes move before production ownership moves
- no legacy `packages/vibe-app` upgrade-validation lane is required or implied
- release migration must remain rollback-safe at every stage
