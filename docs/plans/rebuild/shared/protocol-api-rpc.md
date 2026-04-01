# API, Socket, And RPC Contracts

## Scope

This file defines the concrete HTTP, socket, and RPC surface that Vibe must preserve for Happy
parity. Project and module plans may split ownership differently, but the transport names, route
paths, and callback shapes documented here are fixed unless a shared compatibility decision is
recorded first.

## Fixed HTTP Route Inventory

### Authentication

| Method | Path | Notes |
| --- | --- | --- |
| `POST` | `/v1/auth` | challenge/signature auth body `{ publicKey, challenge, signature }` |
| `POST` | `/v1/auth/request` | create or poll non-account auth request |
| `GET` | `/v1/auth/request/status` | poll non-account auth request state |
| `POST` | `/v1/auth/response` | submit non-account auth approval |
| `POST` | `/v1/auth/account/request` | create or poll account-link request |
| `POST` | `/v1/auth/account/response` | approve account-link request |

Exact request/response DTOs, error bodies, and auth-state semantics for these routes are locked in
`shared/protocol-auth-crypto.md`.

### Sessions

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v1/sessions` | legacy/full list entrypoint |
| `GET` | `/v2/sessions/active` | active sessions only |
| `GET` | `/v2/sessions` | newer list surface |
| `POST` | `/v1/sessions` | create or load a session by tag |
| `GET` | `/v1/sessions/:sessionId/messages` | legacy last-150 history fetch |
| `DELETE` | `/v1/sessions/:sessionId` | delete or archive session |
| `GET` | `/v3/sessions/:sessionId/messages` | paged fetch with `after_seq` and `limit` query params |
| `POST` | `/v3/sessions/:sessionId/messages` | bulk append/idempotent message insert by `localId` |

## Locked Session HTTP Semantics

- `GET /v1/sessions`
  - response body: `SessionListResponse`
  - current Happy route sorts by `updatedAt desc` and limits the result set to 150 sessions
  - each returned session currently includes explicit `lastMessage: null`
  - returned session items do not include `tag`; clients that create or load by tag must treat tag
    as request-side or storage-side state, not as a current response field
- `GET /v2/sessions/active`
  - query params:
    - `limit`: integer, minimum `1`, maximum `500`, default `150`
  - response body: `ActiveSessionListResponse`
  - current Happy route filters to `active = true` sessions whose `activeAt` is within the last 15
    minutes
  - returned session items currently omit `lastMessage`
- `GET /v2/sessions`
  - query params:
    - `cursor?`: string
    - `limit`: integer, minimum `1`, maximum `200`, default `50`
    - `changedSince?`: positive integer millisecond timestamp
  - response body: `CursorPagedSessionListResponse`
  - current Happy pagination sorts by `id desc`, not by `updatedAt`
  - `cursor` must use the fixed format `cursor_v1_${sessionId}`
  - `changedSince` filters by `updatedAt > changedSince` but does not change the pagination sort key
  - returned session items currently omit `lastMessage`
- `POST /v1/sessions`
  - request body: `CreateOrLoadSessionBody`
  - response body: `CreateOrLoadSessionResponse`
  - semantics: load an existing session when `(accountId, tag)` already exists; otherwise create a
    new session record
  - current Happy phase-one handler accepts `agentState` for compatibility with existing clients but
    does not persist that field during create-or-load
  - returned session payload currently includes `lastMessage: null` and does not echo `tag`
- `GET /v1/sessions/:sessionId/messages`
  - response body: `SessionHistoryResponse`
  - current Happy route returns up to 150 messages ordered by `createdAt desc`

### Machines

| Method | Path | Notes |
| --- | --- | --- |
| `POST` | `/v1/machines` | create or load machine record by id |
| `GET` | `/v1/machines` | list machines |
| `GET` | `/v1/machines/:id` | get machine details |

## Locked Machine HTTP Semantics

- `POST /v1/machines`
  - request body: `CreateOrLoadMachineBody`
  - response body: `CreateOrLoadMachineResponse`
  - semantics: load an existing machine when `(accountId, id)` already exists; otherwise create a
    new machine record
  - current Happy route requires encrypted `metadata`, accepts optional encrypted `daemonState`, and
    treats omitted or null `dataEncryptionKey` as the legacy encryption path
  - create path initializes `active = false` so the machine starts offline until later heartbeat
    updates arrive
  - create path initializes `metadataVersion = 1` and initializes `daemonStateVersion = 1` only
    when `daemonState` is provided; otherwise `daemonStateVersion = 0`
  - returned machine payload currently omits `seq` on both create and load paths
- `GET /v1/machines`
  - response body: `MachineHttpRecord[]`
  - current Happy route sorts by `lastActiveAt desc`
  - the route returns a bare array rather than a wrapper object
  - returned machine items currently include `seq`
- `GET /v1/machines/:id`
  - response body: `MachineDetailResponse`
  - returned machine payload currently includes `seq`
  - current Happy route returns `404 { error: 'Machine not found' }` when the machine is absent or
    belongs to another account

### Account And Usage

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v1/account/profile` | account profile |
| `GET` | `/v1/account/settings` | account settings |
| `POST` | `/v1/account/settings` | update account settings |
| `POST` | `/v1/usage/query` | usage aggregation/query surface |

