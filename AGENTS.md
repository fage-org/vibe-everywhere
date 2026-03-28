# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace with one shared crate and three apps. `crates/vibe-core` holds shared protocol types and models. `apps/vibe-relay` is the Axum relay API. `apps/vibe-agent` is the device-side daemon, task runner, and provider adapter layer. `apps/vibe-app` is the Vue 3.5 control UI, and `apps/vibe-app/src-tauri` contains the Tauri shell. In the frontend, keep reusable API/runtime code in `src/lib`, state in `src/stores`, and screens in `src/views`. Do not commit build output from `target/`, `dist/`, or `node_modules/`.

## Build, Test, and Development Commands
- `cargo check -p vibe-relay -p vibe-agent -p vibe-app`: verify all Rust targets compile.
- `cargo test --workspace --all-targets -- --nocapture`: run the relay and agent Rust test suites.
- `./scripts/dual-process-smoke.sh relay_polling`: run the end-to-end relay polling smoke test.
- `cargo run -p vibe-relay`: start the relay on port `8787`.
- `cargo run -p vibe-agent -- --relay-url http://127.0.0.1:8787`: start an agent against the local relay.
- `cd apps/vibe-app && npm ci && npm run dev`: run the Vue control app locally.
- `cd apps/vibe-app && npm ci && npm run build`: run `vue-tsc` and produce the production bundle.
- `cd apps/vibe-app && npm run tauri dev`: launch the desktop shell.

## Coding Style & Naming Conventions
Use `cargo fmt --all` for Rust formatting. Follow Rust defaults: `snake_case` for functions/modules, `PascalCase` for structs/enums, and keep shared protocol changes in `vibe-core`. For Vue and TypeScript, follow the existing Composition API style with `<script setup lang="ts">`, 2-space indentation, `PascalCase` component filenames such as `DashboardView.vue`, and `camelCase` store/actions such as `useControlStore` and `reloadAll`.

## Product And UX Guardrails
- Do not keep growing the control app as one long, all-in-one dashboard. When multiple top-level workflows coexist, they must be separated by explicit navigation such as tabs or route-backed sections.
- Do not present incomplete enterprise, governance, or admin surfaces in the default end-user flow. Hide them behind explicit feature flags or dedicated management areas until they are truly ready.
- Do not render operator or deployment guidance as always-on primary dashboard prose. Detailed self-hosted guidance belongs in docs, drawers, tooltips, or contextual validation, not as the main description for everyday users.
- Do not make display-only capability summaries look interactive. If users cannot switch, download, or open a client from the UI, the surface must not read like a selector.
- Do not claim runtime support that the current client cannot actually detect or represent. Current-client detection, platform metadata, and visible labels must stay aligned.
- On primary product surfaces, prefer showing the current client only. Do not keep other platform identities visible beside it unless the UI offers real actions such as download, install, or open.

## Planning Governance
- The planning entry point is `docs/plans/README.md`. Do not create new ad hoc planning files outside the versioned planning structure unless they are short compatibility pointers.
- Iteration and remediation planning must be versioned. Each active version must have both a `summary` file and a `details` file.
- `summary` files stay concise for lookup; `details` files hold the full implementation, acceptance, and validation guidance.
- When a plan grows too large or the phase changes materially, start a new version instead of rewriting an old one into a different phase.
- Before implementing a remediation item, present its repair modes, state the recommended mode, and ask the user which mode to use.
- After every completed iteration or remediation item, update the active `summary` file, the active `details` file, `PLAN.md`, and any newly required long-term guardrails in this file.
- When a change alters the primary user-facing model, update `README.md`, `README.en.md`, and the relevant manual checklist in `TESTING.md` in the same change set before considering the item complete.
- After pushing to GitHub, monitor the triggered GitHub Actions runs until they either succeed or fail with a clear diagnosis. Do not treat the delivery as complete immediately after `git push`.
- When `main` is pushed, monitor the `CI` workflow. When a release tag such as `vX.Y.Z` is pushed, monitor the `Release` workflow as well.
- If a workflow fails or shows an abnormal state, record the run URL, failing job, conclusion, and required follow-up in the user-facing report before ending the task.

## Testing Guidelines
Prefer focused Rust unit or integration-style tests near parsing, request orchestration, provider mapping, and transport logic; current examples live in `apps/vibe-relay/src/main.rs`, `apps/vibe-agent/src/main.rs`, `apps/vibe-agent/src/workspace_runtime.rs`, and `apps/vibe-agent/src/git_runtime.rs`. Name tests by behavior, for example `claude_tool_use_maps_to_tool_call`. When changing relay or agent control-plane behavior, add or update tests and rerun `cargo test --workspace --all-targets -- --nocapture`; add `./scripts/dual-process-smoke.sh relay_polling` for end-to-end path changes. The frontend still has no dedicated automated harness, so at minimum run `npm run build` and follow the manual checklist in `TESTING.md` when touching `apps/vibe-app`.
- If UI semantics, navigation, visibility gating, or relay/runtime configuration behavior changes, the `TESTING.md` manual regression steps must be updated to match the new product model in the same change set.
- A GitHub push or release cut is not considered fully verified until the corresponding GitHub Actions runs are checked and their final status is reported.
- Do not keep release-critical verification as `best-effort`, forced-success, or non-blocking once a stable harness is available. If CI stability is the problem, fix the harness or move the check out of the required workflow explicitly.
- Test-only loopback or fixed-address defaults are allowed only inside dedicated local/CI harnesses, must stay out of product/runtime defaults, and must be documented as harness-only behavior.

## Configuration And Networking Guardrails
- Do not use `127.0.0.1`, `localhost`, or other loopback addresses as production-facing product defaults for relay/public-origin behavior. Any loopback fallback that remains must be explicitly development-only and documented as such.
- Distinguish relay/public-origin configuration from target-service defaults. A local preview target such as `127.0.0.1` on the device is not the same thing as a client-facing relay address.
- Keep configuration precedence explicit and stable: user choice first, then persisted client config, then relay-provided config, then safe fallback defaults.
- Do not infer hosted versus self-hosted behavior from string matching on domains or addresses. Use explicit deployment metadata or feature flags instead.

## Commit & Pull Request Guidelines
Git history is not available in this workspace snapshot, so use Conventional Commit style, for example `feat(agent): add claude stream-json mapping`. Pull requests should state the affected crate/app, summarize behavior changes, list validation commands run, and include screenshots for UI updates. Call out new `VIBE_*` environment variables or system dependencies explicitly, and never include secrets, auth tokens, or local machine-specific config in commits.
