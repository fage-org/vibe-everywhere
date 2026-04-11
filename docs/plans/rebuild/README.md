# Rebuild Planning Index

## Purpose

This directory is the only active planning tree for the Happy-aligned rebuild of `vibe-remote`.

The repository is intentionally plan-first. No subsystem implementation is allowed to outrun the
relevant shared spec, project plan, and module plan.

## Current Status

See `STATUS.md` for the current phase, batch status, module progress, and acceptance gates.

Wave 10 is now the active planning track for `packages/vibe-app-tauri`.

## Document Tree

### Active Planning Documents

- `STATUS.md`: current phase, batch, module, and gate status at a glance
- `master-summary.md`: Wave 10 high-signal status, milestones, and next work
- `master-details.md`: Wave 10 architecture, dependency order, and acceptance gates
- `execution-plan.md`: authoritative Wave 10 module-by-module implementation order
- `execution-batches.md`: AI dispatch-ready Wave 10 batch list
- `projects/vibe-app-tauri.md`: active Wave 10 project plan
- `modules/vibe-app-tauri/`: active Wave 10 module plans

### Shared Specs (permanent reference)

- `shared/source-crosswalk.md`: Happy-to-Vibe source mapping
- `shared/naming.md`: product naming, binary names, env prefixes
- `shared/data-model.md`: canonical data model
- `shared/protocol-session.md`: session protocol spec
- `shared/protocol-auth-crypto.md`: auth and crypto protocol
- `shared/protocol-api-rpc.md`: API/RPC protocol
- `shared/validation.md`: per-phase validation requirements
- `shared/migration-order.md`: stage-level delivery sequence
- `shared/ui-visual-parity.md`: cross-cutting UI and visual parity rules

### Archived Work (read-only)

- `archive/`: completed project plans, module plans, and historical wave documents
- `archive/wave8/`: Wave 8 desktop-preview planning artifacts (historical)
- `archive/completed-projects/`: done project plans (vibe-wire through vibe-app-logs)
- `archive/completed-modules/`: done module plans (waves 0–7, plus Wave 8 historical modules)
- `archive/historical/`: other superseded documents (parity audit, pre-Wave-9 release model)

For `vibe-app-tauri`, the active planning set is:

- `projects/vibe-app-tauri.md`
- `master-summary.md`
- `master-details.md`
- `execution-plan.md`
- `execution-batches.md`
- `modules/vibe-app-tauri/*`

Wave 8 and Wave 9 planning artifacts now live in `archive/` and must not override the active Wave
10 execution set.

## Wave 10 Glossary

- `active Wave 10 planning track`: the current decision-bearing planning set for
  `packages/vibe-app-tauri`
- `product-contract complete`: a surface whose behavior, platform scope, and active docs agree
- `handoff-only surface`: a route that intentionally hands off to a terminal command, external
  browser flow, or non-app step
- `historical continuity reference`: an old plan or package kept only to answer Vibe-specific
  continuity questions when the active docs are not sufficient

## Working Rules

- Treat `/root/happy` as the source of truth for product concepts, project boundaries, module
  responsibilities, and protocol behavior.
- Treat `shared/ui-visual-parity.md` as the default visual rulebook for app-facing UI work.
- One implementation task must map to one module plan file.
- If work changes a shared contract, update `shared/*.md` first.
- If work changes a project boundary, update the project plan before implementation.
- If a module plan is missing a decision, fill the plan gap first. Do not let AI improvise.

## Recommended Reading Order

First consult `STATUS.md` for current status, then `execution-plan.md` and `execution-batches.md`
for active work. The list below is the coarse reading order for still-active plan documents:

1. `shared/source-crosswalk.md`
2. `shared/naming.md`
3. `shared/ui-visual-parity.md`
4. `shared/data-model.md`
5. `shared/protocol-session.md`
6. `shared/protocol-auth-crypto.md`
7. `shared/protocol-api-rpc.md`
8. `shared/validation.md`
9. `shared/migration-order.md`
10. `master-summary.md`
11. `master-details.md`
12. `execution-plan.md`
13. `execution-batches.md`
14. `projects/vibe-app-tauri.md`
15. `modules/vibe-app-tauri/*`

Completed project and module plans (waves 0–7) are in `archive/completed-projects/` and
`archive/completed-modules/`. Wave 8 planning artifacts are in `archive/wave8/`.

## AI Execution Contract

When dispatching implementation work, always give AI:

- one module plan path
- any referenced shared spec paths
- the owning project plan path
- an explicit instruction to update the plan first if reality differs
- for app-facing UI work, explicitly include `docs/plans/rebuild/shared/ui-visual-parity.md`

Example:

> Implement `docs/plans/rebuild/modules/vibe-app-tauri/validation-and-customer-capability-contract.md` and
> follow `docs/plans/rebuild/projects/vibe-app-tauri.md`,
> `docs/plans/rebuild/shared/data-model.md`,
> `docs/plans/rebuild/shared/protocol-session.md`, and
> `docs/plans/rebuild/shared/protocol-api-rpc.md`.
