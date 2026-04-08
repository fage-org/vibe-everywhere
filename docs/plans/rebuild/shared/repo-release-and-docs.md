# Shared Follow-Up Plan: repo-release-and-docs

## Archival Status

This file is a historical repository-level snapshot.

It records the release and documentation model that existed before the active Wave 9
`packages/vibe-app-tauri` ownership transition. Do not use it as the current source of authority for
release ownership. Current authority lives in:

- root `README.md`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/workflows/app-release.yml`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/release-ota-and-store-migration.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md`

## Purpose

Capture the historical non-blocking repository-level follow-up that landed after the rebuild baseline:

- GitHub Actions coverage for the Rust workspace and imported app validation
- tag-driven GitHub release packaging for the Rust binaries
- a dedicated app-release workflow for web, desktop, and Android packaging at that time
- a root README that explains deployment and day-to-day usage instead of only pointing at plans
- root-level license attribution noting the Happy-aligned origin of the rebuild

This work is packaging/documentation infrastructure. It must not reopen completed wave module scope
or change locked runtime contracts.

## Current Repository Reality

- Rust CI and Rust release packaging remain active.
- The current app release workflow packages `packages/vibe-app-tauri`.
- Deprecated `packages/vibe-app` app release lanes are disabled from the active workflow.
- Treat the decisions below as historical context for continuity review only.

## Scope

In scope:

- root `.github/workflows/*`
- root `README.md`
- root `LICENSE`
- root packaging metadata needed to make tag-driven releases predictable
- historical app-release automation that kept `packages/vibe-app` as the owning package for Expo/EAS, web,
  Android, and the then-shipping desktop path
- a historical non-default `packages/vibe-app-tauri` desktop packaging lane within the shared app-release
  workflow while the rewrite coexisted with `packages/vibe-app`

Out of scope:

- iOS release automation
- moving mobile, web, or the then-default desktop release flows out of `packages/vibe-app`
- redesigning crate or package boundaries
- changing server, CLI, agent, wire, or app protocol behavior

## Decisions

1. CI remains split by toolchain:
   - Rust workspace validation runs with stable Rust
   - app validation runs with Node `24.14.0` and Yarn `1.22.22`
2. GitHub Releases package the Rust binaries only:
   - `vibe`
   - `vibe-agent`
   - `vibe-server`
   - `vibe-app-logs`
3. Historical app packaging used a dedicated GitHub Actions workflow:
   - `packages/vibe-app` remains the owning package for web export, Android packaging, and the
     current default desktop release path
   - web export is built locally on GitHub Actions and uploaded as an artifact
   - desktop bundles for the shipping app are built locally with Tauri on Linux, macOS, and Windows
   - `packages/vibe-app-tauri` may add a non-default desktop packaging lane within the shared
     app-release workflow, with distinct artifacts and channels while coexistence rules remain in
     force
   - Android builds run on the GitHub Actions runner via `expo prebuild --platform android`
     followed by `./gradlew app:bundleRelease app:assembleRelease`, so the workflow emits both AAB
     and APK artifacts without relying on EAS cloud timeout limits or the local EAS wrapper
     overhead
   - manual packaging dispatch may validate and package `packages/vibe-app` and
     `packages/vibe-app-tauri` desktop artifacts in parallel as long as the resulting artifacts
     stay clearly separated and the default shipping path does not change implicitly
4. App-release tags use `app-v*` so the shipping app packaging stays independent from Rust binary
   release tags.
5. Any `vibe-app-tauri` desktop packaging lane must stay distinguishable from the shipping
   `packages/vibe-app` release lane until promotion updates the coexistence rules.
6. Release publishing is tag-driven with `vX.Y.Z` for Rust binaries and `app-v*` for shipping app
   assets.
7. Workspace versioning is centralized at the root `Cargo.toml` so the release tag can be checked
   against a single version source.
8. Root documentation should emphasize:
   - local deployment
   - self-hosting assumptions
   - CLI/app usage flow
   - release workflow expectations

## Implementation Order

1. Add the repository-level follow-up plan.
2. Normalize Rust crate package metadata so version checks can read from the workspace root.
3. Add GitHub Actions CI for Rust and app validation.
4. Add tag-triggered GitHub release packaging for Rust binaries.
5. Add dedicated app packaging automation for web, desktop, and Android.
6. Rewrite the root README around deployment, operations, and usage.
7. Extend the root MIT license with a Happy-origin attribution notice.

## Acceptance Criteria

- historically, push/PR CI validated the Rust workspace and `packages/vibe-app`
- a `vX.Y.Z` tag can produce a GitHub Release with packaged Rust binaries and checksums
- historically, an `app-v*` tag or manual dispatch could package `packages/vibe-app` for web, desktop, and Android
- Android packaging produces both `.aab` and `.apk` artifacts for the shipping app workflow
- any `vibe-app-tauri` desktop packaging lane remains clearly non-default inside the shared
  app-release workflow and does not replace the shipping `packages/vibe-app` release lane without
  a promotion-plan update
- Android packaging no longer depends on EAS cloud build completion to produce an artifact
- the root README documents deployment prerequisites, local bring-up, runtime env vars, and release
  usage
- the root license keeps MIT terms intact while recording the Happy-aligned origin of imported or
  adapted material
