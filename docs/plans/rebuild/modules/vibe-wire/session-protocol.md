# Module Plan: vibe-wire/session-protocol

## Purpose

Implement the canonical session envelope and event union used by the long-term message protocol.

## Happy Source Of Truth

- `packages/happy-wire/src/sessionProtocol.ts`
- `/root/happy/docs/plans/session-protocol-impl.md`
- `/root/happy/docs/plans/session-protocol-unification-v2-draft.md`

## Target Rust/Vibe Location

- crate: `crates/vibe-wire`
- file: `src/session_protocol.rs`

## Responsibilities

- define session roles
- define event variants and payloads
- define envelope invariants
- expose validation helpers used by server and clients

## Non-Goals

- app-side normalized message rendering
- provider-specific message mapping

## Public Types And Interfaces

- `SessionRole`
- `SessionTextEvent`
- `SessionServiceEvent`
- `SessionToolCallStartEvent`
- `SessionToolCallEndEvent`
- `SessionFileEvent`
- `SessionTurnStartEvent`
- `SessionStartEvent`
- `SessionTurnEndStatus`
- `SessionTurnEndEvent`
- `SessionStopEvent`
- `SessionEvent`
- `SessionEnvelope`

## Data Flow

- provider runtimes map raw runtime output into `SessionEnvelope`
- server stores envelopes only as encrypted message content
- app, agent, and CLI parse envelopes using `vibe-wire`

## Dependencies

- `serde`
- optional validation helper crate if needed, but default to local invariant methods

## Implementation Steps

1. Port current Happy event variants exactly as documented in `happy-wire/src/sessionProtocol.ts`.
2. Implement role and event enums with stable serde tagging.
3. Add explicit envelope validation for Happy constraints:
   - `service` must use `agent`
   - `start` must use `agent`
   - `stop` must use `agent`
4. Keep `turn` and `subagent` optional at the type level.
5. Add fixture-based tests for each event variant and each role constraint failure.

## Edge Cases And Failure Modes

- envelope validation must reject mismatched roles for constrained events
- `file.image` metadata must remain optional and partial only in the Happy-supported shape
- future session-protocol variants must be added only by plan update

## Tests

- JSON round-trip per event variant
- valid envelope fixtures
- invalid role/event combination fixtures
- optional `turn` and `subagent` coverage

## Acceptance Criteria

- Rust schema is compatible with current Happy session protocol
- all known event variants are covered by tests
- validation behavior is documented and reusable

## Open Questions

- None. The event surface is intentionally frozen to Happy source-of-truth files.

## Locked Decisions

- implement the current Happy event surface only; do not merge in draft-only event expansions
- validation lives in this module, not in each consumer
- serde field names stay exactly aligned with Happy
