# Happy-Aligned Rebuild Master Details

## Goal

Transform the repository from the removed legacy Vibe implementation into a Happy-shaped
multi-project codebase where:

- package and crate boundaries mirror Happy
- module responsibilities mirror Happy
- protocol semantics mirror Happy
- the app is sourced from Happy directly
- non-app subsystems are rewritten in Rust without changing external behavior

## Final Repository Layout

- `crates/vibe-wire`
  - canonical shared Rust models for messages, protocols, message metadata, voice usage, and
    compatibility helpers
- `crates/vibe-server`
  - Rust server replacing `happy-server`
  - owns HTTP APIs, socket updates, storage, auth, feed/social, and monitoring
- `crates/vibe-agent`
  - Rust remote-control client replacing `happy-agent`
  - owns account auth, session listing, session control, machine RPC, and CLI output
- `crates/vibe-cli`
  - Rust local runtime replacing `happy-cli`
  - owns provider runtimes, daemon, runtime transport, sandbox, persistence, and terminal UI
- `crates/vibe-app-logs`
  - Rust sidecar replacing `happy-app-logs`
- `packages/vibe-app`
  - imported Happy app plus Vibe adaptation layer

## Project Mapping From Happy To Vibe

| Happy project | Vibe target | Language | Delivery rule |
| --- | --- | --- | --- |
| `packages/happy-wire` | `crates/vibe-wire` | Rust | implement first and keep canonical |
| `packages/happy-server` | `crates/vibe-server` | Rust | depend only on `vibe-wire` for shared contracts |
| `packages/happy-agent` | `crates/vibe-agent` | Rust | mirror control-client behavior |
| `packages/happy-cli` | `crates/vibe-cli` | Rust | mirror runtime and provider orchestration |
| `packages/happy-app` | `packages/vibe-app` | TypeScript | import first, adapt second |
| `packages/happy-app-logs` | `crates/vibe-app-logs` | Rust | defer until main path is stable |

## Implementation Principles

- Happy-first parity: do not redesign product boundaries during migration.
- Wire-first authority: protocol and data model decisions belong to `vibe-wire`.
- Explicit divergence: every deliberate deviation from Happy must be written into planning files.
- Minimum working slices: each milestone must deliver one usable vertical path before broadening.
- App-last adaptation: the app consumes locked contracts; it does not define them.

## Global Dependency Graph

1. `shared/source-crosswalk.md`
2. `shared/naming.md`
3. `shared/data-model.md`
4. `shared/protocol-session.md`
5. `shared/protocol-auth-crypto.md`
6. `shared/protocol-api-rpc.md`
7. `vibe-wire`
8. `vibe-server`
9. `vibe-agent`
10. `vibe-cli`
11. `vibe-app`
12. `vibe-app-logs`

No downstream implementation may define its own variant of types that belong in an upstream layer.

## Milestone Breakdown

### M0 - Planning Baseline

- create planning files
- define canonical mapping and naming
- define shared contracts and validation gates

### M1 - Shared Wire

- build Rust crate and module layout
- port message, session, legacy, metadata, and voice schemas
- add serde and validation helpers
- publish compatibility vectors for non-Rust consumers

### M2 - Server Spine

- implement authentication and credential flows
- implement encrypted session CRUD and updates
- implement machine records and update streams
- support minimum endpoints needed by `vibe-agent` and `vibe-app`

### M3 - Remote Agent Client

- implement account auth and credential storage
- implement session and machine listing
- implement session create/send/history/stop/wait
- implement machine RPC support

### M4 - Local Runtime CLI

- implement CLI entrypoints and runtime dispatch
- implement provider runtimes and protocol mapping
- implement daemon control plane and sandbox manager
- implement persistence and resume

### M5 - App Import And Adaptation

- import Happy app source tree
- rename package and environment surfaces to Vibe
- adapt API endpoints, deep links, protocol parsers, and Tauri/Desktop surfaces
- validate app against Rust server and clients

### M6 - Sidecar And Completion

- implement app log sidecar
- close residual parity gaps
- finalize validation coverage

## Cross-Cutting Concerns

- naming migration
- account and session crypto
- versioned encrypted record format
- protocol compatibility during partial implementation
- mobile and desktop app environment mapping
- shared validation and test vectors

## Acceptance Gates

- Gate 0: planning tree is complete and decision-bearing
- Gate 1: `vibe-wire` exposes canonical Rust contracts with tests and vectors
- Gate 2: `vibe-server` supports app and agent minimum paths
- Gate 3: `vibe-agent` can control real sessions end-to-end
- Gate 4: `vibe-cli` can run local providers and stream messages correctly
- Gate 5: `vibe-app` works against Vibe services without Happy-specific branding leakage
- Gate 6: residual utilities, logs, and release scaffolding are aligned

## Change Control Rules

- Do not implement from `master-*` files directly. Use module plans.
- Do not merge a module implementation without updating its status and any affected shared files.
- When a module discovers a missing dependency, add it to the shared or project plan before coding.
- If a Happy source moves or evolves, update `shared/source-crosswalk.md` before adjusting code.
