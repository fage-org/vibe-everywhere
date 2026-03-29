# Vibe Everywhere Iteration Specs v5

Last updated: 2026-03-30

Version note:

- This file is the versioned detailed iteration plan for roadmap epoch `v5`.
- The concise lookup view lives in [`v5-summary.md`](./v5-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This epoch moves the control app from “session-first monitoring console” to
“conversation-first remote coding workspace.” The goal is to make the primary surface feel like a
real AI coding chat product: users pick a device, open a durable conversation, send follow-up
turns into the same provider-native thread, and only open Git/files/raw diagnostics when they
need inspection.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, completion log, verification
  log, and decision log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, workflow, release,
  testing, and documentation guardrails

## Roadmap Overview

| Iteration | Title | Status | Depends On |
| --- | --- | --- | --- |
| 15 | Conversation-First Threaded AI Sessions | in_progress | Iteration roadmap `v4` |

## Shared Guardrails

- The default end-user surface must prioritize the chat transcript over infrastructure cards.
- Relay connection belongs in a compact connect gate or context bar, not a persistent tutorial
  hero after the user is connected.
- Long-lived AI work must be represented as explicit conversations/threads, not only as temporary
  single-run tasks.
- Provider questions that require a user choice must surface inline in the conversation with
  actionable options; if the protocol allows free-form clarification, the UI must also offer a
  custom text path instead of forcing only fixed buttons.
- Raw task events remain valuable for debugging, but they belong in a secondary inspector rather
  than the primary reading path for assistant replies.

## Iteration 15: Conversation-First Threaded AI Sessions

### Goal

Turn the current session dashboard into a durable conversation product with provider-native
continuation and inline user-choice interactions.

### User-Visible Outcome

- `Sessions` becomes a threaded conversation workspace instead of a many-card dashboard.
- Conversations persist and can be reopened later like chat threads.
- Follow-up prompts continue the same native provider conversation for supported providers.
- Git review, workspace browsing, raw trace, and preview move behind compact or collapsible
  inspectors.
- The default app entry now behaves like a Poe-style home: pick a device, then pick one of that
  device's historical projects grouped by working directory.
- Entering a project opens a Telegram-like chat page with a left-top topic-history drawer scoped
  to the current project rather than a global dashboard sidebar.
- Relay/tutorial setup no longer occupies the main screen after a successful connection; server
  configuration lives under `Menu > Settings > Server`.
- Provider-driven choice prompts can be answered inline with selectable options or custom text.

### In Scope

- add a durable conversation model above the existing task/run model
- persist conversation records, linked task runs, provider session handles, and pending interactive
  user-choice requests in relay storage
- add conversation-focused relay APIs for list/detail/create/message-send/archive/respond
- add provider-native continuation for:
  - Codex via `codex exec resume`
  - Claude Code via `claude --resume` / `--continue`
  - OpenCode via its persisted session identifier path
- replace the current default `Sessions` shell with:
  - a device/project home route that groups historical conversations by `device + cwd`
  - a project chat route with a Telegram-like transcript/composer layout
  - a project-scoped topic-history drawer opened from the top-left chrome
- move relay URL and access-token editing into `Menu > Settings > Server`
- keep device runtime inspection under a dedicated `Devices` route instead of mixing it into the
  primary chat surface
- surface inline option-choice prompts with a custom-input fallback when the provider asks the
  user to choose or clarify
- update docs, testing guidance, plan files, and user-facing product descriptions for the new
  model

### Out Of Scope

- no multi-user shared conversation collaboration model
- no full semantic diff review / merge approval workflow
- no enterprise auth redesign beyond current actor boundaries
- no rewrite of shell or port-forward control planes beyond demotion into secondary surfaces

### Acceptance Criteria

- relay exposes conversation APIs and persists conversations across restarts
- each conversation keeps a stable provider-native resume handle where the provider supports one
- follow-up prompts in an existing conversation continue the native provider thread instead of
  replaying prompt history
- the default UI opens on a device/project home rather than a dashboard card grid
- entering a project opens a chat-first page with the transcript and composer primary
- the current project's topic history is reachable from the top-left drawer control
- relay configuration is reachable from `Menu > Settings > Server` instead of the main chat page
- the old tutorial/hero copy no longer remains visible after a successful relay connection
- when the provider asks for a user choice, the app shows option chips inline and supports a
  free-form custom answer path when allowed
- current Git/workspace/raw event data remains reachable from the conversation context

### ACP Completion Track

- Completed in the current tranche:
  truthful ACP capability signaling so only OpenCode is advertised as standard ACP today; stable
  OpenCode `session/update` mapping for `agent_thought_chunk`, `available_commands_update`,
  `session_info_update`, and optional `usage_update`; prompt-usage surfacing when the agent returns
  per-turn usage; and `session/list`-based validation of stored OpenCode session handles before a
  follow-up turn is sent.
- Completed in the current tranche:
  `user-specified` Mode 3 transcript-safe continuation: when an ACP agent advertises
  `sessionCapabilities.resume`, Vibe now uses standard `session/resume` instead of replay-oriented
  `session/load`, and only falls back to the prior `session/list` validation path when resume is
  not advertised.
- Next ACP work:
  implement authenticated ACP startup when agents advertise `authMethods` and the product has a
  real credential flow to supply.
- Next ACP work:
  design transcript-safe support for richer session lifecycle methods beyond `session/resume`.
  `session/load` is still not used for routine conversation continuation because the protocol
  replays prior messages, which would duplicate transcript content in the current
  conversation-first product model. Revisit it only with explicit dedupe/import handling.
- Next ACP work:
  add protocol-extension support (`extMethod` / `extNotification`) if a supported provider needs
  ACP extensions beyond the current core method set.
- Longer-term ACP work:
  do not advertise Codex or Claude Code as ACP until they are backed by a real standard ACP
  transport rather than provider-native CLI shims.

### Validation

- `cargo check -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --workspace --all-targets -- --nocapture`
- `cd apps/vibe-app && npm run build`
- `./scripts/dual-process-smoke.sh relay_polling`
- `./scripts/dual-process-smoke.sh overlay`
- targeted manual QA for:
  - reconnecting to the same relay and reopening prior device/project entries
  - continuing an existing Codex conversation
  - continuing an existing Claude conversation
  - provider-choice prompt rendering and custom-input answer flow
  - desktop and narrow-width device/project home plus topic-drawer layouts

### Iteration Record

- Chosen implementation mode:
  `user-specified Mode C: provider-native threaded continuation plus inline user-choice prompts`
- Implementation notes:
  added durable relay-side conversation storage and APIs, linked task runs, provider-native resume
  handles for Codex / Claude Code / OpenCode, relay-backed pending input requests, and a
  conversation-first `Sessions` UI with compact setup chrome plus secondary Git/workspace
  inspection; corrected provider ACP capability signaling so only OpenCode is advertised as
  standard ACP today; expanded OpenCode ACP update handling to cover stable
  `agent_thought_chunk`, `available_commands_update`, `session_info_update`, and optional
  `usage_update` events; surfaced optional ACP turn-usage summaries; and now prefer standard
  `session/resume` for transcript-safe OpenCode continuation when the agent advertises it, with
  `session/list` validation retained as the compatibility fallback for agents without resume.
- Implementation notes:
  the current UI tranche replaces the default `Sessions` shell with a Poe-like device/project home
  and a Telegram-like project chat page. Historical conversations are now grouped by
  `device + cwd`, topic history is a project-scoped drawer from the top-left chat chrome, the
  bottom navigation is reduced to `Chat`, `Devices`, and `Menu`, and relay URL/token editing moves
  into `Menu > Settings > Server`.
- Implementation notes:
  transcript cleanup now follows `user-specified` Mode 1 for the chat surface: the primary message
  flow keeps only user prompts, assistant replies, and inline provider-choice prompts; raw task
  lifecycle notices such as queue/start/finish, tool activity, and provider stderr are removed from
  the main transcript and reviewed through the secondary `Trace` inspector, with only a lightweight
  per-turn entry preserved in the conversation for discoverability.
- Validation results:
  `cargo check -p vibe-relay -p vibe-agent` passed; `cargo test --workspace --all-targets -- --nocapture`
  passed; `cd apps/vibe-app && npm run build` passed; `./scripts/dual-process-smoke.sh relay_polling`
  passed; `./scripts/dual-process-smoke.sh overlay` passed.
- Remaining follow-up:
  targeted manual QA from `TESTING.md` remains to be run before closing the iteration; broader ACP
  surface completion beyond the current OpenCode stable path still remains for later work,
  including authenticated ACP startup, transcript-safe richer session lifecycle support,
  protocol-extension coverage where providers require it, and any future truthful ACP transport
  work for Codex / Claude Code.
