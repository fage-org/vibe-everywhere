# Rebuild Plan Index

## Active Track

- active planning root: `docs/plans/rebuild/`
- planning index: `docs/plans/rebuild/README.md`
- master summary: `docs/plans/rebuild/master-summary.md`
- master details: `docs/plans/rebuild/master-details.md`
- execution order: `docs/plans/rebuild/execution-plan.md`
- AI batch plan: `docs/plans/rebuild/execution-batches.md`

## Current Phase

- phase: the original Happy-aligned rebuild baseline is complete through Wave 7 (`vibe-wire`, `vibe-server`, `vibe-agent`, `vibe-cli`, `vibe-app`, and `vibe-app-logs` are implemented and validated); Wave 8 is closed as the historical desktop-preview baseline for `packages/vibe-app-tauri`; the active planning milestone is Wave 9, which turns `packages/vibe-app-tauri` into the active Wave 9 replacement package for `packages/vibe-app` across desktop, iOS, Android, and retained web/export ownership while keeping `packages/vibe-app` deprecated from active CI and release ownership
- next milestone: execute the direct `/root/happy/packages/happy-app`-aligned migration into `packages/vibe-app-tauri`, keep `packages/vibe-app` only as a deprecated legacy reference when Happy cannot answer a Vibe-specific continuity question, and keep the old `vibe-app` pipelines disabled

## Rule

Update the relevant rebuild plan files before implementing or changing any subsystem work.
