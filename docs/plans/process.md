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

## Completion Rules

A module implementation is not complete until:

- the module acceptance criteria are met
- the required tests in that module plan are run or explicitly reported blocked
- any affected shared/project plans are updated to reflect scope or contract changes

## Anti-Patterns

- implementing directly from vague summary files
- combining unrelated implementation tasks under one module plan
- letting code define wire contracts before shared specs exist
- keeping planning instructions only in chat instead of checking them into the repository
