# Rebuild Status Dashboard

> **This file is the single source of truth for current project status.**
> Read this file first before any implementation task.
> Update this file whenever a batch, module, or gate changes status.

Last updated: 2026-04-11

## Current Phase

**Wave 10 — Planning Active.** Wave 9 is complete and archived. Wave 10 is the active planning
track for `packages/vibe-app-tauri` and exists to close the gap between route coverage and
product-contract completion.

The historical Wave 9 manual platform validation path remains archived and is not part of the
active repository standard.

## Project Status

| Project | Crate / Package | Status | Wave | Archived |
|---------|----------------|--------|------|----------|
| vibe-wire | `crates/vibe-wire` | ✅ done | 0–5 | Yes — `archive/completed-projects/vibe-wire.md` |
| vibe-server | `crates/vibe-server` | ✅ done | 0–5 | Yes — `archive/completed-projects/vibe-server.md` |
| vibe-agent | `crates/vibe-agent` | ✅ done | 3 | Yes — `archive/completed-projects/vibe-agent.md` |
| vibe-cli | `crates/vibe-cli` | ✅ done | 4–6 | Yes — `archive/completed-projects/vibe-cli.md` |
| vibe-app | `packages/vibe-app` | ⚰️ deprecated | 6–7 | Yes — `archive/completed-projects/vibe-app.md` |
| vibe-app-logs | `crates/vibe-app-logs` | ✅ done | 7 | Yes — `archive/completed-projects/vibe-app-logs.md` |
| vibe-app-tauri | `packages/vibe-app-tauri` | 🔧 Wave 10 planning active | 9–10 | Wave 9 archived; Wave 10 active docs in root |

## Batch Status

| Batch | Description | Status |
|-------|-------------|--------|
| B00–B25 | Waves 0–8 implementation | ✅ done |
| B26 | Promotion evidence and legacy deprecation | ✅ done |
| B27 | Wave 10 planning baseline and capability contract | planned |
| B28 | Settings and connection-center closure | planned |
| B29 | Inbox and notification closure | planned |
| B30 | Remote operations workflow | planned |
| B31 | Platform parity contract | planned |
| B32 | Surface disposition and documentation reset | planned |

## Active Wave 10 Modules (`vibe-app-tauri`)

- `modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
- `modules/vibe-app-tauri/settings-and-connection-center.md`
- `modules/vibe-app-tauri/inbox-and-notification-closure.md`
- `modules/vibe-app-tauri/remote-operations-surfaces.md`
- `modules/vibe-app-tauri/platform-parity-and-browser-contract.md`
- `modules/vibe-app-tauri/social-and-developer-surface-disposition.md`

## Archived Module Plans (Wave 9 — `vibe-app-tauri`)

All Wave 9 module plans are complete and archived. See
`docs/plans/rebuild/archive/completed-modules/` for individual files.

## Acceptance Gates

| Gate | Description | Status |
|------|-------------|--------|
| G0 | Wire protocol compiles, serde round-trips pass | ✅ satisfied |
| G1 | Server boots, health endpoint 200, one session flows end to end | ✅ satisfied |
| G2 | Agent connects, receives session, streams output over machine RPC | ✅ satisfied |
| G3 | CLI launches session, streams, exits cleanly | ✅ satisfied |
| G4 | App renders session feed and composer | ✅ satisfied |
| G5 | Log sidecar tails and serves recent lines | ✅ satisfied |
| G6 | All prior crates pass `cargo test` on CI | ✅ satisfied |
| G7 | `vibe-app-tauri` reaches desktop parity with an explicit promotion/deprecation plan | ✅ satisfied |
| G8 | Wave 10 planning tree exists and is the active source of truth | planned |
| G9 | Customer-visible capability claims are backed by code and platform scope | planned |
| G10 | Settings, notifications, and remote-operations surfaces expose clear product contracts | planned |
| G11 | Desktop/Android/browser support claims are explicit and evidence-backed | planned |
| G12 | Social and developer-only surfaces are formally classified | planned |
| G13 | Active docs and validation commands reflect the Wave 10 standard | planned |

## Next Actions

1. Begin Wave 10 from `modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
2. Keep all Wave 10 implementation work mapped to one active module plan at a time
3. Preserve Wave 9 archival documents as continuity references only

## Archived Work

All completed project plans, module plans, and historical wave documents have been moved to
`docs/plans/rebuild/archive/`. See that directory's README for details.

- Completed projects: `archive/completed-projects/`
- Completed modules (waves 0–9): `archive/completed-modules/`
- Wave 8 historical plans: `archive/wave8/`
- Wave 9 planning documents: `archive/wave9/`
- Other historical docs: `archive/historical/`
