# Iteration Plan v7 Summary

## Scope

- refactor `vibe-agent` to expose a single ACP-first task execution model
- remove CLI as a first-class execution protocol for new tasks and conversations
- keep `OpenCode` on native ACP and move `Codex` behind the existing embedded ACP adapter
- stop advertising `Claude Code` as available until an ACP path exists

## Status

- active
- implementation complete
- validation complete

## Current Target

- unify relay, agent, and app semantics around `execution_protocol = acp`
- keep the current UI structure, but source ACP availability from actual ACP-capable endpoints
- remove `supports_acp` drift between control-plane metadata and runtime behavior

## Dependencies

- `apps/vibe-agent` ACP runtime and provider detection changes
- `apps/vibe-relay` task/conversation creation and task update normalization
- `crates/vibe-core` provider/task/conversation model updates

## Acceptance Summary

- new tasks and conversations are created with ACP execution semantics only
- available ACPs shown in the app match agent-reported runtime availability
- `Codex` tasks execute through the embedded ACP adapter without deprecated CLI flags
- no runtime path still treats CLI as the primary task execution protocol
