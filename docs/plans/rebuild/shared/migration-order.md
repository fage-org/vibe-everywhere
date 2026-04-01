# Migration Order

This file is the stage-level delivery order only.

For the detailed module-by-module execution sequence, use `docs/plans/rebuild/execution-plan.md`.
For direct AI dispatch groupings, use `docs/plans/rebuild/execution-batches.md`.

## Fixed Delivery Sequence

1. `vibe-wire`
2. `vibe-server`
3. `vibe-agent`
4. `vibe-cli`
5. `vibe-app`
6. `vibe-app-logs`

## Stage Definitions

### Stage 1: `vibe-wire`

- inputs:
  - source crosswalk
  - naming rules
  - canonical data model
  - protocol specs
- outputs:
  - stable Rust wire crate
  - compatibility vectors
- done when:
  - all wire module acceptance criteria pass

### Stage 2: `vibe-server`

- inputs:
  - stable `vibe-wire`
  - auth/api/rpc specs
- outputs:
  - minimum auth/session/machine/update backend
  - deterministic session and machine presence handling
  - machine lifecycle and daemon-state update path required by remote control
- done when:
  - `vibe-agent` can be developed against a real server

### Stage 3: `vibe-agent`

- inputs:
  - stable server spine
  - locked auth and session protocols
- outputs:
  - remote-control client with auth, list, create, send, history, stop, wait
- done when:
  - end-to-end remote session control works

### Stage 4: `vibe-cli`

- inputs:
  - stable server and wire layers
  - agent/runtime mapping plans
- outputs:
  - local runtime, provider integrations, daemon, sandbox, and persistence
- done when:
  - local providers can run and stream compatible updates

### Stage 5: `vibe-app`

- inputs:
  - stable wire and minimum server/client contracts
- outputs:
  - imported app adapted to Vibe naming, endpoints, and protocols
- done when:
  - app works with Rust backend path without Happy branding leakage in public surfaces

### Stage 6: `vibe-app-logs`

- inputs:
  - stable app/server operational model
- outputs:
  - sidecar log service parity
- done when:
  - log-sidecar behaviors required by app tooling are implemented

## Gating Rule

If a downstream stage discovers an upstream contract gap, stop implementation and update the
upstream shared or project plan before continuing.
