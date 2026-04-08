# Project Plan: vibe-app-tauri

## Purpose

`vibe-app-tauri` is the next-iteration desktop app project: a new Tauri 2 + web-native
implementation that recreates the current `vibe-app` desktop experience without rewriting or
destabilizing the existing imported Expo / React Native package.

This is a **parallel replacement project**, not an in-place migration of `packages/vibe-app`.

## Current Constraint

- `packages/vibe-app` remains the production baseline for mobile, web export, and the existing
  Tauri-wrapped desktop surface.
- `vibe-app-tauri` must be introduced alongside it as a new project so iteration can proceed
  without breaking the current app.
- UI and behavior targets are parity-first:
  - visual structure should match the current app as closely as practical
  - interaction flows should match the current app as closely as practical
  - protocol / sync / auth behavior must remain compatible with the existing Vibe backend and wire
    contracts

## Hard Decisions

The following decisions are locked for the planning phase and should be treated as execution rules:

1. Shared logic defaults to `packages/vibe-app-tauri` first.
   - reusable logic identified during the rewrite should initially live inside
     `packages/vibe-app-tauri`
   - do not create a new shared package by default
   - do not refactor `packages/vibe-app` aggressively just to create cleaner abstractions
2. UI parity targets desktop-faithful recreation, not redesign.
   - the preferred target is pixel-close recreation of the current desktop-visible UI
   - where device size, browser layout, or desktop ergonomics make exact visual matching
     impractical, the fallback is a maintainable desktop-web implementation that preserves layout
     hierarchy, information density, and interaction semantics
3. `packages/vibe-app` remains the untouched production baseline until explicit promotion.
4. `vibe-app-tauri` is desktop-only in phase one.

## Source Of Truth

- primary UX and behavior source: `packages/vibe-app`
- upstream product reference: `/root/happy/packages/happy-app`
- shared contracts: `crates/vibe-wire`, `crates/vibe-server`, `crates/vibe-agent`, `crates/vibe-cli`
- planning inventories:
  - `docs/plans/rebuild/vibe-app-tauri-wave8-delivery-plan.md`
  - `docs/plans/rebuild/vibe-app-tauri-extraction-inventory.md`
  - `docs/plans/rebuild/vibe-app-tauri-route-inventory.md`
  - `docs/plans/rebuild/vibe-app-tauri-capability-matrix.md`
  - `docs/plans/rebuild/vibe-app-tauri-coexistence-matrix.md`
  - `docs/plans/rebuild/vibe-app-tauri-promotion-baseline.md`
  - `docs/plans/rebuild/vibe-app-tauri-promotion-plan.md`

The desktop rewrite must preserve Vibe behavior first and may diverge only when the relevant module
or shared planning files explicitly record the deviation.

## Target Layout

- package: `packages/vibe-app-tauri`
- expected top-level ownership:
  - desktop web UI shell
  - route tree and layout system for desktop only
  - desktop-friendly state adapters over shared Vibe sync/auth/domain logic
  - Tauri 2 application shell, permissions, updater, and desktop integrations

`packages/vibe-app` stays intact during the rewrite and remains the fallback reference
implementation until `vibe-app-tauri` is explicitly promoted.

## Non-Goals

- replacing the mobile app in phase one
- replacing the current web export in phase one
- redesigning the Vibe product UX before parity is measured
- changing server routes, wire contracts, or account/session semantics to fit the new desktop app
- deleting or heavily restructuring `packages/vibe-app` before `vibe-app-tauri` proves parity

## Migration Thesis

The hard part is not the Tauri shell itself. The hard part is moving from a React Native / Expo UI
and platform model to a desktop-native web UI while preserving behavior.

Key migration realities:

1. The current app is heavily coupled to:
   - `react-native`
   - `expo-router`
   - `react-native-unistyles`
   - `react-native-reanimated`
   - Expo platform modules and React Native component primitives
2. Business logic is only partially reusable as-is:
   - many `sources/sync/*`, `sources/auth/*`, `sources/encryption/*`, `sources/utils/*`, and
     translation assets are good candidates for extraction
   - many current modules still assume Expo / React Native platform APIs, so extraction must happen
     behind explicit seams
3. The project should therefore be split into:
   - reusable domain/state/adaptor packages or local modules
   - a new desktop web UI layer
   - a minimal Tauri 2 shell

## Shared Logic Destination Policy

### Default Rule

