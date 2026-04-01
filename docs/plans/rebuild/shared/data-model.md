# Canonical Data Model

## Purpose

This file defines the canonical cross-project data shapes. These shapes must be implemented first
in `crates/vibe-wire` and referenced everywhere else.

## Core Entities

### SessionRecord

- fields:
  - `id: String`
  - `seq: u64`
  - `created_at: i64`
  - `updated_at: i64`
  - `active: bool`
  - `active_at: i64`
  - `metadata: VersionedEncryptedValue`
  - `agent_state: Option<VersionedNullableEncryptedValue>`
  - `data_encryption_key: Option<String>`
- invariants:
  - `seq` is monotonically increasing per session
  - `metadata` is always present
  - `agent_state` may be null
  - `data_encryption_key == None` implies legacy encryption path

### MachineRecord

- fields:
  - `id`
  - `seq`
  - `created_at`
  - `updated_at`
  - `active`
  - `active_at`
  - `metadata: Option<VersionedMachineEncryptedValue>`
  - `daemon_state: Option<VersionedMachineEncryptedValue>`
  - `data_encryption_key: Option<String>`
- invariants:
  - machine metadata and daemon state version independently
  - machine encryption follows the same account-derived pattern as sessions

### SessionMessage

- fields:
  - `id`
  - `seq`
  - `local_id: Option<String>`
  - `content: EncryptedMessageContent`
  - `created_at`
  - `updated_at`
- invariants:
  - `content.t` is currently `encrypted`
  - decrypted `content.c` resolves to either legacy message content or session-protocol content

### MessageMeta

- fields:
  - `sent_from`
  - `permission_mode`
  - `model`
  - `fallback_model`
  - `custom_system_prompt`
  - `append_system_prompt`
  - `allowed_tools`
  - `disallowed_tools`
  - `display_text`
- invariants:
  - field names serialize to Happy-compatible shapes
  - consumers must preserve unknown-compatible nullability behavior where Happy allows it

### SessionEnvelope

- fields:
  - `id`
  - `time`
  - `role`
  - `turn: Option<String>`
  - `subagent: Option<String>`
  - `ev: SessionEvent`
- invariants:
  - envelope ids are globally unique
  - agent service/start/stop events must use agent role
  - turn-scoped agent events carry `turn` once a turn has started

### AgentState

- minimum canonical fields:
  - `controlled_by_user: bool`
  - `requests: map`
  - provider-specific runtime state
  - lifecycle status
- invariants:
  - waiting/idle logic depends only on stable fields documented in shared specs
  - app and agent must not invent divergent agent-state interpretations

### SessionMetadata

- minimum canonical fields:
  - project path
  - host or machine identity
  - lifecycle state
  - display name / tag / summary
  - provider/runtime hints
- invariants:
  - this is encrypted at rest and in transport update payloads

### UploadRef

- fields:
  - `ref`
  - `name`
  - `size`
  - `mime_type`
  - optional image metadata
- invariants:
  - shared by file-oriented session events and app rendering

## Encryption Wrapper Types

- `VersionedEncryptedValue { version, value }`
- `VersionedNullableEncryptedValue { version, value: Option<String> }`
- `VersionedMachineEncryptedValue { version, value }`

Serialization must remain wire-compatible with Happy:

- numeric `version`
- base64 string payloads
- nullable semantics where present

## Ownership Rules

- `vibe-wire` owns schemas and serde shapes
- `vibe-server` owns persistence and update sequencing
- `vibe-agent` and `vibe-cli` own only client-side projections and wait logic
- `vibe-app` may normalize for UI, but must not redefine canonical storage or transport models
