# Vibe Everywhere Product Evolution Plan

Last updated: 2026-03-28

## Purpose

This file is the authoritative execution record for the repository's product, platform, and
architecture evolution.

Detailed, decision-complete implementation guidance now lives in the versioned planning system
under [`docs/plans/README.md`](./docs/plans/README.md). This file stays concise and records:

- product direction
- current MVP baseline
- active plan-set pointers
- completion, validation, risk, and decision logs
- engineering guardrails that apply to every iteration and remediation phase

The active plan set is:

- planning index: [`docs/plans/README.md`](./docs/plans/README.md)
- process governance: [`docs/plans/process.md`](./docs/plans/process.md)
- active iteration summary: [`docs/plans/iterations/v3-summary.md`](./docs/plans/iterations/v3-summary.md)
- active iteration details: [`docs/plans/iterations/v3-details.md`](./docs/plans/iterations/v3-details.md)
- active remediation summary: [`docs/plans/remediation/v9-summary.md`](./docs/plans/remediation/v9-summary.md)
- active remediation details: [`docs/plans/remediation/v9-details.md`](./docs/plans/remediation/v9-details.md)

Every completed iteration or remediation item must update this file and the active versioned plan
files listed above.

## Product Direction

The target product is:

- an AI-session-first remote development control plane
- available on Web, Tauri Desktop, and Android in the near term
- self-hosted by default, with hosted-compatible deployment metadata later
- suitable for personal and small-team use first
- structurally ready to evolve into a multi-tenant enterprise product later

This means the public product model must be reorganized around:

- devices
- AI sessions
- workspace supervision
- preview access
- notifications
- optional advanced tools such as terminal and raw tunnel access

The product must not continue to present `task`, `shell`, and `port-forward` as equal top-level
user-facing concepts.

## Current MVP Baseline

Delivered today:

- device register, heartbeat, presence, and provider discovery
- relay-backed task create, claim, run, cancel, and event streaming
- relay-backed shell session create, input, output, websocket updates, and close
- relay-first and overlay-assisted port-forward backend flows
- Rust relay, Rust agent, Vue control app, Tauri desktop shell, and Android packaging
- provider integration for Codex, Claude Code, and OpenCode
- self-hosted relay as the default operating model

Not yet productized:

- full diff review, patch approval, and merge supervision
- notifications and async workflow UX
- enterprise-grade auth, audit, role model, and storage abstraction

## Engineering Guardrails

These rules are mandatory for every iteration.

### No Hardcoding

Never hardcode:

- relay addresses such as `127.0.0.1:8787` as a product default for mobile or production use
- API base URLs, websocket URLs, preview addresses, or deployment modes inside UI components
- tenant IDs, user IDs, device IDs, workspace paths, or provider model choices as business constants
- language strings directly inside user-facing Vue templates or component logic
- theme colors directly inside page components when a token or variant should be used
- hosted versus self-hosted behavior by matching fixed domains or string fragments

### Configuration Precedence

Every configurable value must follow the same precedence:

1. explicit user setting
2. persisted client setting
3. relay-provided `app-config`
4. safe fallback default

This precedence must be documented and preserved for:

- relay base URL
- access token
- locale
- theme
- feature visibility
- preview strategy

### Product-Layer Naming

The product layer must converge on:

- `AI Session` over raw `Task`
- `Preview` over raw `Port Forward`
- `Terminal` or `Advanced Console` over raw `Shell Session`

Low-level transport terms such as `overlay`, `bridge`, and `tunnel` must remain advanced or debug
concepts unless the user explicitly enters that layer.

### UI Quality Rules

All new or modified UI must follow these rules:

- use `vue-i18n` for all user-visible strings
- use Tailwind CSS and `shadcn-vue` for component structure
- support `light`, `dark`, and `system` theme modes
- avoid growing `apps/vibe-app/src/styles.css` into a second design system
- preserve Web, Tauri, and Android compatibility

### Completion Recording Rules

When an iteration is completed and verified:

- update the iteration status in this file
- append a completion log entry in this file
- append a verification log entry in this file
- update the matching iteration section in `docs/iteration-specs.md`
- record any deviation, regression, or follow-up before moving to the next iteration

## Iteration Overview

| Iteration | Title | Status |
| --- | --- | --- |
| 0 | Planning Baseline And Governance Reset | completed |
| 1 | Session-First Product Information Architecture | completed |
| 2 | Tailwind And Component System Foundation | completed |
| 3 | Full Internationalization Infrastructure | completed |
| 4 | Theme System And Night Mode | completed |
| 5 | Workspace Browse And File Preview | completed |
| 6 | Git Inspect And Session Supervision | completed |
| 7 | Preview Productization | completed |
| 8 | Notification And Async Workflow | completed |
| 9 | Platform Harmonization | completed |
| 10 | Self-Hosted Server Productization | completed |
| 11 | Enterprise Foundations | completed |
| 12 | Delivery Verification Hardening | completed |
| 13 | Workflow And Release Verification Normalization | completed |

## Current Iteration

Current planned implementation target:

- Iteration 0 through Iteration 11 are completed for the current roadmap baseline.
- Iteration roadmap `v3` is active for workflow and release verification normalization after the
  hosted Linux overlay repair.
- Iteration 12 completed the first delivery-verification hardening phase, including Windows smoke
  coverage and the initial hosted Linux overlay diagnostic split.
- Iteration 13 restores GitHub-hosted Linux `overlay` smoke as a blocking gate in both `CI` and
  `Release`, keeps the hosted runner on the harness-only `no_tun` path, aligns release publish
  dependencies with that gate, and audits workflow/testing/release/plan docs for stale claims.
- The active remediation track remains the problem-driven remediation plan in
  `docs/plans/remediation/v9-summary.md`.
- Remediation plan `v1` is complete.
- Remediation plan `v2` is complete after restoring blocking overlay smoke verification and auditing
  the repository for similar compromised checks.
- Remediation plan `v3` is complete after release hygiene, release notes governance, and
  user-facing onboarding/deployment improvements were implemented and validated locally.
- Remediation plan `v4` is complete after moving overlay fallback into relay runtime behavior,
  restoring the README/developer-doc boundary, and adding bounded Android Gradle caches to CI and
  release workflows.
