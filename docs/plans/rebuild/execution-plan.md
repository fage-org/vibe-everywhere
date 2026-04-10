# Rebuild Execution Plan

## Purpose

This file is the authoritative execution order for the Happy-aligned rebuild. Unlike
`migration-order.md`, which is stage-level only, this file defines the module-level sequencing that
AI implementation tasks should follow.

The AI-dispatch companion file is `execution-batches.md`.

Use this file when deciding:

- which module to implement next
- which modules may run in parallel
- which modules must wait for upstream gates
- which modules need multi-pass implementation instead of a one-shot rewrite

## Execution Rules

- Dispatch one implementation task against one module plan file.
- Do not start a downstream module until its listed prerequisites are actually implemented or
  explicitly stubbed with stable interfaces.
- If a module needs a shared contract that is still ambiguous, stop and update the shared plan
  first.
- Prefer one end-to-end usable slice over broad but incomplete surface area.
- Treat the order below as the default critical path. Parallel work is allowed only where this file
  explicitly marks it safe.
- When an ordered item is completed, mark it inline in this file as `[done]` before starting the
  next item. This completion-tracking rule applies to every subsequent wave as well.
- When a wave is retired as historical baseline material instead of being executed to the end, mark
  the wave status explicitly and relabel any leftover items as historical or moved work so they do
  not read like active backlog.

## Critical Path Summary

1. freeze shared contracts
2. implement `vibe-wire`
3. implement the minimum `vibe-server` spine
4. implement `vibe-agent` against the real server
5. expand `vibe-server` to cover app and CLI support surfaces
6. implement `vibe-cli`
7. import and adapt `vibe-app`
8. implement `vibe-app-logs` only if still needed
9. implement the parallel desktop rewrite in `packages/vibe-app-tauri` after the original rebuild
   baseline is complete
10. expand `packages/vibe-app-tauri` into the full cross-platform replacement for `packages/vibe-app`
    after the desktop preview foundation and planning reset are stable

## Completed Waves (archived)

Detailed module plans for completed waves have been moved to `archive/completed-modules/`.
Completed project plans have been moved to `archive/completed-projects/`.

| Wave | Description | Status |
|------|-------------|--------|
| 0 | Shared Planning Freeze | ✅ `[done]` — all shared contracts frozen |
| 1 | vibe-wire | ✅ `[done]` — `archive/completed-modules/vibe-wire/` |
| 2 | vibe-server Minimum Spine | ✅ `[done]` — `archive/completed-modules/vibe-server/` |
| 3 | vibe-agent | ✅ `[done]` — `archive/completed-modules/vibe-agent/` |
| 4 | vibe-server Support Surface Expansion | ✅ `[done]` — `archive/completed-modules/vibe-server/` |
| 5 | vibe-cli | ✅ `[done]` — `archive/completed-modules/vibe-cli/` |
| 6 | vibe-app | ✅ `[done]` — `archive/completed-modules/vibe-app/` |
| 7 | vibe-app-logs | ✅ `[done]` — `archive/completed-modules/vibe-app-logs/` |
| 8 | vibe-app-tauri Next Desktop Iteration | ⚰️ historical — `archive/wave8/` |

## Wave 9: `vibe-app-tauri` Active App Replacement

### Goal

Turn `packages/vibe-app-tauri` from the historical Wave 8 desktop-preview baseline into the active
Wave 9 replacement package for `packages/vibe-app` across desktop, Android, and retained static
browser export ownership.

### Planning Prerequisites

- `projects/vibe-app-tauri.md` records the Wave 9 project boundary
- `shared/ui-visual-parity.md` exists and is treated as a required cross-cutting rule for any
  user-visible app work
- `vibe-app-tauri-wave9-unified-replacement-plan.md` exists and records the Wave 9 batch layout
- `vibe-app-tauri-wave9-route-and-capability-matrix.md` exists and records route and capability
  priorities directly from `/root/happy/packages/happy-app`
- `vibe-app-tauri-wave9-migration-and-release-plan.md` exists and records release-owner switch and
  rollback rules
- any retained Wave 8 desktop-only planning files are treated as historical references rather than
  competing execution authority

### Order

1. `[done]` `modules/vibe-app-tauri/universal-bootstrap-and-runtime.md`
2. `[done]` `modules/vibe-app-tauri/desktop-shell-and-platform-parity.md` (B19)
3. `[done]` `modules/vibe-app-tauri/auth-and-identity-flows.md` (B19)
4. `[done]` `modules/vibe-app-tauri/mobile-shell-and-navigation.md` (B19–B20)
5. `[done]` `modules/vibe-app-tauri/mobile-native-capabilities.md` (B20)
6. `[done]` `modules/vibe-app-tauri/session-rendering-and-composer.md` (B21)
7. `[done]` `modules/vibe-app-tauri/session-runtime-and-storage.md` (B21)
8. `[done]` `modules/vibe-app-tauri/secondary-routes-and-social.md` (B22)
9. `[done]` `modules/vibe-app-tauri/secondary-surfaces.md` (B22)
10. `[done]` `modules/vibe-app-tauri/release-ota-and-store-migration.md` (B23)
11. `[done]` `modules/vibe-app-tauri/web-export-and-browser-runtime.md` (B24)
12. `[done]` `modules/vibe-app-tauri/universal-bootstrap-and-runtime.md` (B24)
13. `[done]` `modules/vibe-app-tauri/release-and-promotion.md` (B25)
14. 🔧 `modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md` (B26 — pending)

### Why This Order

- runtime bootstrap must exist before the new package can own Tauri desktop, Tauri mobile, and
  retained browser build ownership
- shared core should be extracted before route-level screen migration broadens
- mobile shell, retained static browser export, desktop shell parity, and identity flows define the
  first usable replacement slice
- Happy-aligned style correction starts with the first shell modules and remains in scope for every
  later user-visible Wave 9 module; it is not deferred to a final polish pass
- session runtime must exist before rendering-heavy parity work starts
- native capabilities should harden after the main session flow proves which platform seams are
  required in practice
- secondary surfaces come after the core session path is stable
- release migration must wait for route and capability parity to exist
- promotion and legacy deprecation are last because they depend on explicit rollback-safe release
  ownership

### Output

- `packages/vibe-app-tauri` can act as the active Wave 9 replacement package
- desktop and mobile shells share package-local core modules without protocol forks
- Happy-aligned UI and visual parity correction is treated as part of Wave 9 completion for shell,
  session, and promotion-critical secondary surfaces
- release, OTA, and store ownership can move to the new package by explicit promotion

### Gate To Finish

- `packages/vibe-app-tauri` is approved as the default app path, with hold/rollback and
  legacy-reference rules documented

## Dispatch Format For AI

When assigning work from this file, always provide:

1. the module plan path
2. the owning project plan path
3. the shared spec paths it depends on
4. the immediately previous module in the execution order
5. the gate that must remain true after the change

Example:

> Implement `docs/plans/rebuild/modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md`.
> Follow `docs/plans/rebuild/projects/vibe-app-tauri.md`,
> `docs/plans/rebuild/shared/data-model.md`,
> `docs/plans/rebuild/shared/protocol-session.md`, and
> `docs/plans/rebuild/shared/protocol-api-rpc.md`.