## Locked Account And Usage HTTP Semantics

- `GET /v1/account/profile`
  - response body: `AccountProfileResponse`
  - current Happy emits a fresh `timestamp = Date.now()` on every response rather than a stable
    profile `updatedAt`
  - `connectedServices` is derived from currently connected vendor tokens at read time
- `GET /v1/account/settings`
  - response body: `AccountSettingsResponse`
  - current Happy returns `500 { error: 'Failed to get account settings' }` when the account row
    cannot be loaded or the handler throws
- `POST /v1/account/settings`
  - request body: `UpdateAccountSettingsBody`
  - success/optimistic-concurrency response body: `UpdateAccountSettingsResponse`
  - `expectedVersion` must be an integer `>= 0`
  - version mismatch remains HTTP `200`, not `409`
  - `500` error body: `{ success: false, error: 'Failed to update account settings' }`
  - successful writes emit `update-account` only to user-scoped connections and carry
    `settings: { value, version }`
- `POST /v1/usage/query`
  - request body: `UsageQueryBody`
  - response body: `UsageQueryResponse`
  - omitted or null `groupBy` defaults to `'day'`
  - `startTime` and `endTime` are epoch-second bounds on the wire and are multiplied by `1000`
    before DB comparison
  - when `sessionId` is provided, Happy first verifies account ownership and returns
    `404 { error: 'Session not found' }` on miss
  - current Happy reads matching reports ordered by `createdAt desc`, then returns aggregated
    buckets sorted by `timestamp asc`
  - `500` error body: `{ error: 'Failed to query usage reports' }`

### Connect / OAuth / Vendor Integration

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v1/connect/github/params` | GitHub connect bootstrap |
| `GET` | `/v1/connect/github/callback` | GitHub OAuth callback |
| `POST` | `/v1/connect/github/webhook` | GitHub webhook |
| `DELETE` | `/v1/connect/github` | disconnect GitHub |
| `POST` | `/v1/connect/:vendor/register` | vendor registration |
| `GET` | `/v1/connect/:vendor/token` | vendor token fetch |
| `DELETE` | `/v1/connect/:vendor` | vendor disconnect |
| `GET` | `/v1/connect/tokens` | list integration tokens |

### Social / Feed / User

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v1/feed` | feed list |
| `GET` | `/v1/user/:id` | user profile by id |
| `GET` | `/v1/user/search` | user search |
| `POST` | `/v1/friends/add` | add friend |
| `POST` | `/v1/friends/remove` | remove friend |
| `GET` | `/v1/friends` | list friends |

