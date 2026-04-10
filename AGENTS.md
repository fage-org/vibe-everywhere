# Repository Guidelines

## Purpose

This file defines durable repository-level rules for contributors and coding agents during the
Happy-aligned rebuild.

## Repository Map

- `crates/vibe-wire`: canonical shared protocol, payload, schema, and crypto-facing types ✅ done
- `crates/vibe-server`: Rust service replacing `happy-server` ✅ done
- `crates/vibe-agent`: Rust remote-control client replacing `happy-agent` ✅ done
- `crates/vibe-cli`: Rust local runtime and CLI replacing `happy-cli` ✅ done
- `crates/vibe-app-logs`: Rust log-sidecar replacing `happy-app-logs` ✅ done
- `packages/vibe-app`: ⚰️ **deprecated** — reference-only, no active CI or release ownership
- `packages/vibe-app-tauri`: 🔧 **active** — Wave 9 replacement desktop/mobile/web app
- `docs/plans/rebuild`: the active planning tree
- `docs/plans/rebuild/archive`: completed and historical planning documents (read-only)
- `scripts`: validation and migration helpers

## Source Of Truth

- `/root/happy` is the implementation blueprint for product concepts, project boundaries, module
  responsibilities, and protocol behavior.
- The Vibe rebuild must not diverge from Happy behavior unless the relevant plan file explicitly
  records the deviation first.
- Shared contracts must be defined in `crates/vibe-wire` before they are consumed elsewhere.

## Document Index

Before any implementation task, read `docs/plans/rebuild/STATUS.md` to confirm which module and
batch the task belongs to.

| Document | Purpose |
|----------|---------|
| `PLAN.md` | Top-level status pointer and full document index |
| `docs/plans/rebuild/STATUS.md` | **Start here.** Current phase, batch/module/gate status |
| `docs/plans/process.md` | Workflow rules, update rules, AI dispatch rules, archival rules |
| `docs/plans/rebuild/README.md` | Rebuild track index and reading order |
| `docs/plans/rebuild/master-summary.md` | High-level objectives, milestones, risks |
| `docs/plans/rebuild/master-details.md` | Architecture, dependency graph, acceptance gates |
| `docs/plans/rebuild/execution-plan.md` | Module-level sequencing (active waves in detail) |
| `docs/plans/rebuild/execution-batches.md` | AI dispatch batches (active batches in detail) |
| `docs/plans/rebuild/projects/vibe-app-tauri.md` | Active project plan |
| `docs/plans/rebuild/modules/vibe-app-tauri/` | Active module plans |
| `docs/plans/rebuild/shared/` | Permanent reference specs (naming, protocols, data model) |
| `docs/plans/rebuild/archive/` | Completed and historical plans (read-only) |

If a task's target module is not listed in STATUS.md as active, **stop and ask the user** which
wave or phase it belongs to. Do not default to the current iteration.

## Planning Rules

- Treat `PLAN.md` as the single planning entry point.
- Read `docs/plans/rebuild/STATUS.md` before starting any task to confirm module and batch context.
- Update the relevant file under `docs/plans/rebuild/` before changing code when:
  - scope changes
  - a shared contract changes
  - a module boundary changes
  - a deferred item becomes active work
- Execute one module plan at a time. Do not give AI broad multi-project implementation prompts.
- If a task spans modules, update shared specs first, then project plan, then module plan, then
  code.
- When a module or batch is completed, update STATUS.md in the same commit.

## Toolchain Baseline

- Rust stable with `cargo fmt`
- Node.js `>=24.14.0 <25`
- Yarn `1.22.22`
- Tauri (active — `packages/vibe-app-tauri` is the current app target)
- `cargo check --workspace` for Rust validation
- `yarn workspace vibe-app-tauri test` for app validation

## Validation Baseline

- `cargo check --workspace`
- `cargo test --workspace`
- Plan file reviews against `docs/plans/rebuild/shared/validation.md`
- Per-subsystem validation commands documented in `DEVELOPMENT.md` and `TESTING.md`

## Change Hygiene

- Prefer small, scoped commits aligned to one module plan.
- Do not introduce parallel protocol definitions in server, CLI, agent, or app.
- Avoid speculative abstractions. Match Happy concepts first; refactor only after parity is proven.
- Preserve clear `happy -> vibe` traceability in naming, docs, and source mapping files.

## Archival Rules

- Completed project and module plans live in `docs/plans/rebuild/archive/`, not in the active tree.
- When all modules in a project reach `[done]`, move the project plan and module directory into
  `archive/completed-projects/` and `archive/completed-modules/`.
- When a wave closes, move its wave-level plan files into `archive/wave<N>/`.
- Every archival move must update `STATUS.md` and internal references in the same commit.
- The `archive/` directory is read-only. If an archived module needs revisiting, create a new module
  plan in the active tree instead of editing the archived one.