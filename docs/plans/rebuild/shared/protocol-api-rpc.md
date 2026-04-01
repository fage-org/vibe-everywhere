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
| `POST` | `/v1/auth/request` | create non-account auth request |
| `GET` | `/v1/auth/request/status` | poll non-account auth request state |
| `POST` | `/v1/auth/response` | submit non-account auth approval |
| `POST` | `/v1/auth/account/request` | create or poll account-link request |
| `POST` | `/v1/auth/account/response` | approve account-link request |

### Sessions

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v1/sessions` | legacy/full list entrypoint |
| `GET` | `/v2/sessions/active` | active sessions only |
| `GET` | `/v2/sessions` | newer list surface |
| `POST` | `/v1/sessions` | create session |
| `GET` | `/v1/sessions/:sessionId/messages` | legacy/single-message history fetch |
| `DELETE` | `/v1/sessions/:sessionId` | delete or archive session |
| `GET` | `/v3/sessions/:sessionId/messages` | paged fetch with `after_seq` and `limit` query params |
| `POST` | `/v3/sessions/:sessionId/messages` | bulk append/idempotent message insert by `localId` |

### Machines

| Method | Path | Notes |
| --- | --- | --- |
| `POST` | `/v1/machines` | create/register machine record |
| `GET` | `/v1/machines` | list machines |
| `GET` | `/v1/machines/:id` | get machine details |

### Account And Usage

| Method | Path | Notes |
| --- | --- | --- |
| `GET` | `/v1/account/profile` | account profile |
| `GET` | `/v1/account/settings` | account settings |
| `POST` | `/v1/account/settings` | update account settings |
| `POST` | `/v1/usage/query` | usage aggregation/query surface |

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
| `update` | `Update` union/container from `vibe-wire` | wraps session, machine, artifact, and related durable updates |
| `ephemeral` | `{ type: 'activity', id, active, activeAt, thinking }` | transient activity/presence updates; machine events set `thinking = false` |
| `auth` | `{ success: boolean, user: string }` | connection auth result |
| `error` | `{ message: string }` | transport/auth error surface |
| `rpc-request` | `{ method: string, params: string }` | forwarded RPC request, expects ack callback |
| `rpc-registered` | `{ method: string }` | RPC registration success |
| `rpc-unregistered` | `{ method: string }` | RPC unregister success |
| `rpc-error` | `{ type: string, error: string }` | RPC registration/runtime error |

### Client -> server: session-scoped and general

| Event | Payload shape | Ack/result |
| --- | --- | --- |
| `message` | `{ sid: string, message }` | no typed ack in Happy flow |
| `update-metadata` | `{ sid, expectedVersion, metadata }` | `{ result: 'error' }`, `{ result: 'version-mismatch', version, metadata }`, or `{ result: 'success', version, metadata }` |
| `update-state` | `{ sid, expectedVersion, agentState }` | `{ result: 'error' }`, `{ result: 'version-mismatch', version, agentState }`, or `{ result: 'success', version, agentState }` |
| `session-alive` | `{ sid, time, thinking, mode? }` | no typed ack required |
| `session-end` | `{ sid, time }` | no typed ack required |
| `ping` | no body | ack callback only |
| `usage-report` | `{ key, sessionId, tokens, cost }` | optional callback in Happy server |

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
| `artifact-update` | `{ artifactId, header?, body? }` | `{ result: 'success', ... }`, `{ result: 'error', message }`, or `{ result: 'version-mismatch', header?: { currentVersion, currentData }, body?: { currentVersion, currentData } }` |
| `artifact-create` | Happy-compatible create body | create result object |
| `artifact-delete` | Happy-compatible delete body | delete result object |

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
