# Session And Legacy Message Protocol

## Scope

This file freezes the transport-level message contracts shared across server, app, CLI, and agent.
The Rust source of truth must live in `crates/vibe-wire`.

## Canonical Message Families

There are two active message families:

1. legacy protocol
   - role-discriminated user and agent messages
   - still required for compatibility
   - remains the active production compatibility path in phase one
2. session protocol
   - explicit session envelope and event union
   - frozen reference surface for future convergence
   - must not gain draft-only variants without a shared-plan update

Both must be representable inside the encrypted message content of `SessionMessage`.

## Legacy Protocol

### User message

```json
{
  "role": "user",
  "content": { "type": "text", "text": "..." },
  "localKey": "optional",
  "meta": { "...": "..." }
}
```

### Agent message

```json
{
  "role": "agent",
  "content": { "type": "..." },
  "meta": { "...": "..." }
}
```

Rules:

- user messages are constrained to text content in the canonical legacy schema
- agent messages remain pass-through by `content.type`
- legacy support is mandatory until the app plan explicitly allows retiring it
- `localKey` remains the optional client idempotency field on legacy user messages
- `meta` follows the shared `MessageMeta` contract when present

## Session Protocol Envelope

```json
{
  "id": "<string>",
  "time": 1739347200000,
  "role": "user" | "agent",
  "turn": "<string?>",
  "subagent": "<string?>",
  "ev": { "t": "..." }
}
```

Rules:

- `id` is unique across the session stream
- `time` is a millisecond timestamp
- `role` is limited to `user | agent`
- `turn` is optional but expected on agent turn-scoped events
- `subagent` is optional, reserved for nested agent flows, and must validate as a cuid2 string when
  present
- `role` must satisfy event-specific constraints

## Session Event Union

The canonical event surface mirrors current `happy-wire/src/sessionProtocol.ts`:

- `text`
  - fields: `text`, optional `thinking`
- `service`
  - fields: `text`
  - role must be `agent`
- `tool-call-start`
  - fields: `call`, `name`, `title`, `description`, `args`
  - `args` is `Record<string, unknown>`
- `tool-call-end`
  - fields: `call`
- `file`
  - fields: `ref`, `name`, `size`, optional `mimeType`, optional `image`
  - `image` shape is `{ width, height, thumbhash }`
- `turn-start`
  - no extra fields
- `start`
  - optional `title`
  - role must be `agent`
- `turn-end`
  - fields: `status`
  - `status` in `completed | failed | cancelled`
- `stop`
  - no extra fields
  - role must be `agent`

## Session Message Container

Session-protocol payloads travel inside legacy-style encrypted session messages as:

```json
{
  "role": "session",
  "content": { "...session envelope..." },
  "meta": { "...optional message meta..." }
}
```

Rules:

- this wrapper is distinct from envelope `role`
- encrypted message content must validate as one of:
  - legacy user message
  - legacy agent message
  - session-protocol message
- no other top-level message wrapper is canonical in phase one

## Update Container Shapes

The durable update body union mirrored from Happy must include:

- `new-message`
- `update-session`
- `update-machine`

Each durable update travels inside one container with:

- `id`
- `seq`
- `body`
- `createdAt`

Socket transport compatibility rule:

- the server emits these durable containers on the socket event named `update`
- `new-message`, `update-session`, and `update-machine` are `body` variants inside that container,
  not separate socket event names in phase one

## Compatibility Rules

- `vibe-app` must support both legacy and session-protocol message content
- phase-one imported-app compatibility must assume legacy protocol remains available everywhere
- `vibe-agent` and `vibe-cli` may emit either family during staged migration, but the chosen module
  plan must specify which path is authoritative
- `vibe-server` stores encrypted content opaquely and does not reinterpret message bodies beyond
  validation boundaries
- do not add new event variants from draft Happy planning documents unless this file is updated
  first

## Rust Implementation Rules

- define serde structs/enums in `vibe-wire`
- preserve Happy field names on the wire
- expose validation helpers for server/client use
- include cross-language examples covering every session event variant
- keep legacy and session-protocol decoders side-by-side until an explicit plan retires legacy
