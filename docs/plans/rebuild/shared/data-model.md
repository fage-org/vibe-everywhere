# Canonical Data Model

## Purpose

This file defines the canonical cross-project data shapes. Public shared wire/container types must
be implemented first in `crates/vibe-wire`. Server-owned HTTP/update DTOs that are not yet part of
the phase-one `happy-wire` export set must still align with this document before they are
implemented in their owning modules.

## Serialization Rules

- Rust structs may use `snake_case` field names internally, but all JSON/socket/storage field names
  must serialize to the Happy-compatible `camelCase` names documented below
- encrypted fields (`metadata`, `agentState`, `daemonState`, `content`, `dataEncryptionKey`) are
  standard-base64 strings of raw encrypted bundle bytes unless a protocol spec explicitly says
  otherwise
- version counters are separate numeric fields, not embedded JSON wrappers, on externally visible
  records
- forward-compatible enums should preserve unknown string values where Happy already allows them
- phase-one compatibility is allowed to keep some `happy*` serialized field names inside encrypted
  payloads; these are protocol-compatibility fields, not public branding

## Server-Wire Entities

### SessionRecord

Canonical serialized shape:

- `id: string`
- `tag: string`
- `seq: number`
- `createdAt: number`
- `updatedAt: number`
- `active: boolean`
- `activeAt: number`
- `metadata: string`
- `metadataVersion: number`
- `agentState: string | null`
- `agentStateVersion: number`
- `dataEncryptionKey: string | null`
- invariants:
  - `tag` is the create-or-load lookup key used by `POST /v1/sessions`
  - `tag` is unique within the owning account scope
  - `seq` is monotonically increasing per session
  - `metadata` is always present and encrypted
  - `agentState` may be null and encrypted when present
  - `dataEncryptionKey == null` implies legacy encryption path
  - `metadataVersion` and `agentStateVersion` increment independently

### MachineRecord

Canonical serialized shape:

- `id: string`
- `seq: number`
- `createdAt: number`
- `updatedAt: number`
- `active: boolean`
- `activeAt: number`
- `metadata: string`
- `metadataVersion: number`
- `daemonState: string | null`
- `daemonStateVersion: number`
- `dataEncryptionKey: string | null`
- invariants:
  - current Happy persistence treats `id` as a globally unique primary key even though machine HTTP
    lookups are scoped by `(accountId, id)`
  - `metadata` is always present and encrypted
  - machine metadata and daemon state version independently
  - machine encryption follows the same account-derived pattern as sessions
  - `active` and `activeAt` are server-owned liveness fields, not encrypted metadata fields

### MachineHttpRecord

Canonical serialized shape:

- `id: string`
- `seq: number`
- `createdAt: number`
- `updatedAt: number`
- `active: boolean`
- `activeAt: number`
- `metadata: string`
- `metadataVersion: number`
- `daemonState: string | null`
- `daemonStateVersion: number`
- `dataEncryptionKey: string | null`
- invariants:
  - this is the server-owned HTTP DTO used by `GET /v1/machines` and `GET /v1/machines/:id`
  - current Happy GET machine surfaces include `seq`
  - `metadata` is always present and encrypted on current Happy GET surfaces
  - `activeAt` is the HTTP field name even though Happy persistence stores the timestamp as
    `lastActiveAt`
  - this DTO remains server-owned in phase one and is not automatically promoted into `vibe-wire`

### CreateOrLoadMachineBody

Canonical serialized shape:

- `id: string`
- `metadata: string`
- `daemonState?: string`
- `dataEncryptionKey?: string | null`
- invariants:
  - `id` is matched within the authenticated account scope
  - current Happy persistence also makes `id` globally unique, so reusing the same machine id under
    a different account is a conflict rather than a second machine record
  - `metadata` is required and encrypted
  - current Happy request schema accepts omitted `daemonState` or an encrypted string, but not
    explicit `null`
  - omitted or null `dataEncryptionKey` selects the legacy encryption path
  - current Happy create path stores `daemonState = null` and `daemonStateVersion = 0` when
    `daemonState` is omitted

### CreateOrLoadMachineHttpRecord

Canonical serialized shape:

