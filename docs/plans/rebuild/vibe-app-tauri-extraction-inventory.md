# `vibe-app-tauri` Extraction Inventory

## Purpose

Freeze the reusable-vs-rewrite boundary before `vibe-app-tauri` implementation broadens.

This document is the planning source of truth for what is copied, adapted behind platform seams,
rewritten for desktop, or explicitly deferred.

## Status

- state: `planning baseline`
- update rule: revise this file before any `vibe-app-tauri` module changes the expected ownership of
  a source area

## Classification Rules

- `reusable as-is`: pure TypeScript or portable logic can move package-local without new platform
  seams
- `adapter required`: logic is reusable only after React Native / Expo assumptions are isolated
- `desktop rewrite`: new desktop-web implementation is required; old code is parity reference only
- `deferred`: not part of phase-one desktop scope unless a plan update activates it

## Inventory

| Source area | Initial classification | First owning module | Notes |
| --- | --- | --- | --- |
| `packages/vibe-app/sources/sync/**` | adapter required | `core-logic-extraction` | reducers, parsers, and storage seams must be detached from Expo/runtime assumptions explicitly |
| `packages/vibe-app/sources/auth/**` | adapter required | `core-logic-extraction`, `auth-and-session-state` | auth semantics are reused, but storage and callback handling move behind desktop adapters |
| `packages/vibe-app/sources/realtime/**` | adapter required | `auth-and-session-state` | only the desktop-needed subscription path is in phase-one scope |
| `packages/vibe-app/sources/encryption/**` | reusable as-is | `core-logic-extraction` | keep package-local unless later promotion is explicitly approved |
| `packages/vibe-app/sources/utils/**` | mixed: reusable as-is or adapter required | `core-logic-extraction`, `desktop-platform-adapters` | copy only pure TS helpers directly; rewrite platform helpers behind adapters |
| `packages/vibe-app/sources/text/**` | reusable as-is | `core-logic-extraction` | translations may be mirrored package-local while the old app remains untouched |
| `packages/vibe-app/sources/constants/**` | mixed: reusable as-is or deferred | `core-logic-extraction`, `desktop-shell-and-routing` | avoid carrying forward Expo/mobile-only constants |
| `packages/vibe-app/sources/assets/**` | mixed: reusable as-is or desktop rewrite | `bootstrap-and-package`, `desktop-shell-and-routing`, `secondary-surfaces` | copy/package-local fonts, icons, images, and animations that are required for desktop parity; regenerate only when packaging or web constraints make direct reuse impractical |
| `packages/vibe-app/sources/hooks/**` | mixed: adapter required or desktop rewrite | `core-logic-extraction`, `session-ui-parity` | hooks with RN coupling should not be ported blindly |
| `packages/vibe-app/sources/components/**` | desktop rewrite | `desktop-shell-and-routing`, `session-ui-parity`, `secondary-surfaces` | preserve behavior and hierarchy, but rebuild with desktop-web primitives |
| `packages/vibe-app/sources/modal/**` | desktop rewrite | `desktop-shell-and-routing` | preserve overlay semantics and focus handling, not RN implementation details |
| `packages/vibe-app/sources/app/**` | desktop rewrite | `desktop-shell-and-routing`, `session-ui-parity`, `secondary-surfaces` | route tree and screens are parity references, not direct imports |
| `packages/vibe-app/sources/track/**` | deferred | `release-and-promotion` | analytics/tracking must not disappear by accident, but phase-one parity does not depend on rebuilding telemetry before desktop value and release/privacy needs are reviewed explicitly |
| `packages/vibe-app/sources/types/**` | reusable as-is | `core-logic-extraction`, `bootstrap-and-package` | package-local TS shims and declaration files may be copied as needed without creating a shared package |
| `packages/vibe-app/src-tauri/**` | adapter required | `bootstrap-and-package`, `desktop-platform-adapters` | treat as behavior/config reference only; do not mutate the shipping package path |
| mobile-only camera/media flows | deferred | `desktop-platform-adapters` | activate only if a concrete desktop requirement is approved |
| mobile push, sensor, and location flows | deferred | `desktop-platform-adapters` | no phase-one desktop ownership unless explicitly added |

## Phase 0 Gate

- no `vibe-app-tauri` module work starts until the rows needed for the first usable desktop slice
  are reviewed and this file still matches the project plan

## Change Rule

- if a module discovers that a source area changed classification, update this file first and then
  update the owning module or project plan if the boundary moved
