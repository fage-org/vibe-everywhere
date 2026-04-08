# Rebuild Planning Index

## Purpose

This directory is the only active planning tree for the Happy-aligned rebuild of `vibe-remote`.

The repository is intentionally plan-first. No subsystem implementation is allowed to outrun the
relevant shared spec, project plan, and module plan.

## Document Tree

- `master-summary.md`: high-signal status, milestones, and next work
- `final-parity-audit.md`: closing audit for the original Wave 0-7 rebuild baseline
- `vibe-app-tauri-wave9-unified-replacement-plan.md`: Wave 9 batch plan for turning `vibe-app-tauri` into the active Wave 9 replacement package
- `vibe-app-tauri-wave9-route-and-capability-matrix.md`: Wave 9 route families, parity classes, and cross-platform capability owners
- `vibe-app-tauri-wave9-migration-and-release-plan.md`: Wave 9 release, OTA, store, identifier, migration, and rollback plan
- `vibe-app-tauri-wave8-delivery-plan.md`: historical Wave 8 desktop-preview batch plan
- `vibe-app-tauri-extraction-inventory.md`: historical Wave 8 reusable-vs-rewrite inventory
- `vibe-app-tauri-route-inventory.md`: historical Wave 8 desktop route inventory
- `vibe-app-tauri-capability-matrix.md`: historical Wave 8 desktop capability matrix
- `vibe-app-tauri-coexistence-matrix.md`: historical Wave 8 coexistence rules before the unified replacement boundary
- `vibe-app-tauri-promotion-baseline.md`: historical desktop-only evidence template; use only for continuity review where still relevant
- `vibe-app-tauri-promotion-plan.md`: historical desktop-only promotion/fallback reference
- `vibe-app-tauri-parity-checklist.md`: historical desktop-only parity checklist
- `master-details.md`: global architecture, dependency order, and acceptance gates
- `execution-plan.md`: authoritative module-by-module implementation order
- `execution-batches.md`: AI dispatch-ready batch list derived from the execution plan
- `projects/`: one plan per target project
- `shared/`: cross-cutting source mappings, naming, protocols, data models, and validation rules
- `modules/`: execution-grade plans, one module per file

The authoritative module-by-module implementation sequence lives in `execution-plan.md`.

For `vibe-app-tauri`, the active planning set is:

- `projects/vibe-app-tauri.md`
- `vibe-app-tauri-wave9-unified-replacement-plan.md`
- `vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `vibe-app-tauri-wave9-migration-and-release-plan.md`
- `modules/vibe-app-tauri/*`

Wave 8 desktop-only planning artifacts remain as historical references and must not override Wave 9
route, capability, migration, or promotion rules implicitly.

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
- One implementation task must map to one module plan file.
- If work changes a shared contract, update `shared/*.md` first.
- If work changes a project boundary, update the project plan before implementation.
- If a module plan is missing a decision, fill the plan gap first. Do not let AI improvise.

## Recommended Execution Order

First consult `execution-plan.md` and `execution-batches.md`. The list below is only the coarse reading order for plan
documents:

1. `shared/source-crosswalk.md`
2. `shared/naming.md`
3. `shared/data-model.md`
4. `shared/protocol-session.md`
5. `shared/protocol-auth-crypto.md`
6. `shared/protocol-api-rpc.md`
7. `shared/validation.md`
8. `shared/migration-order.md`
9. `execution-plan.md`
10. `execution-batches.md`
11. `projects/vibe-wire.md`
12. `modules/vibe-wire/*`
13. `projects/vibe-server.md`
14. `modules/vibe-server/*`
15. `projects/vibe-agent.md`
16. `modules/vibe-agent/*`
17. `projects/vibe-cli.md`
18. `modules/vibe-cli/*`
19. `projects/vibe-app.md`
20. `modules/vibe-app/*`
21. `projects/vibe-app-logs.md`
22. `modules/vibe-app-logs/*`
23. `projects/vibe-app-tauri.md`
24. `vibe-app-tauri-wave9-unified-replacement-plan.md`
25. `vibe-app-tauri-wave9-route-and-capability-matrix.md`
26. `vibe-app-tauri-wave9-migration-and-release-plan.md`
27. `historical: vibe-app-tauri-wave8-delivery-plan.md`
28. `historical: vibe-app-tauri-extraction-inventory.md`
29. `historical: vibe-app-tauri-route-inventory.md`
30. `historical: vibe-app-tauri-capability-matrix.md`
31. `historical: vibe-app-tauri-coexistence-matrix.md`
32. `historical: vibe-app-tauri-promotion-baseline.md`
33. `historical: vibe-app-tauri-promotion-plan.md`
34. `historical: vibe-app-tauri-parity-checklist.md`
35. `modules/vibe-app-tauri/*`

## AI Execution Contract

When dispatching implementation work, always give AI:

- one module plan path
- any referenced shared spec paths
- the owning project plan path
- an explicit instruction to update the plan first if reality differs

Example:

> Implement `docs/plans/rebuild/modules/vibe-wire/session-protocol.md` and follow
> `docs/plans/rebuild/shared/protocol-session.md`,
> `docs/plans/rebuild/shared/data-model.md`, and
> `docs/plans/rebuild/projects/vibe-wire.md`.
