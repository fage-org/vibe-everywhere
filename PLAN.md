# Vibe Everywhere — Project Status and Document Index

This file is the single planning entry point. Read it first to understand where the project stands
and where to find every planning document.

## Current Phase

**Wave 9 — Batch B26 (Promotion Evidence and Legacy Deprecation) pending.**

All Waves 0–8 are complete or historical. The remaining work is closing B26: the promotion baseline
sign-off for `packages/vibe-app-tauri` and formal deprecation of `packages/vibe-app` from CI/release
ownership.

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
| vibe-app-tauri | `packages/vibe-app-tauri` | 🔧 active (Wave 9, B26 pending) |

## Document Navigation

### Status and Index

| Document | Purpose |
|----------|---------|
| `docs/plans/rebuild/STATUS.md` | **Start here.** Current phase, batch/module/gate status at a glance |
| `PLAN.md` (this file) | Top-level status pointer and document index |

### Planning Structure

| Document | Purpose |
|----------|---------|
| `docs/plans/README.md` | Repository-level planning index |
| `docs/plans/process.md` | Workflow rules, update rules, AI dispatch rules, archival rules |
| `docs/plans/rebuild/README.md` | Rebuild track index and reading order |
| `docs/plans/rebuild/master-summary.md` | High-level objectives, milestones, risks |
| `docs/plans/rebuild/master-details.md` | Architecture, dependency graph, acceptance gates |
| `docs/plans/rebuild/execution-plan.md` | Module-level sequencing (active waves in detail) |
| `docs/plans/rebuild/execution-batches.md` | AI dispatch batches (active batches in detail) |

### Active Plans (Wave 9)

| Document | Purpose |
|----------|---------|
| `docs/plans/rebuild/projects/vibe-app-tauri.md` | Active project plan |
| `docs/plans/rebuild/modules/vibe-app-tauri/` | Active module plans (12 remaining after archival) |

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
| `archive/completed-projects/` | Done project plans (vibe-wire through vibe-app-logs) |
| `archive/completed-modules/` | Done module plans (waves 0–7) |
| `archive/wave8/` | Wave 8 historical planning documents |
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