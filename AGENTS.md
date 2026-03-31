# Repository Guidelines

## Purpose

This file defines the durable repository-level rules for contributors and coding agents. Keep it
focused on stable engineering workflow, validation, and documentation boundaries rather than
temporary product or release notes.

## Repository Map

- `crates/vibe-core`: shared protocol types, payloads, and cross-process models
- `apps/vibe-relay`: relay service and control-plane HTTP/WebSocket runtime
- `apps/vibe-agent`: device agent, workspace runtime, provider adapters, and task execution
- `apps/vibe-app`: Vue 3 control application
- `apps/vibe-app/src-tauri`: Tauri desktop and Android shell
- `docs/plans`: versioned planning tracks and planning process notes
- `docs/releases`: release note sources rendered by repository scripts
- `scripts`: local validation, packaging, install, and release helper scripts

Do not commit generated output from `target/`, `apps/vibe-app/dist/`, or `apps/vibe-app/node_modules/`.

## Toolchain Baseline

- Rust stable with `cargo fmt`
- `protoc` available locally
- Node.js `>=24.14.0 <25` and npm `>=11.12.0 <12` for `apps/vibe-app`
- Tauri and Android dependencies only when building desktop or Android targets

Match the frontend engine constraints in `apps/vibe-app/package.json`. Do not silently lower or
broaden them in docs or scripts without updating the package metadata and CI together.

## Development Commands

- `cargo run -p vibe-relay`
- `cargo run -p vibe-agent -- --relay-url http://127.0.0.1:8787`
- `cd apps/vibe-app && npm ci && npm run dev`
- `cd apps/vibe-app && npm run tauri dev`

Use `npm ci` for reproducible frontend setup. Prefer repository scripts over ad hoc local command
variants when a script already exists.

## Validation Commands

These are the repository's primary verification entry points and should stay aligned with CI:

- `cargo fmt --all --check`
- `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --locked --workspace --all-targets -- --nocapture`
- `cd apps/vibe-app && npm ci && npm run build`
- `./scripts/dual-process-smoke.sh relay_polling`
- `./scripts/dual-process-smoke.sh overlay`
- `./scripts/verify-release-version.sh`

Validation expectations:

- Rust changes: run at least the relevant `cargo check` and targeted tests; use the full workspace
  test command before merging broad runtime changes.
- Frontend changes: run `npm run build`; run `npm run test` when changing frontend logic with
  existing Vitest coverage or when adding new frontend tests.
- Relay or agent control-plane / transport / overlay changes: include the relevant dual-process
  smoke tests in verification.
- Release/versioning changes: run `./scripts/verify-release-version.sh`, and when validating a tag,
  pass the tag to the script.
- Release/versioning changes: after updating workspace package versions, regenerate and commit
  `Cargo.lock` before running `cargo ... --locked` checks or pushing a release tag.
- Release/versioning changes: ensure `docs/releases/<tag>.md` or `docs/releases/unreleased.md`
  exists before pushing a release tag, because the `Release` workflow renders notes from those
  sources.

## Coding Rules

- Use `cargo fmt --all` for Rust formatting.
- Follow Rust defaults: `snake_case` for functions/modules and `PascalCase` for types.
- Keep protocol contracts shared between relay, agent, and app in `crates/vibe-core`.
- Avoid introducing app-specific protocol copies in `apps/vibe-relay` or `apps/vibe-agent`.
- In the frontend, prefer reusable logic in `src/lib`, state in `src/stores`, and route screens in
  `src/views`.
- Keep Vue code consistent with the existing TypeScript + Composition API style already used in
  `apps/vibe-app`.

## Testing Rules

- Add or update automated tests close to the changed behavior whenever practical.
- Do not rely on frontend build success as a substitute for Rust or transport verification when
  backend behavior changed.
- Treat smoke tests as required verification for changes that affect process orchestration, relay
  polling, overlay wiring, or shell/task execution paths.
- If a change alters documented manual or release validation flow, update `TESTING.md` in the same
  change set.

## Documentation Boundaries

- `DEVELOPMENT.md`: developer onboarding, local setup, and source-build entry point
- `TESTING.md`: validation entry point and regression checklist
- `PLAN.md`: top-level pointer to the currently active planning files
- `docs/plans/README.md`: plan index
- `docs/plans/process.md`: planning workflow and update rules
- `docs/releases/`: release note source files used by release tooling

Keep documents scoped to their role. Do not copy the same instructions across `AGENTS.md`,
`DEVELOPMENT.md`, `TESTING.md`, and planning files unless the duplication is intentional and
durable.

## Planning Rules

- Keep planning versioned under `docs/plans/`.
- Each active planning track should have both a `summary` file and a `details` file.
- Update the active planning files when implementation scope, acceptance criteria, or completion
  state changes.
- Update `PLAN.md` whenever the active plan pointers change.
- Update `AGENTS.md` only when a repository-wide rule or durable workflow constraint changes.

## Release And Delivery Rules

- CI workflow name: `CI`. Release workflow name: `Release`.
- After pushing to GitHub, check the triggered Actions workflows and report the final conclusion.
- Do not treat delivery as complete immediately after `git push`; include workflow outcome in the
  user-facing report.
- Keep version sources aligned across `Cargo.toml`, `apps/vibe-app/package.json`,
  `apps/vibe-app/package-lock.json`, and `apps/vibe-app/src-tauri/tauri.conf.json`.
- Keep `Cargo.lock` aligned with workspace version bumps before pushing `main` or release tags;
  `CI` and `Release` both use `cargo ... --locked` and will fail if the lockfile still points at the
  previous package version.
- Keep release note sources present under `docs/releases/` for each tagged release; the `Release`
  workflow fails if it cannot render notes for the pushed tag.
- When release packaging or release notes behavior changes, update the corresponding scripts and the
  source files under `docs/releases/` in the same change set.

## Change Hygiene

- Prefer small, scoped changes that match the existing module boundaries.
- When behavior changes cross relay, agent, and app, verify the shared contract first and update
  `vibe-core` when needed.
- Do not introduce new top-level process documents when an existing entry point already owns that
  responsibility.
