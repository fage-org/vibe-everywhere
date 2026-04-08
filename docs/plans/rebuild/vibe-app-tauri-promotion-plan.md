# `vibe-app-tauri` Promotion And Deprecation Plan

## Archival Status

This file is historical Wave 8 desktop-only planning material.

Do not use it as active execution authority for Wave 9. Use the active Wave 9 planning set instead:

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/*`

## Purpose

Define the explicit promotion gate, hold or rollback path, and archival sequencing now that
`packages/vibe-app` is deprecated and `packages/vibe-app-tauri` is the sole active desktop path.

## Promotion Gate

Promotion must not proceed until all of the following are true:

- the parity checklist has no remaining required `P0` or `P1` gaps except explicit, approved
  deferrals
- Linux, macOS, and Windows startup validation is recorded in
  `docs/plans/rebuild/vibe-app-tauri-promotion-baseline.md`
- realistic session-load performance and memory review is recorded in
  `docs/plans/rebuild/vibe-app-tauri-promotion-baseline.md`
- side-by-side parity review is recorded in
  `docs/plans/rebuild/vibe-app-tauri-promotion-baseline.md`
- the shared `app-v*` release workflow has produced the active `vibe-app-tauri` desktop artifacts successfully
- archival and rollback notes for `packages/vibe-app` are explicit in this file and related Wave 9 docs

## Rollout Stages

1. Active path hardening
   - `packages/vibe-app-tauri` ships as the sole active desktop artifact
   - docs, helper scripts, and release notes point at `packages/vibe-app-tauri`
2. Promotion candidate review
   - baseline evidence is complete
   - release owner, engineering owner, and product owner review the candidate
3. Sole active desktop confirmation
   - `packages/vibe-app-tauri` is confirmed as the active desktop path
   - bundle naming, updater ownership, and docs remain aligned to the active package
4. Legacy archival tracking
   - the old `packages/vibe-app` desktop path remains deprecated reference-only
   - removal timing is tracked only after the new path has stable production usage

## Fallback Plan

If a promotion candidate regresses before or after the default switch:

- stop further rollout immediately
- keep the last known-good `packages/vibe-app-tauri` artifact set available until the issue is resolved
- publish a follow-up `app-v*` release only after the regression is understood and validated
- record the regression and rollback decision in this file before retrying promotion

## Deprecation Plan

The old desktop path must not be deprecated until:

- the new desktop path has cleared the promotion gate
- the fallback story is verified and documented
- release owners approve removal timing

Current deprecation status:

- `packages/vibe-app` desktop path: deprecated reference-only
- `packages/vibe-app-tauri`: sole active owner
- target switch date: `complete in planning`
- target legacy archival date: `pending`

## Approval

| Role | Reviewer | Status | Notes |
| --- | --- | --- | --- |
| Desktop product owner | pending | pending | |
| Desktop engineering owner | pending | pending | |
| Release owner | pending | pending | |
