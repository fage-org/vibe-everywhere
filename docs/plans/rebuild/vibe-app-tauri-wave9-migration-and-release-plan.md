# Wave 9 Migration And Release Plan: `vibe-app-tauri`

## Purpose

This file records how `packages/vibe-app-tauri` takes over release, updater, identifier, store, and
rollback ownership from `packages/vibe-app`.

## Source Of Truth

### Primary release structure reference

- `/root/happy/packages/happy-app/package.json`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/eas.json`
- `/root/happy/packages/happy-app/release.cjs`
- `/root/happy/packages/happy-app/release-dev.sh`
- `/root/happy/packages/happy-app/release-production.sh`

### Vibe continuity reference

- `packages/vibe-app/package.json`
- `packages/vibe-app/app.config.js`
- `packages/vibe-app/eas.json`
- `.github/workflows/app-release.yml`
- `docs/plans/rebuild/shared/ui-visual-parity.md`

These legacy Vibe paths are reference-only. They are not active CI or release owners anymore.

Happy's Expo/EAS release files remain continuity references only. Wave 9 does not treat them as the
required runtime or release toolchain boundary.

## Migration Rule

Do not move release ownership just because the new package builds. Ownership moves only after:

- the route matrix is satisfied
- the capability matrix is satisfied
- Happy-aligned UI and visual parity correction is complete for shell, session, and
  promotion-critical secondary surfaces, or approved exceptions are documented
- preview releases prove upgrade and rollback safety

## Release Ownership Stages

### Stage 0: Immediate Legacy Deprecation

- `packages/vibe-app` owns no active CI or release lanes
- `packages/vibe-app` remains reference-only when Happy is insufficient
- `packages/vibe-app-tauri` is the only package allowed to receive new app release-lane work

### Stage 1: Unified Preview Ownership

- `packages/vibe-app-tauri` may produce preview desktop, Android APK, and retained static browser
  export artifacts
- all preview identifiers, update channels, and visible release names stay distinguishable from the
  former shipping app history
- no store/default switch yet

### Stage 2: Production-Parity Validation

- `packages/vibe-app-tauri` produces production-candidate artifacts under controlled validation
- no legacy `packages/vibe-app` upgrade-validation lane remains in scope; validation happens against the active replacement package and its own release candidates
- no new fallback release lane is kept on `packages/vibe-app`; rollback must use prior `packages/vibe-app-tauri` artifacts or a later explicitly approved plan update
- user-visible release assets and in-app presentation must pass Happy-aligned visual parity review
  before production ownership can move

### Stage 3: Production Switch

- `packages/vibe-app-tauri` becomes the default release owner
- `packages/vibe-app` remains reference-only and does not regain active release ownership by default
- docs, helper scripts, workflows, and release notes all point to the new owner

### Stage 4: Legacy Retirement

- `packages/vibe-app` remains reference-only
- any remaining legacy retention window is explicitly documented
- deletion or archive can happen only after the retirement window closes

## Identifier And Channel Rules

| Concern | Pre-promotion rule | Promotion requirement |
| --- | --- | --- |
| desktop identifier | use distinct preview identifier and product naming | production identifier can move only after explicit switch approval |
| Android application ids | keep preview ids separate from shipping ids | production switch must preserve upgrade continuity |
| deep-link schemes and universal/app links | preview paths must not steal production ownership from the shipping app | production switch must be coordinated and documented |
| updater/update channels | preview and production channels must remain distinct | production ownership moves only after rollback path is proven |
| Android distribution metadata | preview and production APK naming plus GitHub Release metadata stay separate | production switch must preserve install and rollback continuity |

## Platform Scope Lock

- Android is the only active mobile platform in the current Wave 9 scope.
- Android GitHub Releases distribution is the primary mobile release path.
- APK is the primary Android artifact for preview and production-candidate distribution.
- iOS remains deferred and is excluded from the current release and promotion gate.

## Required Release Inputs In `packages/vibe-app-tauri`

- package-local browser/native build config
- package-local Tauri desktop/mobile config
- Android native project and signing config
- `release.cjs`
- `release-dev.sh`
- `release-production.sh`
- any required app icons, splash assets, and notification assets
- any required browser build metadata, favicon, and browser asset hooks
- any required store / updater / native signing config hooks

## Required Workflow Changes Before Promotion

- `.github/workflows/app-release.yml` or its replacement must package from `packages/vibe-app-tauri`
  for the new default lanes
- workflow inputs must make desktop, Android APK, and retained static browser export ownership
  explicit
- artifact naming must distinguish preview versus default-production ownership during the transition

## Data Migration Review Table

| Continuity area | Platforms | Storage or continuity seam to review explicitly | Owning Wave 9 module | Validation artifact required before switch |
| --- | --- | --- | --- | --- |
| auth credential persistence and token restore | desktop, Android | secure-storage backend, token namespace, resume behavior after upgrade | `auth-and-identity-flows` | upgrade/reinstall restore notes plus restart validation |
| local settings and preference keys | desktop, Android, static export where applicable | settings key namespace, preference defaults, account-backed preference sync | `session-runtime-and-storage`, `secondary-routes-and-social` | settings continuity checklist with before/after key review |
| draft state and compose buffers | desktop, Android | draft storage namespace, unsent composer state, active-session restore semantics | `session-runtime-and-storage`, `session-rendering-and-composer` | draft continuity smoke validation |
| changelog seen-state and onboarding markers | desktop, Android, static export | local seen markers and onboarding flags | `shared-core-from-happy`, `secondary-routes-and-social` | changelog/onboarding continuity review |
| notification permissions and route restoration | Android, desktop where applicable | permission state, notification payload routing, deep-link/session restoration | `mobile-native-capabilities` | real-device notification routing evidence |
| purchase entitlement refresh after upgrade | Android, web where applicable | entitlement cache, restore semantics, post-upgrade refresh timing | `mobile-native-capabilities` | purchase continuity validation notes |
| device-link or QR continuation behavior | desktop, Android, static export where applicable | in-flight auth attempts, callback ownership, QR wait/start state | `auth-and-identity-flows` | auth/link continuity validation notes |
| desktop local state, caches, logs, and app-owned directories | desktop | Tauri/app-owned directories, cache retention, log continuity, file-export locations | `desktop-shell-and-platform-parity`, `release-ota-and-store-migration` | per-OS directory review and smoke validation |
| Android local state, app-owned directories, and signed artifact continuity | Android | Android native project paths, app-owned directories, signing inputs, APK naming, install/upgrade continuity | `universal-bootstrap-and-runtime`, `release-ota-and-store-migration` | Android install/upgrade validation notes plus artifact review |
| analytics opt-in/out state, tracking identity continuity, and review-prompt throttling state | desktop, Android, static export where applicable | analytics preference keys, anonymous/known identity continuity, prompt throttling or suppression state | `release-ota-and-store-migration`, `mobile-native-capabilities` | analytics continuity decision and validation note |

## Rollback Rules

If a production switch candidate regresses:

- pause the broken candidate and fall back to the last known-good `packages/vibe-app-tauri` artifact set or an explicitly approved temporary hold
- keep the broken candidate artifacts distinguishable and non-default
- record the failure mode in this file and in the promotion module plan
- do not reattempt the switch until the parity or migration gap is closed in planning first

## Validation Required Before Production Switch

### Desktop

- Linux startup and core session validation
- macOS startup and core session validation
- Windows startup and core session validation

### Mobile

- Android real-device auth / restore / session / notification / QR / purchase pass

### Release

- preview release scripts from `packages/vibe-app-tauri`
- production-candidate release scripts from `packages/vibe-app-tauri`
- Android APK artifact generation and GitHub Release packaging validation
- Happy-aligned visual parity review for shell, session, and promotion-critical secondary surfaces
- rollback drill documented and exercised at least once before the default switch

### Web And Browser Export

- retained static browser export validation from `packages/vibe-app-tauri`
- favicon, metadata, and browser-only affordance validation where the retained static export path
  uses them

## Promotion Gate

`packages/vibe-app-tauri` may take default release ownership only when:

- the project plan says Wave 9 parity is complete
- the route and capability matrix items are satisfied or explicitly waived in writing
- Happy-aligned UI and visual parity correction is complete for shell, session, and
  promotion-critical secondary surfaces, or approved exceptions are documented
- this file records the exact release-owner switch and rollback path
- the promotion module plan signs off the archival/reference-only policy for `packages/vibe-app`
