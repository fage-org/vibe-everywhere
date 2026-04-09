# Rebuild Plan Index

## Active Track

- active planning root: `docs/plans/rebuild/`
- planning index: `docs/plans/rebuild/README.md`
- master summary: `docs/plans/rebuild/master-summary.md`
- master details: `docs/plans/rebuild/master-details.md`
- execution order: `docs/plans/rebuild/execution-plan.md`
- AI batch plan: `docs/plans/rebuild/execution-batches.md`

## Current Phase

- phase: the original Happy-aligned rebuild baseline is complete through Wave 7 (`vibe-wire`, `vibe-server`, `vibe-agent`, `vibe-cli`, `vibe-app`, and `vibe-app-logs` are implemented and validated); Wave 8 is closed as the historical desktop-preview baseline for `packages/vibe-app-tauri`; Wave 9 has recorded the default-owner switch to `packages/vibe-app-tauri` for desktop, Android APK, and retained web/export ownership while `packages/vibe-app` stays deprecated from active CI and release ownership, but the final promotion evidence gate remains open until the promotion baseline artifact is fully signed off
- next milestone: close the remaining Wave 9 promotion evidence gate, keep the switched default package stable, honor the legacy retention window for `packages/vibe-app`, and only consider archival/removal through a later explicit plan update after the documented retirement gate closes

## Rule

Update the relevant rebuild plan files before implementing or changing any subsystem work.
