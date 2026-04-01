# Module Plan: vibe-server/utility-apis

## Purpose

Implement the remaining supporting API groups that are required by imported clients but do not
justify their own top-level domain crate during parity phase: KV, push tokens, version, and voice.

## Happy Source Of Truth

- `packages/happy-server/sources/app/api/routes/kvRoutes.ts`
- `packages/happy-server/sources/app/api/routes/pushRoutes.ts`
- `packages/happy-server/sources/app/api/routes/versionRoutes.ts`
- `packages/happy-server/sources/app/api/routes/voiceRoutes.ts`
- `packages/happy-server/sources/app/kv/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-server`
- files:
  - `src/api/kv.rs`
  - `src/api/push.rs`
  - `src/api/version.rs`
  - `src/api/voice.rs`

## Responsibilities

- serve KV routes:
  - `GET /v1/kv/:key`
  - `GET /v1/kv`
  - `POST /v1/kv/bulk`
  - `POST /v1/kv`
- serve push-token routes:
  - `POST /v1/push-tokens`
  - `GET /v1/push-tokens`
  - `DELETE /v1/push-tokens/:token`
- serve `POST /v1/version`
- serve `POST /v1/voice/token`

## Non-Goals

- expanding these APIs beyond Happy parity
- turning each support route group into its own top-level project module during phase one

## Public Types And Interfaces

- KV DTOs and service
- push token DTOs and service
- version check DTOs
- voice token DTOs

## Data Flow

- authenticated app or client requests utility API behavior
- handlers validate auth and call the owning support service
- services persist or return Happy-compatible JSON payloads

## Dependencies

- `auth`
- `storage-db`
- `vibe-wire` for voice-related DTOs where applicable

## Implementation Steps

1. Lock route paths and request/response shapes from `shared/protocol-api-rpc.md`.
2. Implement KV storage/query behavior needed by imported app flows.
3. Implement push token registration/list/delete behavior.
4. Implement version check surface used by app/bootstrap flows.
5. Implement voice token issuance surface and wire-compatible DTOs.
6. Add per-route tests and auth-gating tests.

## Edge Cases And Failure Modes

- duplicate push token registration
- KV bulk request partial misses
- version negotiation drift from imported app assumptions
- voice token provider unavailable

## Tests

- KV get/list/bulk/write tests
- push token create/list/delete tests
- version route test
- voice token route test

## Acceptance Criteria

- every remaining support route group in the shared API spec has an owning module plan
- imported clients no longer depend on unowned “misc API” behavior

## Open Questions

- None.

## Locked Decisions

- KV, push, version, and voice stay grouped as support APIs during parity phase; split them only if
  implementation complexity proves the group too large