- Remediation plan `v5` is complete for truthful overlay connectivity reporting, stricter README
  operator-surface boundaries, and bounded Android SDK-component caching.
- Remediation plan `v6` is complete after Windows EasyTier side-by-side packaging plus follow-up
  PowerShell smoke-harness fixes were validated by GitHub-hosted `CI`.
- Remediation plan `v7` is complete after exposing the hosted Linux runner TUN-permission root
  cause with stable harness ports and raw EasyTier stop reasons.
- Remediation plan `v8` is complete after GitHub-hosted `CI` run `23688459204` validated the
  hosted Linux `no_tun` overlay diagnostic path.
- Remediation plan `v9` is complete after GitHub-hosted `CI` run `23689144327` restored hosted
  Linux overlay smoke as a verified required gate, aligned release publish dependencies, and
  reconciled stale workflow/documentation claims.

Most recent completed tranche:

- Iteration 8: activity center, notification deduplication, and return-to-context workflow
- Iteration 9: platform capability runtime helpers and narrow-width dashboard harmonization
- Iteration 10: deployment metadata surfacing and self-hosted operator documentation
- Iteration 11: tenant/user/role/audit/storage foundations across relay, agent, and app

## Iteration Summaries

### Iteration 0: Planning Baseline And Governance Reset

- Objective: replace the old architecture-only plan with a product evolution baseline.
- Scope summary: rewrite `PLAN.md`, add `docs/iteration-specs.md`, migrate prior foundation history, and codify anti-hardcoding rules plus completion logging rules.
- Key deliverables: new top-level plan, detailed per-iteration spec file, explicit update procedure after each verified stage.
- Exit criteria: both files exist, the roadmap is decision-complete, and the documentation includes hardening rules against hardcoded addresses, config, locale, and theme behavior.
- Dependencies: none.
- Notes: completed on 2026-03-27.

### Iteration 1: Session-First Product Information Architecture

- Objective: make AI sessions the single top-level product flow.
- Scope summary: restructure the dashboard, rename task-facing copy to AI session language, and demote shell and preview/tunnel flows to advanced tools.
- Key deliverables: session-first IA, updated copy across README/app UI/config exposure, device/session/workspace-centered navigation model.
- Exit criteria: the app no longer presents `task`, `shell`, and `port-forward` as equal homepage modules.
- Dependencies: Iteration 0.
- Notes: completed on 2026-03-27; detailed implementation and acceptance criteria live in `docs/iteration-specs.md`.

### Iteration 2: Tailwind And Component System Foundation

- Objective: replace the current page-scoped CSS approach with a reusable component system.
- Scope summary: introduce Tailwind CSS, `shadcn-vue`, shared UI primitives, and tokenized design decisions.
- Key deliverables: component primitives, layout primitives, and reduced reliance on page-level CSS.
- Exit criteria: core screens are built on reusable components and no new major UI is added in raw page CSS.
- Dependencies: Iteration 1.
- Notes: completed on 2026-03-27; keep Web, Desktop, and Android compatibility intact.

### Iteration 3: Full Internationalization Infrastructure

- Objective: support Chinese and English across all user-visible app copy.
- Scope summary: introduce `vue-i18n`, locale detection, persistence, translation keys, and state/error translation wrappers.
- Key deliverables: `zh-CN` and `en` locales, language switcher, no hardcoded UI strings in core screens.
- Exit criteria: all user-visible text on primary flows is localized.
- Dependencies: Iteration 1 and Iteration 2.
- Notes: completed on 2026-03-27; provider raw output remains untranslated, but user-facing summaries are now localized.

### Iteration 4: Theme System And Night Mode

- Objective: support light, dark, and system theme modes with token-driven styling.
- Scope summary: add theme runtime, persist selection, map semantic colors to tokens, and adapt components for both modes.
- Key deliverables: global theme switcher, class-based theme handling, component parity in both themes.
- Exit criteria: all core app screens and states are usable in light and dark modes.
- Dependencies: Iteration 2.
- Notes: completed on 2026-03-27; the temporary dark-only root style has been removed and replaced with a persisted light/dark/system theme runtime.

### Iteration 5: Workspace Browse And File Preview

- Objective: let users inspect workspace contents without dropping into shell.
- Scope summary: add read-only workspace browsing, file preview, and access restriction to working root.
- Key deliverables: workspace API, file tree, preview panel, large-file and binary handling.
- Exit criteria: users can browse and inspect workspace files from the session workflow.
- Dependencies: Iteration 1.
- Notes: completed on 2026-03-27; editing remains explicitly out of scope for this round.

### Iteration 6: Git Inspect And Session Supervision

- Objective: let users judge AI output quality through Git-aware supervision surfaces.
- Scope summary: add Git status, changed files, recent commits, and diff summaries to the session workspace.
- Key deliverables: Git inspection API, UI panels, session summary aggregation.
- Exit criteria: users can assess AI changes without manually entering terminal for basic review.
- Dependencies: Iteration 5.
- Notes: completed on 2026-03-27; this round intentionally stops at Git-aware supervision and does not attempt a full review/approval workflow.

### Iteration 7: Preview Productization

- Objective: package raw port-forwarding into a product-facing preview flow.
- Scope summary: rename product surfaces to preview, default to HTTP preview use cases, and keep raw tunnels as advanced tools.
- Key deliverables: preview-centric UI, session/workspace launch points, advanced tunnel diagnostics kept separate.
- Exit criteria: preview becomes a user-understandable product flow rather than a raw networking panel.
- Dependencies: Iteration 1 and Iteration 5.
- Notes: completed on 2026-03-27; low-level port-forward backend behavior stays available behind preview-oriented UI and advanced connection details.

### Iteration 8: Notification And Async Workflow

- Objective: support asynchronous AI-session workflows across Web, Desktop, and Android.
- Scope summary: add session outcome notifications, activity center concepts, and return-to-context links.
- Key deliverables: notification events, client handling, opt-in permission UX where required.
- Exit criteria: users can safely leave the screen and still regain context when sessions need attention.
- Dependencies: Iteration 1, Iteration 3, and Iteration 4.
- Notes: completed on 2026-03-28; the delivered notification surface covers task success/failure/cancel and preview ready/fail with deduplication and in-app/system routing.

### Iteration 9: Platform Harmonization

