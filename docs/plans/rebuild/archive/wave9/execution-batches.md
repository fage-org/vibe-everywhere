# AI Execution Batches

## Purpose

This file converts `execution-plan.md` into direct AI dispatch batches.

Use this file when you want to assign work in grouped waves such as:

- one batch per day
- one batch per AI worker queue
- one batch per review cycle

## Batch Rules

- Batches are ordered. Do not skip ahead unless this file explicitly marks a later batch as safe.
- Inside a batch, still dispatch one module plan per implementation task.
- A batch is complete only when its gate is satisfied.
- If a batch discovers a missing contract, update the relevant shared or project plan before
  continuing.
- If two modules in the same batch touch the same write scope, run them serially even if the batch
  says "parallel allowed".
- If a batch is closed as historical baseline material, mark that status in the batch index and
  relabel any leftover module entries as historical or moved work instead of leaving them looking
  dispatchable.

## Batch Index

| Batch | Focus | Blocking Output | Status |
| --- | --- | --- | --- |
| B00 | planning freeze | all shared contracts and execution docs frozen | ✅ done |
| B01 | wire metadata and legacy/session schemas | `vibe-wire` core shape available | ✅ done |
| B02 | wire containers and voice | complete public `vibe-wire` surface | ✅ done |
| B03 | server config and storage spine | server can start and persist state | ✅ done |
| B04 | server auth, sessions, machines | minimum backend path exists | ✅ done |
| B05 | server presence, router, socket | live backend path exists | ✅ done |
| B06 | agent auth and HTTP control | agent can authenticate and call server | ✅ done |
| B07 | agent live control and CLI UX | remote-control path works end-to-end | ✅ done |
| B08 | server support APIs, files, and images | app/CLI support surface exists | ✅ done |
| B09 | server router/socket finalization and monitoring | server route/socket surface complete | ✅ done |
| B10 | CLI foundation | local CLI architecture exists | ✅ done |
| B11 | CLI daemon and local control plane | daemonized local control works | ✅ done |
| B12 | CLI first provider vertical slice | first provider runtime slice works end-to-end | ✅ done |
| B13 | CLI provider expansion | remaining providers and final command wiring land on stable core | ✅ done |
| B14 | app import baseline | imported app builds in this repo | ✅ done |
| B15 | app adaptation | app works against Vibe services | ✅ done |
| B16 | optional sidecar | app-log sidecar parity if still needed | ✅ done |
| B17 | historical desktop-preview planning freeze and first usable slice | historical Wave 8 baseline | ✅ done |
| B18 | historical desktop promotion planning | historical Wave 8 parity and promotion baseline | ✅ done |
| B19 | unified runtime bootstrap | `vibe-app-tauri` hosts desktop, Android, browser in one package | ✅ done |
| B20 | shared core import from Happy | replacement package owns reusable auth/sync/realtime/core modules | ✅ done |
| B21 | shell surfaces, static browser export, and identity | P0 entry flows and create/link/restore work | ✅ done |
| B22 | session runtime and rendering | desktop and mobile reach real end-to-end session chain | ✅ done |
| B23 | promotion-critical native capabilities | mobile and cross-platform capability blockers closed or waived | ✅ done |
| B24 | secondary route migration | promotion-critical P1 routes wired | ✅ done |
| B25 | release and store migration | `vibe-app-tauri` produces the full app artifact set | ✅ done |
| B26 | promotion evidence and legacy deprecation | default owner-switch recorded and final promotion gate closed | 🔧 pending |

Detailed batch specifications for B00–B25 have been archived. See `archive/completed-modules/` for
the module plans belonging to those batches. The only active batch is B26 below.

## B26: Promotion Evidence And Legacy Deprecation

### Prerequisites

- `B25` complete

### Module Order

1. `modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md`

### Implementation Tasks

1. Confirm every `P0`/`P1` route and `C0`/`C1` capability is either satisfied or explicitly waived
   in writing.
2. Confirm default release ownership is runnable across desktop, Android, and retained static
   browser export outputs.
3. Run and record the final hold/rollback drill against the active Wave 9 replacement package.
4. Update docs, helper scripts, workflow defaults, and release-owner notes so they point to the
   default app path.
5. Record the fallback retention window and eventual retirement policy for `packages/vibe-app`.
6. Produce the final promotion decision record showing that `packages/vibe-app-tauri` is now the
   default app path and `packages/vibe-app` remains reference-only.

### Parallel Allowed

- no; promotion is a single explicit decision point

### Gate

- `packages/vibe-app-tauri` can be confirmed as the default app path with hold/rollback and archival
  rules documented

### Validation Focus

- promotion checklist review
- release-owner switch review
- rollback drill review
- docs/workflow default-owner review

## Direct Prompt Template

Use this template when dispatching a batch item:

> Implement `<module-plan-path>`.
> Follow `<project-plan-path>` and the referenced shared specs.
> Assume all earlier modules in batch `<batch-id>` are complete.
> Treat `<previous-module-plan-path>` as the immediately preceding implementation dependency.
> Preserve the batch gate: `<gate-text>`.
> If code reality differs from the plan, update the plan first before continuing.