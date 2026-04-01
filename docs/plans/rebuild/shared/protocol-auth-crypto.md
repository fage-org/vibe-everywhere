# Authentication And Crypto Protocol

## Scope

This file locks the account, session, and machine crypto model that must remain compatible with
Happy behavior.

## Credential Model

### Agent credentials

- path: `~/.vibe/agent.key`
- stored fields:
  - `token`
  - `secret` as base64

### CLI credentials

- path: `~/.vibe/access.key`
- stored fields are defined by the owning CLI module plans and must remain consistent with Happy
  semantics

## Account Authentication Flow

The remote-control client path mirrors Happy account linking:

1. generate ephemeral box keypair
2. POST account request with public key
3. display QR code using `vibe:///account?...`
4. poll authorization request state
5. on success, decrypt returned account secret bundle
6. store token and secret locally
7. derive content keypair from the account secret for session and machine record access

## Key Derivation Rules

- derive content keypair deterministically from the account secret
- keep the derivation tree compatible with Happy logic
- lock test vectors in module-level tests before consumers depend on the output
- never reimplement the derivation formula differently in multiple crates; centralize reusable
  logic in `vibe-wire` or a shared internal crate if extraction becomes necessary

## Encryption Variants

Two record encryption modes must be supported:

### `legacy`

- no `data_encryption_key`
- use account secret directly
- payloads match Happy legacy bundle format

### `dataKey`

- record includes `data_encryption_key`
- decrypt session or machine data key using the content private key
- decrypt metadata or messages with the resulting per-record data key

## Bundle Formats

The following bundle types must be implemented with compatibility tests:

- account-response public-key encrypted box bundle
- `data_encryption_key` bundle with version byte prefix
- AES/dataKey encrypted payload bundle
- legacy encrypted payload bundle

Module plans must lock:

- exact nonce sizes
- version bytes
- base64/base64url rules
- byte layout order

## Token And Challenge Rules

- token-based HTTP and socket auth remains the primary online auth method after account linking
- challenge-based refresh and signing behavior must remain compatible with Happy semantics where
  present in the source-of-truth modules

## Compatibility Strategy

- `vibe-server` must accept and emit the same encrypted field shapes expected by imported app code
- `vibe-agent` and `vibe-cli` must share vectors and fixtures for crypto behavior
- all cross-language crypto vectors must be generated before app or server integration begins
