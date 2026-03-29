# Iteration Plan v5 Summary

Last updated: 2026-03-29

## Scope

Version 5 starts the next product epoch after the session-first consolidation work. This version
focuses on replacing the current “task dashboard” mental model with a conversation-first remote
coding product:

- long-lived conversations instead of disposable ad hoc sessions
- provider-native resume / continue behavior instead of prompt-history replay
- a chat-first primary surface with secondary inspectors for Git, files, raw trace, and preview
- inline user-choice questions with option chips plus a custom-input fallback when the provider
  asks for clarification or permission

Full implementation detail lives in [`v5-details.md`](./v5-details.md).

## Status

| Iteration | Title | Status |
| --- | --- | --- |
| 15 | Conversation-First Threaded AI Sessions | in_progress |

## Current State

- Iteration 15 core implementation is landed for the product shift from `task/event stream`
  supervision to `conversation/thread` remote coding.
- The implementation target is Mode C (`user-specified`): provider-native threaded continuation
  for supported providers rather than a history-replay approximation.
- Relay, agent, and app now include durable conversation records, provider-native resume handles,
  inline user-choice prompts, and a chat-first `Sessions` surface.
- ACP capability reporting is now aligned with the real implementation surface: only OpenCode is
  advertised as standard ACP, and the OpenCode ACP event mapping now covers the stable
  `agent_thought_chunk` and `available_commands_update` variants.
- ACP completion follow-up is now explicitly tracked inside Iteration 15: OpenCode also validates
  stored session handles through `session/list`, surfaces stable `session_info_update`, and
  consumes optional usage updates when the agent returns them; remaining ACP work is ordered and
  recorded in the detail plan.
- The transcript-safe resume path is now `user-specified` Mode 3: prefer standard ACP
  `session/resume` whenever the agent advertises it, and keep `session/list` validation as the
  compatibility fallback for agents that do not yet support resume.
- Automated validation is complete; targeted manual QA is still pending before the iteration can be
  closed out.

## Lookup Notes

- Need the detailed acceptance criteria or implementation notes:
  read [`v5-details.md`](./v5-details.md).
- Need the previous session-first workflow epoch:
  read [`v4-summary.md`](./v4-summary.md).
- Need the active remediation track:
  read [`../remediation/v11-summary.md`](../remediation/v11-summary.md).
