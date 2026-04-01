# Session And Legacy Message Protocol

## Scope

This file freezes the transport-level message contracts shared across server, app, CLI, and agent.
The Rust source of truth must live in `crates/vibe-wire`.

## Canonical Message Families

There are two active message families:

1. legacy protocol
   - role-discriminated user and agent messages
   - still required for compatibility
2. session protocol
   - explicit session envelope and event union
   - target long-term protocol surface

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
- `turn` is optional but expected on agent turn-scoped events
- `subagent` is optional and reserved for nested agent flows
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
- `tool-call-end`
  - fields: `call`
- `file`
  - fields: `ref`, `name`, `size`, optional `mimeType`, optional `image`
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

## Update Container Shapes

The update container surface mirrored from Happy must include:

- `new-message`
- `update-session`
- `update-machine`

Each update travels in a container with:

- `id`
- `seq`
- `body`
- `createdAt`

## Compatibility Rules

- `vibe-app` must support both legacy and session-protocol message content
- `vibe-agent` and `vibe-cli` may emit either family during staged migration, but the chosen module
  plan must specify which path is authoritative
- `vibe-server` stores encrypted content opaquely and does not reinterpret message bodies beyond
  validation boundaries

## Rust Implementation Rules

- define serde structs/enums in `vibe-wire`
- preserve Happy field names on the wire
- expose validation helpers for server/client use
- include cross-language examples covering every session event variant
