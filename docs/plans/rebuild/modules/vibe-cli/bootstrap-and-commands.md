# Module Plan: vibe-cli/bootstrap-and-commands

## Purpose

Implement the top-level CLI bootstrap, configuration bootstrap, command tree, and dispatch wiring
for `vibe`.

## Happy Source Of Truth

- `packages/happy-cli/bin/happy.mjs`
- `packages/happy-cli/bin/happy-dev.mjs`
- `packages/happy-cli/bin/happy-mcp.mjs`
- `packages/happy-cli/src/index.ts`
- `packages/happy-cli/src/configuration.ts`
- `packages/happy-cli/src/projectPath.ts`
- `packages/happy-cli/src/lib.ts`
- `packages/happy-cli/src/commands/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/main.rs`
  - `src/bootstrap.rs`
  - `src/config.rs`
  - `src/commands/mod.rs`
  - command submodules under `src/commands/`

## Responsibilities

- parse the top-level `vibe` command tree
- bootstrap global configuration before command execution
- route subcommands to auth, daemon, provider, sandbox, resume, and runtime modules
- expose a reusable library/bootstrap surface if the CLI crate needs internal embedding
- centralize exit-code policy and top-level error rendering

## Non-Goals

- provider runtime logic
- wire-schema definition
- low-level utility helpers owned by `utils-and-parsers`

## Public Types And Interfaces

- CLI command enum/tree
- bootstrap context
- top-level command dispatcher

## Data Flow

- process starts in `main`
- bootstrap resolves config, working paths, logging, and runtime prerequisites
- command parser resolves subcommand and flags
- dispatcher invokes the owning domain module and normalizes exit status/output

## Dependencies

### Pass A

- `ui-terminal`
- `utils-and-parsers`

### Pass B additions

- `auth`
- `daemon`
- `sandbox`
- `persistence-resume`
- `builtin-modules`
- provider/runtime modules finalized in:
  - `claude-runtime`
  - `codex-runtime`
  - `gemini-runtime`
  - `openclaw-runtime`
  - `agent-acp`

## Pass Boundaries

- pass A:
  - define config/bootstrap ownership
  - stand up the top-level command tree and parser skeleton
  - keep downstream command handlers behind stable interfaces or placeholders
- pass B:
  - wire real auth, daemon, sandbox, resume, and provider/runtime handlers
  - finalize user-facing command coverage and exit semantics

## Implementation Steps

1. Pass A: port the top-level Happy CLI command surface from `index.ts` and define one typed
   bootstrap context consumed by downstream command handlers.
2. Keep command dispatch out of domain modules so auth, daemon, and provider modules stay callable
   as services.
3. Pass B: wire command families in stable order:
   - auth/connect
   - daemon/status/install
   - sandbox
   - session/runtime entrypoints
4. Lock top-level exit codes and error-to-stderr formatting behavior.
5. Add parsing and dispatch tests for each command family.

## Edge Cases And Failure Modes

- conflicting global flags
- invalid subcommand combinations
- bootstrap failure before logging is initialized
- domain modules returning inconsistent exit semantics

## Tests

- CLI parsing tests
- command dispatch tests
- exit-code mapping tests
- bootstrap context initialization tests

## Acceptance Criteria

- all planned `vibe` command families are reachable from one stable entrypoint
- top-level config/bootstrap logic is not duplicated across modules
- CLI exit behavior is deterministic and test-covered

## Open Questions

- None.

## Locked Decisions

- use `clap` for command parsing
- top-level command wiring is its own module boundary, not hidden inside auth or UI modules
- global configuration bootstrap happens once before command dispatch