- Objective: unify Web, Tauri Desktop, and Android around one product model.
- Scope summary: introduce platform capability mapping, responsive workspace layouts, and shared runtime rules.
- Key deliverables: stable mobile IA, desktop parity, and platform-specific runtime abstractions.
- Exit criteria: the same core session workflow is coherent across all supported clients.
- Dependencies: Iteration 2, Iteration 3, and Iteration 4.
- Notes: completed on 2026-03-28; Web and Tauri Desktop now share explicit platform helpers and Android-width guidance, while iOS remains future work without blocking the current model.

### Iteration 10: Self-Hosted Server Productization

- Objective: make self-hosting a documented, first-class product capability.
- Scope summary: formalize deployment metadata, storage/auth config boundaries, and operator-facing documentation.
- Key deliverables: clearer deployment modes, improved `app-config` semantics, and fewer assumptions about local-only usage.
- Exit criteria: self-hosted deployments do not depend on source reading or hardcoded local-network assumptions.
- Dependencies: Iteration 1 and Iteration 9.
- Notes: completed on 2026-03-28; this round added deployment/storage/auth metadata surfacing plus self-hosted documentation without introducing a hosted control plane.

### Iteration 11: Enterprise Foundations

- Objective: prepare the data model and service boundaries for multi-tenant and auditable use.
- Scope summary: introduce tenant/user/role/audit abstractions, persistence boundaries, and extensible auth interfaces.
- Key deliverables: non-hardcoded tenant/user assumptions, audit logging, role model, persistence abstraction.
- Exit criteria: the repository no longer structurally assumes permanent single-tenant single-user operation.
- Dependencies: Iteration 10.
- Notes: completed on 2026-03-28; enterprise UX remains future work, but tenant/user/role/audit/storage boundaries are now explicit in the backend and visible in the app.

### Iteration 12: Delivery Verification Hardening

- Objective: strengthen cross-platform delivery verification while honestly tracking the unresolved
  GitHub-hosted Linux overlay instability as deferred follow-up work.
- Scope summary: add a Windows-native relay smoke path, keep Linux relay-polling smoke blocking,
  and move GitHub-hosted Linux `overlay` smoke into explicit non-blocking diagnostic jobs.
- Key deliverables: Windows relay-plus-agent smoke coverage in CI, separately named Linux overlay
  diagnostics in `CI` and `Release`, and recorded plan/process tracking for the deferred hosted-runner
  issue.
- Exit criteria: Windows is no longer validated only by compile and packaging checks, and the
  deferred hosted-runner overlay issue is visible in the versioned plan instead of being silently
  dropped.
- Dependencies: Iteration roadmap `v1` baseline.
- Notes: completed on 2026-03-28 as the first post-baseline verification-hardening phase; later
  hosted Linux gate restoration moved into Iteration 13.

### Iteration 13: Workflow And Release Verification Normalization

- Objective: normalize GitHub Actions and repository docs now that the hosted Linux overlay path is
  stable enough to be a required gate again.
- Scope summary: restore hosted Linux overlay smoke as a blocking job in both `CI` and `Release`,
  keep the hosted runner on the harness-only EasyTier `no_tun` path, make release publishing depend
  on the separate Linux overlay gate, and reconcile testing/release/plan docs with that restored
  behavior.
- Key deliverables: blocking hosted Linux overlay jobs, truthful `no_tun` harness usage in both
  workflows, release publish dependency alignment, and documentation that no longer advertises the
  superseded non-blocking diagnostic phase.
- Exit criteria: hosted Linux overlay smoke is required in both `CI` and `Release`, release
  publishing cannot bypass that gate, and the repository docs describe the hosted-runner path
  truthfully.
- Dependencies: Iteration roadmap `v2` and remediation `v8`.
- Notes: completed on 2026-03-28 after GitHub-hosted `CI` run `23689144327` succeeded.

## Completed Foundation Work

The repository already completed substantial groundwork before this product plan replaced the old
architecture governance plan.

- Relay support code was extracted from `apps/vibe-relay/src/main.rs` into dedicated support and domain modules.
- Agent support code was extracted from `apps/vibe-agent/src/main.rs` into configuration, provider, and runtime modules.
- Relay task, shell, and port-forward paths were modularized without changing public APIs.
- Agent task, shell, and port-forward polling/runtime paths were modularized without changing transport behavior.
- Frontend port-forward flows were wired end to end through relay-backed APIs.
- Default capability advertisement was aligned with the actual MVP surface instead of exposing unfinished capabilities by default.

These items are treated as completed foundation work, not future roadmap items.

## Completion Log

