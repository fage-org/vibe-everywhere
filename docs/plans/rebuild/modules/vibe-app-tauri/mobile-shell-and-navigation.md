# Module Plan: vibe-app-tauri/mobile-shell-and-navigation

## Purpose

Recreate the Happy mobile provider stack, route tree, and phone/tablet shell behavior inside
`packages/vibe-app-tauri` using the shared web-native shell rendered through Tauri mobile.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/index.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/inbox/index.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/settings/index.tsx`
- `/root/happy/packages/happy-app/sources/components/MainView.tsx`
- `/root/happy/packages/happy-app/sources/components/SidebarNavigator.tsx`
- `/root/happy/packages/happy-app/sources/components/navigation/Header.tsx`

## Target Location

- `packages/vibe-app-tauri/sources/app/**`
- `packages/vibe-app-tauri/sources/mobile/**`

## Responsibilities

- root provider chain
- mobile route ownership through the package-local web-native router
- phone and tablet shell behavior
- headers, drawer/sidebar, and top-level navigation affordances
- major `P0` entry routes

## Non-Goals

- full session rendering parity
- release migration
- native capability implementation beyond what bootstrapping requires

## Dependencies

- `universal-bootstrap-and-runtime`
- `shared-core-from-happy`

## Implementation Steps

1. Recreate the Happy provider stack and init order in the new package.
2. Port the top-level route tree and route naming semantics.
3. Recreate phone/tablet navigation behavior and shell structure.
4. Port the main/home, inbox, and settings-hub entry surfaces.
5. Validate that mobile entry flows feel like Happy before deeper feature work starts.

## Edge Cases And Failure Modes

- provider ordering drift that breaks auth or realtime state
- tablet and phone behavior collapsing into one incorrect layout
- route names drifting from Happy semantics
- font or safe-area boot timing differences causing broken first paint

## Tests

- route smoke tests for `P0` entry routes
- provider boot smoke checks
- phone and tablet layout validation

## Acceptance Criteria

- Android can boot into a recognizable Happy-aligned shell
- `P0` entry routes exist and are navigable
- route and shell semantics are stable enough for auth and session module work

## Locked Decisions

- mobile shell is reconstructed from Happy directly
- mobile shell stays on the shared web-native app boundary under Tauri mobile; do not reintroduce
  Expo or a React Native shell as the runtime host
- mobile shell visuals and hierarchy must remain governed by
  `docs/plans/rebuild/shared/ui-visual-parity.md` unless a narrower exception is recorded first
