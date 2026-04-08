# `vibe-app-tauri` Coexistence Matrix

## Purpose

Lock the side-by-side operating rules for the shipping `packages/vibe-app` desktop path and the
parallel `packages/vibe-app-tauri` desktop rewrite.

This file exists so bootstrap, auth, packaging, and promotion work do not invent coexistence rules
module by module.

## Status

- state: `planning baseline`
- update rule: revise this file before either desktop package changes default ownership of a shared
  public surface

## Baseline Rules

- `packages/vibe-app` remains the default production desktop path until the promotion plan says
  otherwise
- `packages/vibe-app-tauri` may ship in parallel only with clearly distinguishable package,
  artifact, and updater identities
- shared backend contracts stay the same; coexistence rules apply only to client packaging,
  callback ownership, and local machine state
- phase-one auth callback ownership for `vibe-app-tauri` uses a localhost loopback callback owned
  by the running app process; it does not claim the default production `vibe:///` route while
  `packages/vibe-app` remains the shipping desktop path

## Locked Loopback Ownership Rules

- the loopback listener binds only to `127.0.0.1` and uses an ephemeral per-attempt port owned by
  the initiating desktop process
- `vibe-app-tauri` must not install a shared or always-on background localhost listener just to
  simplify auth handoff
- each callback attempt is correlated with a strong per-attempt `state` value; stale, replayed, or
  mismatched callbacks are rejected
- only the process instance that initiated the auth flow may complete that flow; a second process
  instance must not attach to the first instance's active callback attempt
- if a second auth flow is started inside the same process instance, the implementation must
  explicitly serialize or cancel the previous attempt rather than letting both listeners race
- listener startup failures, port-allocation failures, and timeout expiry are surfaced as explicit
  auth failures and do not fall back silently to browser-only completion

## Matrix

| Concern | Current default owner | Parallel `vibe-app-tauri` rule |
| --- | --- | --- |
| repository package path | `packages/vibe-app` | `packages/vibe-app-tauri` is a separate package and must not replace the existing path |
| bundle identifier / app id | `packages/vibe-app` | use a distinct next-iteration identifier until promotion; do not ship two desktop apps with the same production app id |
| app display naming in public release channels | `packages/vibe-app` | keep the next-iteration package distinguishable and non-default until promotion naming is approved |
| deep-link scheme `vibe:///` | `packages/vibe-app` | production default ownership stays with `packages/vibe-app` until promotion; `vibe-app-tauri` phase-one auth uses a localhost loopback callback instead of claiming the default route |
| localhost auth callback ownership | no separate owner needed for the shipping path today | `vibe-app-tauri` may open a temporary localhost listener for auth/connect completion, but it must be package-local, ephemeral, and not change production deep-link ownership |
| auth attempt ownership and multi-instance behavior | `packages/vibe-app` has no special coexistence rule today | a callback may satisfy only the `vibe-app-tauri` process instance that launched it; same-process concurrent attempts must serialize or cancel explicitly |
| updater channel | `packages/vibe-app` | use a distinct non-default channel until promotion |
| release artifact naming | `packages/vibe-app` | artifact names must clearly distinguish the next-iteration desktop package from the shipping path |
| shared account/session credentials | compatible storage may be reused only if format and callback semantics remain compatible | document the exact reuse rule in auth implementation; do not assume implicit sharing beyond what the auth module validates |
| local UI state, caches, window layout, and desktop-only preferences | `packages/vibe-app` owns its current local state | `vibe-app-tauri` should use package-local subdirectories or keys by default so it can coexist safely |
| logs, temporary files, and desktop-only downloads | `packages/vibe-app` owns its current local behavior | `vibe-app-tauri` should isolate package-local outputs unless an explicit shared path is required and documented |
| CI and release lanes | `packages/vibe-app` retains current production lane | `vibe-app-tauri` must add separate validation and release jobs until promotion; manual packaging workflows may run both desktop lanes in parallel only if artifacts, job names, and publish triggers stay clearly separated |
| default desktop entrypoint in docs and helper scripts | `packages/vibe-app` | keep docs and scripts pointing at the shipping app until promotion sign-off lands |

## Promotion Constraint

- no row in this matrix may transfer default ownership from `packages/vibe-app` to
  `packages/vibe-app-tauri` without a documented promotion/deprecation plan update
