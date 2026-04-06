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
  vibe-app-tauri/   # planned next-iteration desktop rewrite
docs/plans/rebuild/
scripts/
```

## Current Phase

- phase: the original Happy-aligned rebuild plan is complete through `Wave 7`; the next iteration target is the parallel desktop rewrite project `packages/vibe-app-tauri`
- implementation status: `vibe-wire` is implemented and validated; `vibe-server` now ships a single-instance Wave-2 spine plus the Wave-4 support surfaces covering files/images, account/settings/usage, utility APIs, artifacts/access keys, connect/GitHub, social/feed, auxiliary socket APIs, and monitoring; `vibe-agent` now ships the Wave-3 remote-control slice covering auth, HTTP session/machine control, live session socket control, machine RPC, and stable CLI output; `vibe-cli` now ships the Wave-5 local runtime covering auth/connect, command bootstrap, provider execution, daemon control, sandbox policy, persistence/resume, session-protocol mapping, and end-to-end validation across the supported provider paths; `vibe-app` now ships the imported/adapted Wave-6 app baseline with localized workspace bootstrap, `vibe-wire` fixture compatibility tests, centralized Vibe endpoint/env wiring, public Vibe branding, desktop/Tauri bundle identifiers, and successful web export smoke validation; `vibe-app-logs` now ships the Wave-7 minimal sidecar runtime with `POST /logs` ingestion, Vibe home-directory file sinks, root `yarn app-logs` launch support, and startup/ingestion smoke coverage
- immediate goal: keep the completed rebuild baseline stable and execute the next desktop iteration around a new parallel `packages/vibe-app-tauri` project that recreates the current app's desktop UI and behavior without destabilizing `packages/vibe-app`
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
8. `M7 - Parallel Desktop Rewrite`
   - create `packages/vibe-app-tauri`
   - recreate the desktop UX with a Tauri 2 + web-native frontend
   - keep `packages/vibe-app` as the production baseline until promotion

## Critical Risks

- protocol drift between Rust rewrites and imported app behavior
- over-abstraction in Rust before Happy parity is measured
- crypto incompatibility across account, machine, and session records
- app adaptation work starting before server and wire contracts are frozen
- CLI and agent overlapping responsibilities unless source mapping remains explicit

## Current Recommended Next Modules

- `projects/vibe-app-tauri.md`
- `vibe-app-tauri-extraction-inventory.md`
- `vibe-app-tauri-route-inventory.md`
- `vibe-app-tauri-capability-matrix.md`
- `vibe-app-tauri-coexistence-matrix.md`
- future module sequence:
  - `modules/vibe-app-tauri/bootstrap-and-package.md`
  - `modules/vibe-app-tauri/desktop-shell-and-routing.md`
  - `modules/vibe-app-tauri/core-logic-extraction.md`
  - `modules/vibe-app-tauri/desktop-platform-adapters.md`
  - `modules/vibe-app-tauri/auth-and-session-state.md`
  - `modules/vibe-app-tauri/session-ui-parity.md`
  - `modules/vibe-app-tauri/secondary-surfaces.md`
  - `modules/vibe-app-tauri/release-and-promotion.md`
