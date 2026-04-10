# Rebuild Status Dashboard

> **This file is the single source of truth for current project status.**
> Read this file first before any implementation task.
> Update this file whenever a batch, module, or gate changes status.

Last updated: 2026-04-10

## Current Phase

**Wave 9 — Complete.** All batches B00–B26 are done. G7 is satisfied with the promotion baseline
artifact created; manual platform validation sections remain [PENDING] until human sign-off fills them
in. After that, `--promotion-ready` strict validation can pass.

The project has no further batches or open gates. Wave 9 plans have been archived.

## Project Status

| Project | Crate / Package | Status | Wave | Archived |
|---------|----------------|--------|------|----------|
| vibe-wire | `crates/vibe-wire` | ✅ done | 0–5 | Yes — `archive/completed-projects/vibe-wire.md` |
| vibe-server | `crates/vibe-server` | ✅ done | 0–5 | Yes — `archive/completed-projects/vibe-server.md` |
| vibe-agent | `crates/vibe-agent` | ✅ done | 3 | Yes — `archive/completed-projects/vibe-agent.md` |
| vibe-cli | `crates/vibe-cli` | ✅ done | 4–6 | Yes — `archive/completed-projects/vibe-cli.md` |
| vibe-app | `packages/vibe-app` | ⚰️ deprecated | 6–7 | Yes — `archive/completed-projects/vibe-app.md` |
| vibe-app-logs | `crates/vibe-app-logs` | ✅ done | 7 | Yes — `archive/completed-projects/vibe-app-logs.md` |
| vibe-app-tauri | `packages/vibe-app-tauri` | ✅ done | 9 | Yes — `archive/completed-projects/vibe-app-tauri.md` |

## Batch Status

| Batch | Description | Status |
|-------|-------------|--------|
| B00–B25 | Waves 0–8 implementation | ✅ done |
| B26 | Promotion evidence and legacy deprecation | ✅ done |

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
| G7 | `vibe-app-tauri` reaches desktop parity with an explicit promotion/deprecation plan | ✅ satisfied (B26; promotion baseline artifact created; manual platform sign-off [PENDING]) |

## Next Actions

1. Complete manual platform validation in `artifacts/vibe-app-tauri/promotion-baseline.md`
2. Run `node scripts/validate-vibe-app-tauri-promotion.mjs --promotion-ready` after all sections are signed off

## Archived Work

All completed project plans, module plans, and historical wave documents have been moved to
`docs/plans/rebuild/archive/`. See that directory's README for details.

- Completed projects: `archive/completed-projects/`
- Completed modules (waves 0–9): `archive/completed-modules/`
- Wave 8 historical plans: `archive/wave8/`
- Wave 9 planning documents: `archive/wave9/`
- Other historical docs: `archive/historical/`