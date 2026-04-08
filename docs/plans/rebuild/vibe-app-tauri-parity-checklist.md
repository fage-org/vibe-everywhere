# `vibe-app-tauri` Parity Checklist

## Archival Status

This file is historical Wave 8 desktop-only planning material.

Do not use it as active execution authority for Wave 9. Use the active Wave 9 planning set instead:

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/*`

## Purpose

This document is the historical sign-off checklist for promoting `packages/vibe-app-tauri` from a parallel
desktop rewrite into a production-ready desktop app candidate.

It is intentionally checklist-driven and should be updated as implementation progresses.

This file does **not** replace the project plan or module plans. It is now a historical audit artifact from
Wave 8 desktop promotion planning.

## Status Scale

- `not started`
- `in progress`
- `blocked`
- `done`
- `deferred`

## Promotion Rule

Historical Wave 8 rule: `packages/vibe-app-tauri` did not become the default desktop path until the following conditions were met. Any remaining `in progress` markers below are historical and were carried into Wave 9 where still relevant:

- all required P0 and P1 items below are `done`, or
- a remaining gap is explicitly marked `deferred` in planning files with a written rationale, and
- the promotion/deprecation plan is approved

## P0: First Usable Desktop Slice

### Auth And Account

- status: `in progress`
- desktop login flow matches the then-current app semantically
- account restore flow works
- secure credential storage works on desktop
- external browser auth/connect flow returns control to the running app
- localhost loopback callback completes auth without claiming the default `vibe:///` route
- callback validation enforces per-attempt `state`, timeout, and one active auth attempt per
  process instance
- logout and re-auth flows work
- auth callback hardening remains incomplete until cancellation, timeout, stale-attempt, and
  teardown paths have concrete validation artifacts

### Shell And Navigation

- status: `in progress`
- route tree matches current desktop-visible route structure closely enough for side-by-side review
- main chrome/header/sidebar/panel structure is recreated
- core account/settings entry points required for desktop use are reachable from the shell
- keyboard/focus behavior is acceptable on desktop
- modal and overlay semantics are preserved

### Session Core

- status: `in progress`
- session list renders correctly
- session detail shell renders correctly
- message rendering matches current desktop behavior closely
- composer interaction model matches current desktop behavior closely
- active session indicators and resume affordances work
- clipboard integration works for primary session and text-selection flows

### Backend Compatibility

- status: `in progress`
- auth and session flows work against the real Vibe backend
- no protocol fork from `vibe-wire` / `vibe-server` is introduced
- desktop message/update parsing must stay aligned with the existing Vibe compatibility schemas and
  reject invalid payload shapes explicitly

## P1: Required Before Promotion

### Rendering Surfaces

- status: `in progress`
- markdown rendering parity is acceptable
- diff rendering parity is acceptable
- tool rendering parity is acceptable
- file rendering parity is acceptable

### Secondary Product Surfaces

- status: `in progress`
- artifacts flows work
- account/profile/settings detail flows work
- connect/vendor flows used by desktop users work
- changelog/diagnostics surfaces needed for desktop users work
- retained desktop review routes now exist for artifacts, profile detail, machine detail, server config,
  terminal helpers, and text utility flows; deeper backend mutations still remain promotion-scope work
- `artifacts` backend list/detail/create/update/delete is now an active desktop migration item instead
  of a retained local-only placeholder flow
- `user-detail` and `machine-detail` now read live backend data; retained status still applies to
  routes that remain review-only fixtures only
- `settings/account` and `settings/usage` now read live desktop state; retained status is reserved
  for settings routes that still depend on review-only local preview controls
- `settings/features` and `settings/language` now read implementation/runtime-backed state with
  desktop-backed preference persistence rather than hard-coded desktop review fixtures
- `settings/appearance`, `settings/voice`, and `settings/voice/language` now persist desktop-backed
  preference state and sync supported fields through account settings instead of depending on local
  preview-only controls
- `settings/connect/claude` now mirrors the then-current app's explicit terminal-command handoff instead
  of staying documentation-only
- `terminal/index` and `terminal/connect` now expose live desktop helper commands and terminal auth
  approval flow instead of remaining retained-only helper shells
- retained routes must not be reported as promotion-complete until they stop depending on retained
  review-only data

### Desktop Capabilities

- status: `in progress`
- file picker / save dialogs work where needed
- desktop notifications work where required by parity scope
- manual restore can now load backup material through a desktop file picker
- text-selection export and artifact body export now use a desktop save dialog
- desktop notifications now cover successful restore/link and artifact mutation feedback paths

### Packaging

- status: `in progress`
- desktop Tauri bundles are generated reliably for Linux, macOS, and Windows
- packaged app boots successfully on Linux, macOS, and Windows before promotion
- the shared `app-v*` release workflow now packages both the shipping `packages/vibe-app` desktop
  artifacts and the non-default `packages/vibe-app-tauri` desktop preview artifacts in parallel
  without changing the default shipping path

### Performance And Runtime

- status: `in progress`
- realistic session-load startup and first-interaction performance review is completed
- realistic session-load memory review is completed
- any accepted performance or memory gap is documented with rationale before promotion
- the promotion baseline artifact scaffold now exists, but real measurements still need to be
  recorded before sign-off

## P2: Optional Or Late Desktop Scope

### Social / Collaboration

- status: `not started`
- feed/social/friends parity is evaluated
- desktop-only value is confirmed before implementation

### Developer / Diagnostic Extras

- status: `not started`
- non-essential developer surfaces are reviewed and either ported or deferred explicitly

## Explicit Deferrals

Use this section to record approved desktop deferrals.

- `none approved yet`; remaining Wave 8 gaps stay `in progress` unless an owning plan records a
  deliberate deferral with rationale

## Pixel Parity Audit

The preferred target is pixel-close recreation of the current desktop-visible `vibe-app`.

Record exceptions here when exact matching is intentionally relaxed:

- shell chrome and desktop home entry
  - reason exact parity is impractical today: the tabbed desktop entry now mirrors the current app
    more closely, but inbox-social content and some tablet-specific chrome details are still
    migrating
  - what was preserved instead:
    - hierarchy
    - interaction semantics
    - maintainability
- desktop settings hub
  - reason exact parity is impractical today: the grouped settings hub is now closer to the current
    app, but connected-account mutations, machine rows, and some native-only settings actions still
    need deeper migration
  - what was preserved instead:
    - hierarchy
    - interaction semantics
    - maintainability

For each exception, document:

- screen/component
- reason exact parity is impractical
- what was preserved instead:
  - hierarchy
  - information density
  - interaction semantics
  - maintainability

## Validation Checklist

- status: `in progress`
- package bootstrap validation passed
- route-level smoke tests passed
- parser/reducer compatibility checks passed for reused logic
- at least one real desktop chain test passed
- auth/connect callback validation passed against the locked localhost loopback strategy
- desktop shell hotkey routing and palette keyboard rules now have explicit unit-test coverage
- desktop preference persistence now has explicit unit-test coverage for appearance and voice state
- terminal auth approval now has explicit unit-test coverage for authorized and pending request paths
- account settings fetch/update now has explicit unit-test coverage for decrypted reads and
  version-conflict retries
- desktop payload parsing is validated through shared compatibility schemas rather than package-local
  unchecked casts
- Tauri package smoke validation passed
- realistic session-load performance/memory review remains pending before promotion
- side-by-side desktop comparison review remains pending before promotion sign-off

## Release And Promotion Checklist

- status: `in progress`
- release packaging path is stable
- coexistence with `packages/vibe-app` remains documented
- promotion/deprecation plan exists
- final sign-off reviewers are identified
- tracked promotion review artifacts now live in
  `docs/plans/rebuild/vibe-app-tauri-promotion-baseline.md` and
  `docs/plans/rebuild/vibe-app-tauri-promotion-plan.md`
- tagged `app-v*` releases now publish both shipping and preview desktop assets from the same
  workflow while keeping `packages/vibe-app-tauri` clearly non-default

## Notes

- keep this checklist concise and execution-facing
- do not mark an item `done` without a concrete validation artifact
- if an item is impossible or intentionally omitted, mark it `deferred` and point to the owning
  plan update
