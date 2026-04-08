# Wave 9 Unified Replacement Plan: `vibe-app-tauri`

## Purpose

Wave 9 upgrades `packages/vibe-app-tauri` from the historical Wave 8 desktop-preview baseline into
the active Wave 9 replacement package for `packages/vibe-app`.

This file is the execution-facing Wave 9 plan. Use it to decide:

- which batch owns which part of the replacement
- which Happy modules each batch must reference directly
- what counts as done before release ownership can move
- which gates must hold before `packages/vibe-app-tauri` can become the default app path

## Scope

Wave 9 covers all app ownership that historically lived in `packages/vibe-app` and now belongs in
the active Wave 9 replacement package:

- desktop app ownership
- Android app ownership
- retained static browser export ownership
- app config and env ownership
- updater, browser build/export, and store release ownership
- migration and rollback ownership

iOS remains deferred in the current Wave 9 scope and does not block promotion.

Android APK distribution through GitHub Releases is the primary mobile release path in the current
Wave 9 scope.

## Source Of Truth

### Primary Reference

- `/root/happy/packages/happy-app`

### Continuity And Rollback Reference

- `packages/vibe-app` as deprecated historical reference only when Happy is insufficient
- `.github/workflows/app-release.yml`
- `README.md`

### Companion Wave 9 Files

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `vibe-app-tauri-wave9-migration-and-release-plan.md`

## Delivery Rules

- plan from Happy modules first, not from the imported Vibe app tree
- keep `packages/vibe-app` out of active CI/release ownership; use it only as a deprecated reference when Happy is insufficient
- do not treat desktop-responsive web UI as mobile parity
- do not change server or wire behavior to simplify app migration
- move shared logic into `packages/vibe-app-tauri` first; do not create a new shared package by
  default
- UI and visual parity correction is an explicit Wave 9 task, not a post-parity polish pass
- if a user-visible surface ships with Happy-visible style drift, that Wave 9 task is not complete
- route and capability priority lives in `vibe-app-tauri-wave9-route-and-capability-matrix.md`;
  this file summarizes batches and gates but must not become a competing classification source

## Cross-Cutting Priority

### UI And Visual Parity Correction

Wave 9 explicitly includes correcting the current `vibe-app-tauri` style drift from Happy across
desktop, Android, and retained browser export surfaces.

This work is high priority because visual drift currently makes the replacement read as a different
product even when route coverage improves. For Wave 9, route migration without Happy-aligned visual
correction does not count as parity-complete.

## Route Priority Classes

### `P0 replacement-critical`

Must work before Wave 9 can claim a first usable replacement slice:

- `/(app)/index`
- `/(app)/restore/index`
- `/(app)/restore/manual`
- `/(app)/inbox/index`
- `/(app)/new/index`
- `/(app)/session/[id]`
- `/(app)/session/recent`
- `/(app)/settings/index`

### `P1 promotion-critical`

Must work before `packages/vibe-app-tauri` can stand as the default app path:

The authoritative `P1` list lives in `vibe-app-tauri-wave9-route-and-capability-matrix.md`. This
summary is for execution readability only and must be kept in sync with the matrix.

- `/(app)/session/[id]/message/[messageId]`
- `/(app)/session/[id]/info`
- `/(app)/session/[id]/files`
- `/(app)/session/[id]/file`
- `/(app)/settings/account`
- `/(app)/settings/appearance`
- `/(app)/settings/features`
- `/(app)/settings/language`
- `/(app)/settings/voice`
- `/(app)/settings/voice/language`
- `/(app)/settings/usage`
- `/(app)/settings/connect/claude`
- `/(app)/terminal/connect`
- `/(app)/terminal/index`
- `/(app)/artifacts/index`
- `/(app)/artifacts/new`
- `/(app)/artifacts/[id]`
- `/(app)/artifacts/edit/[id]`
- `/(app)/friends/index`
- `/(app)/friends/search`
- `/(app)/user/[id]`
- `/(app)/machine/[id]`
- `/(app)/server`
- `/(app)/changelog`
- `/(app)/text-selection`

### `P2 late or optional`

- `dev/**`
- other developer-only or low-value diagnostics surfaces

## Capability Priority Classes

The authoritative capability classification lives in
`vibe-app-tauri-wave9-route-and-capability-matrix.md`. The labels below define the gate meaning only;
they do not replace the matrix inventory.

### `C0 first-usable replacement`

- capabilities required for the first usable replacement slice across desktop, mobile, and retained
  browser runtime behavior

### `C1 promotion-critical`

- capabilities that must be implemented or explicitly waived before release ownership or default-path
  promotion moves

### `C2 deferred unless explicitly reactivated`

- capabilities that remain out of scope unless a later Wave 9 plan update reactivates them explicitly

## Batch Layout

### `B19`: Unified Runtime Bootstrap

