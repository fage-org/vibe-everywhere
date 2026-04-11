# Wave 10 Execution Batches

## Purpose

This file converts the Wave 10 execution plan into direct AI dispatch batches.

## Batch Rules

- Batches are ordered.
- Inside a batch, still dispatch one module plan per implementation task.
- A batch is complete only when its gate is satisfied.
- If a batch changes what the repository claims is complete, update `PLAN.md`, `STATUS.md`, and
  any touched top-level docs in the same change set.

## Batch Index

| Batch | Focus | Blocking Output | Status |
| --- | --- | --- | --- |
| B27 | Wave 10 planning baseline and capability contract | the active app completion standard is reset | planned |
| B28 | settings and connection-center closure | settings pages stop mixing product and placeholder states | planned |
| B29 | inbox and notification closure | notification sources and user-facing semantics are explicit | planned |
| B30 | remote operations workflow | terminal/server/machine/session surfaces become one coherent flow | planned |
| B31 | platform parity contract | desktop/Android/browser support is classified accurately | planned |
| B32 | surface disposition and documentation reset | social/dev visibility and customer wording are finalized | planned |

## B27: Wave 10 Planning Baseline And Capability Contract

### Prerequisites

- Wave 9 archived

### Module Order

1. `modules/vibe-app-tauri/validation-and-customer-capability-contract.md`

### Gate

- the repository has one active definition of app completion that is stricter than route presence

## B28: Settings And Connection Center

### Prerequisites

- `B27` complete

### Module Order

1. `modules/vibe-app-tauri/settings-and-connection-center.md`

### Gate

- settings and connection surfaces expose clear supported-state contracts

## B29: Inbox And Notification Closure

### Prerequisites

- `B27` complete

### Module Order

1. `modules/vibe-app-tauri/inbox-and-notification-closure.md`

### Gate

- inbox, feed, alerts, and notification semantics are no longer conflated

## B30: Remote Operations Workflow

### Prerequisites

- `B27` complete
- `B28` complete

### Module Order

1. `modules/vibe-app-tauri/remote-operations-surfaces.md`

### Gate

- terminal, machine, server, and session helper routes form one consistent remote-operations story

## B31: Platform Parity Contract

### Prerequisites

- `B28` complete
- `B29` complete
- `B30` complete

### Module Order

1. `modules/vibe-app-tauri/platform-parity-and-browser-contract.md`

### Gate

- desktop, Android, and browser support claims are explicit and evidence-backed

## B32: Surface Disposition And Documentation Reset

### Prerequisites

- `B31` complete

### Module Order

1. `modules/vibe-app-tauri/social-and-developer-surface-disposition.md`

### Gate

- deferred or internal-only surfaces are formally classified and active docs match the resulting
  product contract

## Direct Prompt Template

> Implement `<module-plan-path>`.
> Follow `docs/plans/rebuild/projects/vibe-app-tauri.md` and the referenced shared specs.
> Assume all earlier batches in `execution-batches.md` are complete.
> Preserve the Wave 10 gate: `<gate-text>`.
> If code reality differs from the plan, update the plan first before continuing.
