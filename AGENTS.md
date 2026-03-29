# Repository Guidelines

## Project Structure & Module Organization
This repository is a Rust workspace with one shared crate and three apps. `crates/vibe-core` holds shared protocol types and models. `apps/vibe-relay` is the Axum relay API. `apps/vibe-agent` is the device-side daemon, task runner, and provider adapter layer. `apps/vibe-app` is the Vue 3.5 control UI, and `apps/vibe-app/src-tauri` contains the Tauri shell. In the frontend, keep reusable API/runtime code in `src/lib`, state in `src/stores`, and screens in `src/views`. Do not commit build output from `target/`, `dist/`, or `node_modules/`.

## Build, Test, and Development Commands
- `cargo check -p vibe-relay -p vibe-agent -p vibe-app`: verify all Rust targets compile.
- `cargo test --workspace --all-targets -- --nocapture`: run the relay and agent Rust test suites.
- `./scripts/dual-process-smoke.sh relay_polling`: run the end-to-end relay polling smoke test.
- `./scripts/dual-process-smoke.sh overlay`: run the end-to-end overlay smoke test with graceful
  task and shell fallback coverage plus hard overlay port-forward validation.
- `cargo run -p vibe-relay`: start the relay on port `8787`.
- `cargo run -p vibe-agent -- --relay-url http://127.0.0.1:8787`: start an agent against the local relay.
- `cd apps/vibe-app && npm ci && npm run dev`: run the Vue control app locally.
- `cd apps/vibe-app && npm ci && npm run build`: run `vue-tsc` and produce the production bundle.
- `cd apps/vibe-app && npm run tauri dev`: launch the desktop shell.

## Coding Style & Naming Conventions
Use `cargo fmt --all` for Rust formatting. Follow Rust defaults: `snake_case` for functions/modules, `PascalCase` for structs/enums, and keep shared protocol changes in `vibe-core`. For Vue and TypeScript, follow the existing Composition API style with `<script setup lang="ts">`, 2-space indentation, `PascalCase` component filenames such as `DashboardView.vue`, and `camelCase` store/actions such as `useControlStore` and `reloadAll`.

## Product And UX Guardrails
- Do not keep growing the control app as one long, all-in-one dashboard. When multiple top-level workflows coexist, they must be separated by explicit navigation such as tabs or route-backed sections.
- When AI session launch is the primary workflow, do not isolate relay connection or device-selection prerequisites on a separate top-level page; keep required setup inside the session-first flow and reserve secondary views for management or advanced tools.
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
- If the user has already explicitly specified the concrete repair shape for a remediation item, record that mode as `user-specified` and proceed instead of re-asking the same choice.
- When a change alters the primary user-facing model, update `README.md`, `README.en.md`, and the relevant manual checklist in `TESTING.md` in the same change set before considering the item complete.
- When a change alters developer onboarding, source-build steps, or contributor entry points, update `DEVELOPMENT.md` in the same change set instead of expanding the top-level README files.
- If a change affects release packaging, release notes, or deployment/onboarding flow, update the next-release note source under `docs/releases/` in the same change set.
- After pushing to GitHub, monitor the triggered GitHub Actions runs until they either succeed or fail with a clear diagnosis. Do not treat the delivery as complete immediately after `git push`.
- When `main` is pushed, monitor the `CI` workflow. When a release tag such as `vX.Y.Z` is pushed, monitor the `Release` workflow as well.
- If a workflow fails or shows an abnormal state, record the run URL, failing job, conclusion, and required follow-up in the user-facing report before ending the task.
- Release assets must stay minimal and operator-facing. Do not bundle repository README files or copied build directories into published artifacts when the real deliverable can be uploaded directly.
- Published release asset names must include the shipped version/tag plus platform identity so operators can identify downloads without opening them.
- GitHub Release bodies must come from repository-owned release notes, not only from GitHub auto-generated text.
- Default Linux CLI release archives intended for general self-hosted use must not depend on a
  newer hosted-runner `glibc` baseline by accident. Prefer statically linked `musl` packaging or
  another explicitly verified compatibility strategy, and keep installer asset resolution aligned
  with the shipped Linux target.
- Windows CLI packaging that depends on EasyTier runtime files must keep those DLL/SYS artifacts
  side-by-side with the shipped executables, and installers must copy that packaged layout instead
  of extracting only the `.exe` files.

