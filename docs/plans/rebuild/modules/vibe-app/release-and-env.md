# Module Plan: vibe-app/release-and-env

## Purpose

Align the imported app's release, environment, and packaging behavior with the Vibe repository.

## Happy Source Of Truth

- `packages/happy-app/package.json`
- `packages/happy-app/release.cjs`
- `packages/happy-app/release-*.sh`
- Happy root workspace scripts relevant to the app

## Target Rust/Vibe Location

- package: `packages/vibe-app`
- expected edit areas:
  - package scripts
  - release helpers
  - environment config files

## Responsibilities

- replace Happy-specific release naming and environment assumptions
- define Vibe env tiers and script entrypoints
- document app release prerequisites

## Non-Goals

- CI/release automation for the entire repo outside app scope

## Public Types And Interfaces

- package scripts
- app env variable surface
- release profile names

## Data Flow

- developer selects env/profile
- scripts produce Vibe-branded app builds using Vibe endpoints and identifiers

## Dependencies

- `import-and-build`
- `branding-and-naming-adaptation`
- `api-endpoint-adaptation`
- `desktop-tauri-adaptation`

## Implementation Steps

1. Audit imported scripts for Happy-specific names and paths.
2. Replace them with Vibe script names and env conventions.
3. Document required env variables and release profiles.
4. Add build smoke tests for at least one dev profile.

## Edge Cases And Failure Modes

- profile naming drift across web/mobile/desktop
- script paths still pointing at Happy root files

## Tests

- script resolution tests
- env loading tests
- dev build smoke test

## Acceptance Criteria

- app release/dev scripts operate under Vibe naming and env conventions

## Open Questions

- None.

## Locked Decisions

- keep release/env logic local to the app package
- use `EXPO_PUBLIC_VIBE_*` for app-public runtime variables exposed to Expo/web/JS code
- use `VIBE_` for non-public build, native, and release-only app variables
- finalize release/env behavior only after endpoint, branding, and desktop adaptation are stable
