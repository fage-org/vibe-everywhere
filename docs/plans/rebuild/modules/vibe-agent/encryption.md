# Module Plan: vibe-agent/encryption

## Purpose

Implement client-side crypto helpers required by auth, session decryption, and machine decryption.

## Happy Source Of Truth

- `packages/happy-agent/src/encryption.ts`
- related Happy crypto logic used by `happy-cli` and `happy-app`

## Target Rust/Vibe Location

- crate: `crates/vibe-agent`
- file: `src/encryption.rs`

## Responsibilities

- base64/base64url helpers
- key derivation helpers
- NaCl-compatible box/secretbox helpers
- AES/dataKey helpers

## Non-Goals

- server-side key management
- wire schema definition

## Public Types And Interfaces

- encryption helper functions
- `EncryptionVariant`
- bundle encode/decode helpers

## Data Flow

- auth decrypts account secret bundle
- API resolves session and machine record encryption
- session client decrypts incoming messages and state

## Dependencies

- `vibe-wire` for shared format constants if extracted
- `aes-gcm`
- `base64`
- `hmac`
- `sha2`
- `dryoc` for NaCl-compatible primitives

## Implementation Steps

1. Port byte layouts and helpers from Happy.
2. Add deterministic key-derivation tests from Happy vectors.
3. Implement AES/dataKey and legacy bundle helpers.
4. Keep shared constants aligned with `shared/protocol-auth-crypto.md`.

## Edge Cases And Failure Modes

- base64url padding differences
- wrong key variant causing silent decrypt failure
- cross-language nonce or byte-order mismatch

## Tests

- derivation vector tests
- AES/dataKey round-trip tests
- legacy round-trip tests
- box bundle round-trip tests

## Acceptance Criteria

- agent crypto output matches shared vectors and server expectations

## Open Questions

- None.

## Locked Decisions

- use `dryoc` for NaCl-compatible operations and `aes-gcm` for AES bundles
- all byte-format constants are locked by shared protocol docs
