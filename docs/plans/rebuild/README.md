# Rebuild Planning Index

## Purpose

This directory is the only active planning tree for the Happy-aligned rebuild of `vibe-remote`.

The repository is intentionally plan-first. No subsystem implementation is allowed to outrun the
relevant shared spec, project plan, and module plan.

## Document Tree

- `master-summary.md`: high-signal status, milestones, and next work
- `master-details.md`: global architecture, dependency order, and acceptance gates
- `projects/`: one plan per target project
- `shared/`: cross-cutting source mappings, naming, protocols, data models, and validation rules
- `modules/`: execution-grade plans, one module per file

## Working Rules

- Treat `/root/happy` as the source of truth for product concepts, project boundaries, module
  responsibilities, and protocol behavior.
- One implementation task must map to one module plan file.
- If work changes a shared contract, update `shared/*.md` first.
- If work changes a project boundary, update the project plan before implementation.
- If a module plan is missing a decision, fill the plan gap first. Do not let AI improvise.

## Recommended Execution Order

1. `shared/source-crosswalk.md`
2. `shared/naming.md`
3. `shared/data-model.md`
4. `shared/protocol-session.md`
5. `shared/protocol-auth-crypto.md`
6. `shared/protocol-api-rpc.md`
7. `projects/vibe-wire.md`
8. `modules/vibe-wire/*`
9. `projects/vibe-server.md`
10. `modules/vibe-server/*`
11. `projects/vibe-agent.md`
12. `modules/vibe-agent/*`
13. `projects/vibe-cli.md`
14. `modules/vibe-cli/*`
15. `projects/vibe-app.md`
16. `modules/vibe-app/*`
17. `projects/vibe-app-logs.md`
18. `modules/vibe-app-logs/*`

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
