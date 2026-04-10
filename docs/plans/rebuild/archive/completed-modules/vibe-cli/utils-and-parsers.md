# Module Plan: vibe-cli/utils-and-parsers

## Purpose

Implement the shared internal helper layer for the CLI: low-level utilities, parser helpers,
session metadata factories, deterministic encoding helpers, and system-integration adapters.

## Happy Source Of Truth

- `packages/happy-cli/src/utils/*`
- `packages/happy-cli/src/parsers/*`
- `packages/happy-cli/src/projectPath.ts`
- helper usage across `packages/happy-cli/src/**`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files under:
  - `src/utils/`
  - `src/utils/parsing.rs`

## Responsibilities

- port helper functions reused across providers, daemon, transport, and auth flows
- own special-command parsing and other CLI-local parse helpers
- own session metadata factory helpers that must stay aligned with shared data-model rules
- own deterministic JSON, HMAC/derive-key, time, tmux, env expansion, and process helper utilities
- keep shared helper semantics consistent instead of re-implementing them ad hoc in each provider

## Non-Goals

- canonical shared wire contracts that belong to `vibe-wire`
- top-level command bootstrap that belongs to `bootstrap-and-commands`
- provider-specific business logic

## Public Types And Interfaces

- internal utility functions and helper structs
- parser helpers for special commands and CLI-local syntax
- session metadata factory helpers for runtime modules

## Data Flow

- higher-level CLI modules call helpers for parsing, metadata construction, locking, environment
  expansion, queueing, and OS integration
- helper outputs feed runtime, daemon, sandbox, and transport modules

## Dependencies

- `vibe-wire` for shared schema alignment where needed
- `shared/data-model.md`
- OS/process helper crates as required

## Implementation Steps

1. Inventory Happy helpers by category before porting:
   - crypto/encoding helpers
   - session metadata helpers
   - async queue/iterable helpers
   - file/process/locking helpers
   - tmux and environment helpers
   - command parsers
2. Port helpers that are semantic dependencies of other modules first:
   - `createSessionMetadata`
   - key derivation / HMAC helpers used by auth or encryption
   - environment and path helpers
   - special command parser
3. Move helpers that are only test scaffolding into `testing-fixtures` instead of production code.
4. Add unit tests for helpers with behavior-sensitive compatibility requirements.

## Edge Cases And Failure Modes

- helper drift causing different providers to serialize metadata differently
- parser ambiguity for special commands
- OS-specific helpers behaving differently across Linux/macOS
- race conditions in locking or queue helpers

## Tests

- session metadata factory tests
- parser tests for special commands
- deterministic JSON / encoding tests
- tmux / env expansion tests
- lock and async queue tests

## Acceptance Criteria

- cross-cutting CLI helper logic has one owned module boundary
- runtime modules no longer duplicate metadata, parsing, or low-level process helpers
- helper behavior that affects wire-visible output is test-covered

## Open Questions

- None.

## Locked Decisions

- helper code stays internal to `vibe-cli`; do not promote it to `vibe-wire` unless it defines a
  true cross-project contract
- parsers live with utilities unless they become first-class command modules
