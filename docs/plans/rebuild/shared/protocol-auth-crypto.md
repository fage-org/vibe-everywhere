# Authentication And Crypto Protocol

## Scope

This file locks the account, session, and machine crypto model that must remain compatible with
Happy behavior.

## Wire Encoding Rules

- standard base64 uses RFC 4648 encoding with `=` padding preserved
- base64url is derived from standard base64 by replacing `+ -> -`, `/ -> _`, then stripping all
  trailing `=`
- base64url decode must reverse those substitutions and restore padding using
  `=` repeated `(4 - len % 4) % 4`
- encrypted JSON payloads are always serialized as UTF-8 bytes of `JSON.stringify(...)` before
  encryption and parsed back as JSON after decryption
- unless a section explicitly says otherwise, encrypted blobs on HTTP/socket/storage surfaces use
  standard base64, not base64url

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

1. generate a random 32-byte ephemeral secret and derive a Curve25519 box keypair from it
2. `POST /v1/auth/account/request` with body `{ publicKey }`, where `publicKey` is standard base64
   of the 32-byte ephemeral public key
3. display QR/deep link `vibe:///account?<base64url(publicKey-bytes)>`
4. poll `POST /v1/auth/account/request` once per second for up to 120 seconds with the same
   `{ publicKey }` body
5. pending response shape is `{ state: 'requested' }`
6. authorized response shape is `{ state: 'authorized', token, response }`
7. `response` is a standard-base64 encoded public-key box bundle that encrypts the raw account
   secret to the ephemeral keypair
8. decrypt `response`, persist `{ token, secret }`, then derive the content keypair from the
   account secret for session and machine access

### Deep-link compatibility rule

- external Vibe docs, QR rendering, and user-visible links must use `vibe:///account?...`
- the opaque query payload remains the same Happy-compatible base64url-encoded public key bytes
- if imported phase-one app code still accepts `happy:///account?...`, support it only inside an
  app adapter; do not document it as the primary Vibe scheme

## Auth Endpoint Surface

The following routes are protocol-locked because imported app/agent flows depend on them:

- `POST /v1/auth/account/request`
  - request body: `{ publicKey }`
  - `publicKey` encoding: standard base64 of 32-byte Curve25519 public key
  - response body: `{ state: 'requested' }` or `{ state: 'authorized', token, response }`
- `POST /v1/auth/account/response`
  - app approval route that delivers the encrypted account-secret response for a pending request
- `POST /v1/auth`
  - request body: `{ publicKey, challenge, signature }`
  - all three binary fields serialize as standard base64 strings
- `POST /v1/auth/request`
  - create challenge/approval requests for non-account auth flows
- `GET /v1/auth/request/status`
  - poll status for non-account auth flows
- `POST /v1/auth/response`
  - submit approved response for non-account auth flows

## Key Derivation Rules

- derive the root node with `HMAC-SHA512(key = UTF8("${usage} Master Seed"), data = seed)`
- split the 64-byte HMAC output into:
  - `key = I[0..32)`
  - `chainCode = I[32..64)`
- derive child nodes with `HMAC-SHA512(key = chainCode, data = 0x00 || UTF8(index))`
- `deriveKey(master, usage, path)` means: compute the root using `usage`, then walk each string
  path segment as a child index
- content keypair derivation is compatibility-locked to the literal Happy inputs:
  - `usage = "Happy EnCoder"`
  - `path = ["content"]`
  - this literal must not be renamed to `Vibe EnCoder` until a full protocol migration plan exists
- derive the content box secret key by hashing the derived 32-byte seed with SHA-512 and taking the
  first 32 bytes
- derive the content Curve25519 keypair from that 32-byte secret key using the NaCl box keypair
  constructor
- auth challenge signing derives an Ed25519 signing keypair directly from the 32-byte account
  secret seed
- auth challenges are random 32-byte values signed with detached Ed25519 signatures
- lock test vectors in module-level tests before consumers depend on the output
- never reimplement the derivation formula differently in multiple crates; centralize reusable
  logic in `vibe-wire` or a shared internal crate if extraction becomes necessary

## Encryption Variants

Two record encryption modes must be supported:

### `legacy`

- no `data_encryption_key`
- use the 32-byte account secret directly as the NaCl `secretbox` key
- payloads use the legacy bundle layout `nonce(24) + ciphertext`

### `dataKey`

- record includes `data_encryption_key`
- decrypt session or machine data key using the content private key
- the decrypted data key is a 32-byte AES-256-GCM key
- decrypt metadata, daemon state, agent state, or message content with the resulting per-record
  data key

## Bundle Formats

The following bundle types are fixed and must have cross-language compatibility tests:

| Bundle | Byte layout | Notes |
| --- | --- | --- |
| legacy payload bundle | `nonce(24) + ciphertext` | NaCl `secretbox`; no version byte |
| public-key box bundle | `ephemeralPublicKey(32) + nonce(24) + ciphertext` | NaCl `box`; used for account auth responses and record data keys |
| AES/dataKey payload bundle | `version(1) + nonce(12) + ciphertext + authTag(16)` | AES-256-GCM; only version `0x00` is valid in phase 1 |
| stored `dataEncryptionKey` string | `version(1) + public-key box bundle` | outer version byte is currently `0x00`, then standard-base64 encode the whole byte array |

### `dataEncryptionKey` storage rule

1. encrypt the raw 32-byte record data key to the content public key using the public-key box
   bundle format
2. prepend version byte `0x00`
3. standard-base64 encode the full byte array
4. when reading, standard-base64 decode, require first byte `0x00`, strip it, then decrypt the
   remaining public-key box bundle

### AES/dataKey payload rule

- version byte is mandatory and currently must equal `0x00`
- nonce size is 12 bytes
- auth tag size is 16 bytes
- minimum valid bundle size is `1 + 12 + 16 = 29` bytes

### Legacy payload rule

- nonce size is 24 bytes
- there is no version byte
- decryption failure returns null/error without partial data exposure

## Token And Challenge Rules

- token-based HTTP and socket auth remains the primary online auth method after account linking
- bearer tokens are stored after successful account linking and sent unchanged over HTTP/socket auth
- challenge-based signing uses:
  - `publicKey`: Ed25519 public key derived from the account secret seed
  - `challenge`: random 32-byte challenge encoded as standard base64
  - `signature`: detached Ed25519 signature over the raw challenge bytes, encoded as standard
    base64
- challenge-based refresh and signing behavior must remain compatible with Happy semantics where
  present in the source-of-truth modules

## Compatibility Strategy

- `vibe-server` must accept and emit the same encrypted field shapes expected by imported app code
- `vibe-agent` and `vibe-cli` must share vectors and fixtures for crypto behavior
- all cross-language crypto vectors must be generated before app or server integration begins
- compatibility vectors must cover:
  - base64 and base64url round-trips
  - content keypair derivation
  - legacy payload encrypt/decrypt
  - AES/dataKey payload encrypt/decrypt
  - public-key box bundle encrypt/decrypt
  - `dataEncryptionKey` version-byte wrapping
