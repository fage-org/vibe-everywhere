# Project Plan: vibe-app-tauri

## Purpose

`vibe-app-tauri` starts from the historical Wave 8 desktop-preview baseline and becomes the active
Wave 9 replacement package for `packages/vibe-app`.

The Wave 9 target is not "desktop plus some mobile support." The target is one future app project
that owns:

- desktop runtime and desktop UI
- Android runtime and Android UI
- retained static browser export ownership where the current app still publishes web assets
- app release, update, store, and migration ownership now held by `packages/vibe-app`

iOS is explicitly deferred in the current Wave 9 scope. Do not treat iOS runtime, store, or release
work as promotion-blocking unless a later plan update reactivates it.

Android distribution is APK-first through GitHub Releases in the current Wave 9 scope.

## Current State

- `packages/vibe-app` is now deprecated from active CI and release ownership and remains only as a
  legacy Vibe-specific reference when `/root/happy/packages/happy-app` cannot answer a continuity question.
- `packages/vibe-app-tauri` already exists as the historical desktop-preview package from Wave 8.
- Wave 9 keeps the deprecated legacy package untouched while turning `packages/vibe-app-tauri` into the
  active Wave 9 replacement package and eventual default app path.

## Wave 9 Terms

- `historical Wave 8 desktop-preview baseline`: the closed desktop-only phase that produced the
  starting `packages/vibe-app-tauri` package and related historical planning artifacts
- `active Wave 9 replacement package`: `packages/vibe-app-tauri` while it is the active full-app
  replacement target for `packages/vibe-app`
- `default app path`: the app package that becomes the primary maintained and user-facing path after
  promotion
- `default release ownership`: the package and workflow set that owns primary release lanes after
  promotion
- `historical continuity reference`: a deprecated package or old planning file used only when Happy
  and the active Wave 9 planning set cannot answer a continuity question

## Wave 9 Scope Decision

Wave 9 explicitly changes the project boundary:

- Wave 8: parallel desktop rewrite only
- Wave 9: active full-platform replacement of `packages/vibe-app` using Tauri as the app runtime
  boundary across desktop and mobile, plus a retained browser build/export path

That means the project is no longer desktop-only. It now owns the planning path for desktop,
mobile, retained browser build/export, and release migration.

## Source Of Truth

### Primary Product Reference

Use `/root/happy/packages/happy-app` directly for product behavior, module boundaries, route
families, release structure, and platform capability ownership.

Do not inherit Happy's Expo / React Native / EAS tooling boundary as an implementation requirement.
Wave 9 uses Happy as a product and behavior reference, not as a mandate to keep Expo as the runtime
host.

### Continuity References

Use the following only for Vibe-specific continuity when Happy is insufficient, not for new ownership decisions:

- `packages/vibe-app`
- `.github/workflows/app-release.yml`
- `README.md`
- existing Wave 8 planning files under `docs/plans/rebuild/`

### Shared Contracts

- `crates/vibe-wire`
- `crates/vibe-server`
- `crates/vibe-agent`
- `crates/vibe-cli`

### Active Wave 9 Planning Inputs

- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`

### Historical Wave 8 Continuity References

- historical: `docs/plans/rebuild/vibe-app-tauri-wave8-delivery-plan.md`
- historical: `docs/plans/rebuild/vibe-app-tauri-extraction-inventory.md`
- historical: `docs/plans/rebuild/vibe-app-tauri-route-inventory.md`
- historical: `docs/plans/rebuild/vibe-app-tauri-capability-matrix.md`
- historical: `docs/plans/rebuild/vibe-app-tauri-coexistence-matrix.md`
- historical: `docs/plans/rebuild/vibe-app-tauri-promotion-baseline.md`
- historical: `docs/plans/rebuild/vibe-app-tauri-promotion-plan.md`

These Wave 8 documents are historical reference material only. They may inform continuity checks,
but they do not define the active Wave 9 boundary, priority classes, or promotion gate.

## Locked Decisions

1. Keep the package path as `packages/vibe-app-tauri`.
   - the name is historical now, but the package path remains stable
   - do not create a second replacement package just to get a cleaner name
2. Use `/root/happy/packages/happy-app` as the direct planning reference.
   - do not re-derive product ownership from `packages/vibe-app`
   - use `packages/vibe-app` only when Happy is insufficient for a Vibe-specific continuity check
3. Shared logic lands in `packages/vibe-app-tauri` first.
   - do not create a new shared `vibe-app-core` package by default
   - promote extracted modules later only after replacement parity is proven
4. Wave 9 uses a pure Tauri runtime boundary for app shells.
   - desktop remains Tauri 2 + web-native
   - Android moves to Tauri mobile + web-native ownership in the active scope
   - iOS is deferred until a later plan update explicitly reactivates it
   - retained static browser export remains an explicit build target
   - do not reintroduce Expo, React Native shell ownership, or EAS as the primary app boundary
5. `packages/vibe-app` is deprecated immediately at the CI/release layer.
   - keep it only as a legacy Vibe-specific reference when Happy is insufficient
   - do not re-enable old pipelines unless a later plan update explicitly authorizes that reversal
6. Parity first, redesign later.
   - do not invent a new information architecture during the migration
   - preserve Happy/Vibe route semantics, major interaction flows, and capability ownership first
7. Shared contracts stay upstream.
   - any protocol-shape changes still go through `shared/*.md` and `crates/vibe-wire` first
   - the app replacement must not introduce package-local protocol forks

## Target Layout

- package: `packages/vibe-app-tauri`
- expected top-level ownership:
  - Tauri bootstrap and desktop runtime ownership
  - Android Tauri mobile bootstrap and mobile runtime ownership
  - retained static browser export ownership
  - package-local shared auth/sync/realtime/encryption/text/utils/core modules
  - mobile route tree and screen ownership through the web-native app shell
  - desktop route tree and screen ownership
  - release scripts, Android APK distribution, retained static export, and store submission ownership

Recommended internal layout direction:

```text
packages/vibe-app-tauri/
  index.ts
  browser-build.config.*
  android/
  release-dev.sh
  release-production.sh
  sources/
    app/
    shared/
    mobile/
    desktop/
    browser/
    assets/
    modal/
    hooks/
  src-tauri/
```

The exact folder split may evolve, but the boundary rule is fixed: shared core can be shared,
platform shells cannot be collapsed into one lowest-common-denominator UI layer.

## Android Scope Lock

- Android is the only active mobile platform in Wave 9.
- Android native project files and critical build inputs are treated as first-class, repository-owned
  sources rather than disposable generated output.
- GitHub Releases is the default Android distribution path, with APK artifacts as the primary mobile
  release output.
- iOS remains deferred and must not appear in Wave 9 promotion gates unless explicitly reactivated.

## Non-Goals

- changing backend routes, wire contracts, or account/session semantics to fit the new app
- deleting `packages/vibe-app` before the replacement is promoted
- using Wave 9 as a justification for a broad product redesign
- inventing new mobile-only or desktop-only features unrelated to Happy/Vibe parity
- solving every possible cross-app abstraction before a working replacement exists

## Happy Module Reference Inventory

Wave 9 should plan directly against the Happy app modules below.

### Bootstrap, Config, And Release

- `/root/happy/packages/happy-app/package.json`
- `/root/happy/packages/happy-app/index.ts`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/eas.json`
- `/root/happy/packages/happy-app/release.cjs`
- `/root/happy/packages/happy-app/release-dev.sh`
- `/root/happy/packages/happy-app/release-production.sh`
- `/root/happy/packages/happy-app/src-tauri/**`

### Route And Provider Shell

- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/**`
- `/root/happy/packages/happy-app/sources/components/MainView.tsx`
- `/root/happy/packages/happy-app/sources/components/SidebarNavigator.tsx`
- `/root/happy/packages/happy-app/sources/components/navigation/Header.tsx`

### Auth, State, And Realtime

- `/root/happy/packages/happy-app/sources/auth/**`
- `/root/happy/packages/happy-app/sources/sync/**`
- `/root/happy/packages/happy-app/sources/realtime/**`
- `/root/happy/packages/happy-app/sources/encryption/**`
- `/root/happy/packages/happy-app/sources/hooks/**`
- `/root/happy/packages/happy-app/sources/utils/**`

### Session And Rendering Surfaces

- `/root/happy/packages/happy-app/sources/-session/SessionView.tsx`
- `/root/happy/packages/happy-app/sources/components/SessionsList.tsx`
- `/root/happy/packages/happy-app/sources/components/MessageView.tsx`
- `/root/happy/packages/happy-app/sources/components/AgentInput.tsx`
- `/root/happy/packages/happy-app/sources/components/markdown/**`
- `/root/happy/packages/happy-app/sources/components/diff/**`
- `/root/happy/packages/happy-app/sources/components/tools/**`
- `/root/happy/packages/happy-app/sources/components/autocomplete/**`

### Secondary Surfaces, Text, And Assets

- `/root/happy/packages/happy-app/sources/components/InboxView.tsx`
- `/root/happy/packages/happy-app/sources/components/SettingsView.tsx`
- `/root/happy/packages/happy-app/sources/changelog/**`
- `/root/happy/packages/happy-app/sources/text/**`
- `/root/happy/packages/happy-app/sources/constants/**`
- `/root/happy/packages/happy-app/sources/assets/**`
- `/root/happy/packages/happy-app/sources/modal/**`
- `/root/happy/packages/happy-app/sources/track/**`

## Workstreams

### 1. Universal Runtime Bootstrap

Make one package able to host:

- the retained desktop shell lineage that started in Wave 8
- a Tauri mobile shell on Android
- retained static browser export configuration
- one release/config surface
- shared theme/font/splash/provider bootstrap ownership

### 2. Shared Core From Happy

Port the reusable non-visual logic from Happy into package-local shared modules:

- auth
- sync
- realtime
- encryption
- text
- changelog
- constants
- selected utilities and hooks

### 3. Mobile Shell And Navigation

Recreate the Happy app's provider stack, route tree, phone/tablet navigation rules, and header or
drawer behavior inside `packages/vibe-app-tauri` using the shared web-native app shell rendered
inside Tauri mobile.

### 4. Static Browser Export

Preserve the retained static browser export path as an explicit ownership area instead of treating
web/export as a side effect of mobile tooling.

### 5. Identity And Session Bootstrap

Port:

- account creation
- QR/device linking
- secret-key restore
- credential persistence
- session bootstrap
- profile/bootstrap fetch chains

### 6. Session Runtime And Rendering

Port:

- session list
- session detail
- realtime updates
- message rendering
- composer behavior
- tool/diff/markdown/file rendering

### 7. Native Capability Replacement

Keep desktop and mobile parity explicit for:

- deep links and callback ownership
- secure storage
- camera and QR
- notifications and push routing
- purchases and entitlement refresh
- microphone and voice flows
- file import/export/share
- haptics and platform review prompts

### 8. Secondary Routes And Social Surfaces

Port the remaining user-visible route families and settings detail pages without broad redesign.

### 9. Release, OTA, Store, And Promotion Migration

Keep all new app-release ownership in `packages/vibe-app-tauri`; `packages/vibe-app` remains
reference-only while the Wave 9 replacement package proves runtime and route parity.

## Phase Plan

## Phase 0: Planning Reset

### Goal

Turn the project from a desktop-only rewrite into a full replacement plan.

### Required Outputs

- this project plan
- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- the Wave 9 module plan set under `docs/plans/rebuild/modules/vibe-app-tauri/`

### Gate

- route families, capability owners, release migration rules, and promotion gates are explicit

## Phase 1: Universal Runtime Bootstrap

### Goal

Make `packages/vibe-app-tauri` runnable as:

- desktop preview via Tauri
- Tauri mobile app via Android runtime
- retained static browser export source package

### Acceptance

- `tauri dev` works
- Android Tauri mobile bootstrap and native build path are explicit and validated
- retained static browser export path is explicit and validated
- iOS is recorded as deferred instead of being left as an implied future requirement

## Phase 2: Shared Core Import

### Goal

Port Happy app state and domain modules into package-local shared modules.

### Acceptance

- shared auth/sync/realtime logic compiles without UI-host screen imports
- desktop and Android can both consume the shared modules

## Phase 3: Mobile Shell And Identity

### Goal

Stand up the provider stack, route tree, auth bootstrap, and restore flows for mobile.

### Acceptance

- Android reaches the main authenticated shell
- create-account, device-link, and secret-key restore work

## Phase 4: Session Runtime And Rendering

### Goal

Port the session-heavy core experience.

### Acceptance

- users can open sessions, receive realtime updates, send messages, and render tool output on
  mobile and desktop

## Phase 5: Native Capability And Secondary Surface Completion

### Goal

Close the capability and route gaps that block full replacement.

### Acceptance

- required P1 routes work
- retained static browser export capability works
- required platform capabilities work on real devices

## Phase 6: Release Migration And Promotion

### Goal

Complete release and store ownership in `packages/vibe-app-tauri`, then formalize `packages/vibe-app` as a reference-only legacy path.

### Acceptance

- `packages/vibe-app-tauri` holds default release ownership for desktop/Android APK/static-export lanes
- explicit hold/rollback plans remain documented
- `packages/vibe-app` remains reference-only and must not regain active pipeline ownership without a new plan update

## Recommended Module Plan Breakdown

Wave 8 desktop module plans remain historical reference material for continuity only. New execution
work should follow the Wave 9 module set below and update those files first when reality changes.

Wave 9 work should be executed through the following module plans:

1. `modules/vibe-app-tauri/universal-bootstrap-and-runtime.md`
2. `modules/vibe-app-tauri/shared-core-from-happy.md`
3. `modules/vibe-app-tauri/mobile-shell-and-navigation.md`
4. `modules/vibe-app-tauri/web-export-and-browser-runtime.md`
5. `modules/vibe-app-tauri/desktop-shell-and-platform-parity.md`
6. `modules/vibe-app-tauri/auth-and-identity-flows.md`
7. `modules/vibe-app-tauri/session-runtime-and-storage.md`
8. `modules/vibe-app-tauri/session-rendering-and-composer.md`
9. `modules/vibe-app-tauri/mobile-native-capabilities.md`
10. `modules/vibe-app-tauri/secondary-routes-and-social.md`
11. `modules/vibe-app-tauri/release-ota-and-store-migration.md`
12. `modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md`

No single implementation task should attempt the full migration.

## Validation Strategy

### Required During Migration

- package-level `typecheck`, `test`, `tauri:test`, and `tauri:smoke`
- Android Tauri mobile bootstrap validation and Android native build/dev wiring validation
- retained static browser export smoke validation
- shared-core unit tests for auth/sync/realtime/parser utilities
- route-level smoke checks for mobile and desktop entry flows
- desktop shell checks for keyboard/focus, modal/overlay, clipboard, and required file-dialog flows
- one real backend chain for auth, bootstrap, session load, message send, and realtime receipt
- release-script dry runs and config validation before ownership switches

### Required Before Promotion

- side-by-side parity review against Happy behavior for:
  - home / restore / inbox / new session / session detail / settings
  - message rendering, composer, tool rendering, diff rendering, and file rendering
  - notifications, QR/device-link, voice, purchases, and file import/export where applicable
  - desktop keyboard/focus, modal, clipboard, and required file-dialog semantics
- cross-platform startup validation on Linux, macOS, and Windows
- real-device validation on at least one Android target
- retained static browser export validation
- explicit release migration sign-off for bundle identifiers, updater policy, Android APK
  distribution, and
  rollback paths

## Key Risks

- treating the Wave 8 desktop shell as if it can be reused directly for mobile UI
- underestimating how much Happy behavior lives in Android-native capabilities and platform-specific
  host integration
- drifting from Happy route semantics while trying to "clean up" the app structure
- breaking store, OTA, secure-storage, or purchase continuity during release migration
- broad AI prompts that rewrite shared core, mobile shell, and release scripts all at once

## Acceptance Criteria

- `packages/vibe-app-tauri` can act as the active Wave 9 replacement package for `packages/vibe-app`
  across desktop, Android, and retained static browser export ownership without protocol forks
- the replacement follows Happy behavior first and records any allowed Vibe deviations explicitly
- release, OTA, store, and rollback behavior are documented before the default switch
- `packages/vibe-app` remains a deprecated historical reference only; it is not part of the active pipeline baseline
