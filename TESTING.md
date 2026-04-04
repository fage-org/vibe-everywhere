# Testing

## Current Baseline

`vibe-wire` is implemented. Validation currently focuses on:

- Rust workspace sanity
- `vibe-wire` schema, fixture, and compatibility-vector coverage
- planning completeness for downstream subsystems before implementation

## Current Commands

- `cargo check --workspace`
- `cargo test -p vibe-wire`
- `cargo run --example export-fixtures -p vibe-wire`
- `yarn --cwd scripts install`
- `yarn --cwd scripts validate:vibe-wire-fixtures`
- `HAPPY_ROOT=/path/to/happy yarn --cwd scripts validate:vibe-wire-fixtures`

## Source Of Truth

Implementation-phase validation requirements live in:

- `docs/plans/rebuild/shared/validation.md`
- the `Testing Strategy` and `Acceptance Criteria` sections in each project plan
- the `Tests` section in each module plan

Current non-Rust compatibility vectors for `vibe-wire` are published under `crates/vibe-wire/fixtures/`
and are generated from the Rust fixture source of truth via `cargo run --example export-fixtures -p vibe-wire`.
Happy-side schema validation for those published fixtures is implemented in
`scripts/validate-vibe-wire-fixtures.mjs`.
That validator reads Happy source files from `HAPPY_ROOT`; when `HAPPY_ROOT` is unset it falls back
to `/root/happy` and fails fast with a clear error if the expected checkout is missing.
