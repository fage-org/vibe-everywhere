# `vibe-app-tauri` Parity Checklist

## Purpose

This document is the sign-off checklist for promoting `packages/vibe-app-tauri` from a parallel
desktop rewrite into a production-ready desktop app candidate.

It is intentionally checklist-driven and should be updated as implementation progresses.

This file does **not** replace the project plan or module plans. It is the audit artifact used by
Phase 7 in `projects/vibe-app-tauri.md`.

## Status Scale

- `not started`
- `in progress`
- `blocked`
- `done`
- `deferred`

## Promotion Rule

`packages/vibe-app-tauri` must not become the default desktop path until:

- all required P0 and P1 items below are `done`, or
- a remaining gap is explicitly marked `deferred` in planning files with a written rationale, and
- the promotion/deprecation plan is approved

## P0: First Usable Desktop Slice

### Auth And Account

- status: `not started`
- desktop login flow matches the current app semantically
- account restore flow works
- secure credential storage works on desktop
- external browser auth/connect flow returns control to the running app
- localhost loopback callback completes auth without claiming the default `vibe:///` route
- callback validation enforces per-attempt `state`, timeout, and one active auth attempt per
  process instance
- logout and re-auth flows work

### Shell And Navigation

- status: `not started`
- route tree matches current desktop-visible route structure closely enough for side-by-side review
- main chrome/header/sidebar/panel structure is recreated
- core account/settings entry points required for desktop use are reachable from the shell
- keyboard/focus behavior is acceptable on desktop
- modal and overlay semantics are preserved

### Session Core

- status: `not started`
- session list renders correctly
- session detail shell renders correctly
- message rendering matches current desktop behavior closely
- composer interaction model matches current desktop behavior closely
- active session indicators and resume affordances work
- clipboard integration works for primary session and text-selection flows

### Backend Compatibility

- status: `not started`
- auth and session flows work against the real Vibe backend
- no protocol fork from `vibe-wire` / `vibe-server` is introduced

## P1: Required Before Promotion

### Rendering Surfaces

- status: `not started`
- markdown rendering parity is acceptable
- diff rendering parity is acceptable
- tool rendering parity is acceptable
- file rendering parity is acceptable

### Secondary Product Surfaces

- status: `not started`
- artifacts flows work
- account/profile/settings detail flows work
- connect/vendor flows used by desktop users work
- changelog/diagnostics surfaces needed for desktop users work

### Desktop Capabilities

- status: `not started`
- file picker / save dialogs work where needed
- desktop notifications work where required by parity scope

### Packaging

- status: `not started`
- desktop Tauri bundles are generated reliably for Linux, macOS, and Windows
- packaged app boots successfully on Linux, macOS, and Windows before promotion

### Performance And Runtime

- status: `not started`
- realistic session-load startup and first-interaction performance review is completed
- realistic session-load memory review is completed
- any accepted performance or memory gap is documented with rationale before promotion

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

- `none yet`

## Pixel Parity Audit

The preferred target is pixel-close recreation of the current desktop-visible `vibe-app`.

Record exceptions here when exact matching is intentionally relaxed:

- `none yet`

For each exception, document:

- screen/component
- reason exact parity is impractical
- what was preserved instead:
  - hierarchy
  - information density
  - interaction semantics
  - maintainability

## Validation Checklist

- status: `not started`
- package bootstrap validation passed
- route-level smoke tests passed
- parser/reducer compatibility checks passed for reused logic
- at least one real desktop chain test passed
- auth/connect callback validation passed against the locked localhost loopback strategy
- Tauri package smoke validation passed
- realistic session-load performance/memory review completed
- side-by-side desktop comparison review completed

## Release And Promotion Checklist

- status: `not started`
- release packaging path is stable
- coexistence with `packages/vibe-app` remains documented
- promotion/deprecation plan exists
- final sign-off reviewers are identified

## Notes

- keep this checklist concise and execution-facing
- do not mark an item `done` without a concrete validation artifact
- if an item is impossible or intentionally omitted, mark it `deferred` and point to the owning
  plan update
