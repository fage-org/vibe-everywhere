# Planning Process

## Purpose

This file defines the durable workflow for creating, updating, and consuming planning documents in
this repository.

## Hierarchy

Planning documents must be maintained at five levels:

1. `PLAN.md`
   - top-level pointer to the active planning track
2. `docs/plans/README.md`
   - repository-level planning index
3. active track `master-summary.md` and `master-details.md`
   - high-level goals, gates, and architecture
4. active track `execution-plan.md` and `execution-batches.md`
   - module-level order and AI dispatch batches
5. active track `projects/`, `shared/`, and `modules/`
   - decision-bearing implementation plans

## Required Planning Set

Before implementation begins on a planning track, the following must exist:

- track `README.md`
- track `master-summary.md`
- track `master-details.md`
- track `execution-plan.md`
- track `execution-batches.md` when the track is intended for AI-driven implementation
- one project plan per target project
- shared specs for naming, source mapping, data model, protocol, and validation
- one module plan per implementation task boundary that AI may be asked to execute

## Update Rules

- if a change affects shared contracts, update `shared/*.md` first
- if a change affects a project boundary or ownership split, update the relevant `projects/*.md`
  before implementation
- if a change alters the responsibilities of a default UI shell, route host, or adapter seam,
  update the owning shared/project/module plan before implementation so the structure remains an
  explicit repository rule rather than an implementation accident
- if implementation reveals a missing task boundary, create the missing `modules/*.md` file before
  coding
- if the active track or its entry pointers change, update `PLAN.md` and `docs/plans/README.md`
  together
- if a module plan becomes stale because the source-of-truth Happy behavior changed, update
  `shared/source-crosswalk.md` first, then the affected project/module plans
- if a wave or batch is retired as historical baseline material, mark that closure explicitly in the
  execution docs and label any remaining items as historical or moved work instead of leaving them
  looking like active backlog

## AI Dispatch Rules

- dispatch one implementation task against one module plan file
- always include the owning project plan path
- always include any referenced shared spec paths
- if the assigned module plan lacks a locked decision needed for safe implementation, stop and fill
  the planning gap first
- for `vibe-app-tauri` default-shell work, the dispatch context must name the shell owner, route
  container owner, adapter owner, and form/draft state owner explicitly; do not leave those seams
  to implementation-time interpretation

## Completion Rules

A module implementation is not complete until:

- the module acceptance criteria are met
- the required tests in that module plan are run or explicitly reported blocked
- any affected shared/project plans are updated to reflect scope or contract changes

## Archival Rules

- When all modules in a project reach `[done]`, move the entire `projects/<name>.md` and
  `modules/<name>/` directory into `archive/completed-projects/` and `archive/completed-modules/`.
- When a wave is closed, move its wave-level plan files into `archive/wave<N>/`.
- Every archival move MUST update `STATUS.md` in the same commit.
- Every archival move MUST update internal references in `execution-plan.md`,
  `execution-batches.md`, and `master-summary.md` to point to the new `archive/` paths.
- The `archive/` directory is read-only. No implementation should be dispatched against an archived
  plan. If an archived module needs revisiting, create a new module plan in the active tree instead.

## Navigation Rules (mandatory for AI agents)

- Before any implementation task, read `docs/plans/rebuild/STATUS.md` to confirm the task belongs to
  an active module or batch.
- If the task's target module is not listed in STATUS.md as active, **STOP and ask the user** which
  wave or phase it belongs to — do not default to the current iteration.
- After completing any module or batch, update STATUS.md in the same commit.

## Anti-Patterns

- implementing directly from vague summary files
- combining unrelated implementation tasks under one module plan
- letting code define wire contracts before shared specs exist
- letting a top-level UI shell accumulate route mapping, app-adapter logic, persistence logic, and
  route rendering in one file without an explicit plan update
- keeping planning instructions only in chat instead of checking them into the repository
- leaving `[done]` module plans in the active `modules/` tree past the wave's completion
- leaving historical wave files in the rebuild root directory mixed with active documents
- referencing archived paths as if they are active implementation targets
