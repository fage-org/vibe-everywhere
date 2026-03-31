# Iteration Plan v7 Details

## Goals

- make `vibe-agent` the single ACP client runtime for AI task execution
- standardize task conversation flow, session recovery, and model handling on ACP semantics
- eliminate mixed CLI-versus-ACP behavior from the control plane

## Problem Statements

- the relay and app already present ACP as a first-class concept, but runtime behavior still mixes CLI and ACP paths
- `Codex` currently depends on direct CLI invocation details, including flags that have already drifted
- provider metadata exposes ACP support separately from real runtime behavior, which causes UI and backend mismatches

## Implementation Shape

- remove `supports_acp` from shared provider metadata
- keep `ExecutionProtocol` for compatibility, but normalize all new writes and runtime updates to `Acp`
- make provider detection report `Codex` and `OpenCode` as ACP-capable endpoints and mark `Claude Code` unavailable until an ACP path exists
- route all task execution through ACP handling:
  - `OpenCode`: native ACP process
  - `Codex`: embedded ACP adapter over the Codex JSON event stream
- keep `provider_session_id` as the stored ACP conversation/session handle

### Codex Bridge Detail

- add an internal `vibe-agent` subcommand that runs a local ACP server over stdio for Codex
- have the task runtime spawn that bridge instead of invoking Codex as the primary task process
- let `session/new` return a temporary ACP session id and promote it to the canonical Codex thread id after the first `thread.started` event
- emit canonical session id changes back through ACP `session/update` so relay conversation state stores the real Codex handle for future resume calls
- pass model and execution mode into the bridge process at spawn time and keep resume/new command construction compatible with the current Codex CLI
- support at least `initialize`, `session/new`, `session/resume`, `session/prompt`, `session/cancel`, and `session/set_config_option`

## Acceptance Criteria

- relay-created tasks and conversations persist `execution_protocol = acp`
- task updates received from agents never switch tasks back to `cli`
- `Codex` invocation no longer uses `--ask-for-approval`
- unavailable providers do not show up as selectable ACP options in the app

## Validation Requirements

- `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --locked --workspace --all-targets -- --nocapture`
- `cd apps/vibe-app && npm ci && npm run build`
- `./scripts/dual-process-smoke.sh relay_polling`
- `./scripts/dual-process-smoke.sh overlay`

## Completion Notes

- completed in this iteration:
  - `Codex` now runs behind a local ACP bridge with `session/list`, canonical session promotion, and focused bridge coverage
  - relay, agent, and shared models normalize new task and conversation execution semantics to `acp`
  - validation requirements completed, including workspace tests and both dual-process smoke tests
- `Claude Code` ACP support is explicitly deferred beyond `v7`; keep it unavailable and do not reintroduce the CLI execution path as a fallback
