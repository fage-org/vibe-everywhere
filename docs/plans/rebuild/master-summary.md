# Happy-Aligned Rebuild Master Summary

## Objective

Rebuild `vibe-remote` as a Happy-shaped product family:

- `packages/vibe-app` imported from `happy-app` and retained only as deprecated historical reference
- `packages/vibe-app-tauri` active Wave 9 replacement package aligned directly to `happy-app` for future app ownership
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
  vibe-app/         # deprecated historical reference only
  vibe-app-tauri/   # active Wave 9 replacement package
docs/plans/rebuild/
scripts/
```

## Current Phase

- phase: the original Happy-aligned rebuild plan is complete through `Wave 7`; Wave 8 is closed as
  the historical desktop-preview baseline for `packages/vibe-app-tauri`; Wave 9 has recorded the
  default-owner switch to `packages/vibe-app-tauri` for desktop, Android, and retained browser
  web/export ownership, while the final promotion evidence gate remains open until the baseline
  artifact is fully signed off
- implementation status: `vibe-wire` is implemented and validated; `vibe-server` now ships a single-instance Wave-2 spine plus the Wave-4 support surfaces covering files/images, account/settings/usage, utility APIs, artifacts/access keys, connect/GitHub, social/feed, auxiliary socket APIs, and monitoring; `vibe-agent` now ships the Wave-3 remote-control slice covering auth, HTTP session/machine control, live session socket control, machine RPC, and stable CLI output; `vibe-cli` now ships the Wave-5 local runtime covering auth/connect, command bootstrap, provider execution, daemon control, sandbox policy, persistence/resume, session-protocol mapping, and end-to-end validation across the supported provider paths; `vibe-app` remains only as a deprecated historical reference when Happy is insufficient; `vibe-app-tauri` is the active Wave 9 replacement package; `vibe-app-logs` now ships the Wave-7 minimal sidecar runtime with `POST /logs` ingestion, Vibe home-directory file sinks, root `yarn app-logs` launch support, and startup/ingestion smoke coverage
- immediate goal: close the remaining Wave 9 promotion evidence gate, keep `packages/vibe-app-tauri`
  stable as the switched default path, and honor the documented legacy retention window without
  reopening the deprecated `packages/vibe-app` pipeline
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
   - close the desktop-preview track as historical baseline material
9. `M8 - Unified App Replacement`
   - turn `packages/vibe-app-tauri` into the active Wave 9 replacement package for desktop/mobile/web-export ownership
   - preserve browser web export explicitly
   - finalize release and store ownership in the replacement package

## Critical Risks

- protocol drift between Rust rewrites and imported app behavior
- over-abstraction in Rust before Happy parity is measured
- crypto incompatibility across account, machine, and session records
- app adaptation work starting before server and wire contracts are frozen
- CLI and agent overlapping responsibilities unless source mapping remains explicit

## Current Recommended Next Modules

- `projects/vibe-app-tauri.md`
- `shared/ui-visual-parity.md`
- `vibe-app-tauri-wave9-unified-replacement-plan.md`
- `vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `vibe-app-tauri-wave9-migration-and-release-plan.md`
- immediate next implementation modules:
  - `modules/vibe-app-tauri/universal-bootstrap-and-runtime.md`
  - `modules/vibe-app-tauri/shared-core-from-happy.md`
  - `modules/vibe-app-tauri/mobile-shell-and-navigation.md`
  - `modules/vibe-app-tauri/web-export-and-browser-runtime.md`
  - `modules/vibe-app-tauri/desktop-shell-and-platform-parity.md`
  - priority note: shell/session/secondary-surface style correction is active Wave 9 work, not deferred polish
