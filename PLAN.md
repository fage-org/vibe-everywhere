# Vibe Everywhere — Project Status and Document Index

This file is the single planning entry point. Read it first to understand where the project stands
and where to find every planning document.

## Current Phase

**Wave 9 — Complete.** All batches B00–B26 are done. G7 is satisfied with the promotion baseline
artifact created at `artifacts/vibe-app-tauri/promotion-baseline.md`; manual platform validation
sections remain [PENDING] until human sign-off.

`packages/vibe-app-tauri` is the default app path and active release owner. `packages/vibe-app`
remains reference-only.

For batch-by-batch and module-level status, see `docs/plans/rebuild/STATUS.md`.

## Project Status Overview

| Project | Crate / Package | Status |
|---------|----------------|--------|
| vibe-wire | `crates/vibe-wire` | ✅ done (archived) |
| vibe-server | `crates/vibe-server` | ✅ done (archived) |
| vibe-agent | `crates/vibe-agent` | ✅ done (archived) |
| vibe-cli | `crates/vibe-cli` | ✅ done (archived) |
| vibe-app | `packages/vibe-app` | ⚰️ deprecated (archived) |
| vibe-app-logs | `crates/vibe-app-logs` | ✅ done (archived) |
| vibe-app-tauri | `packages/vibe-app-tauri` | ✅ done (pending archive) |

## Document Navigation

### Status and Index

| Document | Purpose |
|----------|---------|
| `docs/plans/rebuild/STATUS.md` | **Start here.** Current phase, batch/module/gate status at a glance |
| `PLAN.md` (this file) | Top-level status pointer and document index |

### Archived Planning Documents

All active planning has been completed. The following archived directories contain
historical reference material:

- `docs/plans/rebuild/archive/completed-projects/` — Done project plans
- `docs/plans/rebuild/archive/completed-modules/` — Done module plans
- `docs/plans/rebuild/archive/wave8/` — Wave 8 historical plans
- `docs/plans/rebuild/archive/wave9/` — Wave 9 planning documents

### Archived Plans (Wave 9 — complete)

All Wave 9 planning documents have been archived. See `docs/plans/rebuild/archive/` for details.

| Document | Purpose |
|----------|---------|
| `archive/wave9/execution-plan.md` | Archived module-level sequencing |
| `archive/wave9/execution-batches.md` | Archived AI dispatch batches |
| `archive/wave9/master-summary.md` | Archived high-level objectives, milestones, risks |
| `archive/wave9/master-details.md` | Archived architecture, dependency graph, acceptance gates |
| `archive/wave9/vibe-app-tauri-wave9-unified-replacement-plan.md` | Archived Wave 9 execution-facing replacement plan |
| `archive/wave9/vibe-app-tauri-wave9-migration-and-release-plan.md` | Archived migration and release ownership stages |
| `archive/wave9/vibe-app-tauri-wave9-route-and-capability-matrix.md` | Archived route families and capability owners |
| `archive/completed-projects/vibe-app-tauri.md` | Archived project plan |

### Shared Specs (permanent reference)

| Document | Purpose |
|----------|---------|
| `docs/plans/rebuild/shared/naming.md` | Product naming, binary names, env prefixes |
| `docs/plans/rebuild/shared/data-model.md` | Canonical data model |
| `docs/plans/rebuild/shared/protocol-session.md` | Session protocol spec |
| `docs/plans/rebuild/shared/protocol-auth-crypto.md` | Auth and crypto protocol |
| `docs/plans/rebuild/shared/protocol-api-rpc.md` | API/RPC protocol |
| `docs/plans/rebuild/shared/source-crosswalk.md` | Happy-to-Vibe source mapping |
| `docs/plans/rebuild/shared/validation.md` | Per-phase validation requirements |
| `docs/plans/rebuild/shared/migration-order.md` | Stage-level delivery sequence |
| `docs/plans/rebuild/shared/ui-visual-parity.md` | UI parity spec |

### Archived Work (read-only reference)

| Document | Purpose |
|----------|---------|
| `docs/plans/rebuild/archive/README.md` | Archival policy and directory guide |
| `archive/completed-projects/` | Done project plans (vibe-wire through vibe-app-tauri) |
| `archive/completed-modules/` | Done module plans (waves 0–9) |
| `archive/wave8/` | Wave 8 historical planning documents |
| `archive/wave9/` | Wave 9 planning documents |
| `archive/historical/` | Other superseded documents |

### Root-Level Documents

| Document | Purpose |
|----------|---------|
| `CLAUDE.md` | AI agent instructions (Claude Code / OpenCode) |
| `AGENTS.md` | Durable repository rules and document index |
| `DEVELOPMENT.md` | Developer entry point (commands, environment) |
| `TESTING.md` | Validation baseline, PR-ready gate, CI commands |
| `README.md` | Project quick-start and deployment reference |

## Rule

Update the relevant rebuild plan files before implementing or changing any subsystem work.

Read `docs/plans/rebuild/STATUS.md` before starting any new task to confirm which module and batch it
belongs to.