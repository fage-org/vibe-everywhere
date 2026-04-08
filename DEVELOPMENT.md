# Development

## Current State

The repository is beyond bootstrap mode.

- Waves 0-7 of the Happy-aligned rebuild are implemented and validated.
- `packages/vibe-app` is retained only as a deprecated historical reference.
- Active implementation work is Wave 9 for `packages/vibe-app-tauri`.
- Planning remains authoritative for any scope, ownership, or contract change.

## Entry Points

- planning index: `PLAN.md`
- planning tree: `docs/plans/rebuild/README.md`
- master summary: `docs/plans/rebuild/master-summary.md`
- execution order: `docs/plans/rebuild/execution-plan.md`
- AI dispatch batches: `docs/plans/rebuild/execution-batches.md`
- dependency order: `docs/plans/rebuild/shared/migration-order.md`
- validation matrix: `docs/plans/rebuild/shared/validation.md`

## Repository Baseline

- `cargo fmt --all --check`
- `cargo check --workspace --locked`
- `cargo test --workspace --locked`
- `yarn workspace vibe-app-tauri typecheck`
- `yarn workspace vibe-app-tauri test`
- `yarn workspace vibe-app-tauri tauri:test`
- `yarn workspace vibe-app-tauri tauri:smoke`
- `yarn --cwd scripts validate:vibe-app-tauri-promotion`

## Change Rules

- Update the relevant file under `docs/plans/rebuild/` before changing behavior, scope, contracts, or
  module boundaries.
- Treat `packages/vibe-app` as reference-only unless a plan update explicitly reactivates it.
- When a touched module plan defines extra tests or release evidence, run them or report the block
  explicitly.
