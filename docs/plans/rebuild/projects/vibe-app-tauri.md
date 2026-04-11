# Project Plan: vibe-app-tauri

## Purpose

`packages/vibe-app-tauri` remains the default app path and active release owner.

Wave 10 does not change package ownership. It changes the standard by which the package is judged
complete.

The Wave 10 target is to make `vibe-app-tauri` product-contract complete across the active visible
surfaces that remain in the app, with explicit platform scope and customer-safe wording.

## Current State

- Wave 9 is complete and archived.
- `packages/vibe-app-tauri` owns desktop, Android, and retained browser-export app delivery.
- core session, auth/restore, file inspection, and artifact flows are implemented
- some settings, notification, remote-helper, and route-visibility areas remain only partially
  productized even though routes and code exist
- some customer-facing summaries have overstated completion relative to the code

## Wave 10 Terms

- `product-contract complete`: the route behavior, platform scope, validation rules, and active docs
  agree on what is implemented
- `handoff-only surface`: a route that intentionally points the user to a terminal command, external
  browser step, or non-app path instead of completing the integration in-app
- `classified deferral`: a route or capability kept visible, hidden, or disabled by explicit written
  decision rather than by omission

## Source Of Truth

### Primary Product Reference

Use `/root/happy/packages/happy-app` for product semantics, route intent, and platform expectations.

### Active Wave 10 Planning Inputs

- `docs/plans/rebuild/master-summary.md`
- `docs/plans/rebuild/master-details.md`
- `docs/plans/rebuild/execution-plan.md`
- `docs/plans/rebuild/execution-batches.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/*.md`

### Shared Contracts

- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/shared/data-model.md`
- `docs/plans/rebuild/shared/protocol-session.md`
- `docs/plans/rebuild/shared/protocol-api-rpc.md`
- `crates/vibe-wire`

### Historical References

- `docs/plans/rebuild/archive/wave9/*`
- `docs/plans/rebuild/archive/completed-projects/vibe-app-tauri.md`
- `docs/plans/rebuild/archive/completed-modules/*`

Historical references may inform continuity checks, but they do not define the active Wave 10
standard.

## Responsibilities

- visible app route ownership in `packages/vibe-app-tauri`
- app-facing product wording and capability classification
- desktop/Android/browser support contract
- visibility and disposition of partial, deferred, or internal-only app surfaces
- active top-level app validation and documentation alignment

## Non-Goals

- reopening archived Wave 9 release-owner decisions
- re-implementing already-complete Rust services unless Wave 10 planning reveals a missing contract
- broad redesign unrelated to the current partial-completion problem
- reviving `packages/vibe-app`

## Locked Decisions

1. Keep the package path as `packages/vibe-app-tauri`.
2. Keep `packages/vibe-app` deprecated and reference-only.
3. Treat route presence as insufficient evidence of completion.
4. A handoff-only surface may be valid, but it must be described as a handoff-only surface.
5. Platform parity claims must be explicit by surface; no generic "multi-platform complete"
   language without route-level support evidence.
6. Social and developer-only surfaces must be either productized, hidden, or explicitly deferred.

## Wave 10 Workstreams

1. capability contract and validation reset
2. settings and connection-center closure
3. inbox and notification taxonomy
4. remote operations workflow
5. platform parity and browser support contract
6. social/developer surface disposition

## Acceptance Criteria

- active planning docs correctly classify the visible `vibe-app-tauri` product surfaces
- customer-safe wording can be derived from active docs without ad hoc interpretation
- platform support and deferrals are explicit across desktop, Android, and browser export
- the repository no longer relies on implicit route visibility to suggest unfinished product scope
