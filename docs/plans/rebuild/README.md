# Rebuild Planning Index

## Purpose

This directory is the only active planning tree for the Happy-aligned rebuild of `vibe-remote`.

The repository is intentionally plan-first. No subsystem implementation is allowed to outrun the
relevant shared spec, project plan, and module plan.

## Current Status

See `STATUS.md` for the current phase, batch status, module progress, and acceptance gates.

## Document Tree

### Active Planning Documents

- `STATUS.md`: current phase, batch, module, and gate status at a glance
- `master-summary.md`: high-signal status, milestones, and next work
- `master-details.md`: global architecture, dependency order, and acceptance gates
- `execution-plan.md`: authoritative module-by-module implementation order (active waves only)
- `execution-batches.md`: AI dispatch-ready batch list (active batches only)
- `projects/vibe-app-tauri.md`: the only active project plan
- `modules/vibe-app-tauri/`: active Wave 9 module plans
- `vibe-app-tauri-wave9-unified-replacement-plan.md`: Wave 9 batch plan
- `vibe-app-tauri-wave9-route-and-capability-matrix.md`: Wave 9 route families and capability owners
- `vibe-app-tauri-wave9-migration-and-release-plan.md`: Wave 9 release, OTA, migration, and rollback

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
- `vibe-app-tauri-wave9-unified-replacement-plan.md`
- `vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `vibe-app-tauri-wave9-migration-and-release-plan.md`
- `modules/vibe-app-tauri/*`

Wave 8 desktop-only planning artifacts now live in `archive/wave8/` and must not override Wave 9
route, capability, migration, or promotion rules.

## Wave 9 Glossary

- `historical Wave 8 desktop-preview baseline`: the closed desktop-only planning and implementation
  phase that established `packages/vibe-app-tauri` before the full replacement boundary
- `active Wave 9 replacement package`: `packages/vibe-app-tauri` as the current full-platform app
  replacement target for `packages/vibe-app`
- `default app path`: the app package users and maintainers should treat as the primary current path
  after promotion
- `default release ownership`: the release workflows, identifiers, channels, and store lanes that
  define the primary shipping path after promotion
- `historical continuity reference`: an old plan or package kept only to answer Vibe-specific
  continuity or rollback questions when Happy and the active Wave 9 docs are not sufficient

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
10. `execution-plan.md`
11. `execution-batches.md`
12. `projects/vibe-app-tauri.md`
13. `vibe-app-tauri-wave9-unified-replacement-plan.md`
14. `vibe-app-tauri-wave9-route-and-capability-matrix.md`
15. `vibe-app-tauri-wave9-migration-and-release-plan.md`
16. `modules/vibe-app-tauri/*`

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

> Implement `docs/plans/rebuild/modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md` and
> follow `docs/plans/rebuild/projects/vibe-app-tauri.md`,
> `docs/plans/rebuild/shared/data-model.md`,
> `docs/plans/rebuild/shared/protocol-session.md`, and
> `docs/plans/rebuild/shared/protocol-api-rpc.md`.