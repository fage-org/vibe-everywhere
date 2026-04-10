# Project Plan: vibe-wire

## Purpose

`vibe-wire` is the canonical shared-contract crate. Every transport, schema, and protocol type that
crosses subsystem boundaries must live here first.

## Happy Source

- primary source: `packages/happy-wire`
- supporting source: protocol planning docs under `/root/happy/docs/plans/`

## Target Layout

- crate: `crates/vibe-wire`
- expected modules:
  - `messages`
  - `session_protocol`
  - `legacy_protocol`
  - `message_meta`
  - `voice`
  - optional `compat` or `fixtures` only if justified by module plans

## Public Interfaces

- serde-ready structs and enums for:
  - legacy protocol messages
  - session-protocol envelopes and events
  - encrypted message containers
  - update containers
  - message metadata
  - voice token responses
- validation helpers for server and clients
- compatibility fixtures or test-vector helpers

## Internal Module Map

- `messages`: message/update containers and top-level unions
- `session_protocol`: envelope and event union
- `legacy_protocol`: legacy message content wrappers
- `message_meta`: metadata shared by user and session messages
- `voice`: voice token allow/deny responses

## Implementation Order

1. `message_meta`
2. `legacy_protocol`
3. `session_protocol`
4. `messages`
5. `voice`

## Compatibility Requirements

- preserve Happy field names on the wire
- preserve optionality and pass-through behavior where Happy relies on it
- reject explicit JSON `null` for fields that are optional in Happy but not nullable
- do not add Vibe-only fields to shared wire types during parity phase

## Testing Strategy

- unit tests per module
- JSON round-trip tests
- invalid-input tests for optional-vs-nullable field behavior
- fixture coverage for every session event type
- cross-language compatibility vectors where applicable
- cross-language validation against Happy schemas for published fixtures

## Acceptance Criteria

- all shared contracts required by server, app, agent, and CLI are implemented in Rust
- no downstream crate defines duplicate public wire structs
- every module exposes stable docs or comments for field semantics

## Deferred Items

- any wire surface not present in Happy source-of-truth files
- speculative protocol redesigns
