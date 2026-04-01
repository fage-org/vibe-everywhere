# Module Plan: vibe-app/api-endpoint-adaptation

## Purpose

Adapt the imported app to talk to Vibe server endpoints and environment configuration.

## Happy Source Of Truth

- imported `packages/vibe-app` API/config source
- `shared/protocol-api-rpc.md`

## Target Rust/Vibe Location

- package: `packages/vibe-app`
- expected edit areas:
  - config files
  - API client helpers
  - environment loaders

## Responsibilities

- replace Happy server URLs and env names
- align app API clients with Vibe route and socket paths
- isolate endpoint translation seams

## Non-Goals

- protocol parser rewrites

## Public Types And Interfaces

- app config env surface
- API base URL resolution
- socket endpoint configuration

## Data Flow

- app loads Vibe env values
- app API clients call Vibe server routes
- live updates connect to Vibe socket path

## Dependencies

- `import-and-build`
- `shared/naming.md`
- `shared/protocol-api-rpc.md`

## Implementation Steps

1. Identify all Happy server URL and env assumptions.
2. Replace them with Vibe config and defaults.
3. Keep endpoint resolution centralized to one adapter seam.
4. Add integration tests against Vibe server stubs.

## Edge Cases And Failure Modes

- hard-coded Happy domains buried in source
- web/mobile/desktop env divergence

## Tests

- config resolution tests
- API client smoke tests
- socket endpoint connection test

## Acceptance Criteria

- app can talk to Vibe server endpoints without source-wide ad hoc patching

## Open Questions

- None.

## Locked Decisions

- endpoint translation stays centralized
- no duplicate base-URL logic across screens/components
