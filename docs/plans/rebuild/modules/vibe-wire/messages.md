# Module Plan: vibe-wire/messages

## Purpose

Implement the top-level encrypted message and update container types shared by server, app, agent,
and CLI.

## Happy Source Of Truth

- `packages/happy-wire/src/messages.ts`
- `packages/happy-wire/src/sessionProtocol.ts`
- `packages/happy-wire/src/legacyProtocol.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-wire`
- file: `src/messages.rs`
- exported from `src/lib.rs`

## Responsibilities

- define encrypted session message content wrappers
- define session-protocol message wrapper with `role: "session"`
- define versioned encrypted value wrappers
- define update body union and update container types

## Non-Goals

- decrypting payloads
- server persistence logic
- app-side normalization logic

## Public Types And Interfaces

- `SessionMessageContent`
- `SessionMessage`
- `SessionProtocolMessage`
- `MessageContent`
- `VersionedEncryptedValue`
- `VersionedNullableEncryptedValue`
- `VersionedMachineEncryptedValue`
- `UpdateNewMessageBody`
- `UpdateSessionBody`
- `UpdateMachineBody`
- `CoreUpdateBody`
- `CoreUpdateContainer`

## Data Flow

- server stores and emits `SessionMessage`
- agent and CLI decrypt `SessionMessage.content`
- app receives socket event `update` carrying `CoreUpdateContainer`
- `CoreUpdateContainer.body` resolves to `new-message`, `update-session`, or `update-machine`
- downstream projects depend on this file for wire-level JSON shape only

## Dependencies

- `message_meta`
- `legacy_protocol`
- `session_protocol`
- `serde`, `serde_json`

## Implementation Steps

1. Port every exported Zod type from Happy into Rust structs/enums with serde derives.
2. Preserve Happy field names exactly using `#[serde(rename = "...")]` where needed.
3. Represent discriminated unions as tagged enums or validated wrapper enums.
4. Add parsing helpers for JSON round-trip testing.
5. Re-export the public container types from `lib.rs`.

## Edge Cases And Failure Modes

- nullable `localId` and encrypted value wrappers must remain distinguishable from missing fields
- `MessageContent` must reject invalid discriminators
- update body parsing must fail cleanly on unknown `t` values

## Tests

- JSON round-trip for each public type
- one fixture for each update body variant
- one fixture for legacy and session-protocol `MessageContent`
- invalid discriminator tests

## Acceptance Criteria

- every Happy container type in `messages.ts` has a Rust equivalent
- serialization matches Happy field names
- tests cover all update body variants

## Open Questions

- None. Any missing container should be added to the crosswalk before implementation.

## Locked Decisions

- serde is the wire serialization layer
- no downstream crate may redefine these containers
- keep types focused on wire shape; helper methods stay minimal
