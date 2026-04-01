# Project Plan: vibe-app

## Purpose

`vibe-app` is the imported Happy app package adapted to Vibe naming, endpoints, and service
contracts. It is not a rewrite project in phase one.

## Happy Source

- primary source: `packages/happy-app`
- supporting sources: `packages/happy-wire`, relevant Happy planning docs

## Target Layout

- package: `packages/vibe-app`
- phase 1 keeps the imported Happy source layout intact
- Vibe-specific changes are isolated behind small adaptation seams first

## Public Interfaces

- Expo / web / desktop app entrypoints
- app configuration and environment surface
- protocol parsing and UI rendering of session and legacy messages
- deep links and account/session UX

## Internal Module Map

- imported app tree stays mostly intact initially
- adapter seams are introduced for:
  - branding and naming
  - API endpoints
  - protocol compatibility
  - Tauri/desktop configuration
  - release and environment rules

## Implementation Order

1. import and build
2. protocol parser compatibility checks
3. API endpoint adaptation
4. branding and naming adaptation
5. desktop/Tauri adaptation
6. release and environment cleanup

## Compatibility Requirements

- UI behavior should remain as close as practical to imported Happy app behavior
- public branding must shift to Vibe
- app must consume Rust wire/server behavior without protocol forks

## Testing Strategy

- import/build validation
- parser compatibility tests
- real app-to-server integration checks
- desktop shell checks when Tauri work begins

## Acceptance Criteria

- imported app runs against Vibe backend path
- Vibe naming is exposed in public surfaces
- protocol rendering stays compatible with locked shared specs

## Deferred Items

- large-scale visual redesign
- structural app rewrites before parity is achieved
