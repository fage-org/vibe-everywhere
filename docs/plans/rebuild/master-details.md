# Wave 10 Master Details

## Goal

Turn the current `vibe-app-tauri` package from a Wave 9 ownership-complete replacement into a Wave
10 product-contract-complete application.

In Wave 10, "complete" means:

- the route exists
- the route is wired to real state or a deliberate no-op contract
- the route's platform scope is explicit
- the route's behavior is documented for both internal and customer use
- the validation rules measure the real contract instead of route presence alone

## Wave 10 Project Boundary

Wave 10 applies to:

- `packages/vibe-app-tauri`
- active planning documents under `docs/plans/rebuild/`
- active validation and top-level documentation that describe app capability status

Wave 10 does not change:

- upstream Rust service boundaries
- shared protocol ownership in `crates/vibe-wire`
- archived Wave 9 module history
- the decision that `packages/vibe-app` remains deprecated

## Implementation Principles

- Product-contract first: document the supported behavior before changing code.
- No silent upgrades: a surface is not "done" because it renders.
- Platform truth over symmetry: desktop, Android, and browser support may differ, but the
  differences must be explicit.
- Handoff is not integration: command-copy or external-doc pages do not count as full product
  integration.
- Decision traceability: if a surface is kept deferred, hidden, downgraded, or customer-invisible,
  that decision must be written into planning docs.

## Global Dependency Graph

1. `shared/ui-visual-parity.md`
2. `shared/data-model.md`
3. `shared/protocol-session.md`
4. `shared/protocol-api-rpc.md`
5. `projects/vibe-app-tauri.md`
6. `modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
7. `modules/vibe-app-tauri/settings-and-connection-center.md`
8. `modules/vibe-app-tauri/inbox-and-notification-closure.md`
9. `modules/vibe-app-tauri/remote-operations-surfaces.md`
10. `modules/vibe-app-tauri/platform-parity-and-browser-contract.md`
11. `modules/vibe-app-tauri/social-and-developer-surface-disposition.md`

Wave 10 modules may overlap in product area, but the capability contract module must land first so
all later modules inherit the same definition of done.

## Cross-Cutting Concerns

- customer-safe wording for capabilities
- platform-specific capability downgrades
- route visibility versus route support
- connection lifecycle semantics
- notification source taxonomy
- remote-operations object model: server, machine, session, terminal
- archived or deferred capability wording in demo and mock surfaces

## Acceptance Gates

- `G8` Wave 10 planning tree exists and is the active source of truth
- `G9` customer-visible capability claims are backed by code, tests, and platform scope notes
- `G10` settings, notifications, and remote-operations pages expose clear product contracts instead
  of mixed placeholder and real behavior
- `G11` desktop/Android/browser support matrix is explicit and does not overstate parity
- `G12` social and developer-only surfaces are either productized or formally hidden/deferred
- `G13` active docs and validation commands reflect the Wave 10 completion standard

## Wave 10 Exit Criteria

Wave 10 is complete only when:

- the active app-facing routes that remain visible to users can be classified as implemented,
  deliberately limited, or explicitly unsupported
- no active top-level document overstates app completion relative to code
- all customer-facing app claims can be reproduced from active planning docs without extra chat
  context
- remaining deferred routes and helper surfaces are visibly marked or removed from customer-facing
  claims

## Change Control Rules

- Do not implement from this file directly; use module plans.
- Do not mark a Wave 10 surface complete until the module acceptance criteria and validation rules
  both reflect product-level closure.
- If a Wave 10 change reclassifies a route or capability, update the capability-contract module and
  project plan before code.