- `id: string`
- `createdAt: number`
- `updatedAt: number`
- `active: boolean`
- `activeAt: number`
- `metadata: string`
- `metadataVersion: number`
- `daemonState: string | null`
- `daemonStateVersion: number`
- `dataEncryptionKey: string | null`
- invariants:
  - this is the server-owned HTTP DTO used by `POST /v1/machines`
  - current Happy `POST /v1/machines` responses omit `seq` on both create and load paths
  - field names otherwise match the current create-or-load response surface
  - this DTO remains server-owned in phase one and is not automatically promoted into `vibe-wire`

### CreateOrLoadMachineResponse

Canonical serialized shape:

- `machine: CreateOrLoadMachineHttpRecord`
- invariants:
  - response shape is identical on both create and load-by-id paths

### MachineDetailResponse

Canonical serialized shape:

- `machine: MachineHttpRecord`
- invariants:
  - canonical response wrapper for `GET /v1/machines/:id`

### SessionHttpRecord

Canonical serialized shape:

- `id: string`
- `seq: number`
- `createdAt: number`
- `updatedAt: number`
- `active: boolean`
- `activeAt: number`
- `metadata: string`
- `metadataVersion: number`
- `agentState: string | null`
- `agentStateVersion: number`
- `dataEncryptionKey: string | null`
- `lastMessage?: SessionMessage | null`
- invariants:
  - this is the server-owned HTTP DTO used by `/v1/sessions`, `/v2/sessions/active`,
    `/v2/sessions`, and `POST /v1/sessions`
  - `tag` is intentionally not part of the current phase-one session HTTP response payloads even
    though create-or-load is keyed by tag
  - `/v1/sessions` and `POST /v1/sessions` currently emit explicit `lastMessage: null` placeholders
  - `/v2` session list surfaces currently omit `lastMessage`
  - this DTO remains server-owned in phase one and is not automatically promoted into `vibe-wire`

### SessionListResponse

Canonical serialized shape:

- `sessions: SessionHttpRecord[]`
- invariants:
  - canonical response wrapper for `GET /v1/sessions`

### ActiveSessionListResponse

Canonical serialized shape:

- `sessions: SessionHttpRecord[]`
- invariants:
  - canonical response wrapper for `GET /v2/sessions/active`
  - route-level active-window semantics are locked in `shared/protocol-api-rpc.md`

### CursorPagedSessionListResponse

Canonical serialized shape:

- `sessions: SessionHttpRecord[]`
- `nextCursor: string | null`
- `hasNext: boolean`
- invariants:
  - canonical response wrapper for `GET /v2/sessions`
  - cursor formatting and pagination semantics are locked in `shared/protocol-api-rpc.md`

### CreateOrLoadSessionBody

Canonical serialized shape:

- `tag: string`
- `metadata: string`
- `agentState?: string | null`
- `dataEncryptionKey?: string | null`
- invariants:
  - `tag` is matched within the authenticated account scope
  - `metadata` is required and encrypted
  - omitted or null `dataEncryptionKey` selects the legacy encryption path
  - current Happy phase-one handlers accept `agentState` for compatibility but do not persist it
    during create-or-load

### CreateOrLoadSessionResponse

Canonical serialized shape:

- `session: SessionHttpRecord`
- invariants:
  - response shape is identical on both create and load-by-tag paths

### SessionHistoryResponse

Canonical serialized shape:

- `messages: SessionMessage[]`
- invariants:
  - canonical response wrapper for `GET /v1/sessions/:sessionId/messages`
  - route-level ordering and limit semantics are locked in `shared/protocol-api-rpc.md`

### SessionMessage

Canonical serialized shape:

- `id: string`
- `seq: number`
- `localId: string | null`
- `content: SessionMessageContent`
- `createdAt: number`
- `updatedAt: number`
- invariants:
  - `content` is always the encrypted wrapper defined below
  - decrypted `content.c` resolves to either legacy message content or session-protocol content
  - `localId` is the idempotency key for v3 bulk message insertion
  - decoders must accept omitted `localId` from Happy-compatible producers and normalize it to the
    same local nullable projection as `localId = null`

### SessionMessageContent

Canonical serialized shape:

- `t: 'encrypted'`
- `c: string`
- invariants:
  - `t` is currently fixed to `encrypted`
  - `c` is a standard-base64 encrypted payload bundle

### LegacyUserMessage

Canonical serialized shape:

- `role: 'user'`
- `content: { type: 'text'; text: string }`
- `localKey?: string`
- `meta?: MessageMeta`
- invariants:
  - legacy user messages are text-only at this wire layer
  - `localKey` remains a legacy decrypted-payload field and is distinct from outer
    `SessionMessage.localId`

### LegacyAgentMessage

Canonical serialized shape:

- `role: 'agent'`
- `content: { type: string, ...unknown }`
- `meta?: MessageMeta`
- invariants:
  - `content.type` is required
  - unknown additional keys inside `content` must round-trip unchanged during parity phase

### LegacyMessageContent

Canonical serialized union:

- `LegacyUserMessage`
- `LegacyAgentMessage`
- invariants:
  - discriminates by top-level `role`
  - only `user` and `agent` are valid legacy discriminators

### SessionProtocolMessage

Canonical serialized shape:

- `role: 'session'`
- `content: SessionEnvelope`
- `meta?: MessageMeta`
- invariants:
  - this wrapper is distinct from `SessionEnvelope.role`
  - this is the only canonical wrapper for session-protocol payloads inside encrypted message
    content

### MessageContent

Canonical serialized union:

- `LegacyUserMessage`
- `LegacyAgentMessage`
- `SessionProtocolMessage`
- invariants:
  - discriminates first by top-level `role`
  - decoders must reject unknown top-level `role` values at this wire layer

### MessageMeta

Canonical serialized fields:

- `sentFrom?: string`
- `permissionMode?: 'default' | 'acceptEdits' | 'bypassPermissions' | 'plan' | 'read-only' | 'safe-yolo' | 'yolo'`
- `model?: string | null`
- `fallbackModel?: string | null`
- `customSystemPrompt?: string | null`
- `appendSystemPrompt?: string | null`
- `allowedTools?: string[] | null`
- `disallowedTools?: string[] | null`
- `displayText?: string`
- invariants:
  - field names serialize to Happy-compatible shapes
  - shared by both legacy decrypted payloads and `SessionProtocolMessage`
  - consumers must preserve unknown-compatible nullability behavior where Happy allows it

### SessionRole

Canonical serialized enum:

- `'user'`
- `'agent'`

### SessionEnvelope

Canonical serialized fields:

- `id: string`
- `time: number`
- `role: SessionRole`
- `turn?: string`
- `subagent?: string`
- `ev: SessionEvent`
- invariants:
  - `role` is limited to the current session role enum
  - `subagent` must be a cuid2 string when present
  - envelope ids are globally unique
  - agent service/start/stop events must use agent role
  - turn-scoped agent events carry `turn` once a turn has started

### VersionedEncryptedValue

Canonical serialized shape:

- `version: number`
- `value: string`
- invariants:
  - `value` is an encrypted standard-base64 payload
  - used when the updated field is required when present in the update body

### VersionedNullableEncryptedValue

Canonical serialized shape:

- `version: number`
- `value: string | null`
- invariants:
  - `value = null` is only valid for fields whose canonical stored value may be cleared
  - phase-one canonical use is `UpdateSessionBody.agentState`

### VersionedMachineEncryptedValue

Canonical serialized shape:

- `version: number`
- `value: string`
- invariants:
  - serialized shape matches `VersionedEncryptedValue`
  - remains a distinct public type so machine update paths do not silently diverge from session
    update paths

### UpdateNewMessageBody

Canonical serialized shape:

- `t: 'new-message'`
- `sid: string`
- `message: SessionMessage`
- invariants:
  - `sid` is the owning session id for `message`
  - `message.seq` remains session-local ordering; container sequencing is separate

### UpdateSessionBody

Canonical serialized shape:

- `t: 'update-session'`
- `id: string`
- `metadata?: VersionedEncryptedValue | null`
- `agentState?: VersionedNullableEncryptedValue | null`
- invariants:
  - `id` is the session id
  - omitted or null update fields mean the container does not carry a new value for that field
  - when present, each wrapper version supersedes the consumer's last known version for that field

### UpdateMachineBody

Canonical serialized shape:

- `t: 'update-machine'`
- `machineId: string`
- `metadata?: VersionedMachineEncryptedValue | null`
- `daemonState?: VersionedMachineEncryptedValue | null`
- `active?: boolean`
- `activeAt?: number`
- invariants:
  - `machineId` is the machine record id
  - metadata and daemon-state wrappers version independently
  - `active` and `activeAt` may be emitted without encrypted field changes for presence-only
    updates

