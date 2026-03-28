# Vibe Everywhere Product Evolution Plan

Last updated: 2026-03-28

## Purpose

This file is the authoritative execution record for the repository's product, platform, and
architecture evolution.

Detailed, decision-complete implementation guidance lives in
[`docs/iteration-specs.md`](./docs/iteration-specs.md). This file stays concise and records:

- product direction
- current MVP baseline
- iteration status and exit criteria
- completion, validation, risk, and decision logs
- engineering guardrails that apply to every iteration

Every completed iteration must update both this file and
[`docs/iteration-specs.md`](./docs/iteration-specs.md).

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

## Current Iteration

Current planned implementation target:

- Iteration 0 through Iteration 11 are completed for the current roadmap baseline.
- No further iteration is active in this file until a new roadmap revision is defined.

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
