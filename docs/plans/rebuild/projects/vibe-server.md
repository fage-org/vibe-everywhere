# Project Plan: vibe-server

## Purpose

`vibe-server` is the Rust backend replacing Happy server responsibilities: auth, sessions,
machines, update fanout, storage, feed/social, and supporting APIs needed by the app and clients.

## Happy Source

- primary source: `packages/happy-server`
- supporting source: `packages/happy-wire` and server-focused Happy planning docs

## Target Layout

- crate: `crates/vibe-server`
- top-level module groups:
  - `api`
  - `auth`
  - `events`
  - `sessions`
  - `machines`
  - `presence`
  - `storage`
  - `social`
  - `feed`
  - `github`
  - `monitoring`
  - `config`

## Public Interfaces

- HTTP API routes
- socket update namespace and events
- encrypted session and machine persistence
- machine registration, machine metadata, and daemon-state lifecycle
- service-level configuration and startup surface

## Internal Module Map

- `api`: route registration and response shaping
- `auth`: auth guards, QR/account flows, token lifecycle
- `events`: update routing and fanout
- `sessions`: CRUD, history, lifecycle, session update generation
- `machines`: registration, list/detail, metadata, daemon state, and machine-scoped socket writes
- `presence`: session/machine heartbeat caching, DB flush, active state, and timeout handling
- `account/usage`: account profile, settings, and usage query APIs
- `integrations`: generic connect/vendor token routes plus GitHub-specific flows
- `artifacts/access-keys`: opaque artifact bodies and session-machine access-key APIs
- `utility-apis`: KV, push tokens, version, and voice routes
- `storage`: db/files/redis/image subsystems
- `social` / `feed` / `github`: Happy feature parity modules
- `monitoring`: metrics and service health

## Implementation Order

1. versions/config plus storage foundation
2. auth and event-router pass A
3. session lifecycle and machine lifecycle
4. presence plus minimum socket/API surface
5. files/images and support APIs needed by imported clients
6. connect, GitHub, social, and feed domains
7. event-router/socket/API finalization
8. monitoring

## Compatibility Requirements

- wire shapes must come from `vibe-wire`
- imported app paths must not need server-specific protocol forks
- endpoint and socket behavior should match Happy semantics unless a plan explicitly records a
  Vibe-only adaptation

## Testing Strategy

- route tests
- update stream tests
- encrypted record storage tests
- integration tests with `vibe-agent` and `vibe-cli`

## Acceptance Criteria

- supports minimum auth/session/machine/update flows used by downstream projects
- machine registration and daemon-state updates work with explicit version control
- session and machine presence transitions are deterministic and test-covered
- update sequencing and encrypted record handling are stable
- route and socket behavior are documented and test-covered

## Deferred Items

- performance optimizations not needed for parity
- feature expansions beyond Happy parity