### CoreUpdateBody

Canonical serialized union:

- `UpdateNewMessageBody`
- `UpdateSessionBody`
- `UpdateMachineBody`
- invariants:
  - discriminates by body field `t`
  - unknown `t` values must fail wire-level validation

### CoreUpdateContainer

Canonical serialized shape:

- `id: string`
- `seq: number`
- `body: CoreUpdateBody`
- `createdAt: number`
- invariants:
  - this is the minimal durable update container family currently exported by
    `packages/happy-wire/src/messages.ts`
  - this subset is sufficient for Wave 1 `vibe-wire` parity and the minimum remote-control path
  - `seq` is allocated independently from session-local or machine-local record sequence counters
  - `body` contains exactly one discriminated update variant
  - phase-one socket `update` transport also carries late support-domain durable update bodies that
    are transport-locked in `shared/protocol-api-rpc.md` until their owning module plans promote
    them into `vibe-wire`

### ActivityEphemeralUpdate

Canonical serialized shape:

- `type: 'activity'`
- `id: string`
- `active: boolean`
- `activeAt: number`
- `thinking?: boolean`
- invariants:
  - `id` is the session id
  - current Happy emitters default `thinking` to `false` when omitted by the caller

### MachineActivityEphemeralUpdate

Canonical serialized shape:

- `type: 'machine-activity'`
- `id: string`
- `active: boolean`
- `activeAt: number`
- invariants:
  - `id` is the machine id
  - this is the primary live machine-presence update family in current Happy flows

### UsageEphemeralUpdate

Canonical serialized shape:

- `type: 'usage'`
- `id: string`
- `key: string`
- `tokens: Record<string, number>`
- `cost: Record<string, number>`
- `timestamp: number`
- invariants:
  - `id` is the session id
  - `tokens` and `cost` preserve provider/model subtotal keys plus any `total` key emitted by the
    reporter

### MachineStatusEphemeralUpdate

Canonical serialized shape:

- `type: 'machine-status'`
- `machineId: string`
- `online: boolean`
- `timestamp: number`
- invariants:
  - this legacy family remains transport-locked even though current Happy emitters primarily use
    `machine-activity`

### EphemeralUpdate

Canonical serialized union:

- `ActivityEphemeralUpdate`
- `MachineActivityEphemeralUpdate`
- `UsageEphemeralUpdate`
- `MachineStatusEphemeralUpdate`
- invariants:
  - discriminates by top-level field `type`
  - unknown `type` values must fail transport-level validation

### GitHubProfile

Canonical serialized shape:

- `id: number`
- `login: string`
- `type: string`
- `site_admin: boolean`
- `avatar_url: string`
- `gravatar_id: string | null`
- `name: string | null`
- `company: string | null`
- `blog: string | null`
- `location: string | null`
- `email: string | null`
- `hireable: boolean | null`
- `bio: string | null`
- `twitter_username: string | null`
- `public_repos: number`
- `public_gists: number`
- `followers: number`
- `following: number`
- `created_at: string`
- `updated_at: string`
- `private_gists?: number`
- `total_private_repos?: number`
- `owned_private_repos?: number`
- `disk_usage?: number`
- `collaborators?: number`
- `two_factor_authentication?: boolean`
- `plan?: { collaborators: number; name: string; space: number; private_repos: number }`
- invariants:
  - phase-one account and user-facing APIs preserve the full Happy GitHub profile field family,
    not only the reduced app-local parsing subset
  - field names remain GitHub-compatible, not Vibe-renamed
  - consumers that only read a subset must tolerate additional GitHub-compatible keys unchanged

### StoredImageRef

Canonical serialized shape:

- `width: number`
- `height: number`
- `thumbhash: string`
- `path: string`
- invariants:
  - this is the server-side stored image reference shape used in persistence and storage helpers
  - `path` is the storage-relative object path, not a public URL

### ImageRef

Canonical serialized shape:

- `width: number`
- `height: number`
- `thumbhash: string`
- `path: string`
- `url: string`
- invariants:
  - this is the client-facing/public transport image reference shape
  - `url` is derived from server/public storage configuration using `path`
  - account, user, and feed-facing APIs must emit `ImageRef`, not `StoredImageRef`

### AccountProfile

Canonical serialized shape:

- `firstName: string | null`
- `lastName: string | null`
- `username: string | null`
- `avatar: ImageRef | null`
- `github: GitHubProfile | null`
- `settings: { value: string | null; version: number } | null`
- `connectedServices: string[]`
- invariants:
  - this is the shared server-side aggregate DTO, not the exact `GET /v1/account/profile` response
  - `update-account` durable updates carry root `id` plus only a partial subset of
    `firstName`, `lastName`, `username`, `avatar`, `github`, and/or `settings`
  - `connectedServices` is HTTP-only in phase one and is not currently emitted by
    `update-account`

### AccountProfileResponse

Canonical serialized shape:

- `id: string`
- `timestamp: number`
- `firstName: string | null`
- `lastName: string | null`
- `username: string | null`
- `avatar: ImageRef | null`
- `github: GitHubProfile | null`
- `connectedServices: string[]`
- invariants:
  - `GET /v1/account/profile` uses this shape
  - account settings are fetched separately via `/v1/account/settings` in Happy phase one
  - imported app consumers must tolerate additive account fields they do not currently read

### AccountSettingsResponse

Canonical serialized shape:

- `settings: string | null`
- `settingsVersion: number`
- invariants:
  - `GET /v1/account/settings` uses this shape
  - `settingsVersion` is the optimistic-concurrency counter for account settings writes

### UpdateAccountSettingsBody

Canonical serialized shape:

- `settings: string | null`
- `expectedVersion: number`
- invariants:
  - `expectedVersion` must be an integer `>= 0`
  - `POST /v1/account/settings` uses optimistic concurrency against `settingsVersion`

### UpdateAccountSettingsResponse

Canonical serialized union:

- `{ success: true, version: number }`
- `{ success: false, error: 'version-mismatch', currentVersion: number, currentSettings: string | null }`
- invariants:
  - this is the HTTP `200` response family for `POST /v1/account/settings`
  - fatal transport errors use a separate `500 { success: false, error }` body documented in
    `shared/protocol-api-rpc.md`

### UsageQueryBody

Canonical serialized shape:

- `sessionId?: string | null`
- `startTime?: number | null`
- `endTime?: number | null`
- `groupBy?: 'hour' | 'day' | null`
- invariants:
  - `startTime` and `endTime` serialize as epoch seconds on the wire
  - omitted or null `groupBy` defaults to `'day'`

### UsageBucket

Canonical serialized shape:

- `timestamp: number`
- `tokens: Record<string, number>`
- `cost: Record<string, number>`
- `reportCount: number`
- invariants:
  - `timestamp` is an epoch-second bucket boundary rounded down to the selected `groupBy` period
  - `tokens` and `cost` keep the original usage-report key families rather than collapsing them to
    a single total

### UsageQueryResponse

Canonical serialized shape:

- `usage: UsageBucket[]`
- `groupBy: 'hour' | 'day'`
- `totalReports: number`
- invariants:
  - aggregated `usage` buckets are sorted by `timestamp asc`
  - `totalReports` counts the raw reports matched before bucketing

### KvEntry

Canonical serialized shape:

- `key: string`
- `value: string`
- `version: number`
- invariants:
  - single-get, list, and bulk-get KV APIs all reuse this value container

### KvListResponse

Canonical serialized shape:

- `items: KvEntry[]`
- invariants:
  - `GET /v1/kv` wraps list results in an object rather than returning a bare array

### KvBulkGetBody

Canonical serialized shape:

- `keys: string[]`
- invariants:
  - `POST /v1/kv/bulk` requires `1..100` keys

### KvBulkGetResponse

Canonical serialized shape:

- `values: KvEntry[]`
- invariants:
  - bulk-get omits missing keys rather than returning positional null placeholders

### KvMutation

Canonical serialized shape:

- `key: string`
- `value: string | null`
- `version: number`
- invariants:
  - callers use `version = -1` when creating a brand-new key in Happy phase one
  - `value = null` deletes the key when the version check succeeds

### KvMutateBody

Canonical serialized shape:

- `mutations: KvMutation[]`
- invariants:
  - `POST /v1/kv` requires `1..100` mutations per request

### KvMutateResponse

Canonical serialized union:

