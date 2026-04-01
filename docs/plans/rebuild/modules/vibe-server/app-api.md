# Module Plan: vibe-server/app-api

## Purpose

Implement the top-level HTTP API surface that exposes Vibe server functionality to app, agent, and
CLI clients.

## Happy Source Of Truth

- `packages/happy-server/sources/app/api/api.ts`
- `packages/happy-server/sources/app/api/types.ts`
- route modules under `packages/happy-server/sources/app/**`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/api/mod.rs`
  - `src/api/routes.rs`
  - `src/api/types.rs`

## Responsibilities

- register HTTP routes
- attach shared auth and state extractors
- enforce request/response wire compatibility
- group domain routes by subsystem

## Non-Goals

- business logic implementation inside handlers
- socket update transport

## Public Types And Interfaces

- server router builder
- shared API state
- typed request/response DTOs where needed beyond `vibe-wire`

## Data Flow

- HTTP request enters Axum router
- auth middleware enriches request context
- handler delegates to domain module
- response serializes Happy-compatible JSON shape

## Dependencies

- `auth`
- `session-lifecycle`
- `feed`
- `social`
- `github`
- `versions-and-config`
- `axum`, `tokio`, `serde`

## Implementation Steps

1. Build Axum router with versioned prefixes matching Happy route structure.
2. Add request context containing user id, start time, and service state.
3. Define typed DTOs only where the shared wire crate does not already own the shape.
4. Keep handlers thin; delegate to domain services.
5. Add integration tests per route group.

## Edge Cases And Failure Modes

- route registration drift from Happy path names
- inconsistent error payloads across route groups
- missing auth context on protected routes

## Tests

- router smoke test
- protected route auth test
- per-domain happy-path JSON shape tests
- error mapping tests

## Acceptance Criteria

- versioned routes are registered and documented
- protected routes consistently receive auth context
- response shapes match planned contracts

## Open Questions

- None.

## Locked Decisions

- use `axum` as the HTTP framework
- keep route modules thin and domain-specific
- use shared wire types whenever a contract is cross-project
