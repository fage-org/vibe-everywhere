# Module Plan: vibe-wire/voice

## Purpose

Implement the shared voice token allow/deny response schema.

## Happy Source Of Truth

- `packages/happy-wire/src/voice.ts`

## Target Rust/Vibe Location

- crate: `crates/vibe-wire`
- file: `src/voice.rs`

## Responsibilities

- define allow and deny response shapes
- expose the top-level discriminated union

## Non-Goals

- voice billing logic
- ElevenLabs integration logic

## Public Types And Interfaces

- `VoiceTokenAllowed`
- `VoiceTokenDenied`
- `VoiceTokenResponse`

## Data Flow

- server computes voice token response
- app and clients consume it as a discriminated wire object

## Dependencies

- `serde`

## Implementation Steps

1. Port allow and deny variants.
2. Encode the `allowed` discriminator directly in the type model.
3. Port deny reasons exactly.
4. Add round-trip and invalid discriminator tests.

## Edge Cases And Failure Modes

- deny reason values must remain constrained to the Happy set
- numeric usage fields must preserve integer semantics

## Tests

- allow response round-trip
- deny response round-trip
- invalid reason test

## Acceptance Criteria

- Rust union is wire-compatible with Happy voice responses

## Open Questions

- None.

## Locked Decisions

- keep this module isolated and small; do not merge with broader billing or auth models
