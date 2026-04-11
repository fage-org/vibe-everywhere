# Wave 10 Execution Plan

## Purpose

This file is the authoritative execution order for Wave 10.

Wave 10 is a product-contract correction wave. The goal is not to broaden the app first; the goal
is to make the active `vibe-app-tauri` surfaces defensible, classifiable, and safe to describe.

The AI-dispatch companion file is `execution-batches.md`.

## Execution Rules

- Dispatch one implementation task against one module plan file.
- Do not start a downstream module until its prerequisites are implemented or deliberately frozen in
  planning.
- Product wording, route visibility, and validation rules are execution dependencies, not post-work
  cleanups.
- Prefer closing one end-to-end product claim over spreading work across many surfaces.
- Mark a module `[done]` in this file only after its code, docs, and validation obligations are all
  satisfied.

## Completed Waves (archived)

| Wave | Description | Status |
|------|-------------|--------|
| 0 | Shared Planning Freeze | ✅ archived |
| 1 | vibe-wire | ✅ archived |
| 2 | vibe-server Minimum Spine | ✅ archived |
| 3 | vibe-agent | ✅ archived |
| 4 | vibe-server Support Surface Expansion | ✅ archived |
| 5 | vibe-cli | ✅ archived |
| 6 | vibe-app | ✅ archived |
| 7 | vibe-app-logs | ✅ archived |
| 8 | vibe-app-tauri desktop-preview baseline | ⚰️ historical |
| 9 | vibe-app-tauri active replacement ownership | ✅ archived |

## Wave 10: `vibe-app-tauri` Product Contract Closure

### Goal

Align the active app package's route behavior, customer claims, and validation rules so the
repository stops overstating completion.

### Planning Prerequisites

- `master-summary.md`
- `master-details.md`
- `projects/vibe-app-tauri.md`
- shared specs already frozen from prior waves

### Order

1. `modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
2. `modules/vibe-app-tauri/settings-and-connection-center.md`
3. `modules/vibe-app-tauri/inbox-and-notification-closure.md`
4. `modules/vibe-app-tauri/remote-operations-surfaces.md`
5. `modules/vibe-app-tauri/platform-parity-and-browser-contract.md`
6. `modules/vibe-app-tauri/social-and-developer-surface-disposition.md`

### Why This Order

- the capability contract must be locked before any app module can be classified correctly
- settings/connection pages are the most visible source of mixed real and handoff-only behavior
- inbox/notifications need their own product taxonomy before platform parity claims are rewritten
- remote-operations pages depend on the same classification work but need a separate workflow pass
- platform parity should be documented after the underlying surfaces are classified
- social and developer surfaces should be resolved last because they depend on the visibility rules
  set by all earlier modules

### Output

- a Wave 10 capability contract that matches code reality
- a cleaner app surface map for settings, notifications, and remote operations
- an explicit desktop/Android/browser support matrix
- a written disposition for social and developer-only surfaces

### Gate To Finish

- every active customer-facing app capability has an evidence-backed completion class and platform
  scope

## Dispatch Format For AI

When assigning work from this file, always provide:

1. the module plan path
2. the owning project plan path
3. the shared spec paths it depends on
4. the immediately previous module in the execution order
5. the Wave 10 gate that must remain true after the change
