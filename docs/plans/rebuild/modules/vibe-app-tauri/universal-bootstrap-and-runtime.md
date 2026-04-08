# Module Plan: vibe-app-tauri/universal-bootstrap-and-runtime

## Purpose

Turn `packages/vibe-app-tauri` into one app package that can host the Wave 8 desktop shell and a
new Expo mobile shell without depending on `packages/vibe-app` to boot.

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

## Target Location

- `packages/vibe-app-tauri`
- package root bootstrap files
- Expo runtime entrypoints
- desktop and mobile shell directories

## Responsibilities

- add Expo-compatible bootstrap ownership to the existing package
- keep Tauri desktop boot working
- define package-local scripts, env resolution, and build outputs
- establish the package-internal layout for `desktop`, `mobile`, and `shared` code
- preserve theme, font, splash, and provider bootstrap ownership from Happy
- ensure Android and iOS prebuild paths exist without relying on `packages/vibe-app`

## Non-Goals

- screen parity
- auth/session implementation
- release-owner switch

## Dependencies

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`

## Implementation Steps

1. Add package-root Expo bootstrap files aligned to Happy's package structure.
2. Preserve the existing Tauri shell and keep its outputs package-local.
3. Define a stable package layout for shared core, mobile shell, and desktop shell code.
4. Port theme, font, splash, and provider bootstrap order from Happy's root layout and theme files.
5. Add package scripts for desktop dev/build/test and Expo start/prebuild/test flows.
6. Wire env/config resolution so preview and production modes are explicit.
7. Validate that Android and iOS prebuild can run without touching `packages/vibe-app`.

## Edge Cases And Failure Modes

- path alias collisions between Vite/Tauri and Expo
- bootstraps that still import assets or config from `packages/vibe-app`
- theme/provider ordering drift from Happy root layout semantics
- font or splash bootstrap timing regressions
- desktop and mobile scripts sharing output directories accidentally
- env resolution drift between desktop and Expo paths

## Tests

- package install verification
- `yarn workspace vibe-app-tauri tauri:dev` smoke check
- Expo bootstrap smoke check
- provider/theme/font/splash bootstrap smoke check
- Android prebuild smoke check
- iOS prebuild smoke check

## Acceptance Criteria

- `packages/vibe-app-tauri` can boot as desktop and Expo/mobile package
- theme, font, splash, and provider bootstrap ownership are explicit and package-local
- scripts, envs, and outputs are package-local
- `packages/vibe-app` is no longer required to bootstrap the replacement package

## Locked Decisions

- keep the package path as `packages/vibe-app-tauri`
- preserve separate desktop and mobile shells inside one package
