# Module Plan: vibe-app-tauri/platform-parity-and-browser-contract

## Status

- planned for Wave 10

## Purpose

Replace broad multi-platform completion claims with a per-surface desktop/Android/browser support
contract.

## Source Of Truth

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- archived Wave 9 route/capability and migration plans for continuity only

## Target Location

- active planning docs
- active top-level app documentation
- any platform support matrix maintained for `packages/vibe-app-tauri`

## Responsibilities

- define per-surface platform support classes
- define retained browser-export contract
- define Android support boundaries
- define desktop support boundaries where desktop remains the richest runtime
- ensure the formal default shell (`src/AppV2.tsx`) matches the active platform contract instead of
  inheriting behavior only from the legacy shell
- enforce explicit shell structure so platform support is expressed through named route/adapter seams
  instead of single-file implicit behavior

## Non-Goals

- reactivating iOS in Wave 10
- claiming parity where the current app only has shell-level rendering support

## Dependencies

- `settings-and-connection-center`
- `inbox-and-notification-closure`
- `remote-operations-surfaces`

## Implementation Steps

1. Build a Wave 10 support matrix for visible surfaces.
2. Classify each surface per platform as complete, limited, handoff-only, read-only, or
   unsupported.
3. Rewrite active platform wording to match the matrix.
4. Define browser-export support as a retained contract, not a generic full-web claim unless the
   evidence supports it.
5. Enforce parity for user-visible failure handling so Android surfaces surface async backend/native
   failures with the same clarity as desktop instead of failing silently.
6. Keep default-shell route mapping explicit by route key or another documented surface contract;
   do not rely on broad section-based fallbacks.
7. Keep feed/session/backend projections in adapter modules so desktop/Android/browser capability
   differences can be reviewed directly at the seam.

## Edge Cases And Failure Modes

- desktop assumptions leaking into Android/browser claims
- browser-export availability being mistaken for fully interactive parity
- Android shell coverage being overstated as route-complete product parity
- async actions writing only desktop-visible error state while Android routes show no actionable
  feedback
- defaulting to `AppV2` while key routes still depend on legacy-shell-only state shape or placeholder
  actions

## Tests

- platform route smoke coverage review
- support-matrix documentation review against actual code paths

## Acceptance Criteria

- desktop/Android/browser support claims are explicit by surface
- active docs no longer use vague "multi-platform complete" wording
- browser-export support is described in a bounded, testable way
- desktop and Android shells both expose visible error feedback for backend/native action failures
- the default `AppV2` shell is the artifact reviewed for platform parity, not only `src/App.tsx`
- AppV2 structure makes platform support reviewable by seam: shell orchestration, route containers,
  and adapter modules are separate and map to the documented contract

## Locked Decisions

- platform truth is more important than symmetric messaging
- browser export is not automatically equivalent to full browser product support
