# Module Plan: vibe-app-tauri/session-runtime-and-storage

## Purpose

Port the shared session bootstrap, storage, profile, and realtime runtime needed for the replacement
app to load real data.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `/root/happy/packages/happy-app/sources/sync/**`
- `/root/happy/packages/happy-app/sources/realtime/**`
- `/root/happy/packages/happy-app/sources/hooks/**`
- `/root/happy/packages/happy-app/sources/components/SessionsList.tsx`
- `/root/happy/packages/happy-app/sources/components/MainView.tsx`

## Target Location

- `packages/vibe-app-tauri/sources/shared/session/**`
- `packages/vibe-app-tauri/sources/shared/realtime/**`

## Responsibilities

- bootstrap account/profile/session state
- realtime subscription ownership
- local app state and settings persistence
- session inventory loading
- profile, machine, feed, and server-config supporting state as needed by `P0` and `P1` routes

## Non-Goals

- final rendering parity for timeline or composer
- store-release migration

## Dependencies

- `shared-core-from-happy`
- `auth-and-identity-flows`

## Implementation Steps

1. Port Happy sync and storage modules into the shared layer.
2. Port realtime session and account bootstrap logic.
3. Recreate the selectors and hooks needed by the session list and shell routes.
4. Validate reconnect, resume, and state-refresh behavior.
5. Keep wire parsing and reducer semantics aligned to shared crate contracts.

## Edge Cases And Failure Modes

- reducer behavior drifting from Happy state expectations
- realtime reconnect gaps
- hidden coupling to old local-setting keys or storage formats
- profile/bootstrap state loading in a different order than Happy expects

## Tests

- shared state unit tests
- realtime reconnect tests
- session inventory smoke tests
- profile/bootstrap chain validation against a real backend

## Acceptance Criteria

- desktop and mobile can load the core authenticated session state from the real backend
- state is stable enough for session rendering and secondary-route modules

## Locked Decisions

- shared runtime state lives below the UI shells
- storage and realtime changes must not fork protocol rules away from `vibe-wire`
- runtime/state work must preserve the Happy-aligned UI expectations defined in
  `docs/plans/rebuild/shared/ui-visual-parity.md` rather than forcing alternate shell structures
