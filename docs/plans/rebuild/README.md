# Rebuild Planning Index

## Purpose

This directory is the only active planning tree for the Happy-aligned rebuild of `vibe-remote`.

The repository is intentionally plan-first. No subsystem implementation is allowed to outrun the
relevant shared spec, project plan, and module plan.

## Document Tree

- `master-summary.md`: high-signal status, milestones, and next work
- `final-parity-audit.md`: closing audit for the original Wave 0-7 rebuild baseline
- `vibe-app-tauri-parity-checklist.md`: desktop rewrite promotion checklist and sign-off artifact
- `vibe-app-tauri-wave8-delivery-plan.md`: Wave 8 feature inventory, batch plan, and execution checklist
- `vibe-app-tauri-extraction-inventory.md`: reusable-vs-rewrite inventory for the new desktop project
- `vibe-app-tauri-route-inventory.md`: route and surface parity inventory for desktop migration
- `vibe-app-tauri-capability-matrix.md`: desktop capability requirements and replacement matrix
- `vibe-app-tauri-coexistence-matrix.md`: side-by-side rules for `vibe-app` and `vibe-app-tauri`
- `master-details.md`: global architecture, dependency order, and acceptance gates
- `execution-plan.md`: authoritative module-by-module implementation order
- `execution-batches.md`: AI dispatch-ready batch list derived from the execution plan
- `projects/`: one plan per target project
- `shared/`: cross-cutting source mappings, naming, protocols, data models, and validation rules
- `modules/`: execution-grade plans, one module per file

The authoritative module-by-module implementation sequence lives in `execution-plan.md`.

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
24. `vibe-app-tauri-wave8-delivery-plan.md`
25. `vibe-app-tauri-extraction-inventory.md`
26. `vibe-app-tauri-route-inventory.md`
27. `vibe-app-tauri-capability-matrix.md`
28. `vibe-app-tauri-coexistence-matrix.md`
29. `vibe-app-tauri-parity-checklist.md`
30. `modules/vibe-app-tauri/*`

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
