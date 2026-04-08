# Module Plan: vibe-app-tauri/universal-bootstrap-and-runtime

## Purpose

Turn `packages/vibe-app-tauri` into one app package that can host the Wave 8 desktop shell, Tauri
mobile ownership, and the retained browser build/export path without depending on `packages/vibe-app`
to boot.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `/root/happy/packages/happy-app/package.json`
- `/root/happy/packages/happy-app/index.ts`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/eas.json`
- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/theme.css`
- `/root/happy/packages/happy-app/sources/theme.ts`
- `/root/happy/packages/happy-app/sources/theme.gen.ts`
- `/root/happy/packages/happy-app/sources/unistyles.ts`
- `/root/happy/packages/happy-app/src-tauri/**`
- current `packages/vibe-app-tauri/package.json`
- current `packages/vibe-app-tauri/src-tauri/**`

## Wave 9 Canonical Inputs

- package-root runtime entrypoints for desktop, Android, and retained static browser export
- `packages/vibe-app-tauri/android/**` as the canonical Android native project path
- repository-owned Android native project and critical build inputs
- package-local browser build/export config
- package-local Tauri desktop/mobile config
- package-local release/dev scripts for the active Wave 9 replacement package

## Target Location

- `packages/vibe-app-tauri`
- `packages/vibe-app-tauri/android/**`
- package root bootstrap files
- mobile/browser runtime entrypoints
- desktop and mobile shell directories

## Responsibilities

- add Tauri-mobile-compatible and browser-build bootstrap ownership to the existing package
- keep Tauri desktop boot working
- define package-local scripts, env resolution, and build outputs
- establish the package-internal layout for `desktop`, `mobile`, and `shared` code
- preserve theme, font, splash, and provider bootstrap ownership from Happy
- ensure Android mobile runtime/build paths exist without relying on `packages/vibe-app`

## Non-Goals

- screen parity
- auth/session implementation
- release-owner switch

## Dependencies

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`

## Implementation Steps

1. Add package-root bootstrap files for Tauri desktop, Android Tauri-mobile ownership, and retained
   static browser export aligned to the new Wave 9 boundary.
2. Preserve the existing Tauri shell and keep its outputs package-local.
3. Define a stable package layout for shared core, mobile shell, and desktop shell code.
4. Port theme, font, splash, and provider bootstrap order from Happy's root layout and theme files.
5. Add package scripts for desktop dev/build/test, Android mobile build/dev flows, and retained
   static browser export validation.
6. Wire env/config resolution so preview and production modes are explicit.
7. Validate that the repository-owned Android native project and build path work without touching
   `packages/vibe-app`.

## Edge Cases And Failure Modes

- path alias collisions between desktop, mobile, and browser build paths
- bootstraps that still import assets or config from `packages/vibe-app`
- theme/provider ordering drift from Happy root layout semantics
- font or splash bootstrap timing regressions
- desktop and mobile scripts sharing output directories accidentally
- env resolution drift between desktop, mobile, and browser paths

## Tests

- package install verification
- `yarn workspace vibe-app-tauri tauri:dev` smoke check
- mobile bootstrap smoke check
- provider/theme/font/splash bootstrap smoke check
- Android mobile-path smoke check
- retained static browser export smoke check

## Acceptance Criteria

- `packages/vibe-app-tauri` can boot as desktop package, define Tauri mobile ownership, and produce
  the retained static browser export output
- Android native project ownership is explicit and repository-managed
- theme, font, splash, and provider bootstrap ownership are explicit and package-local
- scripts, envs, and outputs are package-local
- `packages/vibe-app` is no longer required to bootstrap the replacement package

## Locked Decisions

- keep the package path as `packages/vibe-app-tauri`
- preserve separate desktop and mobile shells inside one package
