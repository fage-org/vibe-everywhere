# Development

## Current State

The repository is beyond bootstrap mode.

- Waves 0-7 of the Happy-aligned rebuild are implemented and validated.
- `packages/vibe-app` is retained only as a deprecated historical reference.
- Wave 9 owner-switch work has moved the default app path to `packages/vibe-app-tauri`, with final promotion evidence still tracked in `artifacts/vibe-app-tauri/promotion-baseline.md`.
- Planning remains authoritative for any scope, ownership, or contract change.

## Entry Points

- planning index and status: `PLAN.md`
- planning tree: `docs/plans/rebuild/README.md`
- status dashboard: `docs/plans/rebuild/STATUS.md`
- master summary: `docs/plans/rebuild/master-summary.md`
- execution order: `docs/plans/rebuild/execution-plan.md`
- AI dispatch batches: `docs/plans/rebuild/execution-batches.md`
- planning rules: `docs/plans/process.md`

For validation commands, see `TESTING.md`.

## Repository Baseline

Quick check: `cargo check --workspace && yarn workspace vibe-app-tauri test`

Full validation: see `TESTING.md`.

## Change Rules

- Update the relevant file under `docs/plans/rebuild/` before changing behavior, scope, contracts, or
  module boundaries.
- Treat `packages/vibe-app` as reference-only unless a plan update explicitly reactivates it.
- When a touched module plan defines extra tests or release evidence, run them or report the block
  explicitly.
