# Canonical Data Model

## Purpose

This file defines the canonical cross-project data shapes. These shapes must be implemented first
in `crates/vibe-wire` and referenced everywhere else.

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
- `metadata: string | null`
- `metadataVersion: number`
- `daemonState: string | null`
- `daemonStateVersion: number`
- `dataEncryptionKey: string | null`
- invariants:
  - machine metadata and daemon state version independently
  - machine encryption follows the same account-derived pattern as sessions
  - `active` and `activeAt` are server-owned liveness fields, not encrypted metadata fields

### SessionMessage

Canonical serialized shape:

- `id: string`
- `seq: number`
- `localId: string | null`
- `content: { t: 'encrypted', c: string }`
- `createdAt: number`
- `updatedAt: number`
- invariants:
  - `content.t` is currently `encrypted`
  - `content.c` is a standard-base64 encrypted payload
  - decrypted `content.c` resolves to either legacy message content or session-protocol content
  - `localId` is the idempotency key for v3 bulk message insertion

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
  - consumers must preserve unknown-compatible nullability behavior where Happy allows it

### SessionEnvelope

Canonical serialized fields:

- `id: string`
- `time: number`
- `role: string`
- `turn?: string`
- `subagent?: string`
- `ev: SessionEvent`
- invariants:
  - envelope ids are globally unique
  - agent service/start/stop events must use agent role
  - turn-scoped agent events carry `turn` once a turn has started

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
- optional image metadata as defined by the owning session/artifact protocol module
- invariants:
  - shared by file-oriented session events and app rendering

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
- public Vibe branding still applies to binaries, paths, docs, env vars, package names, and
  user-visible surfaces
- renaming canonical encrypted field names requires synchronized server, app, agent, CLI, and
  migration-tool updates; do not do it opportunistically

## Ownership Rules

- `vibe-wire` owns schemas and serde shapes
- `vibe-server` owns persistence and update sequencing
- `vibe-agent` and `vibe-cli` own only client-side projections and wait logic
- `vibe-app` may normalize for UI, but must not redefine canonical storage or transport models
