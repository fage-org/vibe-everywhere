# Module Plan: vibe-app-tauri/auth-and-session-state

## Purpose

Stand up desktop auth, account restore, session state bootstrap, and backend connectivity in the
new package.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-coexistence-matrix.md`
- `packages/vibe-app/sources/auth/**`
- `packages/vibe-app/sources/sync/**`
- desktop auth/session flows in current `packages/vibe-app`

## Target Location

- `packages/vibe-app-tauri`
- auth state
- account/session bootstrap
- backend client adapters

## Responsibilities

- desktop login and account restore flow
- credential persistence via desktop-safe storage
- initial session/account state loading
- real backend chain wiring for core desktop flows

## Non-Goals

- full UI parity for every screen
- mobile-only auth/device flows

## Dependencies

- `core-logic-extraction`
- `desktop-shell-and-routing`
- `desktop-platform-adapters`

## Implementation Steps

1. Consume the auth-critical adapter layer defined in the capability and coexistence documents.
2. Implement desktop-safe credential persistence.
3. Implement auth/connect completion against the locked localhost loopback callback strategy rather
   than taking over the shipping `vibe:///` production path.
4. Enforce loopback listener constraints explicitly:
   - bind to `127.0.0.1` only
   - require exact per-attempt `state` validation
   - allow only one active auth attempt per process instance
   - tear down listeners after success, failure, or timeout
5. Port auth/account restore flow semantics.
6. Connect session/account bootstrap state to the real Vibe backend.
7. Validate auth/session lifecycle against the current app behavior.

## Edge Cases And Failure Modes

- secure storage mismatch on desktop
- OAuth/external-browser callback gaps
- localhost port-allocation failure or listener startup failure
- stale, replayed, or wrong-instance callback delivery
- concurrent auth attempts racing inside the same desktop process
- state bootstrap drift from the current app
- reconnect/resume logic depending on hidden Expo APIs

## Tests

- auth state smoke tests
- credential persistence tests
- callback state / timeout / listener-teardown tests
- one real desktop auth + session bootstrap chain

## Acceptance Criteria

- desktop app can authenticate, restore account state, and load session/account state from the
  real backend

## Locked Decisions

- desktop auth semantics must match current app behavior first
- phase-one desktop auth callbacks return through a localhost loopback flow while `packages/vibe-app`
  retains default ownership of `vibe:///`
- localhost callback listeners bind only to `127.0.0.1`, remain one-shot and short-lived, and may
  satisfy only the active auth attempt of the initiating process instance
- platform-specific credential handling lives behind desktop adapters, not inside UI code
