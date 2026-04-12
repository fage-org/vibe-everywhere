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
- `src/AppV2.tsx` is now the intended formal UI shell, but it is not yet fully integrated with the
  current `useWave8Desktop` state contract and still contains placeholder behavior
- `src/App.tsx` remains as a legacy shell implementation and migration reference, but it is no
  longer the product target for new UI work

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
- formal default-shell ownership for `src/AppV2.tsx`
- explicit adapter ownership between the Wave 10 backend/state contract and Happy-aligned UI
- default-shell structure rules for route containers, app adapters, and local form/draft state

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
7. `src/AppV2.tsx` is the formal default UI and must be brought to full product usability instead
   of routing users back to legacy shell behavior.
8. The current Rust/backend-supported notification and feed model is the source of truth for Inbox
   behavior; `AppV2` must adapt to that model rather than inventing an independent notification API.
9. `src/App.tsx` may remain temporarily as migration fallback/reference, but it must not define the
   acceptance standard for Wave 10 UI completeness.
10. `src/AppV2.tsx` is a shell orchestrator, not a feature bucket. Route mapping, feed/session
    projection, settings-section assembly, and draft/form persistence must live in explicit route or
    adapter modules.
11. Unsupported or unproductized routes must be classified explicitly in the default shell; broad
    route-section fallbacks do not count as support.

## Wave 10 Workstreams

1. capability contract and validation reset
2. settings and connection-center closure
3. inbox and notification taxonomy
4. remote operations workflow
5. platform parity and browser support contract
6. social/developer surface disposition
7. AppV2 default-shell takeover, adapter integration, and legacy-shell retirement

## Acceptance Criteria

- active planning docs correctly classify the visible `vibe-app-tauri` product surfaces
- customer-safe wording can be derived from active docs without ad hoc interpretation
- platform support and deferrals are explicit across desktop, Android, and browser export
- the repository no longer relies on implicit route visibility to suggest unfinished product scope
- `AppV2` owns the primary desktop/Android/browser shell experience and is validated against the
  real `useWave8Desktop` state contract
- home, session, inbox, and settings flows in `AppV2` are product-usable against the current
  backend without relying on placeholder or legacy-only behavior
- the active planning docs define which AppV2 logic belongs to shell orchestration, route
  containers, backend adapters, and draft/form state modules so future changes cannot collapse those
  seams implicitly
