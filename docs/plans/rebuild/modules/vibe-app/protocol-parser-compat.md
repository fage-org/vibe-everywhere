# Module Plan: vibe-app/protocol-parser-compat

## Purpose

Ensure the imported app can parse and render both legacy and session-protocol messages produced by
the Rust backend path.

## Happy Source Of Truth

- `packages/happy-app/sources/**` sync/reducer/parser code that is later imported into
  `packages/vibe-app`
- `packages/happy-wire/src/messages.ts`
- `packages/happy-wire/src/sessionProtocol.ts`

## Target Rust/Vibe Location

- package: `packages/vibe-app`
- expected edit areas:
  - protocol parsing
  - message normalization
  - reducer or state handling

## Responsibilities

- validate parser compatibility with `vibe-wire`
- adapt imported parser code only where required by Rust parity differences already documented
- keep UI behavior stable

## Non-Goals

- protocol redesign

## Public Types And Interfaces

- app-side normalized message model
- parser entrypoints

## Data Flow

- app receives encrypted/decrypted message payloads from server sync
- parser validates wire shape
- reducer normalizes to UI-ready structures

## Dependencies

- `import-and-build`
- `shared/protocol-session.md`
- `shared/data-model.md`
- `vibe-wire` fixtures

## Implementation Steps

1. Inventory imported parser and reducer assumptions.
2. Add wire fixtures generated from `vibe-wire`.
3. Patch only the compatibility seams required for Vibe server outputs.
4. Verify both legacy and session-protocol flows.

## Edge Cases And Failure Modes

- turn-start/turn-end semantics diverging from Happy expectations
- file/tool/service events rendering incorrectly
- parent/sidechain relationships breaking in nested outputs

## Tests

- parser fixture tests for every session event variant
- legacy message compatibility tests
- reducer integration tests for turn lifecycle and tool events

## Acceptance Criteria

- app parser/rendering works with Rust-generated payloads for both protocol families

## Open Questions

- None.

## Locked Decisions

- parser compatibility is validated against `vibe-wire` fixtures, not hand-made app-only samples
