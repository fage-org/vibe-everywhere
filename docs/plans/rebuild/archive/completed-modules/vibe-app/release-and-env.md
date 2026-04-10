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
- prevent the imported app from silently shipping against legacy Happy-owned Expo/EAS/Firebase or
  store metadata

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
3. Move Expo/EAS ownership, update URLs, and native-service file selection behind explicit Vibe env
   variables instead of defaulting to legacy Happy infrastructure.
4. Make store auto-submit opt-in so local release scripts do not target stale App Store / Play
   metadata by accident.
5. Document required env variables and release profiles, including the server-side
   `VIBE_IOS_STORE_URL` and `VIBE_ANDROID_STORE_URL` required by version-check/update prompts.
6. Add build smoke tests for at least one dev profile.

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
- Expo/EAS ownership metadata, update URLs, and Google services config must be env-driven; Wave 6
  must not hardcode the legacy Happy release infrastructure as the default Vibe path
- release scripts may support store auto-submit, but only through explicit Vibe opt-in variables
- finalize release/env behavior only after endpoint, branding, and desktop adaptation are stable
