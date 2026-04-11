# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`vibe-remote` is a Happy-aligned remote coding stack with Rust services/clients and a Tauri-based desktop/mobile app. It replaces the legacy `packages/vibe-app` with `packages/vibe-app-tauri` as the active release owner.

**Architecture:**
- `crates/vibe-wire`: Shared protocol and compatibility fixtures
- `crates/vibe-server`: HTTP + Socket.IO backend (Axum, in-memory stores by default)
- `crates/vibe-agent`: Remote-control CLI for machines and sessions
- `crates/vibe-cli`: Local runtime wrapper for AI agents (`claude`, `codex`, `gemini`, `openclaw`, `acp`)
- `crates/vibe-app-logs`: Optional remote app log receiver
- `packages/vibe-app-tauri`: Active desktop/Android/browser app (Tauri v2, React, Vite)
- `packages/vibe-app`: Deprecated legacy reference only

## Common Commands

### Rust Workspace
```bash
# Check and test entire workspace
cargo fmt --all --check
cargo check --workspace --locked
cargo test --workspace --locked

# Run specific crate tests
cargo test -p vibe-server
cargo test -p vibe-cli

# Run the server locally
export VIBE_MASTER_SECRET='replace-with-a-long-random-secret'
cargo run -p vibe-server -- --host 0.0.0.0 --port 3005

# Run CLI commands
cargo run -p vibe-cli -- auth login
cargo run -p vibe-cli -- codex run --tag demo --prompt "Summarize this repository"
cargo run -p vibe-cli -- daemon start

# Run agent commands
cargo run -p vibe-agent -- auth login
cargo run -p vibe-agent -- machines --active
cargo run -p vibe-agent -- spawn --machine <machine-id> --path ~/project --agent codex

# Run app log receiver
cargo run -p vibe-app-logs -- --host 0.0.0.0 --port 8787
```

### App (vibe-app-tauri)
```bash
# Install dependencies
yarn install --frozen-lockfile

# Development
yarn app                              # Start Tauri dev mode (desktop)
yarn workspace vibe-app-tauri tauri:dev

# Testing
yarn workspace vibe-app-tauri typecheck
yarn workspace vibe-app-tauri test    # Vitest
yarn workspace vibe-app-tauri tauri:test  # Rust tests

# Build
yarn workspace vibe-app-tauri build   # Desktop production build
yarn workspace vibe-app-tauri desktop:smoke  # Debug build, no bundle

# Mobile (Android)
yarn workspace vibe-app-tauri mobile:android:dev     # Dev mode
yarn workspace vibe-app-tauri mobile:android:preview:apk  # Debug APK
yarn workspace vibe-app-tauri mobile:android:production-candidate:apk  # Release APK

# Browser export
yarn workspace vibe-app-tauri browser:build
yarn workspace vibe-app-tauri browser:build:preview

# Full validation (CI-like)
yarn workspace vibe-app-tauri validate:promotion
```

### Validation Scripts
```bash
# Validate app promotion and release planning inputs
yarn --cwd scripts validate:vibe-app-tauri-promotion

# Validate release inputs
yarn app:release:validate

# Record metrics
yarn app:metrics

# Validate vibe-wire fixtures (run when vibe-wire changes)
cargo run --example export-fixtures -p vibe-wire
yarn --cwd scripts validate:vibe-wire-fixtures
```

## Environment Variables

**Server:**
- `VIBE_MASTER_SECRET` (required): Auth/crypto root secret
- `VIBE_SERVER_HOST` (default: 0.0.0.0)
- `VIBE_SERVER_PORT` (default: 3005)
- `VIBE_WEBAPP_URL` (default: https://app.vibe.engineering)

**Clients:**
- `VIBE_SERVER_URL`: API base URL (default: http://127.0.0.1:3005)
- `VIBE_WEBAPP_URL`: Web redirect base
- `VIBE_HOME_DIR`: Local state dir (default: ~/.vibe)

**Storage (optional S3):**
- `VIBE_S3_ENDPOINT`, `VIBE_S3_REGION`, `VIBE_S3_ACCESS_KEY`, `VIBE_S3_SECRET_KEY`, `VIBE_S3_BUCKET`

## Project Structure Notes

- The server uses process-local in-memory stores by default (no Postgres/Redis required for local dev)
- `vibe-cli` wraps external AI agents and manages local daemon mode for remote spawning
- `vibe-agent` is the control-side CLI for remote machines/sessions
- App has three build modes: `desktop-*`, `mobile-*`, `browser-*` (controlled via Vite modes)
- Tauri v2 is used with custom Android project preparation scripts

## CI/Release

- **CI**: `.github/workflows/ci.yml` - Rust workspace + vibe-app-tauri validation
- **Release**: `.github/workflows/release.yml` - Triggered on `v*` tags, builds binaries
- **App Release**: `.github/workflows/app-release.yml` - Triggered on `app-v*` tags

Version is defined in root `Cargo.toml` `[workspace.package].version` and must match release tags.

## Planning

Active planning lives in `docs/plans/rebuild/`. Read `docs/plans/rebuild/STATUS.md` first to confirm current phase and batch. Update plan files before implementing subsystem changes. See `PLAN.md` for the full document index.