- 2026-03-27: Replaced the old architecture governance plan with a product evolution baseline in `PLAN.md`.
- 2026-03-27: Added `docs/iteration-specs.md` as the decision-complete per-iteration implementation document.
- 2026-03-27: Marked Iteration 0 as completed and set Iteration 1 as the next execution target.
- 2026-03-27: Added hardening rules that forbid hardcoded relay addresses, locale strings, and theme/config behavior in future iterations.
- 2026-03-27: Completed Iteration 1 by restructuring the dashboard into a device rail, AI session list, and session workspace instead of three equal low-level panels.
- 2026-03-27: Demoted terminal and preview tunnel flows into an advanced tools section while preserving the existing shell and port-forward backend paths.
- 2026-03-27: Updated `README.md` and `README.en.md` to describe the product as an AI-session-first remote development control plane.
- 2026-03-27: Completed Iteration 2 by wiring Tailwind CSS v4, `@tailwindcss/vite`, `shadcn-vue`, and reusable UI primitives into the Vue app.
- 2026-03-27: Migrated the main dashboard from page-scoped CSS classes to a component-first layout using shadcn button, card, badge, input, textarea, scroll-area, and separator primitives.
- 2026-03-27: Removed the hardcoded Vite relay proxy target and replaced it with environment-driven proxy configuration.
- 2026-03-27: Completed Iteration 3 by adding `vue-i18n` bootstrap, locale detection and persistence, localized document titles, and bilingual locale bundles for `zh-CN` and `en`.
- 2026-03-27: Migrated the session-first dashboard, status labels, filter options, empty states, advanced-tool panels, and relay/auth copy to translation keys and added an in-app language switcher.
- 2026-03-27: Replaced the frontend port validation string literal path with an error-code-backed translation flow so validation does not depend on hardcoded English matching.
- 2026-03-27: Completed Iteration 4 by adding a persisted `light` / `dark` / `system` theme runtime, system-theme detection, and a non-hardcoded app-config theme hook in the app shell.
- 2026-03-27: Re-tokenized the global background and rebuilt the main dashboard surfaces around semantic theme classes so light and dark modes no longer depend on the old dark-only wrapper.
- 2026-03-27: Added an in-app theme switcher alongside the locale switcher and removed the temporary `.dark` root wrapper from `App.vue`.
- 2026-03-27: Completed Iteration 5 by adding relay/agent workspace browse and file-preview APIs with explicit request/claim/complete flows instead of hidden shell execution.
- 2026-03-27: Added a workspace browser and read-only file preview panel to the session workspace, scoped to the selected session root and guarded against path escape.
- 2026-03-27: Added focused workspace unit tests on the agent side for path-boundary enforcement and text-preview behavior.
- 2026-03-27: Set Iteration 6 as the next execution target.
- 2026-03-27: Completed Iteration 6 by adding relay/agent Git inspect APIs with explicit request/claim/complete flows instead of shell transcript scraping.
- 2026-03-27: Added a session-supervision summary card and a Git inspect panel with branch context, workspace-scoped changed files, recent commits, and direct handoff into workspace file preview.
- 2026-03-27: Added Git inspect capability advertisement, a relay feature flag for the new surface, and focused agent Git tests for non-repository and changed-repository cases.
- 2026-03-27: Set Iteration 7 as the next execution target.
- 2026-03-27: Completed Iteration 7 by turning the raw port-forward panel into a preview-oriented flow with session/workspace launch controls, preview URLs, and open-in-browser affordances.
- 2026-03-27: Demoted raw relay and target endpoint details into an advanced connection section while keeping the underlying port-forward runtime, status filters, and close controls intact.
- 2026-03-27: Added mobile loopback warning behavior for preview URLs and kept preview addresses derived from runtime relay host/port data instead of fixed client-side domains.
- 2026-03-27: Set Iteration 8 as the next execution target.
- 2026-03-28: Completed Iteration 8 by adding an activity center, notification deduplication, task/preview outcome classification, and return-to-context actions in the control app.
- 2026-03-28: Completed Iteration 9 by adding shared platform helpers, platform capability matrix rendering, explicit client/deployment guidance, and tighter narrow-width dashboard behavior.
- 2026-03-28: Completed Iteration 10 by extending `AppConfig` with deployment/auth/storage metadata, updating the Tauri shell config path, and adding `docs/self-hosted.md`.
- 2026-03-28: Completed Iteration 11 by introducing tenant/user/membership/audit/store abstractions, actor-aware relay write/read boundaries, agent-provided actor headers, and audit/event governance UI.
- 2026-03-28: Marked the Iteration 0-11 baseline roadmap as complete and reset the current target to future planning.
- 2026-03-28: Added `docs/problem-remediation-plan.md` to track the post-baseline dashboard, deployment-guidance, platform-surfacing, and loopback-default repair work item by item.
- 2026-03-28: Introduced the versioned planning system under `docs/plans/`, with shared process governance plus `summary` and `details` files for iteration plan v1 and remediation plan v1.
- 2026-03-28: Started iteration roadmap `v2` to track post-baseline delivery verification hardening and the deferred GitHub-hosted Linux overlay instability.
- 2026-03-28: Implemented Iteration 12 locally by adding Windows relay smoke coverage in CI, moving GitHub-hosted Linux `overlay` smoke into explicit non-blocking diagnostic jobs in both `CI` and `Release`, and recording the deferred issue in the versioned plan and workflow guardrails.
- 2026-03-28: Completed Remediation R1 by replacing the single-route control dashboard with route-backed `Sessions`, `Devices`, `Connections`, and `Advanced` sections inside a shared responsive shell.
- 2026-03-28: Added desktop sidebar navigation and mobile bottom navigation so the primary workflows no longer coexist as one long screen.
- 2026-03-28: Completed Remediation R2 by gating the `Governance And Audit` surface behind the explicit `governance_audit_console` feature flag and hiding it from the default user flow.
- 2026-03-28: Aligned governance visibility with audit-event loading so the default app path no longer fetches or renders that unfinished enterprise-facing surface.
- 2026-03-28: Completed Remediation R3 by removing inline deployment/operator guidance and relay warning copy from the primary `Connections` screen.
- 2026-03-28: Replaced the deployment card description with neutral metadata-oriented copy and kept self-hosted detail in documentation links instead of persistent dashboard prose.
- 2026-03-28: Completed Remediation R4 by keeping loopback fallback only on explicit debug/development paths and removing silent loopback defaults from relay/public-origin product behavior.
- 2026-03-28: Cleaned up relay public-origin and forward-host derivation, made preview creation fail explicitly when no public relay host is configured, limited desktop/agent loopback bootstrap to dev paths, and fixed the connections screen to fall back when relay-public-origin is an empty string.
- 2026-03-28: Completed Remediation R5 by replacing the misleading client-capability matrix with a current-client runtime summary plus a read-only control-client form explanation.
- 2026-03-28: Kept deployment metadata visible, restored governance gating to the actual governance card, and removed the impression that Web/Desktop/Android can be switched from inside the current page.
- 2026-03-28: Completed Remediation R6 by adding explicit Android-native current-client detection and aligning platform-related defaults with the clients the UI can actually represent.
- 2026-03-28: Simplified the main connections surface back to the current client only, while keeping mobile relay behavior safe by treating mobile user agents as explicit-remote relay clients even when they are running in a browser.
- 2026-03-28: Completed Remediation R7 by reconciling README, testing guidance, planning-process rules, and repository guardrails with the repaired navigation, visibility, and networking model.
- 2026-03-28: Added durable manual QA expectations for sidebar/bottom-nav sections, current-client-only platform surfacing, governance default hiding, and development-only loopback behavior.
- 2026-03-28: Started remediation plan `v2` to repair release-verification integrity after finding
  that the release workflow still treated `overlay` smoke as best-effort.
