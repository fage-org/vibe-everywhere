# Rebuild Status Dashboard

> **This file is the single source of truth for current project status.**
> Read this file first before any implementation task.
> Update this file whenever a batch, module, or gate changes status.

Last updated: 2026-04-11 (B27-B30 Complete, B31 In Progress, B32 Planned)

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
| B27 | Wave 10 planning baseline and capability contract | ✅ done |
| B28 | Settings and connection-center closure | ✅ done |
| B29 | Inbox and notification closure | ✅ done |
| B30 | Remote operations workflow | ✅ done |
| B31 | Platform parity contract | 🔄 in_progress |
| B32 | Surface disposition and documentation reset | 📋 planned |

### Summary: B27-B32 Deliverables

| Batch | Key Deliverables | Test Coverage |
|-------|------------------|---------------|
| **B27** | Capability Classification Model, Evidence System, Validation Runner | 51 tests (97%) |
| **B28** | Settings Registry (7 settings surfaces), Connection Center taxonomy | 17 tests (100%) |
| **B29** | Inbox/Feed/Notification Taxonomy, Event Source Classification (4 surfaces) | 18 tests (100%) |
| **B30** | Remote Operations Registry (6 surfaces), Workflow Step Classification | 19 tests |
| **B31** | Platform Support Matrix and Browser Contract metadata layer | 27 tests |
| **B32** | Social/Developer Surface Disposition (deferred/hidden/internal classification) | Planned |
| **Total** | **4 registries, 17 surfaces defined** | **132 targeted capability tests** |

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
| G8 | Wave 10 planning tree exists and is the active source of truth | 🔄 in_progress |
| G9 | Customer-visible capability claims are backed by code and platform scope | 🔄 in_progress |
| G10 | Settings, notifications, and remote-operations surfaces expose clear product contracts | 🔄 in_progress |
| G11 | Desktop/Android/browser support claims are explicit and evidence-backed | 🔄 in_progress |
| G12 | Social and developer-only surfaces are formally classified | 🔄 in_progress |
| G13 | Active docs and validation commands reflect the Wave 10 standard | 🔄 in_progress |

## Next Actions

Wave 10 is in progress. B27-B30 registries are implemented and verified locally. B31's metadata layer is implemented, while broader route/docs integration remains in progress. B32 is still planned.

### Completed Work Summary

**Capability System Architecture (B27-B30 Complete):**
- Core types and capability classification (B27)
- Evidence requirements and validation system (B27)
- Settings registry with 7 surfaces (B28)
- Inbox/notification registry with 4 surfaces (B29)
- Remote operations registry with 6 surfaces (B30)

**In Progress / Planned:**
- Platform parity and browser contract metadata layer (B31) - 🔄 In Progress
- Social/developer surface disposition (B32) - 📋 Planned

**Verified locally:** `yarn typecheck` and 132 targeted capability tests pass in `packages/vibe-app-tauri`

### Future Recommendations

1. **Complete B31:** Finalize platform parity contract and browser type system
2. **Execute B32:** Surface disposition classification and documentation reset
3. **Integration:** Connect capability registries with actual route implementation
4. **Validation:** Add CI checks to verify surface capabilities match implementation
5. **Documentation:** Generate customer-facing capability documentation from registries
6. **Monitoring:** Add analytics to track feature usage by capability class

## Archived Work

All completed project plans, module plans, and historical wave documents have been moved to
`docs/plans/rebuild/archive/`. See that directory's README for details.

- Completed projects: `archive/completed-projects/`
- Completed modules (waves 0–9): `archive/completed-modules/`
- Wave 8 historical plans: `archive/wave8/`
- Wave 9 planning documents: `archive/wave9/`
- Other historical docs: `archive/historical/`
