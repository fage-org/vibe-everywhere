# API, Socket, And RPC Contracts

## Scope

This file defines the server-facing transport categories that Vibe subsystems must support. Exact
route and payload locking happens in project and module plans, but the surface categories are fixed
here.

## HTTP API Groups

- authentication
  - account request/response
  - token refresh or related auth maintenance
- sessions
  - list
  - list active
  - create
  - delete
  - history/messages
- machines
  - list
  - detail if required by Happy parity
- feed
  - get/post flows
- social
  - relationships, friends, usernames
- github
  - connect/disconnect/profile sync
- kv/storage support if required by imported app behavior

## Socket Update Surface

Primary socket path mirrors Happy updates behavior:

- namespace/path: `/v1/updates`
- auth payload includes:
  - `token`
  - `clientType`
  - session or machine scoping when required

Required update body categories:

- `new-message`
- `update-session`
- `update-machine`

Required client behaviors:

- reconnect support
- state cache updates
- decrypted message emission
- idle/wait detection using metadata and agent state

## Machine RPC Surface

The remote-control path requires a machine-scoped RPC contract used by `vibe-agent` and backed by
`vibe-server` plus `vibe-cli` or local runtime components.

At minimum, the RPC surface must cover:

- list controllable machines
- open or attach to sessions
- send user input
- stop or interrupt
- query active state

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