- 2026-03-28: Completed Remediation v2 R1 by stabilizing the same-host overlay smoke harness with a
  test-only loopback bootstrap path, deterministic overlay node IP, and richer timeout diagnostics.
- 2026-03-28: Completed Remediation v2 R2 by auditing the repository for similar compromised
  verification patterns and removing the release overlay smoke forced-success bypass.
- 2026-03-28: Started remediation plan `v3` to clean up release artifacts, version release asset
  names, make release notes repository-owned, and rewrite the operator onboarding path.
- 2026-03-28: Completed Remediation v3 R1 by shrinking release packaging to meaningful binaries and
  installers only, and by versioning all published asset names.
- 2026-03-28: Completed Remediation v3 R2 by moving GitHub Release bodies to repository-owned note
  sources under `docs/releases/` and wiring release-note rendering into the release workflow.
- 2026-03-28: Completed Remediation v3 R3 by rewriting the top-level README flow around operators,
  replacing the self-hosted note with a deployment guide, and adding Linux/Windows relay bootstrap
  installers with auto-start setup.
- 2026-03-28: Started remediation plan `v4` to repair overlay runtime fallback behavior, restore
  README ownership boundaries, and speed up Android CI/release jobs with bounded caches.
- 2026-03-28: Completed Remediation v4 R1 by adding relay-side overlay bridge health tracking,
  startup/connect timeouts, background recovery probes, and transport suppression until a bridge
  becomes healthy again.
- 2026-03-28: Completed Remediation v4 R2 by rewriting the top-level README files back to
  user/operator entry points, splitting developer instructions into `DEVELOPMENT.md`, and
  codifying the documentation-surface rule in repository guardrails.
- 2026-03-28: Completed Remediation v4 R3 by enabling dependency-aware Gradle caching for Android
  jobs in both `CI` and `Release`.
- 2026-03-28: Started remediation plan `v6` to align Windows EasyTier runtime packaging across
  smoke validation, release archives, and the Windows relay installer after finding that the
  installer extracted only `vibe-relay.exe`.
- 2026-03-28: Completed Remediation v6 R1 by switching Windows smoke to a package-style
  side-by-side runtime directory, routing Windows CLI archive staging through the shared runtime
  staging script, and making the Windows relay installer copy the packaged EasyTier runtime files.
- 2026-03-28: Closed remediation plan `v6` after GitHub-hosted `CI` run `23687362451` reported
  `Windows Compatibility` success and the Windows relay smoke passed after the follow-up
  PowerShell script fixes.
- 2026-03-28: Started remediation plan `v7` to stabilize GitHub-hosted Linux overlay diagnostics
  by reserving unique multi-protocol harness ports and preserving the raw embedded EasyTier agent
  stop reason in smoke artifacts.
- 2026-03-28: Completed Remediation v7 R1 locally by replacing independent anonymous-port probes in
  `scripts/dual-process-smoke.sh` with harness-local reservations that keep relay, agent, and
  target helper ports unique and TCP/UDP-capable, and by preserving the original EasyTier stop
  reason in agent overlay status instead of overwriting it with a generic wrapper message.
- 2026-03-28: Closed remediation plan `v7` after GitHub-hosted `CI` run `23687951251` proved the
  stabilized harness and raw-error propagation changes, and isolated the next Linux hosted-runner
  issue as a distinct TUN-permission limitation.
- 2026-03-28: Started remediation plan `v8` to repair the GitHub-hosted Linux overlay diagnostic
  with a harness-scoped EasyTier `no_tun` path instead of changing product defaults.
- 2026-03-28: Completed Remediation v8 R1 locally by exposing EasyTier `no_tun` in relay and
  agent config, enabling it only from the hosted Linux overlay diagnostic job, and rewriting the
  Linux no_tun smoke path to assert truthful overlay fallback behavior without claiming impossible
  hosted-runner bridge reachability.
- 2026-03-28: Closed remediation plan `v8` after GitHub-hosted `CI` run `23688459204` validated
  the hosted Linux `no_tun` overlay diagnostic path.
- 2026-03-28: Started iteration roadmap `v3` to normalize GitHub Actions and repository docs after
  the hosted Linux overlay repair.
- 2026-03-28: Started remediation plan `v9` to restore hosted Linux overlay smoke as a required
  gate, align release publish dependencies, and reconcile stale workflow/documentation claims.
- 2026-03-28: Implemented Iteration 13 locally by restoring hosted Linux overlay smoke as a
  blocking job in both `CI` and `Release`, keeping the hosted runner on the harness-only
  `VIBE_TEST_EASYTIER_NO_TUN=1` path, aligning release publish dependencies, and auditing workflow,
  testing, release-note, and plan docs for superseded non-blocking overlay claims.
- 2026-03-28: Implemented Remediation v9 R1 locally by renaming and re-gating the hosted Linux
  overlay jobs, enabling the missing hosted `no_tun` env in `Release`, making GitHub Release
  publish depend on the separate Linux overlay gate, and adding a durable guardrail that publish
  jobs must depend on all separate required release-verification gates.
- 2026-03-28: Closed Iteration 13 and remediation `v9` after GitHub-hosted `CI` run `23689144327`
  succeeded with `Verify`, `Linux Overlay Smoke`, `Windows Compatibility`, and `Android Mobile`.
- 2026-03-26: Completed relay and agent runtime modularization, frontend port-forward MVP wiring, and capability-boundary alignment as foundational architecture work.

## Verification Log