### KV / Push / Version / Voice

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v1/kv/:key` | single key lookup |
| `GET` | `/v1/kv` | key scan/list surface |
| `POST` | `/v1/kv/bulk` | bulk key lookup |
| `POST` | `/v1/kv` | write/update key |
| `POST` | `/v1/push-tokens` | register push token |
| `GET` | `/v1/push-tokens` | list push tokens |
| `DELETE` | `/v1/push-tokens/:token` | remove push token |
| `POST` | `/v1/version` | app/server version check |
| `POST` | `/v1/voice/token` | voice access token |

## Locked Utility HTTP Semantics

- `GET /v1/kv/:key`
  - response body: `KvEntry`
  - current Happy route schema marks the `200` response nullable, but the handler returns
    `404 { error: 'Key not found' }` instead of `null` when the key is absent
  - `500` error body: `{ error: 'Failed to get value' }`
- `GET /v1/kv`
  - query params:
    - `prefix?`: string
    - `limit`: integer, minimum `1`, maximum `1000`, default `100`
  - response body: `KvListResponse`
  - `500` error body: `{ error: 'Failed to list items' }`
- `POST /v1/kv/bulk`
  - request body: `KvBulkGetBody`
  - response body: `KvBulkGetResponse`
  - request limits: `1..100` keys
  - `500` error body: `{ error: 'Failed to get values' }`
- `POST /v1/kv`
  - request body: `KvMutateBody`
  - success/conflict response body: `KvMutateResponse`
  - each mutation must include `version`; callers use `-1` when creating a new key
  - version mismatches return HTTP `409`, not HTTP `200`
  - `500` error body: `{ error: 'Failed to mutate values' }`
- `POST /v1/push-tokens`
  - request body: `{ token: string }`
  - success body: `{ success: true }`
  - current Happy performs an idempotent upsert on `(accountId, token)` and refreshes `updatedAt`
    on duplicate registration
  - `500` error body: `{ error: 'Failed to register push token' }`
- `GET /v1/push-tokens`
  - response body: `PushTokenListResponse`
  - current Happy sorts by `createdAt desc`
  - `500` error body: `{ error: 'Failed to get push tokens' }`
- `DELETE /v1/push-tokens/:token`
  - success body: `{ success: true }`
  - current Happy deletes via `deleteMany`, so deleting a missing token still succeeds
  - `500` error body: `{ error: 'Failed to delete push token' }`
- `POST /v1/version`
  - auth: none
  - request body: `VersionCheckBody`
  - response body: `VersionCheckResponse`
  - `app_id` is accepted for compatibility but unused by the current handler
  - outdated iOS and Android responses currently return Happy-branded store URLs and must stay
    compatibility-locked until a shared deviation is approved
  - non-iOS and non-Android platforms return `{ updateUrl: null }`
- `POST /v1/voice/token`
  - request body: `{ agentId: string }`
  - response body: `VoiceTokenResponse`
  - `500` error body: `{ error: string }`
  - current deny-path emits `{ allowed: false, reason: 'voice_limit_reached', usedSeconds, limitSeconds, agentId }`
  - the locked wire union still includes `reason = 'subscription_required'` because
    `happy-wire` exports that branch

### Artifacts And Access Keys

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v1/artifacts` | list artifacts |
| `POST` | `/v1/artifacts` | create artifact |
| `GET` | `/v1/artifacts/:id` | get artifact |
| `POST` | `/v1/artifacts/:id` | update artifact |
| `DELETE` | `/v1/artifacts/:id` | delete artifact |
| `GET` | `/v1/access-keys/:sessionId/:machineId` | fetch access key |
| `POST` | `/v1/access-keys/:sessionId/:machineId` | create access key |
| `PUT` | `/v1/access-keys/:sessionId/:machineId` | rotate/update access key |

## Locked Artifact And Access-Key HTTP Semantics

- `GET /v1/artifacts`
  - response body: `ArtifactInfo[]`
  - current Happy sorts by `updatedAt desc`
  - `500` error body: `{ error: 'Failed to get artifacts' }`
- `GET /v1/artifacts/:id`
  - response body: `Artifact`
  - `404` error body: `{ error: 'Artifact not found' }`
  - `500` error body: `{ error: 'Failed to get artifact' }`
- `POST /v1/artifacts`
  - request body: `CreateArtifactBody`
  - response body: `Artifact`
  - `id` must be a UUID string
  - if the artifact id already exists for the same account, Happy returns the existing record with
    HTTP `200` and does not create a duplicate
  - if the artifact id already exists for another account, Happy returns
    `409 { error: 'Artifact with this ID already exists for another account' }`
  - newly created artifacts initialize `headerVersion = 1`, `bodyVersion = 1`, and `seq = 0`
  - successful creates emit `new-artifact` only to user-scoped connections
  - `500` error body: `{ error: 'Failed to create artifact' }`
