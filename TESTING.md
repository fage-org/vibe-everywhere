# Testing Guide

## Purpose

This file is the testing entry point for repository changes. It is intentionally reduced to the
minimum structure needed for later rewrite.

## Core Commands

```bash
cargo fmt --all --check
cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app
cargo test --locked --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm run build
./scripts/dual-process-smoke.sh relay_polling
./scripts/dual-process-smoke.sh overlay
```

## Minimum Checklist

- run the relevant Rust checks and tests
- build the frontend when touching `apps/vibe-app`
- run smoke tests when changing relay or agent control-plane behavior
- check GitHub Actions after push

## Release Validation

- keep version sources aligned across `Cargo.toml`, `apps/vibe-app/package.json`,
  `apps/vibe-app/package-lock.json`, and `apps/vibe-app/src-tauri/tauri.conf.json`
- after a workspace version bump, regenerate and commit `Cargo.lock` before relying on
  `cargo ... --locked` checks or pushing a release tag
- run `./scripts/verify-release-version.sh <tag>` before creating or updating a release tag
- make sure `docs/releases/<tag>.md` or `docs/releases/unreleased.md` exists before pushing the tag,
  because the `Release` workflow renders notes from those files

## Manual Regression Placeholder

The detailed manual regression checklist will be rewritten later. Add product-specific manual cases
here when the new documentation set is ready.
