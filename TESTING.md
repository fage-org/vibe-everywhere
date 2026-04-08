# Testing

## Current Baseline

`vibe-wire`, `vibe-app-tauri`, and `vibe-app-logs` are the active validation focus. `packages/vibe-app` is deprecated and excluded from the automated baseline unless Happy cannot answer a Vibe-specific continuity question. Validation currently focuses on:

- Rust workspace sanity
- `vibe-wire` schema, fixture, and compatibility-vector coverage
- `vibe-app-tauri` typecheck, test, desktop-shell, and promotion-doc validation
- `vibe-app-logs` startup and ingestion smoke coverage

## Current Commands

- `cargo check --workspace`
- `cargo test -p vibe-wire`
- `cargo test -p vibe-app-logs`
- `cargo run --example export-fixtures -p vibe-wire`
- `yarn workspace vibe-app-tauri typecheck`
- `yarn workspace vibe-app-tauri test`
- `yarn workspace vibe-app-tauri tauri:test`
- `yarn workspace vibe-app-tauri tauri:smoke`
- `yarn workspace vibe-app-tauri validate:promotion`
- `yarn app:metrics`
- `yarn --cwd scripts validate:vibe-app-tauri-promotion`
- `yarn app:promotion-ready`
- `yarn app-logs --help`
- `yarn --cwd scripts install`
- `yarn --cwd scripts validate:vibe-wire-fixtures`
- `yarn --cwd scripts metrics:vibe-app-tauri`
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