- `{ success: true, results: Array<{ key: string, version: number }> }`
- `{ success: false, errors: Array<{ key: string, error: 'version-mismatch', version: number, value: string | null }> }`
- invariants:
  - the success branch is returned with HTTP `200`
  - the version-mismatch branch is returned with HTTP `409`

### PushTokenRecord

Canonical serialized shape:

- `id: string`
- `token: string`
- `createdAt: number`
- `updatedAt: number`
- invariants:
  - `GET /v1/push-tokens` returns records sorted by `createdAt desc`

### PushTokenListResponse

Canonical serialized shape:

- `tokens: PushTokenRecord[]`
- invariants:
  - the route wraps push tokens inside a `tokens` object property

### VersionCheckBody

Canonical serialized shape:

- `platform: string`
- `version: string`
- `app_id: string`
- invariants:
  - `app_id` remains compatibility-locked snake_case because Happy currently accepts it that way
  - current Happy handlers read `app_id` but do not use it for routing decisions

### VersionCheckResponse

Canonical serialized shape:

- `updateUrl: string | null`
- invariants:
  - `POST /v1/version` always returns HTTP `200`
  - current Happy outdated-platform URLs remain Happy-branded until a shared compatibility
    decision records a Vibe-specific change

### VoiceTokenAllowed

Canonical serialized shape:

- `allowed: true`
- `token: string`
- `agentId: string`
- `elevenUserId: string`
- `usedSeconds: number`
- `limitSeconds: number`
- invariants:
  - this is the allowed branch of `VoiceTokenResponse`

### VoiceTokenDenied

Canonical serialized shape:

- `allowed: false`
- `reason: 'voice_limit_reached' | 'subscription_required'`
- `usedSeconds: number`
- `limitSeconds: number`
- `agentId: string`
- invariants:
  - current Happy route currently emits `voice_limit_reached`
  - `subscription_required` remains part of the locked wire union because `happy-wire` exports it

### VoiceTokenResponse

Canonical serialized union:

- `VoiceTokenAllowed`
- `VoiceTokenDenied`
- invariants:
  - discriminates by top-level field `allowed`

### ArtifactInfo

Canonical serialized shape:

- `id: string`
- `header: string`
- `headerVersion: number`
- `dataEncryptionKey: string`
- `seq: number`
- `createdAt: number`
- `updatedAt: number`
- invariants:
  - `header` and `dataEncryptionKey` are standard-base64 encrypted payload strings
  - this is the artifact shape used when the body is intentionally omitted

### Artifact

Canonical serialized shape:

- `id: string`
- `header: string`
- `headerVersion: number`
- `body: string`
- `bodyVersion: number`
- `dataEncryptionKey: string`
- `seq: number`
- `createdAt: number`
- `updatedAt: number`
- invariants:
  - `header`, `body`, and `dataEncryptionKey` are standard-base64 encrypted payload strings
  - this is the full artifact read/write transport shape, not the partial update shape

### CreateArtifactBody

Canonical serialized shape:

- `id: string`
- `header: string`
- `body: string`
- `dataEncryptionKey: string`
- invariants:
  - `id` must be a UUID string
  - `header`, `body`, and `dataEncryptionKey` are standard-base64 encrypted payload strings

### UpdateArtifactBody

Canonical serialized shape:

- `header?: string`
- `expectedHeaderVersion?: number`
- `body?: string`
- `expectedBodyVersion?: number`
- invariants:
  - Happy only applies a header mutation when both `header` and `expectedHeaderVersion` are
    supplied
  - Happy only applies a body mutation when both `body` and `expectedBodyVersion` are supplied
  - omitted pairs are treated as no-op fields rather than validation errors in the current handler

### UpdateArtifactResponse

Canonical serialized union:

- `{ success: true, headerVersion?: number, bodyVersion?: number }`
- `{ success: false, error: 'version-mismatch', currentHeaderVersion?: number, currentBodyVersion?: number, currentHeader?: string, currentBody?: string }`
- invariants:
  - this is the HTTP `200` response family for `POST /v1/artifacts/:id`
  - mismatch details are included only for the artifact parts that actually failed the version
    check

### AccessKeyRecord

Canonical serialized shape:

- `data: string`
- `dataVersion: number`
- `createdAt: number`
- `updatedAt: number`
- invariants:
  - access-key fetch/create APIs preserve serialized field name `dataVersion`
  - update-success APIs collapse the incremented counter to generic field name `version`

