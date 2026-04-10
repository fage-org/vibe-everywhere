# Plans Index

## Purpose

This directory is the repository-level entry point for all planning tracks.

Only one planning track should be active for implementation at a time. `PLAN.md` points at the
currently active track and its summary/detail files.

## Active Track

- active track: `docs/plans/rebuild/`
- status dashboard: `docs/plans/rebuild/STATUS.md`
- active track index: `docs/plans/rebuild/README.md`
- active track summary: `docs/plans/rebuild/master-summary.md`
- active track details: `docs/plans/rebuild/master-details.md`
- active track execution order: `docs/plans/rebuild/execution-plan.md`
- active track AI batch plan: `docs/plans/rebuild/execution-batches.md`
- archived work: `docs/plans/rebuild/archive/`

## Track Rules

- keep project plans under the active track's `projects/`
- keep shared contracts under the active track's `shared/`
- keep execution-grade module plans under the active track's `modules/`
- do not implement from `master-*` files directly; dispatch work from module plan files
- when the active track changes, update `PLAN.md` and this file together
- when all modules in a project reach `[done]`, move the project plan and module directory to `archive/`
- completed and historical work lives in `archive/`; never dispatch against an archived plan

## Process Pointer

Planning workflow and maintenance rules live in `docs/plans/process.md`.