Goal:

- make `packages/vibe-app-tauri` able to host the desktop shell, Android Tauri mobile ownership,
  and the retained static browser export path without Expo

Owning modules:

1. `modules/vibe-app-tauri/universal-bootstrap-and-runtime.md`

Primary Happy references:

- `/root/happy/packages/happy-app/package.json`
- `/root/happy/packages/happy-app/index.ts`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/eas.json`
- `/root/happy/packages/happy-app/src-tauri/**`

Output:

- package can boot Tauri desktop entrypoints, define Tauri mobile entry ownership, and produce the
  retained static browser export output
- scripts, env resolution, and build outputs are package-local

Gate:

- desktop boot, Android mobile runtime ownership, and static browser export all work without mutating
  `packages/vibe-app`

### `B20`: Shared Core Import From Happy

Goal:

- move the reusable Happy app logic into package-local shared modules

Owning modules:

1. `modules/vibe-app-tauri/shared-core-from-happy.md`

Primary Happy references:

- `/root/happy/packages/happy-app/sources/auth/**`
- `/root/happy/packages/happy-app/sources/sync/**`
- `/root/happy/packages/happy-app/sources/realtime/**`
- `/root/happy/packages/happy-app/sources/encryption/**`
- `/root/happy/packages/happy-app/sources/text/**`
- `/root/happy/packages/happy-app/sources/changelog/**`
- `/root/happy/packages/happy-app/sources/constants/**`
- `/root/happy/packages/happy-app/sources/utils/**`

Output:

- shared auth, sync, realtime, encryption, and text logic lives in `packages/vibe-app-tauri`

Gate:

- desktop and mobile can both consume the shared modules without importing screen-level RN code

### `B21`: Shell Surfaces, Browser Runtime, And Identity

Goal:

- stand up the web-native provider stack, retained static browser export path, desktop
  shell/adapters, Android mobile shell behavior under Tauri mobile, route shell, and account
  bootstrap flows
- correct shell-level visual drift so the replacement reads as Happy rather than as a Wave 8 review UI

Owning modules:

1. `modules/vibe-app-tauri/mobile-shell-and-navigation.md`
2. `modules/vibe-app-tauri/web-export-and-browser-runtime.md`
3. `modules/vibe-app-tauri/desktop-shell-and-platform-parity.md`
4. `modules/vibe-app-tauri/auth-and-identity-flows.md`

Primary Happy references:

- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/index.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/restore/index.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/restore/manual.tsx`
- `/root/happy/packages/happy-app/sources/components/MainView.tsx`
- `/root/happy/packages/happy-app/sources/components/SidebarNavigator.tsx`
- `/root/happy/packages/happy-app/package.json`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/sources/theme.css`
- `/root/happy/packages/happy-app/sources/unistyles.ts`
- `/root/happy/packages/happy-app/sources/auth/**`

Output:

- phone and tablet shells load on Android under the Tauri mobile boundary
- retained static browser export exists
- desktop shell, keyboard/focus behavior, and required desktop adapters are explicit in the active
  Wave 9 module set
- create-account, device-link, and secret-key restore flows work
- shell-level typography, grouped surfaces, brand assets, spacing, and navigation hierarchy are
  recognizably Happy-aligned

Gate:

- `P0 replacement-critical` entry flows are live across desktop and Android and the retained static
  browser export path is explicit
- obvious shell-level style drift from Happy is removed for the primary entry surfaces

### `B22`: Session Runtime And Storage

Goal:

- stand up shared session state, profile/bootstrap state, and realtime subscriptions

Owning modules:

1. `modules/vibe-app-tauri/session-runtime-and-storage.md`

Primary Happy references:

- `/root/happy/packages/happy-app/sources/sync/**`
- `/root/happy/packages/happy-app/sources/realtime/**`
- `/root/happy/packages/happy-app/sources/hooks/**`
- `/root/happy/packages/happy-app/sources/components/SessionsList.tsx`

Output:

- session list, profile state, server config, and realtime subscriptions are live

Gate:

- session inventory and bootstrap state load consistently on desktop and mobile

### `B23`: Session Rendering And Composer

Goal:

- port the message timeline, composer, and tool rendering surfaces
- correct session-surface visual drift so timeline, composer, and renderers stay in the Happy visual family

Owning modules:

1. `modules/vibe-app-tauri/session-rendering-and-composer.md`

Primary Happy references:

- `/root/happy/packages/happy-app/sources/-session/SessionView.tsx`
- `/root/happy/packages/happy-app/sources/components/MessageView.tsx`
- `/root/happy/packages/happy-app/sources/components/AgentInput.tsx`
- `/root/happy/packages/happy-app/sources/components/markdown/**`
- `/root/happy/packages/happy-app/sources/components/diff/**`
- `/root/happy/packages/happy-app/sources/components/tools/**`
- `/root/happy/packages/happy-app/sources/components/autocomplete/**`

Output:

- users can work inside real sessions on desktop and mobile
- session density, renderer styling, and composer treatment are recognizably Happy-aligned

Gate:

- one end-to-end session interaction chain works on both platform families
- session surfaces no longer depend on visibly non-Happy styling to remain usable

### `B24`: Native Capability Replacement

Goal:

- close the platform capability gaps that still block parity or release ownership

Owning modules:

1. `modules/vibe-app-tauri/mobile-native-capabilities.md`

Primary Happy references:

- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/sources/components/qr/**`
- `/root/happy/packages/happy-app/sources/realtime/RealtimeVoiceSession.tsx`
- `/root/happy/packages/happy-app/sources/sync/pushRegistration.ts`
- `/root/happy/packages/happy-app/sources/sync/purchases.ts`
- `/root/happy/packages/happy-app/sources/sync/revenueCat/**`
- `/root/happy/packages/happy-app/sources/utils/notificationRouting.ts`
- `/root/happy/packages/happy-app/sources/utils/requestReview.ts`
- `/root/happy/packages/happy-app/sources/utils/microphonePermissions.ts`

Output:

- notifications, purchases, voice, camera/QR, sharing, and related native flows are explicit and
  testable under the pure Tauri mobile/runtime direction

Gate:

- every `C1 promotion-critical` capability has either an implementation or an explicit documented
  deferral approved in the plan

### `B25`: Secondary Routes And Social Surfaces

Goal:

- close the non-core but user-visible route families
- close the remaining user-visible visual parity gaps on `P1` routes while wiring those route families

Owning modules:

1. `modules/vibe-app-tauri/secondary-routes-and-social.md`

Primary Happy references:

- `/root/happy/packages/happy-app/sources/app/(app)/artifacts/**`
- `/root/happy/packages/happy-app/sources/app/(app)/friends/**`
- `/root/happy/packages/happy-app/sources/app/(app)/user/**`
- `/root/happy/packages/happy-app/sources/app/(app)/machine/**`
- `/root/happy/packages/happy-app/sources/app/(app)/settings/**`
- `/root/happy/packages/happy-app/sources/app/(app)/terminal/**`
- `/root/happy/packages/happy-app/sources/app/(app)/changelog.tsx`
- `/root/happy/packages/happy-app/sources/components/InboxView.tsx`
- `/root/happy/packages/happy-app/sources/components/SettingsView.tsx`

Output:

- all required `P1 promotion-critical` routes are wired
- all required `P1 promotion-critical` routes are wired with Happy-aligned visual hierarchy and styling

Gate:

- no user-visible promotion-critical route remains missing
- no user-visible promotion-critical route remains obviously style-divergent from Happy without a
  documented exception

### `B26`: Release, OTA, And Store Migration

Goal:

- finalize app release ownership inside `packages/vibe-app-tauri` for desktop, Android, and retained
  static browser export outputs without Expo/EAS as the primary toolchain

Owning modules:

1. `modules/vibe-app-tauri/release-ota-and-store-migration.md`

Primary Happy references:

- `/root/happy/packages/happy-app/release.cjs`
- `/root/happy/packages/happy-app/release-dev.sh`
- `/root/happy/packages/happy-app/release-production.sh`
- `/root/happy/packages/happy-app/eas.json`
- `/root/happy/packages/happy-app/app.config.js`

Output:

- `packages/vibe-app-tauri` can produce desktop, Android APK, and retained static browser export
  release outputs

Gate:

- preview and production release lanes are explicit, reproducible, and rollback-safe

### `B27`: Promotion And `vibe-app` Deprecation

Goal:

- switch the default app path only after parity and default release ownership are both proven

Owning modules:

1. `modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md`

Output:

- explicit production switch, hold/rollback, and legacy-reference notes exist

Gate:

- `packages/vibe-app-tauri` is approved as the default app path

## Cross-Platform Validation Matrix

### Desktop

- Linux startup and core session validation
- macOS startup and core session validation
- Windows startup and core session validation

### Mobile

- one Android real-device pass for auth, restore, session load, message send, notification, and QR

### Shared Core

- parser/reducer/auth/realtime unit tests
- config/env resolution checks
- no shared-core imports of RN screen modules

### Web Export

- retained static browser export validation

## Completion Rule

Wave 9 is complete only when:

- `packages/vibe-app-tauri` holds default release ownership for the full app path
- the route and capability matrix is satisfied
- shell, session, and promotion-critical secondary surfaces have completed Happy-aligned UI and
  visual parity correction or have documented approved exceptions
- migration and rollback documents are signed off
- `packages/vibe-app-tauri` is the default app path and active Wave 9 replacement package, while
  `packages/vibe-app` remains only as a deprecated historical reference