## Documentation And Workflow Guardrails
- `README.md` and `README.en.md` are user/operator entry points. Keep them focused on what the product does, how to deploy it, how to connect to it, and where to download it.
- Do not write internal governance, anti-hardcoding policy, planning workflow, release-process mandates, or project-management requirements into the top-level README files.
- Contributor, source-build, and local development instructions belong in `DEVELOPMENT.md` or another dedicated developer document, not in the top-level README files.
- Do not use the top-level README files as a navigation hub for `DEVELOPMENT.md`, `TESTING.md`, `AGENTS.md`, `PLAN.md`, or versioned planning documents. Those files may exist at the repository root, but they are not primary README entry points.
- Operator docs may explain required runtime inputs and deployment steps, but they should do so as operational guidance rather than repository-governance language.
- When optimizing CI or release performance with caches, scope cache keys to dependency manifests, lockfiles, or explicit tool-version inputs. Do not add broad caches with unclear invalidation, especially for large SDK trees.

## Testing Guidelines
Prefer focused Rust unit or integration-style tests near parsing, request orchestration, provider mapping, and transport logic; current examples live in `apps/vibe-relay/src/main.rs`, `apps/vibe-agent/src/main.rs`, `apps/vibe-agent/src/workspace_runtime.rs`, and `apps/vibe-agent/src/git_runtime.rs`. Name tests by behavior, for example `claude_tool_use_maps_to_tool_call`. When changing relay or agent control-plane behavior, add or update tests and rerun `cargo test --workspace --all-targets -- --nocapture`; add `./scripts/dual-process-smoke.sh relay_polling` and `./scripts/dual-process-smoke.sh overlay` for end-to-end transport path changes. The frontend still has no dedicated automated harness, so at minimum run `npm run build` and follow the manual checklist in `TESTING.md` when touching `apps/vibe-app`.
- If UI semantics, navigation, visibility gating, or relay/runtime configuration behavior changes, the `TESTING.md` manual regression steps must be updated to match the new product model in the same change set.
- A GitHub push or release cut is not considered fully verified until the corresponding GitHub Actions runs are checked and their final status is reported.
- Do not keep release-critical verification as `best-effort`, forced-success, or non-blocking once a stable harness is available. If CI stability is the problem, fix the harness or move the check out of the required workflow explicitly.
- If a required release verification gate lives in a separate workflow job for clarity or parallelism,
  the final publish job must explicitly depend on it instead of depending only on packaging jobs.
- If a known unstable diagnostic must remain non-blocking for a period, isolate it in a clearly
  named diagnostic job, keep it out of the required verify path, and record the deferred root-cause
  in the active versioned plan before merging.
- Test-only loopback or fixed-address defaults are allowed only inside dedicated local/CI harnesses, must stay out of product/runtime defaults, and must be documented as harness-only behavior.
- Hosted-runner capability workarounds such as EasyTier `no_tun` must stay behind explicit harness-only configuration and must not silently become product/runtime defaults.
- Local or CI harnesses that allocate multiple listener ports for cooperating processes must track
  reserved ports across the whole harness and validate every required protocol binding instead of
  repeatedly probing anonymous TCP ports independently.
- If README, deployment docs, or developer-entry documents move or change materially, the manual verification and release/onboarding checks in `TESTING.md` must be updated in the same change set.
- Windows smoke or installer validation that depends on EasyTier runtime files must exercise the
  packaged side-by-side layout rather than assuming raw `target/` outputs are sufficient.

## Configuration And Networking Guardrails
- Do not use `127.0.0.1`, `localhost`, or other loopback addresses as production-facing product defaults for relay/public-origin behavior. Any loopback fallback that remains must be explicitly development-only and documented as such.
- Distinguish relay/public-origin configuration from target-service defaults. A local preview target such as `127.0.0.1` on the device is not the same thing as a client-facing relay address.
- Keep configuration precedence explicit and stable: user choice first, then persisted client config, then relay-provided config, then safe fallback defaults.
- Do not infer hosted versus self-hosted behavior from string matching on domains or addresses. Use explicit deployment metadata or feature flags instead.
- Do not trust client-supplied `x-vibe-*` tenant, user, or role headers for relay identity. Relay auth must come from configured control tokens, explicit enrollment tokens, or issued device credentials.
- Do not use the human control-plane token as the default long-lived device secret when a dedicated enrollment token plus issued device credential flow is available.

## Commit & Pull Request Guidelines
Git history is not available in this workspace snapshot, so use Conventional Commit style, for example `feat(agent): add claude stream-json mapping`. Pull requests should state the affected crate/app, summarize behavior changes, list validation commands run, and include screenshots for UI updates. Call out new `VIBE_*` environment variables or system dependencies explicitly, and never include secrets, auth tokens, or local machine-specific config in commits.
