# Module Plan: vibe-wire/legacy-protocol

## Purpose

Implement the legacy user and agent message wrappers that remain necessary for compatibility.

## Happy Source Of Truth

- `packages/happy-wire/src/legacyProtocol.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-wire`
- file: `src/legacy_protocol.rs`

## Responsibilities

- define legacy user message schema
- define legacy agent message schema
- define the legacy discriminated union used in encrypted message content

## Non-Goals

- interpreting agent `content.type`
- converting legacy messages to session protocol

## Public Types And Interfaces

- `UserMessage`
- `AgentMessage`
- `LegacyMessageContent`

## Data Flow

- user text input from app, CLI, or agent may still emit legacy user messages
- agent outputs may still travel as legacy agent messages during staged migration
- app parser must keep understanding this family until retirement is explicitly planned

## Dependencies

- `message_meta`
- `serde`
- `serde_json::Value` for pass-through agent content

## Implementation Steps

1. Implement exact user and agent wrapper types.
2. Preserve pass-through semantics for agent content while requiring `type`.
3. Add tagged enum parsing for the legacy discriminated union.
4. Re-export from `lib.rs`.

## Edge Cases And Failure Modes

- legacy agent messages must allow unknown content keys
- user messages must reject non-text content in the canonical schema
- optional `localKey` and `meta` fields must serialize compatibly

## Tests

- user message JSON round-trip
- agent message pass-through round-trip
- discriminated union parse tests
- invalid user-content tests

## Acceptance Criteria

- legacy content shapes match Happy exactly
- pass-through agent content retains unknown fields
- compatibility tests cover legacy parse failures

## Open Questions

- None. The legacy shape is fixed until a future retirement plan exists.

## Locked Decisions

- represent agent content as a typed wrapper around generic JSON value with required `type`
- do not narrow legacy agent content during parity phase