- extracted or rewritten logic should land inside `packages/vibe-app-tauri` first
- `packages/vibe-app` remains the source of truth reference, but not the place where the new
  abstractions are incubated

### Allowed Early Reuse Sources

The following categories may be copied, adapted, or wrapped into `packages/vibe-app-tauri`:

- protocol parsing helpers
- sync/domain utilities
- auth/account/session state helpers
- translation/text assets
- pure TypeScript utility modules

### Not Allowed Early

During phases 0 through 4, do not:

- create a general-purpose `packages/vibe-app-core` by default
- perform broad mechanical rewrites inside `packages/vibe-app`
- move large React Native / Expo coupled modules out of `packages/vibe-app` in one pass
- optimize for elegance at the expense of stability of the current app

### Promotion Rule For Shared Extraction

A module may only be promoted into a future shared package after:

- the `vibe-app-tauri` version of the feature already works
- the logic has proven cross-app value
- extracting it will not destabilize `packages/vibe-app`

## Public Interfaces

- desktop executable distributed via Tauri 2 bundles
- desktop route/navigation surface
- desktop auth/account/session UX
- desktop session list, session detail, agent input, artifacts, settings, feed/social, and utility
  surfaces
- deep links and desktop-local integrations required for parity

## Project Strategy

Build `vibe-app-tauri` as a **fresh desktop-first frontend** with explicit compatibility seams back
to the current app.

Do **not** attempt a big-bang in-place conversion of `packages/vibe-app`.

Instead:

1. freeze desktop parity targets against the current app
2. extract reusable non-UI logic behind stable seams
3. build a new desktop route/layout shell
4. migrate feature slices one by one
5. validate parity continuously against the current desktop behavior

## Workstreams

### 1. Architecture And Extraction

- identify code that can be reused without React Native primitives
- identify code that must be wrapped behind platform adapters
- define the shared desktop package boundaries

### 2. Desktop Design System And Layout

- choose the web UI stack for the new package
- recreate the current Vibe information architecture and visual hierarchy
- map React Native style/token usage to desktop web equivalents
- define the desktop parity tolerances for spacing, typography, breakpoints, and panel resizing

### 3. Navigation And Screen Migration

- rebuild route groups, stack behavior, drawer/panel patterns, and session navigation
- preserve current desktop-visible flows before introducing any redesign

### 4. Platform Integration

- replace Expo-only platform modules with desktop-compatible implementations
- define Tauri commands/plugins for filesystem, notifications, clipboard, external browser, and
  other desktop-specific needs

### 5. Validation And Promotion

- verify route-by-route parity
- run side-by-side smoke comparisons with the existing `vibe-app`
- promote only after the replacement is viable for desktop use

## Phased Implementation Plan

## Phase 0: Planning Freeze

### Goal

Create a migration map before any code lands.

### Required Outputs

- this project plan
- module plans for the desktop rewrite slices listed below
- `docs/plans/rebuild/vibe-app-tauri-extraction-inventory.md`
  - records which `vibe-app` modules are reusable as-is, reusable after adapter seams,
    desktop-only rewrites, or out of scope for phase one
- `docs/plans/rebuild/vibe-app-tauri-route-inventory.md`
  - records route parity targets, screen parity targets, component parity targets, and shell
    invariants for desktop review
- `docs/plans/rebuild/vibe-app-tauri-capability-matrix.md`
  - records auth-critical, session-critical, and promotion-scope desktop capability replacement
    requirements
- `docs/plans/rebuild/vibe-app-tauri-coexistence-matrix.md`
  - records bundle, deep-link, updater, local-storage, and release-artifact coexistence rules with
    `packages/vibe-app`

### Gate To Phase 1

- no implementation begins until the reusable-vs-rewrite boundary, desktop parity inventory, and
  side-by-side coexistence rules are explicit in those files

## Phase 1: Bootstrap New Package

### Goal

Create `packages/vibe-app-tauri` without touching the production role of `packages/vibe-app`.

### Scope

- package bootstrap
- Tauri 2 shell bootstrap
- desktop web frontend bootstrap
- build scripts, lint/test/typecheck wiring
- release packaging entrypoints

### Acceptance

- a blank desktop app boots locally
- Tauri bundle generation works
- the package has its own CI/release hooks without mutating `vibe-app`

## Phase 2: Extract Reusable Core

### Goal

Move non-visual logic behind stable seams the new desktop app can consume.

### Scope

