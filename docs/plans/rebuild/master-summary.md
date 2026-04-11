# Wave 10 Master Summary

## Objective

Use Wave 10 to convert the remaining `vibe-app-tauri` partials, retained shells, and deferred
product surfaces into an evidence-backed product contract.

Wave 9 proved route coverage, package ownership, and the default-path switch. Wave 10 does not
re-open the rebuild foundation. It narrows the gap between:

- code that exists
- routes that render
- capabilities that are actually product-complete
- capabilities that are safe to describe to customers

The Wave 10 target is a smaller but more defensible product statement, not a broader surface area
with unclear completion status.

## Current Repository Shape

```text
crates/
  vibe-wire/
  vibe-server/
  vibe-agent/
  vibe-cli/
  vibe-app-logs/
packages/
  vibe-app/         # deprecated historical reference only
  vibe-app-tauri/   # active app package and Wave 10 planning target
docs/plans/rebuild/
scripts/
```

## Current Phase

- phase: `Wave 10` planning active
- implementation status: Wave 9 is complete and archived; Wave 10 is plan-first and has not started
  implementation
- immediate goal: define the active Wave 10 execution set for the unfinished or over-claimed
  `vibe-app-tauri` product surfaces before any new code changes begin
- authoritative execution sequence: `docs/plans/rebuild/execution-plan.md`
- AI dispatch batches: `docs/plans/rebuild/execution-batches.md`

## Wave 10 Scope

Wave 10 is limited to `packages/vibe-app-tauri` and the planning/docs surface that defines how its
capabilities are represented.

Wave 10 focuses on:

- settings pages that mix real state with placeholder or handoff-only behavior
- inbox and notification behavior that is not yet a clear product contract
- terminal, server, machine, and remote-operations pages that need one coherent workflow
- desktop/Android/browser parity claims that currently overstate what is actually complete
- social and developer-only surfaces that need an explicit product decision instead of implicit
  drift
- validation and customer-facing capability language that must match code reality

Wave 10 explicitly does **not** reopen:

- `vibe-wire`
- `vibe-server`
- `vibe-agent`
- `vibe-cli`
- `vibe-app-logs`
- archived Wave 9 promotion closeout rules

## Milestones

1. `M10.0 - Wave 10 Planning Baseline`
   - create the active Wave 10 planning set
   - classify remaining app gaps by product impact
   - define the active acceptance gates
2. `M10.1 - Product Contract Correction`
   - stop treating route presence as feature completion
   - define what is customer-safe to claim
3. `M10.2 - Settings, Notifications, and Remote Operations`
   - convert partial settings and remote-helper pages into coherent product flows
4. `M10.3 - Platform Contract And Surface Disposition`
   - formalize desktop/Android/browser support
   - decide social and developer-route disposition
5. `M10.4 - Validation And Documentation Reset`
   - align validation, status, and customer language with Wave 10 completion rules

## Critical Risks

- route-level coverage continuing to be mistaken for product-level completion
- customer-facing claims outrunning code reality
- desktop/Android/browser support being described at one common denominator that is not true on any
  single platform
- helper or handoff pages being presented as finished integrations
- deferred social/dev surfaces staying visible long enough to be misread as active commitments

## Active Wave 10 Planning Inputs

- `docs/plans/rebuild/master-details.md`
- `docs/plans/rebuild/execution-plan.md`
- `docs/plans/rebuild/execution-batches.md`
- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/settings-and-connection-center.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/inbox-and-notification-closure.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/remote-operations-surfaces.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/platform-parity-and-browser-contract.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/social-and-developer-surface-disposition.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/validation-and-customer-capability-contract.md`

## Recommended First Modules

1. `modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
2. `modules/vibe-app-tauri/settings-and-connection-center.md`
3. `modules/vibe-app-tauri/inbox-and-notification-closure.md`

Wave 10 should not start from UI expansion. It should start by tightening the completion contract,
then use that contract to drive the remaining app modules.
