## Highlights

- Overlay bridge connections now fall back cleanly and recover automatically when the bridge
  becomes reachable again.
- Top-level README files are user-facing again, and developer/source-build entry points now live in
  `DEVELOPMENT.md`.
- Android CI and release jobs now restore Gradle dependency caches from repository-owned dependency
  inputs.

## Included Iterations And Remediations

- Remediation v4 R1: overlay bridge runtime fallback, timeout handling, and auto-recovery probes
- Remediation v4 R2: README user-facing rewrite and developer guide split
- Remediation v4 R3: CI and release cache optimization for Android/Gradle jobs

## Operator Notes

- Existing relay and agent deployment flows stay the same. The main operator-facing changes are
  clearer top-level docs and more resilient overlay fallback behavior in the relay.

## Validation

- `cargo fmt --all`
- `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --locked --workspace --all-targets -- --nocapture`
- `cd apps/vibe-app && npm run build`
- `bash -n scripts/install-relay.sh`
- `./scripts/render-release-notes.sh v0.0.0 >/dev/null`
- `./scripts/dual-process-smoke.sh relay_polling`
- `./scripts/dual-process-smoke.sh overlay`
- GitHub Actions monitoring remains pending until the next push/tag event.
