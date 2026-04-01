# Happy-Aligned Rebuild Master Summary

## Objective

Rebuild `vibe-remote` as a Happy-shaped product family:

- `packages/vibe-app` imported from `happy-app` and adapted
- `crates/vibe-wire` rewritten in Rust from `happy-wire`
- `crates/vibe-server` rewritten in Rust from `happy-server`
- `crates/vibe-agent` rewritten in Rust from `happy-agent`
- `crates/vibe-cli` rewritten in Rust from `happy-cli`
- `crates/vibe-app-logs` rewritten in Rust from `happy-app-logs`

The target is concept, project, module, and protocol parity first. Rust-native refinements come
only after parity is proven.

## Target Repository Shape

```text
crates/
  vibe-wire/
  vibe-server/
  vibe-agent/
  vibe-cli/
  vibe-app-logs/
packages/
  vibe-app/
docs/plans/rebuild/
scripts/
```

## Current Phase

- phase: baseline reset and planning scaffold
- implementation status: not started
- immediate goal: freeze shared contracts needed by `vibe-wire`
- authoritative execution sequence: `docs/plans/rebuild/execution-plan.md`
- AI dispatch batches: `docs/plans/rebuild/execution-batches.md`

## Milestones

1. `M0 - Planning Baseline`
   - reset repository
   - create planning tree
   - freeze mapping, naming, canonical data model, and dependency order
2. `M1 - Shared Wire`
   - implement `vibe-wire`
   - define canonical Rust types and serialization contracts
   - establish compatibility vectors
3. `M2 - Server Spine`
   - implement authentication, sessions, updates, and storage minimum path in `vibe-server`
4. `M3 - Remote Agent Client`
   - implement `vibe-agent` auth, session control, and machine control paths
5. `M4 - Local Runtime CLI`
   - implement `vibe-cli` runtime, providers, daemon, sandbox, and message mapping
6. `M5 - App Import And Adaptation`
   - import `happy-app`
   - adapt naming, endpoints, protocols, and desktop shell
7. `M6 - Sidecar And Completion`
   - implement `vibe-app-logs`
   - close parity gaps and validation matrix

## Critical Risks

- protocol drift between Rust rewrites and imported app behavior
- over-abstraction in Rust before Happy parity is measured
- crypto incompatibility across account, machine, and session records
- app adaptation work starting before server and wire contracts are frozen
- CLI and agent overlapping responsibilities unless source mapping remains explicit

## Current Recommended Next Modules

- `modules/vibe-wire/messages.md`
- `modules/vibe-wire/session-protocol.md`
- `modules/vibe-wire/legacy-protocol.md`
- `modules/vibe-wire/message-meta.md`
- `modules/vibe-wire/voice.md`
