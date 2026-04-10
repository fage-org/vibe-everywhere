# Module Plan: vibe-cli/testing-fixtures

## Purpose

Implement reusable fixtures, integration environment helpers, and runtime test scaffolding for the
CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/testing/*`
- runtime/provider integration tests across `packages/happy-cli/src/**`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files under:
  - `tests/fixtures/`
  - `tests/integration/`
  - helper modules in `src/testing/` if needed

## Responsibilities

- define reusable provider/runtime fixtures
- define integration environment bootstrap helpers
- keep end-to-end test setup deterministic

## Non-Goals

- production runtime logic

## Public Types And Interfaces

- test-only fixtures and harnesses

## Data Flow

- integration tests bootstrap environment
- fixtures produce provider or server state
- tests assert wire-compatible outputs

## Dependencies

### Pass A

- implemented CLI core through `claude-runtime`
- `vibe-server` test environment

### Pass B additions

- remaining provider runtimes
- finalized command wiring from `bootstrap-and-commands` pass B

## Pass Boundaries

- pass A:
  - establish the first provider/runtime harness around the first end-to-end provider slice
  - prove that the shared CLI core can be regression-tested without ad hoc setup
- pass B:
  - broaden the fixture matrix across the remaining providers and late command wiring

## Implementation Steps

1. Pass A: port Happy integration environment concepts into Rust test harnesses for the first
   provider slice.
2. Add fixture data for provider outputs and session protocol mappings.
3. Pass B: extend the harness and fixtures across the remaining provider/runtime modules.
4. Keep test harness code out of production modules where possible.

## Edge Cases And Failure Modes

- flaky process timing
- fixture drift from real provider output

## Tests

- harness self-tests
- one integration smoke test per provider path as it lands

## Acceptance Criteria

- CLI modules can be regression-tested without re-creating scaffolding in each test file

## Open Questions

- None.

## Locked Decisions

- test scaffolding lives with the CLI crate and is not shared as runtime code
- shared helpers live under `tests/fixtures/`, while Cargo integration entrypoints may remain flat
  `tests/*.rs` crates during Wave 5 as long as they reuse the shared harnesses instead of
  re-creating setup logic
