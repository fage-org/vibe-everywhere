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

## Edge Cases And Failure Modes

- desktop assumptions leaking into Android/browser claims
- browser-export availability being mistaken for fully interactive parity
- Android shell coverage being overstated as route-complete product parity
- async actions writing only desktop-visible error state while Android routes show no actionable
  feedback

## Tests

- platform route smoke coverage review
- support-matrix documentation review against actual code paths

## Acceptance Criteria

- desktop/Android/browser support claims are explicit by surface
- active docs no longer use vague "multi-platform complete" wording
- browser-export support is described in a bounded, testable way
- desktop and Android shells both expose visible error feedback for backend/native action failures

## Locked Decisions

- platform truth is more important than symmetric messaging
- browser export is not automatically equivalent to full browser product support