### GetAccessKeyResponse

Canonical serialized shape:

- `accessKey: AccessKeyRecord | null`
- invariants:
  - `GET /v1/access-keys/:sessionId/:machineId` returns `accessKey: null` when the parents exist
    but no key has been created yet

### CreateAccessKeyBody

Canonical serialized shape:

- `data: string`
- invariants:
  - `data` is the encrypted access-key payload string stored for the `(sessionId, machineId)` pair

### CreateAccessKeyResponse

Canonical serialized shape:

- `success: true`
- `accessKey: AccessKeyRecord`
- invariants:
  - current Happy create-success responses always include the created `accessKey`
  - create conflicts and transport errors use distinct non-`200` error bodies documented in
    `shared/protocol-api-rpc.md`

### UpdateAccessKeyBody

Canonical serialized shape:

- `data: string`
- `expectedVersion: number`
- invariants:
  - `expectedVersion` must be an integer `>= 0`

### UpdateAccessKeyResponse

Canonical serialized union:

- `{ success: true, version: number }`
- `{ success: false, error: 'version-mismatch', currentVersion: number, currentData: string }`
- invariants:
  - this is the HTTP `200` response family for `PUT /v1/access-keys/:sessionId/:machineId`
  - fatal transport errors use a separate `500 { success: false, error }` body documented in
    `shared/protocol-api-rpc.md`

### AgentState

- canonical serialized shape:
  - `controlledByUser?: boolean | null`
  - `requests?: Record<string, PendingToolRequest> | null`
  - `completedRequests?: Record<string, CompletedToolRequest> | null`
- `PendingToolRequest` serialized fields:
  - `tool: string`
  - `arguments: unknown`
  - `createdAt?: number | null`
- `CompletedToolRequest` serialized fields:
  - `tool: string`
  - `arguments: unknown`
  - `createdAt?: number | null`
  - `completedAt?: number | null`
  - `status: 'canceled' | 'denied' | 'approved'`
  - `reason?: string | null`
  - `mode?: string | null`
  - `allowedTools?: string[] | null`
  - `decision?: 'approved' | 'approved_for_session' | 'denied' | 'abort' | null`
- invariants:
  - waiting/idle logic depends only on stable fields documented in shared specs
  - app and agent must not invent divergent agent-state interpretations
  - canonical field name is `allowedTools`
  - decoders must accept legacy alias `allowTools` when reading existing Happy-derived data and
    normalize it to `allowedTools`
  - encoders must emit only `allowedTools`

### SessionMetadata

- required on all writers:
  - `path: string`
  - `host: string`
- baseline fields that locally created sessions must populate:
  - `version?: string`
  - `os?: string`
  - `machineId?: string`
  - `homeDir?: string`
  - `happyHomeDir?: string`
  - `happyLibDir?: string`
  - `happyToolsDir?: string`
  - `startedFromDaemon?: boolean`
  - `hostPid?: number`
  - `startedBy?: 'daemon' | 'terminal'`
  - `lifecycleState?: string`
  - `lifecycleStateSince?: number`
  - `flavor?: string | null`
  - `sandbox?: unknown | null`
  - `dangerouslySkipPermissions?: boolean | null`
- optional UI/runtime extension fields:
  - `name?: string`
  - `summary?: { text: string; updatedAt: number }`
  - `claudeSessionId?: string`
  - `codexThreadId?: string`
  - `tools?: string[]`
  - `slashCommands?: string[]`
  - `models?: Array<{ code: string; value: string; description?: string | null }>`
  - `currentModelCode?: string`
  - `operatingModes?: Array<{ code: string; value: string; description?: string | null }>`
  - `currentOperatingModeCode?: string`
  - `thoughtLevels?: Array<{ code: string; value: string; description?: string | null }>`
  - `currentThoughtLevelCode?: string`
  - `archivedBy?: string`
  - `archiveReason?: string`
- invariants:
  - this is encrypted at rest and in transport update payloads
  - locally created sessions should initialize:
    - `controlledByUser = false` in `AgentState`
    - `lifecycleState = 'running'`
    - `lifecycleStateSince = Date.now()`
  - `happyHomeDir`, `happyLibDir`, and `happyToolsDir` remain compatibility-locked serialized field
    names in phase one

### MachineMetadata

