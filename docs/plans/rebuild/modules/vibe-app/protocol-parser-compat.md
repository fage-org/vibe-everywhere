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
- normalize server-owned late support-domain durable updates where imported app code currently keeps
  richer app-local projections
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
- `shared/protocol-api-rpc.md`
- `shared/data-model.md`
- `vibe-wire` fixtures

## Implementation Steps

1. Inventory imported parser and reducer assumptions, including late support-domain update bodies.
2. Add wire fixtures generated from `vibe-wire`.
3. Add adapter normalization only where imported app schemas are wider than the canonical server
   transport.
4. Patch only the compatibility seams required for Vibe server outputs.
5. Verify both legacy/session-protocol flows and late support-domain updates.

## Edge Cases And Failure Modes

- turn-start/turn-end semantics diverging from Happy expectations
- file/tool/service events rendering incorrectly
- server-emitted `relationship-updated` or `new-feed-post` bodies not matching imported app-local
  parser assumptions
- parent/sidechain relationships breaking in nested outputs

## Tests

- parser fixture tests for every session event variant
- legacy message compatibility tests
- durable update normalization tests for app-local projections such as relationship and feed updates
- reducer integration tests for turn lifecycle and tool events

## Acceptance Criteria

- app parser/rendering works with Rust-generated payloads for both protocol families and for the
  late support-domain durable updates consumed by the imported sync layer

## Open Questions

- None.

## Locked Decisions

- parser compatibility is validated against `vibe-wire` fixtures, not hand-made app-only samples
- normalization of late support-domain durable updates stays in the app adapter/parser seam; it must
  not rewrite the canonical server transport documented in shared specs
