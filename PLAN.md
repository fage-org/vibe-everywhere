# Vibe Everywhere Architecture Governance Plan

## Background

This file tracks the ongoing architecture-governance iterations for the repository.

The current project is already a working personal-edition MVP, but it still has several
engineering issues:

- `vibe-relay` and `vibe-agent` both concentrate too much logic in `main.rs`.
- Shared capability declarations are ahead of some fully delivered product capabilities.
- Backend capability coverage is ahead of frontend capability coverage.

The goal of each iteration is to reduce structural risk without changing public behavior.

## Success Criteria

- `apps/vibe-relay/src/main.rs` and `apps/vibe-agent/src/main.rs` keep shrinking by extracting stable support and domain modules.
- Existing HTTP APIs, protocol field names, and user-visible control flows remain unchanged.
- Rust compile/test and frontend build continue to pass after each refactor round.

## Iteration 1

- [x] Create and maintain this plan file as the single execution record for the iteration.
- [x] Extract stable relay support modules for configuration, auth, and persistence/store helpers.
- [x] Extract stable agent support modules for runtime/config helpers and provider logic.
- [x] Keep shared capability notes aligned with current implementation boundaries in this plan.
- [x] Run verification and record results in this file.

## Iteration 2

- [x] Split relay task handlers and task runtime helpers into a dedicated module.
- [x] Split relay shell handlers and shell bridge/runtime helpers into a dedicated module.
- [x] Split relay port-forward handlers and tunnel/overlay helpers into a dedicated module.
- [x] Keep relay routes, payloads, and task/shell/forward state transitions unchanged.
- [x] Re-run relay and workspace validation and record the results in this file.

## Iteration 3

- [x] Split agent task execution, ACP runtime, and task polling logic into a dedicated module.
- [x] Split agent shell session polling and relay shell runtime into a dedicated module.
- [x] Split agent port-forward polling and tunnel runtime into a dedicated module.
- [x] Rewire `task_bridge` and `shell_bridge` to the extracted agent runtime modules without changing transport behavior.
- [x] Re-run agent and workspace validation and record the results in this file.

## Iteration 4

- [x] Extend the frontend shared types and API client for port-forward list/create/close flows.
- [x] Add control-store state, polling, selection, and create/close actions for port forwards.
- [x] Add dashboard port-forward management UI with create, filter, list, inspect, and close flows.
- [x] Preserve the existing relay APIs and use polling for forward status updates instead of introducing new transport paths.
- [x] Run frontend and workspace verification and record the results in this file.

## Iteration 5

- [x] Align the agent-advertised device capabilities with the currently delivered MVP surface.
- [x] Keep future capability enum variants in the shared protocol without advertising unfinished surfaces by default.
- [x] Add a regression test that locks the default capability advertisement to the current MVP boundary.
- [x] Re-run validation and record the results in this file.

## Remaining Backlog

- [ ] Upgrade authentication, audit, and persistence for production-oriented deployment.

## Capability Notes

Current MVP-delivered flows:

- Device register/heartbeat/presence
- Task create/claim/run/cancel/event streaming
- Relay shell session create/input/output/close
- Relay-first and overlay-assisted port-forward backend flows
- Codex/OpenCode preferred ACP execution and Claude Code CLI execution
- Default agent capability advertisement is limited to `ai_session` and `shell`; future enum variants remain protocol placeholders until those product surfaces exist.

Not fully productized in these iterations:

- Full `file_sync` product surface
- Full `workspace_browse` product surface
- Full `notifications` product surface

## Completion Log

