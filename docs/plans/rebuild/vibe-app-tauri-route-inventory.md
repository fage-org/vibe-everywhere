# `vibe-app-tauri` Route Inventory

## Purpose

Freeze the desktop-visible route and surface parity scope before the new route tree is implemented.

This file is the planning source of truth for which routes and shell surfaces are required in the
first usable slice, which are required before promotion, and which are deferred.

## Status

- state: `planning baseline`
- update rule: revise this file before adding, removing, or deferring a desktop-visible route in
  `vibe-app-tauri`

## Shell Invariants

The new desktop shell must preserve these baseline expectations unless a plan update records an
exception:

- recognizable header/sidebar/main-panel structure
- desktop-accessible entry points for account/settings and session flows
- modal and overlay semantics that preserve focus and action ownership
- keyboard navigation and focus order that remain acceptable for desktop use
- session resume and active-state affordances in the same general locations users expect today
- every currently desktop-visible route must be classified here as `P0`, `P1`, `P2`, or
  `deferred`; do not rely on implied grouping alone

## Inventory

| Route or surface | Promotion class | First owning module | Notes |
| --- | --- | --- | --- |
| shell chrome: header, sidebar, main panel, modal/overlay/focus behavior | `P0` | `desktop-shell-and-routing` | must exist before deeper feature migration starts |
| `/(app)/index` shell landing / default desktop entry route | `P0` | `desktop-shell-and-routing` | preserve current desktop entry behavior and avoid dead-end placeholder routing |
| `/(app)/restore/index` and `/(app)/restore/manual` | `P0` | `auth-and-session-state` | semantic parity is required for the first usable auth/account restore slice |
| auth/login/account restore entry flows | `P0` | `auth-and-session-state` | includes real callback-driven completion, not browser-only completion |
| `/(app)/inbox/index` | `P0` | `session-ui-parity` | primary session list route for the first usable slice |
| `/(app)/new/index` | `P0` | `session-ui-parity` | session creation entry point remains part of the first usable desktop session path |
| `/(app)/session/[id]` | `P0` | `session-ui-parity` | includes timeline shell, composer, and primary state loading |
| `/(app)/session/recent` | `P0` | `session-ui-parity` | preserve active-session resume affordance and route entry point |
| core account/settings entry via `/(app)/settings/index` | `P0` | `desktop-shell-and-routing`, `auth-and-session-state` | users must be able to reach settings/account entry points and return to the main shell |
| `/(app)/session/[id]/file`, `/(app)/session/[id]/files`, and `/(app)/session/[id]/info` | `P1` | `session-ui-parity` | promotion-scope detail surfaces tied to session rendering/file inspection; file detail must remain deep-linkable with explicit file selection in route state |
| `/(app)/artifacts/index`, `/(app)/artifacts/new`, `/(app)/artifacts/[id]`, and `/(app)/artifacts/edit/[id]` | `P1` | `secondary-surfaces` | required before promotion |
| `/(app)/settings/account`, `/(app)/settings/appearance`, `/(app)/settings/features`, `/(app)/settings/language`, `/(app)/settings/usage`, `/(app)/settings/voice`, and `/(app)/settings/voice/language` | `P1` | `secondary-surfaces` | detailed settings/account flows required before promotion |
| `/(app)/settings/connect/claude` and other retained connect/vendor flows | `P1` | `secondary-surfaces` | implement only the routes with real desktop value, but classify them explicitly here |
| `/(app)/user/[id]` | `P1` | `secondary-surfaces` | profile/detail route needed for promotion if still desktop-visible |
| `/(app)/changelog` | `P1` | `secondary-surfaces` | required before promotion if still desktop-visible |
| `/(app)/terminal/index` and `/(app)/terminal/connect` | `P1` | `secondary-surfaces` | terminal-link/connect utility flow remains explicit rather than implicit under “connect” |
| `/(app)/server` | `P1` | `secondary-surfaces` | self-hosted/server-config surface should not disappear silently if desktop users rely on it |
| `/(app)/machine/[id]` | `P1` | `secondary-surfaces` | machine detail/remote-control utility surface remains in parity scope unless explicitly deferred later |
| `/(app)/text-selection` | `P1` | `secondary-surfaces` | utility route that supports primary copy/export text workflows on desktop |
| `/(app)/friends/index` and `/(app)/friends/search` | `P2` | `secondary-surfaces` | confirm desktop value before implementation |
| `/(app)/dev/**` | `P2` | `secondary-surfaces` | review route-by-route and either port or defer explicitly before promotion |

## Rendering-Specific Parity Notes

The following are treated as route-adjacent parity requirements even when implemented as reusable
components rather than standalone pages:

- message timeline rendering
- markdown rendering
- diff rendering
- file rendering
- tool rendering

These surfaces belong to `session-ui-parity` for the first usable session slice and become
promotion-blocking when the parity checklist marks them as `P1`.

## Change Rule

- if a route or desktop-visible surface is omitted, mark it `deferred` here and in the parity
  checklist before implementation continues
