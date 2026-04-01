# Repository Guidelines

## Purpose

This file defines durable repository-level rules for contributors and coding agents during the
Happy-aligned rebuild.

## Repository Map

- `crates/vibe-wire`: canonical shared protocol, payload, schema, and crypto-facing types
- `crates/vibe-server`: Rust service replacing `happy-server`
- `crates/vibe-agent`: Rust remote-control client replacing `happy-agent`
- `crates/vibe-cli`: Rust local runtime and CLI replacing `happy-cli`
- `crates/vibe-app-logs`: Rust log-sidecar replacing `happy-app-logs`
- `packages/vibe-app`: imported `happy-app` codebase plus Vibe-specific adapters
- `docs/plans/rebuild`: the only active planning tree
- `scripts`: validation and migration helpers as they are introduced

## Source Of Truth

- `/root/happy` is the implementation blueprint for product concepts, project boundaries, module
  responsibilities, and protocol behavior.
- The Vibe rebuild must not diverge from Happy behavior unless the relevant plan file explicitly
  records the deviation first.
- Shared contracts must be defined in `crates/vibe-wire` before they are consumed elsewhere.

## Planning Rules

- Treat `PLAN.md` as the single planning entry point.
- Update the relevant file under `docs/plans/rebuild/` before changing code when:
  - scope changes
  - a shared contract changes
  - a module boundary changes
  - a deferred item becomes active work
- Execute one module plan at a time. Do not give AI broad multi-project implementation prompts.
- If a task spans modules, update shared specs first, then project plan, then module plan, then
  code.

## Toolchain Baseline

- Rust stable with `cargo fmt`
- Node.js `>=24.14.0 <25`
- Yarn `1.22.22`
- Tauri / Expo / mobile toolchains only when `packages/vibe-app` import begins

## Validation Baseline

Until functional code lands, keep the skeleton healthy:

- `cargo check --workspace`
- plan file reviews against `docs/plans/rebuild/shared/validation.md`

Once a subsystem is implemented, expand validation in the owning project plan and keep it aligned
with the shared validation matrix.

## Change Hygiene

- Prefer small, scoped commits aligned to one module plan.
- Do not introduce parallel protocol definitions in server, CLI, agent, or app.
- Avoid speculative abstractions. Match Happy concepts first; refactor only after parity is proven.
- Preserve clear `happy -> vibe` traceability in naming, docs, and source mapping files.