- auth/session/account domain adapters
- sync client and reducer interfaces
- wire parsing compatibility
- shared i18n/text assets where practical
- utility modules that do not require React Native primitives
- extraction targets remain package-local to `vibe-app-tauri` unless explicitly promoted later

### Acceptance

- `vibe-app-tauri` can authenticate and read core session/account state using extracted logic
- extracted modules compile without importing React Native UI primitives

## Phase 3: Desktop Shell And Navigation Parity

### Goal

Recreate the current desktop route tree and main layout structure.

### Scope

- route tree and navigation primitives
- app chrome, header, sidebar/inbox/session list shell
- desktop settings shell
- modal/overlay/focus management for desktop interactions

### Acceptance

- users can navigate the main desktop flows without placeholder dead ends
- layout hierarchy mirrors the current app closely enough for side-by-side review

## Phase 4: Core Session UX Parity

### Goal

Port the session-heavy flows first, since they define most of the product value.

### Scope

- session list
- session detail
- message rendering
- input composer
- tool / diff / markdown / file rendering
- session resume / active session indicators

### Acceptance

- one end-to-end desktop session flow works against the real Vibe backend
- the major session UI states are parity-complete enough for internal dogfooding

## Phase 5: Secondary Surface Migration

### Goal

Close the non-core but user-visible parity gaps.

### Scope

- artifacts
- account/settings/profile
- connect/vendor flows
- feed/social/friends where still relevant to desktop
- changelog, diagnostics, and utility pages

### Acceptance

- no major desktop-visible route remains missing for parity scope

## Desktop Parity Inventory

This inventory defines the expected parity scope before module work begins.

### P0: Must Match In The First Usable Desktop Slice

- auth/login/account restore flows
- inbox / session list shell
- session detail shell
- message rendering
- composer interaction model
- active session indicators and resume affordances
- core settings/account entry points required for desktop use

### P1: Required Before Promotion

- artifacts
- account/profile/settings detail flows
- connect/vendor flows used by desktop users
- changelog and diagnostics surfaces
- markdown, diff, file, and tool rendering parity

### P2: Nice-To-Have Or Late Desktop Parity

- feed/social/friends surfaces if still relevant to desktop
- developer and diagnostics pages not required for primary desktop workflows

### Explicit Desktop Deferrals

- mobile-only camera/media flows unless a concrete desktop requirement is recorded first
- mobile push/device-specific surfaces that do not have a meaningful desktop analog

## UI Parity Standard

### Primary Requirement

- the target is pixel-close recreation of the current desktop-visible `vibe-app` UI

### Allowed Desktop Adjustments

The rewrite may diverge only where needed for:

- desktop viewport differences
- browser layout constraints
- keyboard/focus ergonomics
- maintainability of the desktop web implementation

These adjustments must preserve:

- information architecture
- hierarchy and grouping
- panel relationships
- interaction semantics
- user expectation for where actions and state live

### Not Allowed Without Explicit Plan Update

- stylistic redesign
- simplified layouts that materially change information density
- moving actions to unrelated surfaces just because the web implementation is easier that way

## Phase 6: Desktop Capability Replacement

### Goal

Replace remaining Expo-only platform assumptions with Tauri/web-native implementations.

### Scope

- secure storage
- notifications
- external browser / OAuth callbacks
- clipboard
- file open/save dialogs
- camera/media only if actually required for desktop parity; otherwise explicitly defer

## Desktop Capability Matrix

### Required For Phase-One Desktop Viability

- secure credential storage
- external browser / OAuth callback handling
- localhost loopback callback handling that returns control to the running app without hijacking the
  shipping `vibe-app` deep-link path
- clipboard integration

The following are promotion-scope capabilities rather than first-slice requirements unless a module
plan explicitly promotes them earlier for a concrete desktop flow:

- file picker / save dialogs where current desktop behavior depends on them
- desktop notifications if current desktop flows depend on them materially

### Allowed To Degrade Gracefully

- browser-only fallbacks for some external-link flows
- desktop-specific modal or focus behavior differences that preserve the same action semantics

### Explicitly Deferred Unless New Scope Is Approved

- camera capture
- mobile sensor/location-dependent behavior not critical for desktop use
- other mobile-only Expo capability surfaces with no current desktop value

### Acceptance

- desktop app no longer depends on mobile-only Expo capabilities for supported desktop flows

## Phase 7: Validation, Packaging, And Promotion

### Goal

Prove parity, package the app cleanly, and define promotion criteria.

