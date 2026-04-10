# Project Plan: vibe-app-logs

## Purpose

`vibe-app-logs` is the Rust sidecar replacing `happy-app-logs`. It is intentionally deferred until
the main backend and app path are stable.

Wave 7 is now active because the imported app still contains a remote log-server workflow used by
the developer screen and console-capture path.

## Happy Source

- primary source: `packages/happy-app-logs`

## Target Layout

- crate: `crates/vibe-app-logs`
- expected modules:
  - `server`
  - `config`
  - `main`

## Public Interfaces

- binary: `vibe-app-logs`
- local or service-facing log ingestion endpoints required by the app ecosystem
- root helper command: `yarn app-logs`

## Internal Module Map

- `server/config/main`: minimal sidecar bootstrap grouped under one module plan because the Happy
  source tree is only one runtime file plus package metadata

## Wave 7 Feature Inventory

- HTTP sidecar listening on the app-facing remote log port (`8787` by default)
- preferred port override via `VIBE_APP_LOGS_PORT` with legacy `PORT` fallback compatibility
- `POST /logs` ingestion endpoint with permissive CORS for the imported app web/native dev flows
- file sink under `~/.vibe/app-logs/` or `VIBE_HOME_DIR/app-logs/`
- mirrored stdout logging for local developer visibility
- root helper entrypoint so app copy can truthfully tell developers to run `yarn app-logs`

## Implementation Order

1. log server bootstrap and integration wiring

## Compatibility Requirements

- only implement behavior needed by imported app tooling
- avoid growing this subsystem before the main path is stable

## Testing Strategy

- smoke tests for startup and ingestion
- app-facing integration tests if the imported app depends on it

## Acceptance Criteria

- app tooling requirements that depend on the sidecar are satisfied

## Deferred Items

- unrelated observability expansion
