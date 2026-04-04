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
  - `src/api/types.rs`
  - route-group modules under `src/api/`

## Responsibilities

- register HTTP routes
- attach shared auth and state extractors
- enforce request/response wire compatibility
- group domain routes by subsystem
- preserve exact Happy-compatible versioned path names for:
  - auth
  - sessions
  - machines
  - account and usage
  - connect/github
  - social/feed/user
  - kv
  - push tokens
  - artifacts
  - access keys
  - version
  - voice

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

### Pass A

- `auth`
- `session-lifecycle`
- `machine-lifecycle`
- `versions-and-config`
- `axum`, `tokio`, `serde`

### Pass B additions

- `account-and-usage`
- `connect-vendors`
- `artifacts-and-access-keys`
- `utility-apis`
- `feed`
- `social`
- `github`

## Pass Boundaries

- pass A:
  - stand up the router skeleton, shared middleware, and the minimum auth/session/machine routes
  - lock the request context and route registration shape used by later route groups
- pass B:
  - register the remaining account, connect, utility, social, feed, artifact, and GitHub route
    groups after their services exist
  - keep handlers thin and avoid reworking pass-A router structure

## Implementation Steps

1. Pass A: build the Axum router with versioned prefixes matching Happy route structure.
2. Add request context containing user id, start time, and service state.
3. Lock the full route inventory from `shared/protocol-api-rpc.md` before implementing handlers.
4. Define typed DTOs only where the shared wire crate does not already own the shape.
5. Keep handlers thin; delegate to domain services.
6. Pass B: register the remaining route groups once their domain services are real.
7. Add integration tests per route group.

## Edge Cases And Failure Modes

- route registration drift from Happy path names
- inconsistent error payloads across route groups
- missing auth context on protected routes

## Tests

- router smoke test
- protected route auth test
- per-domain happy-path JSON shape tests
- error mapping tests

## Follow-Up Status

- Wave 2 validation hardening completed for pass-A HTTP routes, including automated coverage for
  `/v2` session queries, forward-paged `/v3` history reads, machine listing, and delete-path
  regressions.

## Acceptance Criteria

- versioned routes are registered and documented
- protected routes consistently receive auth context
- response shapes match planned contracts
- no route groups required by imported Happy clients are missing from the router

## Open Questions

- None.

## Locked Decisions

- use `axum` as the HTTP framework
- keep route modules thin and domain-specific
- use shared wire types whenever a contract is cross-project