### Scope

- desktop regression validation
- performance and memory checks on realistic session loads
- release artifacts
- migration / rollout notes
- explicit promotion gate for when `vibe-app-tauri` can become the preferred desktop app

### Acceptance

- release bundles are generated reliably
- the parity checklist in `docs/plans/rebuild/vibe-app-tauri-parity-checklist.md` is signed off
- promotion/deprecation plan for the old desktop path is documented

## Recommended Module Plan Breakdown

When implementation starts, create and execute module plans in roughly this order:

1. `modules/vibe-app-tauri/bootstrap-and-package.md`
2. `modules/vibe-app-tauri/desktop-shell-and-routing.md`
3. `modules/vibe-app-tauri/core-logic-extraction.md`
4. `modules/vibe-app-tauri/desktop-platform-adapters.md`
5. `modules/vibe-app-tauri/auth-and-session-state.md`
6. `modules/vibe-app-tauri/session-ui-parity.md`
7. `modules/vibe-app-tauri/secondary-surfaces.md`
8. `modules/vibe-app-tauri/release-and-promotion.md`

No single implementation task should attempt the full migration.

## UI Parity Rules

The new desktop app must preserve:

- route structure and entry points where desktop users already rely on them
- major information density and panel structure
- session rendering semantics
- composer and tool interaction semantics
- settings/account flows

Allowed changes in early phases:

- replacement of RN-specific interaction plumbing with web-native equivalents
- desktop-specific keyboard/focus handling where it improves parity on desktop
- implementation-driven internal state and component refactors

Not allowed in early phases:

- visual redesign for novelty
- collapsing multiple current screens into a new IA before parity is demonstrated
- changing session semantics or data dependencies to simplify the rewrite

## Validation Strategy

### Required During Migration

- package-level typecheck and test wiring for `vibe-app-tauri`
- route-level smoke tests for desktop navigation
- parser / reducer compatibility checks against `vibe-wire` fixtures where reused
- real desktop app -> Vibe backend chain for auth and sessions
- release-bundle smoke checks for Tauri
- review updates to the extraction inventory, route inventory, capability matrix, and coexistence
  matrix whenever a module discovers a new boundary decision

### Required Before Promotion

- side-by-side parity audit against current `vibe-app` desktop behavior
- explicit review of:
  - auth/connect flow
  - session list
  - session detail
  - message rendering
  - composer actions
  - artifacts/settings/account flows
- packaging and startup validation on Linux, macOS, and Windows

## Promotion Platform Matrix

The promotion gate for `vibe-app-tauri` is cross-platform desktop parity, not a single-OS pilot.

- required before promotion:
  - Linux desktop package boots successfully
  - macOS desktop package boots successfully
  - Windows desktop package boots successfully
- allowed before promotion:
  - day-to-day development or first-slice validation may start on fewer platforms
- not allowed:
  - declaring promotion-ready status from a single-platform validation result

## Parallel Operation Rules

- `packages/vibe-app` remains the default shipping app during all early phases
- `packages/vibe-app-tauri` ships as an additional desktop-focused package until promotion
- release automation, package naming, and CI must keep the two projects distinguishable
- bundle identifiers, updater channels, and callback ownership must stay distinguishable while the
  two desktop paths coexist
- shared credentials may reuse documented compatible storage only where the coexistence matrix says
  it is safe; package-local caches, logs, and window/UI state should stay isolated by default
- no default desktop entrypoint should be flipped to `vibe-app-tauri` without an explicit
  promotion/deprecation plan update

## Key Risks

- underestimating the amount of UI code tied to React Native primitives
- trying to preserve too much of the current UI layer instead of extracting only real domain logic
- desktop-only rewrites drifting from current behavior before parity is measured
- platform capability gaps around secure storage, notifications, OAuth, and file integrations
- AI implementation attempts that are too broad and rewrite multiple desktop subsystems at once

## Recommended Execution Policy

- do not start implementation until the extraction inventory exists
- migrate one feature slice at a time
- require a parity demo after each major phase
- keep `packages/vibe-app` shipping until `vibe-app-tauri` clears promotion gates

## Acceptance Criteria

- `packages/vibe-app-tauri` exists as a separate project and does not destabilize `packages/vibe-app`
- the desktop UI and interactions are substantially recreated, not redesigned first
- the new desktop app works against current Vibe backend contracts without protocol forks
- the old desktop path can remain in service until the new package is explicitly promoted
