# Rebuild Status Dashboard

> **This file is the single source of truth for current project status.**
> Read this file first before any implementation task.
> Update this file whenever a batch, module, or gate changes status.

Last updated: 2026-04-10

## Current Phase

**Wave 9 — Batch B26 (Promotion Evidence and Legacy Deprecation) pending**

All Waves 0–8 are complete or historical. The only remaining open batch is B26, which requires a
promotion baseline artifact sign-off before `packages/vibe-app` can be formally retired from CI and
release ownership.

## Project Status

| Project | Crate / Package | Status | Wave | Archived |
|---------|----------------|--------|------|----------|
| vibe-wire | `crates/vibe-wire` | ✅ done | 0–5 | Yes — `archive/completed-projects/vibe-wire.md` |
| vibe-server | `crates/vibe-server` | ✅ done | 0–5 | Yes — `archive/completed-projects/vibe-server.md` |
| vibe-agent | `crates/vibe-agent` | ✅ done | 3 | Yes — `archive/completed-projects/vibe-agent.md` |
| vibe-cli | `crates/vibe-cli` | ✅ done | 4–6 | Yes — `archive/completed-projects/vibe-cli.md` |
| vibe-app | `packages/vibe-app` | ⚰️ deprecated | 6–7 | Yes — `archive/completed-projects/vibe-app.md` |
| vibe-app-logs | `crates/vibe-app-logs` | ✅ done | 7 | Yes — `archive/completed-projects/vibe-app-logs.md` |
| vibe-app-tauri | `packages/vibe-app-tauri` | 🔧 active | 9 | No — active project plan at `projects/vibe-app-tauri.md` |

## Batch Status

| Batch | Description | Status |
|-------|-------------|--------|
| B00–B25 | Waves 0–8 implementation | ✅ done |
| B26 | Promotion evidence and legacy deprecation | 🔧 pending |

## Active Module Plans (Wave 9 — `vibe-app-tauri`)

| Module Plan | Batch | Status |
|-------------|-------|--------|
| `modules/vibe-app-tauri/auth-and-identity-flows.md` | B19 | ✅ done |
| `modules/vibe-app-tauri/desktop-shell-and-platform-parity.md` | B19 | ✅ done |
| `modules/vibe-app-tauri/mobile-shell-and-navigation.md` | B19–B20 | ✅ done |
| `modules/vibe-app-tauri/mobile-native-capabilities.md` | B20 | ✅ done |
| `modules/vibe-app-tauri/session-rendering-and-composer.md` | B21 | ✅ done |
| `modules/vibe-app-tauri/session-runtime-and-storage.md` | B21 | ✅ done |
| `modules/vibe-app-tauri/secondary-routes-and-social.md` | B22 | ✅ done |
| `modules/vibe-app-tauri/secondary-surfaces.md` | B22 | ✅ done |
| `modules/vibe-app-tauri/release-ota-and-store-migration.md` | B23 | ✅ done |
| `modules/vibe-app-tauri/web-export-and-browser-runtime.md` | B24 | ✅ done |
| `modules/vibe-app-tauri/universal-bootstrap-and-runtime.md` | B24 | ✅ done |
| `modules/vibe-app-tauri/release-and-promotion.md` | B25 | ✅ done |
| `modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md` | B26 | 🔧 pending |

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
| G7 | `vibe-app-tauri` reaches desktop parity with an explicit promotion/deprecation plan | 🔧 pending (B26) |

## Next Actions

1. Complete promotion baseline artifact for B26
2. Sign off on `packages/vibe-app` deprecation gate
3. Once B26 closes, archive `promotion-and-vibe-app-deprecation.md` and update this file

## Archived Work

All completed project plans, module plans, and historical wave documents have been moved to
`docs/plans/rebuild/archive/`. See that directory's README for details.

- Completed projects: `archive/completed-projects/`
- Completed modules (waves 0–7): `archive/completed-modules/`
- Wave 8 historical plans: `archive/wave8/`
- Other historical docs: `archive/historical/`