# Module Plan: vibe-app-tauri/auth-and-identity-flows

## Purpose

Port create-account, QR/device-link, secret-key restore, credential persistence, and identity
bootstrap flows into `packages/vibe-app-tauri`.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `/root/happy/packages/happy-app/sources/auth/**`
- `/root/happy/packages/happy-app/sources/app/(app)/index.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/restore/index.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/restore/manual.tsx`
- `/root/happy/packages/happy-app/sources/components/qr/**`

## Target Location

- `packages/vibe-app-tauri/sources/shared/auth/**`
- mobile and desktop auth bootstrap surfaces

## Responsibilities

- create account
- device link / QR flow
- secret-key restore
- credential persistence and restore
- auth bootstrap into session/account state
- desktop/mobile callback ownership rules
- desktop localhost loopback callback security and lifecycle rules where desktop auth still depends on
  external-browser handoff

## Non-Goals

- full settings/profile parity
- release migration

## Dependencies

- `shared-core-from-happy`
- `mobile-shell-and-navigation`
- existing desktop auth module work where still relevant

## Implementation Steps

1. Port Happy auth helpers into the shared layer.
2. Recreate create-account semantics.
3. Recreate QR/device-link semantics with explicit mobile and desktop callback ownership rules.
4. Recreate secret-key restore semantics.
5. Implement credential persistence per platform behind explicit adapters.
6. Keep the desktop callback contract explicit while Wave 9 is active:
   - bind loopback listeners to `127.0.0.1` only
   - mint and validate a strong per-attempt `state`
   - keep listeners one-shot and short-lived
   - allow only one active auth attempt per desktop app process instance unless the plan changes
   - fail visibly on listener startup, timeout, or stale-callback cases rather than degrading silently
7. Validate restart and re-auth behavior across desktop and mobile.

## Edge Cases And Failure Modes

- credential persistence drift across platform backends
- QR flow drift from expected link semantics
- restore keys accepted or rejected differently from Happy
- preview and production deep-link ownership collisions
- desktop loopback listeners binding too broadly or surviving beyond the active auth attempt
- stale, replayed, or wrong-instance callbacks getting accepted

## Tests

- create-account tests
- QR/device-link flow tests
- secret-key restore tests
- credential restore smoke tests across app restart boundaries
- desktop callback state / timeout / listener-teardown tests
- desktop per-process auth-attempt ownership tests

## Acceptance Criteria

- users can create, link, restore, and resume accounts in `packages/vibe-app-tauri`
- auth semantics match Happy behavior closely enough for upgrade and rollback confidence
- desktop callback validation artifacts exist when loopback auth remains part of the active path

## Locked Decisions

- identity behavior follows Happy semantics first
- desktop and mobile callback strategies may differ internally, but not semantically
- desktop auth callbacks use a localhost loopback flow until a later Wave 9 plan update records a
  replacement strategy
- localhost callback listeners bind only to `127.0.0.1`, remain one-shot and short-lived, and must
  validate per-attempt `state`