- 2026-03-27: Verified `PLAN.md` and `docs/iteration-specs.md` existence and iteration coverage after the planning-baseline rewrite.
- 2026-03-27: `cd apps/vibe-app && npm run build` succeeded after the session-first dashboard rewrite.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the session-first dashboard rewrite.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` succeeded after the session-first dashboard rewrite.
- 2026-03-27: `cd apps/vibe-app && npm run build` succeeded after the Tailwind CSS and `shadcn-vue` migration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the Tailwind CSS and `shadcn-vue` migration.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` succeeded after the Tailwind CSS and `shadcn-vue` migration.
- 2026-03-27: `cd apps/vibe-app && npm run build` succeeded after the Iteration 3 internationalization migration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the Iteration 3 internationalization migration.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` succeeded after the Iteration 3 internationalization migration.
- 2026-03-27: `cd apps/vibe-app && npm run build` succeeded after the Iteration 4 theme-system migration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the Iteration 4 theme-system migration.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` succeeded after the Iteration 4 theme-system migration.
- 2026-03-27: `cd apps/vibe-app && npm run build` succeeded after the Iteration 5 workspace-browser integration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the Iteration 5 workspace-browser integration.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` succeeded after the Iteration 5 workspace-browser integration.
- 2026-03-27: `cd apps/vibe-app && npm run build` succeeded after the Iteration 6 Git-supervision integration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the Iteration 6 Git-supervision integration.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` succeeded after the Iteration 6 Git-supervision integration.
- 2026-03-27: `cd apps/vibe-app && npm run build` succeeded after the Iteration 7 preview-productization integration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the Iteration 7 preview-productization integration.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` succeeded after the Iteration 7 preview-productization integration.
- 2026-03-28: `cargo fmt --all` succeeded after the Iteration 8-11 integration tranche.
- 2026-03-28: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the Iteration 8-11 integration tranche.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after the Iteration 8-11 integration tranche.
- 2026-03-28: `cargo test --workspace --all-targets -- --nocapture` succeeded after the Iteration 8-11 integration tranche.
- 2026-03-28: documentation-only verification completed for `docs/problem-remediation-plan.md` and the related `PLAN.md`/`AGENTS.md` updates before any code-level remediation begins.
- 2026-03-28: documentation-only verification completed for the versioned planning structure under `docs/plans/` plus the compatibility-pointer updates in `docs/iteration-specs.md` and `docs/problem-remediation-plan.md`.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation R1 route-backed navigation refactor.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation R2 governance/audit feature gating.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation R3 deployment-guidance relocation.
- 2026-03-28: `cargo fmt --all` succeeded after Remediation R4 loopback/public-origin cleanup.
- 2026-03-28: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after Remediation R4 loopback/public-origin cleanup.
- 2026-03-28: `cargo test --workspace --all-targets -- --nocapture` succeeded after Remediation R4 loopback/public-origin cleanup.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation R4 loopback/public-origin cleanup.
- 2026-03-28: `./scripts/dual-process-smoke.sh relay_polling` succeeded after Remediation R4 loopback/public-origin cleanup.
- 2026-03-28: `cargo fmt --all` succeeded after Remediation R5 platform-surface semantic correction.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation R5 platform-surface semantic correction.
- 2026-03-28: `cargo fmt --all` succeeded after Remediation R6 current-client detection alignment.
- 2026-03-28: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after Remediation R6 current-client detection alignment.
- 2026-03-28: `cargo test --workspace --all-targets -- --nocapture` succeeded after Remediation R6 current-client detection alignment.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation R6 current-client detection alignment.
- 2026-03-28: file-content review completed after Remediation R7 docs/tests/process realignment.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation R7 docs/tests/process realignment.
- 2026-03-28: `cargo fmt --all --check` succeeded after Remediation v2 verification-integrity
  repair.
- 2026-03-28: `./scripts/dual-process-smoke.sh relay_polling` succeeded after Remediation v2
  verification-integrity repair.
- 2026-03-28: `./scripts/dual-process-smoke.sh overlay` succeeded after Remediation v2
  verification-integrity repair.
- 2026-03-28: repository search for forced-success or best-effort verification markers confirmed
  that no release-critical smoke test remains intentionally non-blocking after Remediation v2.
- 2026-03-28: documentation-only verification completed for remediation plan `v3`, including the
  active-plan pointer changes and the release-notes/process governance updates.
- 2026-03-28: `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app` succeeded after
  Remediation v3 release/workflow and versioning changes.
- 2026-03-28: `cargo test --locked --workspace --all-targets -- --nocapture` succeeded after
  Remediation v3 release/workflow and onboarding changes.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation v3 README and
  deployment-doc rewrites.
- 2026-03-28: `./scripts/dual-process-smoke.sh relay_polling` succeeded after Remediation v3
  release asset packaging changes.
- 2026-03-28: `./scripts/dual-process-smoke.sh overlay` succeeded after Remediation v3 release
  asset packaging changes.
- 2026-03-28: `bash -n scripts/install-relay.sh` and `./scripts/render-release-notes.sh v0.1.4`
  succeeded after Remediation v3 deployment and release-note additions.
- 2026-03-28: `cargo fmt --all` succeeded after Remediation v6 Windows runtime packaging
  alignment.
- 2026-03-28: `cargo check --locked -p vibe-relay -p vibe-agent` succeeded after Remediation v6
  Windows runtime packaging alignment.
- 2026-03-28: `./scripts/render-release-notes.sh v0.0.0 >/dev/null` succeeded after Remediation
  v6 release-note updates.
- 2026-03-28: local PowerShell parser validation for `scripts/stage-windows-runtime.ps1`,
  `scripts/dual-process-smoke.ps1`, and `scripts/install-relay.ps1` could not be run because
  `pwsh` was not installed in the local environment; Windows GitHub Actions validation is still
  required after push.
- 2026-03-28: GitHub-hosted Windows `CI` run `23687362451` completed successfully after
  Remediation v6 Windows runtime packaging plus follow-up PowerShell harness fixes; job
  `69009094041` (`Windows Compatibility`) and its `Run Windows relay smoke test` step both
  succeeded.
- 2026-03-28: `bash -n scripts/dual-process-smoke.sh` succeeded after Remediation v7 Linux
  overlay diagnostic harness stabilization.
- 2026-03-28: `cargo fmt --all --check` succeeded after Remediation v7 Linux overlay diagnostic
  harness stabilization.
- 2026-03-28: `cargo check --locked -p vibe-relay -p vibe-agent` succeeded after Remediation v7
  Linux overlay diagnostic harness stabilization.
- 2026-03-28: `./scripts/dual-process-smoke.sh overlay` succeeded after Remediation v7 Linux
  overlay diagnostic harness stabilization.
- 2026-03-28: `./scripts/dual-process-smoke.sh relay_polling` succeeded after Remediation v7
  Linux overlay diagnostic harness stabilization.
- 2026-03-28: GitHub-hosted `CI` run `23687951251` confirmed Remediation v7's raw-error
  propagation and exposed the hosted Linux EasyTier failure as `rust tun error Operation not
  permitted (os error 1)`.
- 2026-03-28: `cargo fmt --all --check` succeeded after Remediation v8 hosted Linux no_tun
  overlay diagnostic changes.
- 2026-03-28: `cargo check --locked -p vibe-relay -p vibe-agent` succeeded after Remediation v8
  EasyTier no_tun config additions.
- 2026-03-28: `bash -n scripts/dual-process-smoke.sh` succeeded after Remediation v8 hosted Linux
  no_tun smoke-path updates.
- 2026-03-28: `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay` succeeded
  after Remediation v8 hosted Linux no_tun diagnostic changes.
- 2026-03-28: `./scripts/dual-process-smoke.sh relay_polling` succeeded after Remediation v8
  shared smoke-path updates.
- 2026-03-28: `cargo fmt --all --check` succeeded after Iteration 13 workflow and documentation
  normalization changes.
- 2026-03-28: `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app` succeeded after
  Iteration 13 workflow and documentation normalization changes.
- 2026-03-28: `cargo test --locked --workspace --all-targets -- --nocapture` succeeded after
  Iteration 13 workflow and documentation normalization changes.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Iteration 13 workflow and
  documentation normalization changes.
- 2026-03-28: `bash -n scripts/dual-process-smoke.sh` succeeded after Iteration 13 workflow and
  documentation normalization changes.
- 2026-03-28: `./scripts/dual-process-smoke.sh relay_polling` succeeded after Iteration 13
  workflow and documentation normalization changes.
- 2026-03-28: `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay` succeeded
  after Iteration 13 hosted Linux gate-restoration changes.
- 2026-03-28: `./scripts/render-release-notes.sh v0.1.6 >/dev/null` succeeded after the Iteration
  13 release-note updates.
- 2026-03-28: GitHub-hosted `CI` run `23689144327` completed successfully after Iteration 13 and
  remediation `v9`; jobs `69013607968` (`Verify`), `69013725596` (`Linux Overlay Smoke`),
  `69013725588` (`Windows Compatibility`), and `69013725582` (`Android Mobile`) all succeeded.
- 2026-03-28: `cargo fmt --all` succeeded after Remediation v4 runtime, documentation, and
  workflow updates.
- 2026-03-28: `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app` succeeded after
  Remediation v4 runtime, documentation, and workflow updates.
- 2026-03-28: `cargo test --locked --workspace --all-targets -- --nocapture` succeeded after
  Remediation v4 runtime, documentation, and workflow updates.
- 2026-03-28: `cd apps/vibe-app && npm run build` succeeded after Remediation v4 README rewrite,
  developer-guide split, and runtime changes.
- 2026-03-28: `bash -n scripts/install-relay.sh` and `./scripts/render-release-notes.sh v0.0.0`
  succeeded after Remediation v4 documentation and workflow updates.
- 2026-03-28: `./scripts/dual-process-smoke.sh relay_polling` succeeded after Remediation v4
  runtime, documentation, and workflow updates.
- 2026-03-28: `./scripts/dual-process-smoke.sh overlay` succeeded after Remediation v4 overlay
  runtime fallback and auto-recovery changes.
- 2026-03-28: `cargo fmt --all --check` succeeded after Iteration 12 delivery-verification
  updates.
- 2026-03-28: `cargo test --locked --workspace --all-targets -- --nocapture` succeeded after
  Iteration 12 delivery-verification updates.
- 2026-03-28: `bash -n scripts/dual-process-smoke.sh` succeeded after Iteration 12
  delivery-verification updates.
- 2026-03-28: `./scripts/dual-process-smoke.sh relay_polling` succeeded after Iteration 12
  delivery-verification updates.
- 2026-03-28: `./scripts/render-release-notes.sh v0.0.0 >/dev/null` succeeded after Iteration 12
  delivery-verification updates.
- 2026-03-28: local execution of `scripts/dual-process-smoke.ps1` could not be performed because
  `pwsh` was not installed in the local environment; Windows GitHub Actions validation remains
  required after push.
- 2026-03-26: `cargo fmt --all` succeeded after the capability-advertisement alignment.
- 2026-03-26: `cargo test -p vibe-agent -- --nocapture` succeeded after the capability-advertisement alignment.
- 2026-03-26: `cd apps/vibe-app && npm run build` succeeded after the capability-advertisement alignment.
- 2026-03-26: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the capability-advertisement alignment.

## Risk And Decision Log

- 2026-03-27: Decided to keep `shell` and `port-forward` as platform capabilities, but demote them to advanced product surfaces instead of removing them.
- 2026-03-27: Decided to prioritize Web, Tauri Desktop, and Android before iOS because Android packaging already exists in the current repository.
- 2026-03-27: Decided that i18n must cover all user-visible product copy in Chinese and English, rather than introducing a partial translation layer.
- 2026-03-27: Decided that the design system migration will use Tailwind CSS plus `shadcn-vue`, not another component library or additional page-scoped CSS expansion.
- 2026-03-27: Iteration 1 intentionally kept existing relay and agent APIs unchanged; session-first naming is currently a product-layer/UI rewrite rather than a wire-level rename.
- 2026-03-27: Interactive manual UI smoke against a live relay/agent session was not run in this turn; compile and test coverage passed, but visual regression risk remains until Iteration 2 work begins.
- 2026-03-27: Iteration 2 kept native `<select>` controls for the current screen while migrating the rest of the foundation to shadcn primitives; full select/dropdown convergence can continue in later UI iterations.
- 2026-03-27: Interactive visual regression against a live relay/agent pair was still not run after the component-system migration, so spacing and mobile behavior should be manually checked before release.
- 2026-03-27: Iteration 3 localizes product-owned copy, enum labels, and explicit client validation errors, but intentionally leaves raw provider output and backend-returned freeform error text untranslated.
- 2026-03-27: Interactive locale-switch and persisted-locale smoke was not run against live Web, Tauri, and Android clients in this turn; runtime compile/test coverage passed, but manual QA is still required before release.
- 2026-03-27: Iteration 4 defaults theme behavior to `system`, persists explicit user selection locally, and keeps relay-provided theme handling optional through a non-hardcoded config extractor for future server support.
- 2026-03-27: Interactive theme smoke across Web, Tauri, and Android was not run in this turn; build/test coverage passed, but manual verification of contrast, system-theme switching, and mobile layout is still required before release.
- 2026-03-27: Iteration 5 constrains workspace browsing to the resolved session root, which is stricter than the long-term “working root” envelope; broader browsing inside the whole working root can be revisited deliberately later if product needs justify it.
- 2026-03-27: Interactive workspace-browser QA against a live relay/agent pair was not run in this turn; compile/test coverage passed, but manual verification is still needed for large directories, binary previews, and Android-width usability.
- 2026-03-27: Iteration 6 scopes changed-file and diff summaries to the selected session workspace when the session cwd is nested inside a larger repository, while branch/upstream and recent-commit context remain repo-wide.
- 2026-03-27: The Git inspect surface is capability-advertised and productized even when `git` is unavailable on the device; in that case the panel returns a `git_unavailable` state instead of failing closed or forcing terminal fallback.
- 2026-03-27: Interactive Git-supervision QA against a live relay/agent pair was not run in this turn; compile/test coverage passed, but manual verification is still needed for long changed-file lists, deleted-file behavior, and mobile-width ergonomics.
- 2026-03-27: Iteration 7 derives preview URLs from the active relay host/port runtime data and current relay context, but intentionally defaults the product-facing link shape to HTTP for common local web-preview use cases.
- 2026-03-27: Preview launch now lives in the session workspace while raw relay and target endpoints remain available in an advanced connection block; this is a product-layer packaging change, not a new network transport.
- 2026-03-27: Interactive preview QA against live Web, Tauri, and Android clients was not run in this turn; compile/test coverage passed, but manual verification is still needed for browser-opening behavior, loopback warnings, and self-hosted forward-host configuration.
- 2026-03-28: Iteration 8 routes “system notifications” through the active runtime's Notification API when supported; there is still no background push service or OS-specific native notification bridge.
- 2026-03-28: Iteration 9 isolates client detection and relay-URL preference rules in frontend runtime helpers, but the current capability matrix still models mobile-native support at the product level rather than maintaining separate Web-mobile and Android-native forks.
- 2026-03-28: Iteration 10 keeps `external` storage as a compatibility placeholder that still reuses file-backed persistence until a real external store is introduced.
- 2026-03-28: Iteration 11 enforces tenant and role boundaries in relay handlers and records audit trails, but identity is still derived from configured defaults or explicit headers rather than a full signed user-session model.
- 2026-03-28: Decided to run the next repair tranche as a problem-driven remediation track instead of reopening the completed Iteration 0-11 roadmap table.
- 2026-03-28: Remediation R1 intentionally keeps governance, deployment guidance, and platform semantics present inside the new `Connections` route so that R2-R5 can still be repaired item by item without silently changing their chosen modes.
- 2026-03-28: Remediation R2 keeps the governance/audit implementation code available behind a dedicated feature flag instead of deleting it, so future enterprise work can re-enable it deliberately without default-path leakage.
- 2026-03-28: Remediation R3 deliberately removes warning-style deployment prose from the main UI; future network/default cleanup belongs to R4 rather than reintroducing operator copy into the dashboard.
- 2026-03-28: Remediation R4 keeps `0.0.0.0` wildcard listening valid for relay bind addresses and bridge listeners, but no longer treats wildcard or loopback values as production-safe public relay origins.
- 2026-03-28: Remediation R4 preserves `127.0.0.1` as a valid preview target default for device-local services while requiring explicit non-empty relay/public-origin configuration for product-facing preview links outside debug fallback.
- 2026-03-28: Remediation R5 treats platform information as descriptive runtime metadata, not as an in-page selector; any future download/open/install actions must become explicit product actions rather than visual implication.
- 2026-03-28: Remediation R6 keeps the product promise aligned with what the runtime can actually identify: Android native is explicit, mobile web remains `web`, and the default UI avoids showing other platform identities when the user cannot switch to them there.
- 2026-03-28: Remediation R7 makes documentation and manual verification part of the feature completion bar; UI/model changes are not complete until README, TESTING, plan records, and guardrails are updated in the same tranche.
- 2026-03-28: Remediation v2 keeps test-only loopback and fixed overlay node IP behavior confined
  to the smoke harness; product/runtime defaults remain governed by the non-hardcoded public-origin
  rules from Remediation R4.
- 2026-03-28: Remediation v2 classified the remaining Android signing `exit 0` branch in the
  release workflow as a packaging fallback for missing secrets, not as a compromised test gate.
- 2026-03-28: Iteration 12 keeps GitHub-hosted Linux `overlay` smoke visible as diagnostic signal
  instead of deleting it, but explicitly removes it from the required verify path until the hosted-runner
  root cause is reopened and repaired.
- 2026-03-28: Iteration 12 currently has only static local review for the new Windows smoke script
  because the local environment does not provide `pwsh`; GitHub-hosted Windows validation is the
  next required check after push.
- 2026-03-28: Remediation v4 keeps overlay recovery scoped to future work selection rather than
  trying to migrate an already running task/session/forward back onto overlay mid-flight.
- 2026-03-28: Remediation v4 reaffirms that top-level README files are user/operator surfaces,
  while contributor workflow and source-build guidance belong in `DEVELOPMENT.md`.
- 2026-03-28: Remediation v4 chooses Gradle dependency caches keyed by repository inputs instead of
  broad Android SDK caches to reduce build time without increasing stale-cache risk materially.
- 2026-03-28: Remediation v8 keeps EasyTier `no_tun` strictly as a harness-only hosted-runner
  workaround; product/runtime defaults remain unchanged and still expect explicit non-test
  networking behavior.
- 2026-03-28: Remediation v8 intentionally treats hosted Linux no_tun preview coverage as
  transport-and-lifecycle validation only; preview byte-path verification remains covered by the
  stable relay-polling smoke while the no_tun websocket-reset gap is deferred for a later repair.
- 2026-03-28: Iteration 13 keeps hosted Linux overlay smoke as a separate required workflow job
  instead of folding it back into `verify`, so hosted-runner artifacts and failure boundaries stay
  explicit while the release workflow can still parallelize packaging after `verify`.
- 2026-03-28: Iteration 13 requires the final release publish job to depend on every separate
  required verification gate, not only on packaging jobs, so a failed release-critical smoke path
  cannot be bypassed by workflow structure alone.
