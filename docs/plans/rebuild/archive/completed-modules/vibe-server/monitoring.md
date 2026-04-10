# Module Plan: vibe-server/monitoring

## Purpose

Implement metrics and basic operational visibility equivalents needed by the Vibe server.

## Happy Source Of Truth

- `packages/happy-server/sources/app/monitoring/metrics.ts`
- `packages/happy-server/sources/app/monitoring/metrics2.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/monitoring/mod.rs`
  - `src/monitoring/metrics.rs`

## Responsibilities

- expose service metrics
- track request and update throughput
- provide operational counters needed for parity and debugging

## Non-Goals

- full observability platform rollout

## Public Types And Interfaces

- metrics registry bootstrap
- metrics route handler if exposed

## Data Flow

- HTTP, socket, and storage operations increment counters
- metrics endpoint exports current snapshot

## Dependencies

- `app-api`
- `socket-updates`
- `prometheus`-compatible crate

## Implementation Steps

1. Define baseline metric set.
2. Wire counters/histograms into request and update paths.
3. Expose metrics endpoint if required.
4. Add tests for registry and endpoint output.

## Edge Cases And Failure Modes

- duplicate metric registration
- missing labels or unbounded cardinality

## Tests

- metrics registry test
- endpoint snapshot test

## Acceptance Criteria

- core service paths have basic metrics coverage

## Open Questions

- None.

## Locked Decisions

- keep metric cardinality low
- instrument only core parity paths initially
