# `vibe-app-tauri` Promotion And Deprecation Plan

## Purpose

Define the explicit promotion gate, fallback path, and deprecation sequencing for moving desktop
default ownership from `packages/vibe-app` to `packages/vibe-app-tauri`.

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
- the shared `app-v*` release workflow has produced distinct `vibe-app-tauri` desktop preview
  bundles successfully alongside the shipping app assets
- `packages/vibe-app` remains the default desktop path until explicit approval is recorded here

## Rollout Stages

1. Parallel preview only
   - `packages/vibe-app-tauri` ships as a non-default desktop preview asset in the shared
     `app-v*` release workflow
   - docs, helper scripts, release notes, and asset naming continue to treat `packages/vibe-app`
     as the default desktop path
2. Promotion candidate review
   - baseline evidence is complete
   - release owner, engineering owner, and product owner review the candidate
3. Default desktop switch
   - `packages/vibe-app-tauri` becomes the documented default desktop path
   - bundle naming, updater ownership, and docs are updated in the coexistence matrix
4. Legacy path deprecation
   - the old `packages/vibe-app` desktop path is documented as deprecated
   - removal timing is tracked only after the new path has stable production usage

## Fallback Plan

If a promotion candidate regresses before or after the default switch:

- stop further rollout immediately
- keep the current `packages/vibe-app` desktop release lane available until the issue is resolved
- publish a follow-up `app-v*` release only after the regression is understood and validated
- record the regression and rollback decision in this file before retrying promotion

## Deprecation Plan

The old desktop path must not be deprecated until:

- the new desktop path has cleared the promotion gate
- the fallback story is verified and documented
- release owners approve removal timing

Current deprecation status:

- `packages/vibe-app` desktop path: active default owner
- `packages/vibe-app-tauri`: parallel preview owner
- target switch date: `pending`
- target legacy deprecation date: `pending`

## Approval

| Role | Reviewer | Status | Notes |
| --- | --- | --- | --- |
| Desktop product owner | pending | pending | |
| Desktop engineering owner | pending | pending | |
| Release owner | pending | pending | |
