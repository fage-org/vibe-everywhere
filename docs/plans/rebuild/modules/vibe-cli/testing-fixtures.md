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

- all implemented CLI modules
- `vibe-server` test environment

## Implementation Steps

1. Port Happy integration environment concepts into Rust test harnesses.
2. Add fixture data for provider outputs and session protocol mappings.
3. Keep test harness code out of production modules where possible.

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