- 2026-03-26: Created this plan file and kept it as the execution record for the first refactor-first iteration.
- 2026-03-26: Extracted relay support code from `apps/vibe-relay/src/main.rs` into `apps/vibe-relay/src/config.rs`, `apps/vibe-relay/src/auth.rs`, and `apps/vibe-relay/src/store.rs`.
- 2026-03-26: Updated `apps/vibe-relay/src/main.rs` to consume the extracted relay support modules and fixed the `Arc<RelayConfig>` ownership path introduced during the refactor.
- 2026-03-26: Extracted agent support code from `apps/vibe-agent/src/main.rs` into `apps/vibe-agent/src/config.rs` and `apps/vibe-agent/src/providers.rs`.
- 2026-03-26: Removed obsolete duplicate helper implementations from `apps/vibe-agent/src/main.rs` so the module split is single-source and compile-safe.
- 2026-03-26: Kept capability notes aligned with current MVP boundaries instead of overstating unfinished product surfaces.
- 2026-03-26: Extracted relay task handlers and task runtime logic into `apps/vibe-relay/src/tasks.rs`.
- 2026-03-26: Extracted relay shell handlers and shell bridge/runtime logic into `apps/vibe-relay/src/shell.rs`.
- 2026-03-26: Extracted relay port-forward handlers and tunnel/overlay logic into `apps/vibe-relay/src/port_forwards.rs`.
- 2026-03-26: Reduced `apps/vibe-relay/src/main.rs` to router composition, shared state, generic lifecycle functions, and cross-domain helpers while preserving existing tests.
- 2026-03-26: Extracted agent task polling, task execution, ACP runtime, and terminal management logic into `apps/vibe-agent/src/task_runtime.rs`.
- 2026-03-26: Extracted agent shell session polling and relay shell runtime logic into `apps/vibe-agent/src/shell_runtime.rs`.
- 2026-03-26: Extracted agent port-forward polling and relay tunnel runtime logic into `apps/vibe-agent/src/port_forward_runtime.rs`.
- 2026-03-26: Rewired `apps/vibe-agent/src/task_bridge.rs` and `apps/vibe-agent/src/shell_bridge.rs` to consume the extracted runtime helpers instead of relying on `main.rs` implementations.
- 2026-03-26: Reduced `apps/vibe-agent/src/main.rs` to agent bootstrap, registration, heartbeat, shared state, and top-level module wiring.
- 2026-03-26: Extended `apps/vibe-app/src/types.ts` and `apps/vibe-app/src/lib/api.ts` to cover relay-backed port-forward types and create/close API calls.
- 2026-03-26: Extended `apps/vibe-app/src/stores/control.ts` with port-forward state, polling, selection sync, and create/close actions.
- 2026-03-26: Added a dedicated port-forward management panel to `apps/vibe-app/src/views/DashboardView.vue` and supporting styles in `apps/vibe-app/src/styles.css`.
- 2026-03-26: Completed the first frontend port-forward surface using existing relay APIs and polling-only synchronization.
- 2026-03-26: Aligned the default agent capability advertisement in `apps/vibe-agent/src/main.rs` with the currently delivered MVP surface instead of declaring unfinished capabilities by default.
- 2026-03-26: Added a regression test to keep `file_sync`, `workspace_browse`, and `notifications` out of the default advertised capability set until those flows are implemented.

## Verification Log

- 2026-03-26: `cargo fmt --all` succeeded after the capability-advertisement alignment.
- 2026-03-26: `cargo test -p vibe-agent -- --nocapture` succeeded after the capability-advertisement alignment, 17 tests passed.
- 2026-03-26: `cd apps/vibe-app && npm run build` succeeded after the capability-advertisement alignment.
- 2026-03-26: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the capability-advertisement alignment.
- 2026-03-26: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the frontend port-forward iteration.
- 2026-03-26: `cd apps/vibe-app && npm run build` succeeded after the frontend port-forward iteration.
- 2026-03-26: `cargo fmt --all` succeeded.
- 2026-03-26: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded.
- 2026-03-26: `cargo test -p vibe-agent -- --nocapture` succeeded, 16 tests passed.
- 2026-03-26: `cargo test -p vibe-relay -- --nocapture` succeeded, 27 tests passed.
- 2026-03-26: `cd apps/vibe-app && npm run build` succeeded.
- 2026-03-26: `cargo fmt --all` succeeded after the relay domain-module extraction.
- 2026-03-26: `cargo test -p vibe-relay -- --nocapture` succeeded after the relay domain-module extraction, 27 tests passed.
- 2026-03-26: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the relay domain-module extraction.
- 2026-03-26: `cd apps/vibe-app && npm run build` succeeded after the relay domain-module extraction.
- 2026-03-26: `cargo fmt --all` succeeded after the agent runtime-module extraction.
- 2026-03-26: `cargo check -p vibe-agent` succeeded after the agent runtime-module extraction.
- 2026-03-26: `cargo test -p vibe-agent -- --nocapture` succeeded after the agent runtime-module extraction, 16 tests passed.
- 2026-03-26: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the agent runtime-module extraction.
- 2026-03-26: `cd apps/vibe-app && npm run build` succeeded after the agent runtime-module extraction.
