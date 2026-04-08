# `vibe-app-tauri` Capability Matrix

## Archival Status

This file is historical Wave 8 desktop-only planning material.

Do not use it as active execution authority for Wave 9. Use the active Wave 9 planning set instead:

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/*`

## Purpose

Record the historical desktop capability replacements defined for the Wave 8 `vibe-app-tauri` rewrite without silently
depending on Expo/mobile runtime behavior.

This file is also the source of truth for which adapter work is auth-critical and must land before
`auth-and-session-state`.

## Status

- state: `historical Wave 8 capability reference`
- update rule: use this file as historical desktop-only reference; active full-app capability ownership now belongs to the Wave 9 planning set
- current execution note:
  - desktop file load/save dialogs now back manual restore import plus text/artifact export flows
  - desktop notifications now back restore/link success and artifact mutation feedback paths
  - broader promotion-scope sign-off still remains pending until cross-platform review is complete

## Capability Classes

- `P0 auth-critical`: required before desktop auth/account restore can be considered viable
- `P0 session-critical`: required for the first usable desktop session slice
- `P1 promotion-critical`: required before promotion to the then-default desktop path
- `deferred`: out of phase-one scope unless explicitly activated

## Locked Phase-One Callback Strategy

During the historical coexistence period with the shipping `packages/vibe-app` desktop path:

- `packages/vibe-app-tauri` must not take default ownership of the production `vibe:///` deep-link
  scheme
- desktop auth/connect callbacks for `vibe-app-tauri` use a localhost loopback callback owned by
  the running desktop app process
- phase-one desktop backend connections must prefer `https`; plain `http` is reserved for
  `127.0.0.1` / `localhost` development and loopback flows unless a later plan explicitly widens
  that rule
- the callback listener must bind only to `127.0.0.1` on an ephemeral port chosen at auth start;
  it must never bind to wildcard interfaces such as `0.0.0.0`
- each auth attempt must mint a strong per-attempt `state` value and reject callbacks whose
  `state`, age, or request shape do not match the active attempt exactly
- the listener is one-shot and short-lived: it accepts only the active in-flight attempt, tears
  down after success/failure/timeout, and must not remain as a long-running background listener
- if listener startup or port allocation fails, the auth flow fails visibly rather than silently
  degrading to a browser-only completion path
- auth listener ownership is per initiating app process instance; concurrent auth attempts inside
  the same process must serialize or cancel explicitly, and a different process instance must not
  be allowed to attach to an already-active attempt
- if a temporary compatibility deep-link is needed for local development, it must stay
  non-default, undocumented as the primary production path, and recorded in the coexistence matrix
- browser-only completion without a callback handoff is not sufficient for the phase-one auth flow;
  the app must regain authenticated state directly after the external browser step completes

## Matrix

| Capability | Class | Initial rule | First owning module | Notes |
| --- | --- | --- | --- | --- |
| secure credential storage | `P0 auth-critical` | must exist before desktop login/account restore is accepted | `desktop-platform-adapters` | consumed by `auth-and-session-state` |
| external browser launch for auth/connect | `P0 auth-critical` | must support the same auth semantics as the current desktop path and return control to the running app | `desktop-platform-adapters` | browser-only completion without app handoff does not satisfy P0 auth viability |
| OAuth / callback return handling | `P0 auth-critical` | phase-one callback handling uses a localhost loopback callback and must not steal production callback ownership from `packages/vibe-app` while both apps coexist | `desktop-platform-adapters` | coordinated with the coexistence matrix |
| localhost callback listener security and lifecycle | `P0 auth-critical` | bind only to `127.0.0.1`, validate per-attempt `state`, allow only one active attempt per process instance, and tear down on success/failure/timeout | `desktop-platform-adapters`, `auth-and-session-state` | do not leave listener behavior implicit or implementation-defined |
| deep-link / callback route ownership | `P0 auth-critical` | `vibe-app-tauri` stays off the default production `vibe:///` route during coexistence; any temporary compatibility route must remain non-default until promotion | `desktop-platform-adapters` | see coexistence rules |
| session-safe clipboard integration | `P0 session-critical` | required where current desktop behavior depends on copy/paste flows | `desktop-platform-adapters` | first usable slice should not regress expected clipboard flows |
| file open/save dialogs | `P1 promotion-critical` | required where artifacts or file actions are desktop-visible today | `desktop-platform-adapters` | may harden after the first usable slice |
| notifications | `P1 promotion-critical` | required only if current desktop flows materially depend on them | `desktop-platform-adapters` | document explicit deferral if omitted |
| updater channel integration | `P1 promotion-critical` | must remain separate from the shipping desktop path before promotion | `release-and-promotion` | coordinates with coexistence rules |
| bundle/startup packaging hooks | `P1 promotion-critical` | must validate reliably on Linux, macOS, and Windows before promotion | `bootstrap-and-package`, `release-and-promotion` | packaging is not a substitute for parity |
| camera/media capture | `deferred` | do not implement without a concrete desktop requirement | `desktop-platform-adapters` | mobile-only by default |
| sensor/location/device-specific APIs | `deferred` | keep out of scope unless a plan update activates them | `desktop-platform-adapters` | mobile-only by default |

## Change Rule

- if a capability moves between classes, update this file and any affected module plan before code
  changes continue
