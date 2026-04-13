# Rebuild Status Dashboard

> **This file is the single source of truth for current project status.**
> Read this file first before any implementation task.
> Update this file whenever a batch, module, or gate changes status.

Last updated: 2026-04-13 (Phase 1 Module C complete)

## Current Phase

**Phase 1 — Implementation Progress.** Module C (Machine Management) is now complete with full UI interaction. Modules A and B have minor items remaining. See `PHASE1_IMPLEMENTATION_PLAN.md` for details.

### Active Plan Documents

- `HAPPY_GAP_ANALYSIS.md` - Complete gap analysis between Happy and Vibe-Remote
- `PHASE1_IMPLEMENTATION_PLAN.md` - Detailed Phase 1 implementation plan (3 modules, 26 tasks)

## Project Status

| Project | Crate / Package | Status | Wave | Archived |
|---------|----------------|--------|------|----------|
| vibe-wire | `crates/vibe-wire` | ✅ done | 0–5 | Yes — `archive/completed-projects/vibe-wire.md` |
| vibe-server | `crates/vibe-server` | ✅ done | 0–5 | Yes — `archive/completed-projects/vibe-server.md` |
| vibe-agent | `crates/vibe-agent` | ✅ done | 3 | Yes — `archive/completed-projects/vibe-agent.md` |
| vibe-cli | `crates/vibe-cli` | ✅ done | 4–6 | Yes — `archive/completed-projects/vibe-cli.md` |
| vibe-app | ~~`packages/vibe-app`~~ | ⚰️ removed | 6–7 | Yes — `archive/completed-projects/vibe-app.md` |
| vibe-app-logs | `crates/vibe-app-logs` | ✅ done | 7 | Yes — `archive/completed-projects/vibe-app-logs.md` |
| vibe-app-tauri | `packages/vibe-app-tauri` | 🔧 Wave 10 planning active | 9–10 | Wave 9 archived; Wave 10 active docs in root |

## Legacy UI Cleanup (2026-04-13)

The following cleanup was completed:

- **AppV2 is now the sole UI shell** — All routes use AppV2
- **Legacy App.tsx removed** — The 283KB monolithic component has been deleted
- **packages/vibe-app deleted** — The deprecated package has been removed
- **wave8 terminology cleaned** — Renamed to desktop-* naming:
  - `wave8-client.ts` → `desktop-client.ts`
  - `wave8-wire.ts` → `desktop-wire.ts`
  - `useWave8Desktop` → `useDesktopState`
  - `Wave8Client` → `DesktopClient`

## Batch Status

| Batch | Description | Status |
|-------|-------------|--------|
| B00–B25 | Waves 0–8 implementation | ✅ done |
| B26 | Promotion evidence and legacy deprecation | ✅ done |
| B27 | Wave 10 planning baseline and capability contract | ✅ done |
| B28 | Settings and connection-center closure | ✅ done |
| B29 | Inbox and notification closure | ✅ done |
| B30 | Remote operations workflow | ✅ done |
| B31 | Platform parity contract | ✅ done |
| B32 | Surface disposition and documentation reset | ✅ done |

### Summary: B27-B32 Deliverables

| Batch | Key Deliverables | Test Coverage |
|-------|------------------|---------------|
| **B27** | Capability Classification Model, Evidence System, Validation Runner | 51 tests (97%) |
| **B28** | Settings Registry (7 settings surfaces), Connection Center taxonomy | 17 tests (100%) |
| **B29** | Inbox/Feed/Notification Taxonomy, Event Source Classification (4 surfaces) | 18 tests (100%) |
| **B30** | Remote Operations Registry (6 surfaces), Workflow Step Classification | 19 tests |
| **B31** | Platform Support Matrix and Browser Contract metadata layer | 27 tests |
| **B32** | Social/Developer Surface Disposition (deferred/hidden/internal classification) | Completed |

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
| G8 | Wave 10 planning tree exists and is the active source of truth | ✅ satisfied |
| G9 | Customer-visible capability claims are backed by code and platform scope | ✅ satisfied |
| G10 | Settings, notifications, and remote-operations surfaces expose clear product contracts | ✅ satisfied |
| G11 | Desktop/Android/browser support claims are explicit and evidence-backed | ✅ satisfied |
| G12 | Social and developer-only surfaces are formally classified | ✅ satisfied |
| G13 | Active docs and validation commands reflect the Wave 10 standard | ✅ satisfied |

## Architecture Summary

### UI Shell

**AppV2** is the sole UI shell for the `vibe-app-tauri` package. It uses:

- **Design Token System**: TypeScript-based tokens (`design-system/tokens.ts`)
- **Modular Component Architecture**: Organized into layers (ui, layout, surfaces, renderers, routes)
- **Desktop State Management**: `useDesktopState` hook (formerly `useWave8Desktop`)

### Route Coverage

All routes are handled by AppV2:
- `home`, `inbox`, `new-session`, `session-recent`, `session-detail`
- `settings-*` (all settings routes)
- `restore-index`, `restore-manual` (account restoration)
- `session-info`, `session-message`, `session-files`, `session-file` (session details)

## Next Actions

Phase 1 implementation is largely complete. Remaining items:

1. **Module A: 消息渲染增强** (可选后续)
   - 代码块行号显示设置联动
   - Diff 渲染器设置联动
   - 工具特定渲染器 (Bash, FileRead, Edit)

**All critical functionality is implemented and tested.**

## Archived Work

All completed project plans, module plans, and historical wave documents have been moved to
`docs/plans/rebuild/archive/`. See that directory's README for details.

- Completed projects: `archive/completed-projects/`
- Completed modules (waves 0–9): `archive/completed-modules/`
- Wave 8 historical plans: `archive/wave8/`
- Wave 9 planning documents: `archive/wave9/`
- Other historical docs: `archive/historical/`