- `POST /v1/artifacts/:id`
  - request body: `UpdateArtifactBody`
  - success/optimistic-concurrency response body: `UpdateArtifactResponse`
  - Happy only applies a header change when both `header` and `expectedHeaderVersion` are present;
    the same pairing rule applies to `body` and `expectedBodyVersion`
  - version mismatches remain HTTP `200`, not `409`
  - successful writes increment artifact `seq` and emit `update-artifact` only to user-scoped
    connections
  - `404` error body: `{ error: 'Artifact not found' }`
  - `500` error body: `{ error: 'Failed to update artifact' }`
- `DELETE /v1/artifacts/:id`
  - success body: `{ success: true }`
  - successful deletes emit `delete-artifact` only to user-scoped connections
  - `404` error body: `{ error: 'Artifact not found' }`
  - `500` error body: `{ error: 'Failed to delete artifact' }`
- `GET /v1/access-keys/:sessionId/:machineId`
  - response body: `GetAccessKeyResponse`
  - current Happy verifies both the session and machine belong to the authenticated account before
    looking up the key
  - when the parents exist but no key has been created, the route returns
    `{ accessKey: null }` with HTTP `200`
  - `404` error body: `{ error: 'Session or machine not found' }`
  - `500` error body: `{ error: 'Failed to get access key' }`
- `POST /v1/access-keys/:sessionId/:machineId`
  - request body: `CreateAccessKeyBody`
  - response body: `CreateAccessKeyResponse`
  - current Happy verifies both parents before create and initializes `dataVersion = 1`
  - `404` error body: `{ error: 'Session or machine not found' }`
  - `409` error body: `{ error: 'Access key already exists' }`
  - `500` error body: `{ error: 'Failed to create access key' }`
- `PUT /v1/access-keys/:sessionId/:machineId`
  - request body: `UpdateAccessKeyBody`
  - success/optimistic-concurrency response body: `UpdateAccessKeyResponse`
  - current Happy looks up the existing access key directly and returns
    `404 { error: 'Access key not found' }` when absent
  - version mismatches remain HTTP `200`, not `409`
  - `500` error body: `{ success: false, error: 'Failed to update access key' }`

## Locked v3 Session Message Semantics

- `GET /v3/sessions/:sessionId/messages`
  - query params:
    - `after_seq`: integer, minimum `0`, default `0`
    - `limit`: integer, minimum `1`, maximum `500`, default `100`
  - response body: `{ messages, hasMore }`
- `POST /v3/sessions/:sessionId/messages`
  - request body: `{ messages: Array<{ content: string, localId: string }> }`
  - request limits: `1..100` messages
  - idempotency rule: deduplicate by the first occurrence of each `localId`
  - response body: `{ messages: Array<{ id, seq, localId, createdAt, updatedAt }> }`

## Socket Transport

Primary socket path mirrors Happy updates behavior:

- namespace/path: `/v1/updates`
- transports: `websocket`, `polling`
- ping timeout: `45_000 ms`
- ping interval: `15_000 ms`
- upgrade timeout: `10_000 ms`
- connect timeout: `20_000 ms`
- auth payload includes:
  - `token`
  - `clientType`: `session-scoped | user-scoped | machine-scoped | undefined`
  - `sessionId?`
  - `machineId?`
- validation rules:
  - missing `token` is a hard disconnect error
  - `clientType = session-scoped` requires `sessionId`
  - `clientType = machine-scoped` requires `machineId`

Required client behaviors:

- reconnect support
- state cache updates from `update` events
- decrypted message emission
- idle/wait detection using metadata, agent state, and ephemeral activity

## Socket Event Inventory

### Server -> client

