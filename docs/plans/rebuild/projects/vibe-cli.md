# Project Plan: vibe-cli

## Purpose

`vibe-cli` is the Rust replacement for Happy CLI: local runtime orchestration, provider adapters,
daemon, sandbox, persistence, resume, and terminal interaction.

## Happy Source

- primary source: `packages/happy-cli`
- supporting sources: `packages/happy-wire`, `packages/happy-server`, `packages/happy-agent`

## Target Layout

- crate: `crates/vibe-cli`
- expected module groups:
  - `agent`
  - `api`
  - `providers`
  - `daemon`
  - `sandbox`
  - `persistence`
  - `resume`
  - `ui`
  - `utils`
  - `main`

## Public Interfaces

- binary: `vibe`
- provider/runtime entrypoints
- daemon lifecycle commands
- auth and connect flows
- sandbox and local runtime behavior

## Internal Module Map

- `agent`: provider-independent abstractions and transport
- `api`: server/session/machine communication
- `providers`: Claude, Codex, Gemini, OpenClaw, ACP
- `daemon`: local control plane and process orchestration
- `sandbox`: runtime isolation and policy mapping
- `persistence`: local state storage
- `resume`: resume/attach behavior
- `ui`: terminal presentation and interaction
- `utils`: low-level helpers that are not shared wire contracts

## Implementation Order

1. agent core and transport
2. API client and auth helpers
3. daemon
4. session protocol mapper
5. provider runtimes
6. sandbox
7. persistence and resume
8. terminal UI

## Compatibility Requirements

- provider behavior must map to Happy concepts before Rust-specific cleanup
- session message and update emission must remain compatible with app expectations
- local persistence and daemon conventions must not silently diverge from Happy behavior

## Testing Strategy

- provider-level unit/integration tests
- daemon tests
- protocol mapper tests
- persistence/resume tests
- real end-to-end runtime tests against `vibe-server`

## Acceptance Criteria

- at least one provider path works end-to-end
- daemon and sandbox behaviors match the planned surface
- session protocol and legacy outputs are validated against `vibe-wire`

## Deferred Items

- major CLI UX redesign
- new providers not represented in Happy
