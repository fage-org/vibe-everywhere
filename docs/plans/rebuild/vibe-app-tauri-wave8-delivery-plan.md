# Wave 8 Delivery Plan: `vibe-app-tauri`

## Purpose

This document is the execution-facing delivery plan for Wave 8. It translates the existing
project, module, parity, route, capability, extraction, and coexistence plans into one practical
feature checklist for implementation.

Use this file when deciding:

- what Wave 8 must deliver
- which features belong to the first usable desktop slice versus promotion readiness
- which module owns each feature area
- what “done” means for each phase

This file does not replace the authoritative module plans. It is the Wave 8 roll-up view that ties
them together.

## Scope

Wave 8 introduces a new parallel desktop-only package:

- package target: `packages/vibe-app-tauri`
- product goal: recreate the current desktop-visible `vibe-app` UX and behavior with a Tauri 2 +
  web-native implementation
- delivery rule: `packages/vibe-app` remains the default shipping desktop path until explicit
  promotion

## Source Of Truth

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-parity-checklist.md`
- `docs/plans/rebuild/vibe-app-tauri-route-inventory.md`
- `docs/plans/rebuild/vibe-app-tauri-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-extraction-inventory.md`
- `docs/plans/rebuild/vibe-app-tauri-coexistence-matrix.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/*.md`

## Delivery Rules

- parity first; do not redesign before parity is measured
- `packages/vibe-app-tauri` is a separate package, not an in-place rewrite of `packages/vibe-app`
- extracted logic lands in `packages/vibe-app-tauri` first unless a later plan explicitly promotes
  it
- desktop auth must use the locked localhost loopback callback strategy while coexistence remains in
  force
- P0 first, then P1, then P2; do not broaden scope early

## Batch Layout

### `B17`: First Usable Desktop Slice

Goal:

- create a real `packages/vibe-app-tauri`
- make the desktop app boot
- make users authenticate and reach the core session flow against the real Vibe backend

Owning modules:

1. `bootstrap-and-package`
2. `desktop-shell-and-routing`
3. `core-logic-extraction`
4. `desktop-platform-adapters`
5. `auth-and-session-state`
6. `session-ui-parity`

### `B18`: Promotion Readiness

Goal:

- close required promotion-scope parity gaps
- package and validate the desktop app cleanly
- define explicit promotion/deprecation rules without switching the default path early

Owning modules:

1. `secondary-surfaces`
2. `release-and-promotion`

## Feature Inventory

### A. Package And Bootstrap

Owner:

- `modules/vibe-app-tauri/bootstrap-and-package.md`

Feature points:

- create `packages/vibe-app-tauri` with package metadata and workspace integration
- add a desktop web frontend bootstrap that runs independently of `packages/vibe-app`
- add a Tauri 2 shell with isolated bundle identifiers and package-local state directories
- define package-local `dev`, `build`, `typecheck`, `test`, and `release` scripts
- keep build outputs, artifacts, updater channels, and local state isolated from `packages/vibe-app`
- add package-local CI/release hooks without mutating the shipping app lane

Definition of done:

- package exists in-repo
- local desktop dev boot works
- Tauri bundle smoke build works
- scripts do not depend on mutating `packages/vibe-app`

### B. Desktop Shell And Routing

Owner:

- `modules/vibe-app-tauri/desktop-shell-and-routing.md`

Feature points:

- recreate the desktop shell chrome: header, sidebar, main panel, modal/overlay/focus handling
- rebuild the route tree for desktop-only navigation
- preserve desktop entry routes and expected route semantics
- preserve keyboard/focus behavior acceptable for desktop use
- keep layout hierarchy and information density close to current desktop behavior

P0 routes:

- `/(app)/index`
- `/(app)/restore/index`
- `/(app)/restore/manual`
- `/(app)/inbox/index`
- `/(app)/new/index`
- `/(app)/session/[id]`
- `/(app)/session/recent`
- `/(app)/settings/index`

Definition of done:

- shell routes are navigable
- no placeholder dead-end exists in the P0 shell flow
- side-by-side route and shell review is possible

### C. Core Logic Extraction

Owner:

- `modules/vibe-app-tauri/core-logic-extraction.md`

Feature points:

- extract minimum auth/session/account/sync logic needed for the new desktop app
- copy or adapt pure TS logic into `packages/vibe-app-tauri`
- isolate Expo/RN assumptions behind adapter seams
- keep extraction package-local during early phases
- copy required text, utility, encryption, constants, asset, and TS shim inputs only where needed

Primary extraction sources:

- `sources/sync/**`
- `sources/auth/**`
- `sources/encryption/**`
- `sources/utils/**`
- `sources/text/**`
- `sources/constants/**`
- `sources/assets/**`
- `sources/types/**`

Explicitly not required early:

- new shared-core package
- broad refactors inside `packages/vibe-app`
- mobile-only device capabilities

Definition of done:

- extracted logic compiles without React Native UI imports
- auth/session bootstrap code in `vibe-app-tauri` uses package-local extracted seams

### D. Desktop Platform Adapters

Owner:

- `modules/vibe-app-tauri/desktop-platform-adapters.md`

Feature points:

- secure credential storage
- session-safe clipboard integration
- external browser launch for auth/connect
- localhost loopback callback listener for auth completion
- file open/save dialogs where desktop parity requires them
- notifications where parity requires them

Locked auth callback requirements:

- bind only to `127.0.0.1`
- use an ephemeral per-attempt port
- validate a per-attempt `state`
- keep listeners one-shot and short-lived
- allow only one active auth attempt per process instance
- reject stale, replayed, or wrong-instance callbacks

Definition of done:

- auth-critical flows no longer depend on Expo/mobile-only platform APIs
- clipboard and later desktop adapters behave correctly for supported flows

### E. Auth And Session State

Owner:

- `modules/vibe-app-tauri/auth-and-session-state.md`

Feature points:

- desktop login flow
- account restore flow
- logout and re-auth flow
- credential persistence via desktop-safe storage
- backend client wiring for auth/session/account bootstrap
- session/account initial state loading from the real backend
- callback-driven auth completion through the locked localhost loopback strategy

Definition of done:

- desktop app can authenticate
- account restore works
- session/account bootstrap works against the real Vibe backend

### F. Session UI Parity

Owner:

- `modules/vibe-app-tauri/session-ui-parity.md`

Feature points:

- session list shell
- session detail shell
- message timeline rendering
- composer and input interaction model
- active-session and resume affordances
- markdown rendering
- diff rendering
- tool rendering
- file rendering

Primary P0 expectation:

- one real desktop session flow works end-to-end

Promotion-scope rendering surfaces:

- markdown
- diff
- tool
- file

Definition of done:

- users can open a session, read messages, and send input end-to-end
- session UI is good enough for internal dogfooding before broader surface migration

### G. Secondary Product Surfaces

Owner:

- `modules/vibe-app-tauri/secondary-surfaces.md`

Feature points:

- artifacts routes and detail/edit/create flows
- account/profile/settings detail screens
- connect/vendor flows retained for desktop users
- changelog and diagnostics routes
- terminal utility routes
- self-hosted server config route
- machine detail route
- text-selection utility route

P2 late-scope or optional items:

- friends/social routes if desktop value is confirmed
- developer-only routes after route-by-route review

Definition of done:

- all required P1 routes are present
- any omitted P2 route is explicitly marked deferred

### H. Release, Validation, And Promotion

Owner:

- `modules/vibe-app-tauri/release-and-promotion.md`

Feature points:

- package-local release scripts
- desktop CI/release automation
- artifact naming and channel isolation from the shipping app
- parity checklist maintenance
- Linux, macOS, and Windows startup validation before promotion
- realistic session-load performance review
- realistic session-load memory review
- explicit fallback/deprecation strategy for the current desktop path

Definition of done:

- release artifacts are produced reliably
- coexistence rules remain documented
- promotion gate is explicit and sign-off ready

## P0 / P1 / P2 Summary

### P0: First Usable Desktop Slice

- separate `packages/vibe-app-tauri` package and Tauri shell
- desktop shell, route tree, and core navigation
- desktop login/account restore/logout/re-auth
- secure storage plus callback-driven auth completion
- core session list and session detail flow
- message rendering plus composer interaction model
- active-session and resume affordances
- clipboard behavior needed by the primary session/text-selection flows
- one real app-tauri -> backend session chain

### P1: Required Before Promotion

- markdown/diff/tool/file rendering parity
- artifacts flows
- account/profile/settings detail flows
- connect/vendor flows used by desktop users
- changelog/diagnostics routes
- file picker/save dialogs where needed
- notifications where needed
- Linux, macOS, and Windows package/startup validation
- realistic session-load performance and memory review
- promotion/deprecation plan

### P2: Late Or Optional Scope

- friends/social surfaces after desktop-value review
- developer-only routes after route-by-route review
- telemetry/tracking after release/privacy review
- mobile-only camera, sensor, and location capabilities unless reactivated by plan update

## Recommended Implementation Slices

Implement Wave 8 in these vertical slices:

1. bootstrap slice
   - create package
   - boot Tauri shell
   - wire scripts and package-local state
2. shell slice
   - route tree
   - shell chrome
   - modal/focus primitives
3. auth slice
   - secure storage
   - external browser launch
   - localhost callback listener
   - account restore/login/logout
4. session vertical slice
   - session list
   - session detail
   - composer
   - end-to-end backend flow
5. promotion-scope surfaces
   - artifacts/settings/connect/changelog/server/machine/text-selection
6. packaging and promotion
   - release lane
   - artifact validation
   - parity sign-off

Each slice should leave behind:

- code that boots or runs
- tests or smoke validation
- checklist updates
- explicit deferred notes for any omission

## Quality Gates

Wave 8 is not complete unless all of the following are true:

- `packages/vibe-app-tauri` exists and runs as a separate package
- no protocol fork from `vibe-wire` or `vibe-server` was introduced
- auth uses the locked localhost loopback strategy during coexistence
- P0 and P1 checklist items are either `done` or explicitly `deferred` with rationale
- cross-platform desktop startup validation exists before promotion
- performance/memory review exists before promotion
- `packages/vibe-app` remains the default shipping desktop path until promotion is approved

## Immediate Next Work

Start with the first `B17` module in order:

1. `modules/vibe-app-tauri/bootstrap-and-package.md`
2. `modules/vibe-app-tauri/desktop-shell-and-routing.md`
3. `modules/vibe-app-tauri/core-logic-extraction.md`

Do not start `secondary-surfaces` or `release-and-promotion` before the P0 auth and session slice
is real.