| Event | Payload shape | Notes |
| --- | --- | --- |
| `update` | `CoreUpdateContainer` outer envelope `{ id: string, seq: number, body, createdAt: number }` | durable per-user sync container; `seq` is monotonic per user and `body` is either `CoreUpdateBody` or one of the late support-domain durable families locked below |
| `ephemeral` | `EphemeralUpdate` union | transient presence/usage/status updates; not replayed after reconnect |
| `error` | `{ message: string }` | transport/auth error surface before disconnect |
| `rpc-request` | `{ method: string, params: string }` | forwarded same-user RPC request; target socket must resolve the ack callback with the opaque result payload |
| `rpc-registered` | `{ method: string }` | RPC registration success |
| `rpc-unregistered` | `{ method: string }` | RPC unregister success |
| `rpc-error` | `{ type: string, error: string }` | RPC registration/runtime error |

## Locked Durable Update Families

The socket `update` event always uses the same outer envelope as `CoreUpdateContainer`:
`{ id, seq, body, createdAt }`.
Only the `body` discriminator family varies.

### Core `vibe-wire` durable family

- `new-message`
  - `{ t: 'new-message', sid, message: { id, seq, content, localId?, createdAt, updatedAt } }`
- `update-session`
  - `{ t: 'update-session', id, metadata?: { value, version } | null, agentState?: { value: string | null, version } | null }`
- `update-machine`
  - `{ t: 'update-machine', machineId, metadata?: { value, version }, daemonState?: { value, version }, active?: boolean, activeAt?: number }`
  - `happy-wire` preserves optional `active` and `activeAt`, even though current Happy emitters
    mostly use `machine-activity` ephemeral events for liveness

### Server-owned late support-domain durable family

- `new-session`
  - `{ t: 'new-session', id, seq, metadata, metadataVersion, agentState, agentStateVersion, dataEncryptionKey, active, activeAt, createdAt, updatedAt }`
- `delete-session`
  - `{ t: 'delete-session', sid }`
- `update-account`
  - `{ t: 'update-account', id, settings?, github?, username?, firstName?, lastName?, avatar? }`
  - current Happy emitters do not send `connectedServices` or `timestamp` via this update family
- `new-machine`
  - `{ t: 'new-machine', machineId, seq, metadata, metadataVersion, daemonState, daemonStateVersion, dataEncryptionKey, active, activeAt, createdAt, updatedAt }`
- `new-artifact`
  - `{ t: 'new-artifact', artifactId, seq, header, headerVersion, body, bodyVersion, dataEncryptionKey, createdAt, updatedAt }`
- `update-artifact`
  - `{ t: 'update-artifact', artifactId, header?: { value, version }, body?: { value, version } }`
- `delete-artifact`
  - `{ t: 'delete-artifact', artifactId }`
- `relationship-updated`
  - `{ t: 'relationship-updated', uid, status, timestamp }`
  - imported `packages/vibe-app` code currently normalizes a richer relationship projection with
    `fromUserId`, `toUserId`, `action`, and optional user records; that remains an app-adapter
    concern and does not change the canonical server-emitted transport shape
- `new-feed-post`
  - `{ t: 'new-feed-post', id, body, cursor, createdAt }`
  - imported `packages/vibe-app` code currently carries additional app-local feed fields such as
    `repeatKey`; any defaulting or projection widening remains an app-adapter concern and does not
    change the canonical server-emitted transport shape
- `kv-batch-update`
  - `{ t: 'kv-batch-update', changes: Array<{ key, value, version }> }`

Until the owning module plans promote these late support-domain bodies into `vibe-wire`, this
document remains the transport source of truth for them. Imported app schemas may keep wider
app-local projections for some of these bodies, but any normalization belongs in app adapter seams
rather than in the canonical server transport.

## Locked Ephemeral Families

- `activity`
  - `{ type: 'activity', id: sessionId, active, activeAt, thinking?: boolean }`
- `machine-activity`
  - `{ type: 'machine-activity', id: machineId, active, activeAt }`
- `usage`
  - `{ type: 'usage', id: sessionId, key, tokens, cost, timestamp }`
- `machine-status`
  - `{ type: 'machine-status', machineId, online, timestamp }`

Notes:

- clients must not treat `ephemeral` events as durable replayable state
- `machine-status` stays transport-locked because it exists in Happy's protocol/event-router
  definitions even though current Happy flows primarily emit `machine-activity` and the imported
  app currently ignores `machine-status`

### Client -> server: session-scoped and general

