# Shared Follow-Up Plan: repo-release-and-docs

## Purpose

Capture the non-blocking repository-level follow-up needed after the rebuild baseline:

- GitHub Actions coverage for the Rust workspace and imported app validation
- tag-driven GitHub release packaging for the Rust binaries
- a root README that explains deployment and day-to-day usage instead of only pointing at plans
- root-level license attribution noting the Happy-aligned origin of the rebuild

This work is packaging/documentation infrastructure. It must not reopen completed wave module scope
or change locked runtime contracts.

## Scope

In scope:

- root `.github/workflows/*`
- root `README.md`
- root `LICENSE`
- root packaging metadata needed to make tag-driven releases predictable

Out of scope:

- store submission automation for `packages/vibe-app`
- moving app release flows out of `packages/vibe-app`
- redesigning crate or package boundaries
- changing server, CLI, agent, wire, or app protocol behavior

## Decisions

1. CI remains split by toolchain:
   - Rust workspace validation runs with stable Rust
   - app validation runs with Node `24.14.0` and Yarn `1.22.22`
2. GitHub Releases package the Rust binaries only:
   - `vibe`
   - `vibe-agent`
   - `vibe-server`
   - `vibe-app-logs`
3. Release publishing is tag-driven with `vX.Y.Z` tags.
4. Workspace versioning is centralized at the root `Cargo.toml` so the release tag can be checked
   against a single version source.
5. Root documentation should emphasize:
   - local deployment
   - self-hosting assumptions
   - CLI/app usage flow
   - release workflow expectations

## Implementation Order

1. Add the repository-level follow-up plan.
2. Normalize Rust crate package metadata so version checks can read from the workspace root.
3. Add GitHub Actions CI for Rust and app validation.
4. Add tag-triggered GitHub release packaging for Rust binaries.
5. Rewrite the root README around deployment, operations, and usage.
6. Extend the root MIT license with a Happy-origin attribution notice.

## Acceptance Criteria

- push/PR CI validates the Rust workspace and `packages/vibe-app`
- a `vX.Y.Z` tag can produce a GitHub Release with packaged Rust binaries and checksums
- the root README documents deployment prerequisites, local bring-up, runtime env vars, and release
  usage
- the root license keeps MIT terms intact while recording the Happy-aligned origin of imported or
  adapted material
