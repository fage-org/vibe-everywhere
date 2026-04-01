# Project Plan: vibe-app-logs

## Purpose

`vibe-app-logs` is the Rust sidecar replacing `happy-app-logs`. It is intentionally deferred until
the main backend and app path are stable.

## Happy Source

- primary source: `packages/happy-app-logs`

## Target Layout

- crate: `crates/vibe-app-logs`
- expected modules:
  - `server`
  - `config`
  - `main`

## Public Interfaces

- binary: `vibe-app-logs`
- local or service-facing log ingestion endpoints required by the app ecosystem

## Internal Module Map

- `server`: log-sidecar runtime
- `config`: configuration and port/env handling

## Implementation Order

1. config
2. log server
3. integration wiring

## Compatibility Requirements

- only implement behavior needed by imported app tooling
- avoid growing this subsystem before the main path is stable

## Testing Strategy

- smoke tests for startup and ingestion
- app-facing integration tests if the imported app depends on it

## Acceptance Criteria

- app tooling requirements that depend on the sidecar are satisfied

## Deferred Items

- unrelated observability expansion