- required canonical synced fields:
  - `host: string`
  - `platform: string`
  - `happyCliVersion: string`
  - `homeDir: string`
  - `happyHomeDir: string`
- optional canonical synced fields:
  - `happyLibDir?: string`
  - `username?: string`
  - `arch?: string`
  - `displayName?: string`
  - `cliAvailability?: { claude: boolean, codex: boolean, gemini: boolean, openclaw: boolean, detectedAt: number }`
  - `resumeSupport?: { rpcAvailable: boolean, requiresSameMachine: boolean, requiresHappyAgentAuth: boolean, happyAgentAuthenticated: boolean, detectedAt: number }`
- not canonical machine-metadata-on-write:
  - `daemonLastKnownStatus`
  - `daemonLastKnownPid`
  - `shutdownRequestedAt`
  - `shutdownSource`
- invariants:
  - daemon/runtime status belongs in `DaemonState`, not `MachineMetadata`
  - app-local caches may denormalize daemon state into convenience fields, but server/client
    encoders must not treat those convenience fields as canonical machine metadata
  - `happyCliVersion`, `happyHomeDir`, and `happyLibDir` remain compatibility-locked serialized
    field names in phase one
  - `resumeSupport.requiresHappyAgentAuth` and `resumeSupport.happyAgentAuthenticated` also remain
    compatibility-locked serialized field names in phase one

### DaemonState

- canonical serialized fields:
  - `status: 'running' | 'shutting-down' | string`
  - `pid?: number`
  - `httpPort?: number`
  - `startedAt?: number`
  - `shutdownRequestedAt?: number`
  - `shutdownSource?: 'mobile-app' | 'cli' | 'os-signal' | 'unknown' | string`
- invariants:
  - this is encrypted independently from `MachineMetadata`
  - unknown string enum values must round-trip
  - app-layer aliases like `happy-app` / `happy-cli` are local projection concerns, not canonical
    synced daemon-state values

### DaemonPersistedState

This is `vibe-cli` local process state, not a synced server record:

- `pid: number`
- `httpPort: number`
- `startTime: number`
- `startedWithCliVersion: string`
- `daemonLogPath: string`
- `lastHeartbeat?: number`
- invariants:
  - stored only under local daemon persistence
  - never sent over server APIs as the canonical machine record

### UploadRef

Canonical serialized fields:

- `ref: string`
- `name: string`
- `size: number`
- `mimeType: string`
- `image?: { width: number; height: number; thumbhash: string }`
- invariants:
  - this is the planning alias for the non-discriminator payload carried by `SessionEvent` when
    `t = 'file'`
  - shared by file-oriented session events and app rendering
  - image metadata is optional and only present for image-capable file refs
  - this is distinct from server-side `StoredImageRef` and public `ImageRef` records used for
    avatars and stored image objects

## Local Projection Rules

The following fields are app-local or client-local projections and must not be promoted into
canonical server storage schemas without a shared-plan change:

- session-local UI/runtime projection fields:
  - `thinking`
  - `thinkingAt`
  - `presence`
  - `todos`
  - `draft`
  - `permissionMode`
  - `modelMode`
  - `latestUsage`
- machine-local convenience projection fields:
  - `daemonLastKnownStatus`
  - `daemonLastKnownPid`
  - app-computed shutdown labels copied out of `DaemonState`
- local Git/UI helper entities such as `GitStatus`

## Compatibility Notes

- phase-one wire compatibility is allowed to preserve Happy-prefixed field names inside encrypted
  metadata and machine records
- phase-one HTTP compatibility is also allowed to preserve Happy route-local quirks such as
  request-only session `tag`, explicit `lastMessage: null` placeholders, and accepted-but-ignored
  `agentState` on `POST /v1/sessions`
- public Vibe branding still applies to binaries, paths, docs, env vars, package names, and
  user-visible surfaces
- renaming canonical encrypted field names requires synchronized server, app, agent, CLI, and
  migration-tool updates; do not do it opportunistically

## Ownership Rules

- `vibe-wire` owns schemas and serde shapes that are intentionally promoted into the shared wire
  crate
- `vibe-server` owns persistence, update sequencing, and server-defined HTTP/update DTOs that are
  not yet part of the shared wire crate
- `vibe-agent` and `vibe-cli` own only client-side projections and wait logic
- `vibe-app` may normalize for UI, but must not redefine canonical storage or transport models
