# Module Plan: vibe-cli/builtin-modules

## Purpose

Implement local helper modules equivalent to Happy built-ins such as ripgrep, difftastic, watcher,
and proxy helpers.

## Happy Source Of Truth

- `packages/happy-cli/src/modules/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files under:
  - `src/modules/common/`
  - `src/modules/ripgrep/`
  - `src/modules/difftastic/`
  - `src/modules/watcher/`
  - `src/modules/proxy/`

## Responsibilities

- provide local helper capabilities relied on by provider runtimes
- keep module responsibilities scoped and testable

## Non-Goals

- inventing new tool modules beyond Happy parity

## Public Types And Interfaces

- helper module APIs per builtin capability

## Data Flow

- provider runtime requests helper capability
- builtin module performs local work and returns structured result

## Dependencies

- `sandbox`
- OS/process helper crates as needed

## Implementation Steps

1. Port builtin modules in dependency order:
   - common
   - ripgrep
   - difftastic
   - watcher
   - proxy
2. Keep each builtin isolated in its own module tree.
3. Add unit tests matching Happy behaviors.

## Edge Cases And Failure Modes

- host tool missing
- file watcher race conditions
- proxy lifecycle leaks

## Tests

- per-module unit tests
- missing-tool fallback tests
- watcher/proxy smoke tests

## Acceptance Criteria

- provider runtimes have access to required local helper modules with stable APIs

## Open Questions

- None.

## Locked Decisions

- keep builtin capabilities as internal modules, not separate crates during parity phase
