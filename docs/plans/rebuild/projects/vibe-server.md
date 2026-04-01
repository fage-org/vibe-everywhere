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
- service-level configuration and startup surface

## Internal Module Map

- `api`: route registration and response shaping
- `auth`: auth guards, QR/account flows, token lifecycle
- `events`: update routing and fanout
- `sessions`: CRUD, history, lifecycle, session update generation
- `presence`: active state and timeout handling
- `storage`: db/files/redis/image subsystems
- `social` / `feed` / `github`: Happy feature parity modules
- `monitoring`: metrics and service health

## Implementation Order

1. auth
2. session lifecycle and updates
3. storage backends required by sessions
4. machines and presence
5. feed/social/github
6. monitoring

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
- update sequencing and encrypted record handling are stable
- route and socket behavior are documented and test-covered

## Deferred Items

- performance optimizations not needed for parity
- feature expansions beyond Happy parity