| Event | Payload shape | Ack/result |
| --- | --- | --- |
| `message` | `{ sid: string, message: string, localId?: string }` | no typed ack in Happy flow |
| `update-metadata` | `{ sid, expectedVersion, metadata }` | `{ result: 'error' }`, `{ result: 'version-mismatch', version, metadata }`, or `{ result: 'success', version, metadata }` |
| `update-state` | `{ sid, expectedVersion, agentState }` | `{ result: 'error' }`, `{ result: 'version-mismatch', version, agentState }`, or `{ result: 'success', version, agentState }` |
| `session-alive` | `{ sid, time, thinking? }` | no typed ack required |
| `session-end` | `{ sid, time }` | no typed ack required |
| `ping` | no body | ack callback returns `{}` |
| `usage-report` | `{ key: string, sessionId?: string, tokens: Record<string, number> & { total: number }, cost: Record<string, number> & { total: number } }` | optional callback: `{ success: true, reportId, createdAt, updatedAt }` or `{ success: false, error }` |

### Client -> server: machine-scoped

| Event | Payload shape | Ack/result |
| --- | --- | --- |
| `machine-alive` | `{ machineId, time }` | no typed ack required |
| `machine-update-metadata` | `{ machineId, metadata, expectedVersion }` | `{ result: 'error' }`, `{ result: 'version-mismatch', version, metadata }`, or `{ result: 'success', version, metadata }` |
| `machine-update-state` | `{ machineId, daemonState, expectedVersion }` | `{ result: 'error' }`, `{ result: 'version-mismatch', version, daemonState }`, or `{ result: 'success', version, daemonState }` |

### Client -> server: auxiliary socket APIs

| Event | Payload shape | Ack/result |
| --- | --- | --- |
| `access-key-get` | `{ sessionId, machineId }` | `{ ok: true, accessKey: { data, dataVersion, createdAt, updatedAt } \| null }` or `{ ok: false, error }` |
| `artifact-read` | `{ artifactId }` | `{ result: 'success', artifact: { id, header, headerVersion, body, bodyVersion, seq, createdAt, updatedAt } }` or `{ result: 'error', message }` |
| `artifact-update` | `{ artifactId, header?: { data, expectedVersion }, body?: { data, expectedVersion } }` | `{ result: 'success', header?: { version, data }, body?: { version, data } }`, `{ result: 'error', message }`, or `{ result: 'version-mismatch', header?: { currentVersion, currentData }, body?: { currentVersion, currentData } }` |
| `artifact-create` | `{ id, header, body, dataEncryptionKey }` | `{ result: 'success', artifact: { id, header, headerVersion, body, bodyVersion, seq, createdAt, updatedAt } }` or `{ result: 'error', message }` |
| `artifact-delete` | `{ artifactId }` | `{ result: 'success' }` or `{ result: 'error', message }` |

## Machine RPC Surface

RPC uses same-user socket forwarding. The server-side contract is fixed as follows:

- client -> server:
  - `rpc-register`
  - `rpc-unregister`
  - `rpc-call`
- server -> client:
  - `rpc-request`
  - `rpc-registered`
  - `rpc-unregistered`
  - `rpc-error`
- `rpc-call` callback result shape is always:
  - success: `{ ok: true, result }`
  - failure: `{ ok: false, error }`
- `rpc-request` forwarded payload shape is `{ method, params }`
- RPC timeout is 30 seconds
- server rejects self-calls on the same socket
- method registration scope is per-user, not global across users

### Phase-one machine RPC method names

The imported Happy clients currently depend on these method identifiers:

- `spawn-happy-session`
- `resume-happy-session`
- `stop-session`
- `stop-daemon`

Phase one must preserve these strings for compatibility. If Vibe-specific RPC names are desired,
add aliases only after app and CLI adapter plans explicitly cover the migration.

## Error Model

Transport and API plans must preserve distinct handling for:

- `401` authentication expired
- `403` forbidden
- `404` not found
- `4xx` validation/request issues
- `5xx` server faults
- socket connect and reconnect errors

CLI and agent plans must document the exact user-facing error text policy.

## Versioning Strategy

- prefer additive compatibility in transport contracts
- do not fork protocol versions per client
- if a field or endpoint must diverge from Happy, record the deviation in shared docs and add an
  explicit compatibility adapter plan
