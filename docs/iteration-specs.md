# Vibe Everywhere Iteration Specs

Last updated: 2026-03-28

## How To Use This File

This file is the decision-complete implementation guide for every planned iteration.

Use it together with [`PLAN.md`](../PLAN.md):

- `PLAN.md` records status, decisions, completion, and verification.
- this file defines the implementation shape, acceptance criteria, and post-completion update rules for each iteration.

When an iteration is completed:

- update the matching status in `PLAN.md`
- append completion and verification entries in `PLAN.md`
- fill in `Implementation Notes`, `Validation Results`, `Deviations`, and `Follow-up Items` in this file

Do not move to the next iteration with known hardcoded config, address, locale, or theme shortcuts left undocumented.

## Global Product Principles

- The product is `AI session first`.
- `Task`, `shell`, and `port-forward` are backend/runtime concepts, not permanent top-level UX concepts.
- Shell and raw tunnels remain supported, but they become advanced tools behind the primary session workflow.
- The control app must work as one product across Web, Tauri Desktop, and Android.
- Self-hosted relay/server support is a first-class product requirement, not a debugging convenience.
- Enterprise evolution must build on the same substrate rather than a separate forked architecture.

## Global Frontend Principles

- All user-visible strings must be routed through i18n keys.
- Tailwind CSS plus `shadcn-vue` is the only approved UI foundation for new or rebuilt surfaces.
- Theme handling must support `light`, `dark`, and `system` consistently.
- Semantic tokens must drive color, spacing, radius, elevation, and state styling.
- Shared business components must sit above raw UI primitives so the product can evolve without rewriting page layouts repeatedly.
- Mobile layouts must be intentionally designed, not obtained by squeezing desktop panels into narrow widths.

## Global Backend Principles

- Existing relay and agent APIs can remain stable during the early product-rename phase.
- New product capabilities should be introduced as explicit APIs or feature flags, not as hidden shell commands.
- Capability advertisement must match real delivered surfaces.
- Relay, agent, and app must not drift in naming, capability flags, or field semantics.
- Access to workspace browsing must remain bounded to the configured working root unless a future iteration explicitly broadens the security model.

## Global Safety And Configuration Principles

### Forbidden Shortcuts

The implementation must not hardcode:

- relay base URLs
- websocket endpoints
- preview hosts or preview ports
- fixed deployment domains
- tenant IDs, user IDs, or device IDs as long-term business defaults
- workspace paths or provider model names inside product UI logic
- locale strings inside components
- theme colors as scattered inline literals

### Configuration Precedence

For relay URL, access token, locale, theme, and feature visibility, always use:

1. explicit user choice
2. persisted client config
3. relay-provided `app-config`
4. safe fallback default

### Mobile Networking Rule

- Mobile clients must never assume `localhost` or `127.0.0.1` is a valid relay endpoint.
- Preview links must be created from returned server data or resolved config, never by concatenating fixed client-side hostnames.

### Completion Rule

No iteration is complete until:

- the code or docs changes are merged into the execution record
- validation is run
- the result is recorded in `PLAN.md`
- the result is also recorded in the iteration section below

## Shared Validation Baseline

Every iteration must validate at least:

```bash
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cargo test --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm run build
```

Frontend-heavy iterations must also validate:

- locale switch behavior
- light/dark/system theme behavior
- Android-width layout
- Tauri shell startup assumptions that depend on client config
- no hardcoded `localhost` or production URL assumptions

Documentation-only iterations may replace compile validation with file existence and content checks, but must record that this was a docs-only change.

## Iteration 0: Planning Baseline And Governance Reset

### Goal

Replace the old architecture-only planning record with a product-evolution execution baseline that can drive future implementation and verification.

### User-Visible Outcome

- The repository now has one authoritative roadmap in `PLAN.md`.
- The repository now has one detailed per-iteration specification file in `docs/iteration-specs.md`.
- The team has explicit anti-hardcoding and completion-recording rules.

### In Scope

- Rewrite `PLAN.md`.
- Add `docs/iteration-specs.md`.
- Preserve the completed foundation work from the previous architecture-governance phase in summarized form.
- Mark Iteration 0 as completed.
- Set the next actionable target to Iteration 1.
- Add update rules that require both files to be kept in sync after each completed stage.

### Out Of Scope

- No product UI or API implementation changes.
- No renaming of backend APIs.
- No feature work outside planning artifacts.

### Implementation Plan

- Convert `PLAN.md` into a concise product evolution file with direction, guardrails, iteration table, historical foundation summary, completion log, verification log, and risk/decision log.
- Create `docs/iteration-specs.md` with one section per iteration and explicit acceptance criteria.
- Write hard rules against hardcoded relay addresses, locale strings, theme handling, and deployment assumptions.
- Include exact completion-recording steps to be performed after every verified iteration.

### Acceptance Criteria

- Both files exist in the repo.
- `PLAN.md` names the current product direction and next execution target.
- `docs/iteration-specs.md` contains Iteration 0 through Iteration 11 with acceptance criteria.
- Both files explicitly forbid hardcoded addresses and configuration shortcuts.
- Both files explicitly require updating records after each completed stage.

### Validation

- Verify file existence.
- Verify `PLAN.md` contains the iteration overview.
- Verify `docs/iteration-specs.md` contains all iteration headings.

### Failure Or Rollback Conditions

- If the new plan drops the historical foundation record entirely, restore that history in summarized form before closing the iteration.
- If the detailed spec file does not cover every planned iteration, the iteration remains incomplete.

### Post-Completion Update Rules

- Record the completion date in `PLAN.md`.
- Record the verification command output summary in `PLAN.md`.
- Update this section with actual implementation and validation notes.

### Implementation Notes

- 2026-03-27: Replaced the old architecture-governance `PLAN.md` with a product-evolution execution plan.
- 2026-03-27: Added `docs/iteration-specs.md` and aligned it with the new plan.

### Validation Results

- 2026-03-27: documentation-only verification completed by confirming `PLAN.md` and `docs/iteration-specs.md` exist and contain iteration coverage.

### Deviations

- None.

### Follow-up Items

- Begin Iteration 1 from the session-first information architecture change set.

## Iteration 1: Session-First Product Information Architecture

### Goal

Restructure the control app around AI sessions so the product no longer looks like three parallel low-level consoles.

### User-Visible Outcome

- The homepage is centered on devices, AI sessions, and the current session workspace.
- Shell and Preview/Tunnel remain available but are clearly advanced tools.
- Product language consistently says `AI Session`.

### In Scope

- Replace task-centric UI copy with session-centric product language.
- Redesign the dashboard information architecture.
- Reorganize navigation and layout to emphasize device selection, session list, and session workspace.
- Move shell and preview/tunnel entry points out of the primary top-level panels.
- Update README and public app-facing product wording to match the session-first model.

### Out Of Scope

- No backend API renaming.
- No workspace browse or Git inspection delivery yet.
- No removal of shell or port-forward runtimes.

### Information Architecture Changes

- Primary layout becomes:
  - device rail or list
  - AI session list
  - session workspace
- Advanced tools move into:
  - device detail actions
  - session workspace secondary actions
  - a dedicated advanced tools drawer, sheet, or tab
- Health and provider state should support the session workflow rather than dominate the layout.

### Frontend Changes

- Introduce product-layer view models that map `TaskRecord` to `AiSessionView`.
- Rename page headings, buttons, and filters from `Task` to `AI Session`.
- Replace the current three-panel equal-weight dashboard with a session-first structure.
- Reduce visibility of raw transport labels.
- Preserve access to terminal and raw tunnel diagnostics through secondary surfaces.

### Relay And Agent Changes

- No wire-level API changes required.
- `app-config` may gain feature flags for advanced tool visibility.
- Capability handling stays compatible with the current relay and agent model.

### Configuration And Hardening Requirements

- Do not hardcode “default relay” product copy that assumes desktop-local operation.
- Do not hide advanced tools by deleting their data flows; only demote their UX surface.
- Do not derive product wording from enum names directly in the template; map through view models and i18n keys.

### Test Cases

- Select a device and create an AI session from the main flow.
- See AI session history without touching shell or port-forward views.
- Open terminal from an advanced tool path.
- Open preview/tunnel from an advanced tool path.
- Verify mobile-width layout preserves the session-first flow.

### Acceptance Criteria

- The homepage no longer presents `task`, `shell`, and `port-forward` as equal first-class modules.
- The primary CTA creates an AI session, not a raw task or raw shell.
- Users can still reach terminal and preview features without backend regression.
- No existing API routes or protocol field names are broken.

### Failure Or Rollback Conditions

- If the redesign hides shell or preview completely without a reachable advanced path, this iteration is incomplete.
- If the UI still visibly reads as three equal low-level consoles, this iteration is incomplete.

### Post-Completion Update Rules

- Record the exact pages and store/view-model changes in `Implementation Notes`.
- Record regression coverage for task creation, shell access, and preview access in `Validation Results`.

### Implementation Notes

- 2026-03-27: Changed the control-store default task scope to `selected_device` so AI sessions follow the currently selected device by default.
- 2026-03-27: Added task-selection synchronization to the selected device, matching the existing shell and port-forward selection behavior.
- 2026-03-27: Rebuilt `DashboardView.vue` around a session-first layout: devices in the sidebar, AI sessions in a dedicated list, and a session workspace as the primary panel.
- 2026-03-27: Moved terminal and preview-tunnel controls into a lower-priority advanced tools section instead of exposing them as equal top-level surfaces.
- 2026-03-27: Updated top-level README copy in both Chinese and English to describe the product as an AI-session-first remote development control plane.

### Validation Results

- 2026-03-27: `cd apps/vibe-app && npm run build` passed.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` passed.

### Deviations

- 2026-03-27: Backend API routes and shared protocol names were intentionally left unchanged; the session-first model is currently enforced in the product layer and documentation.
- 2026-03-27: No interactive manual UI smoke test was run against a live relay/agent pair in this turn.

### Follow-up Items

- Carry the new session-first layout into the Tailwind CSS and `shadcn-vue` migration in Iteration 2 instead of expanding legacy page-scoped CSS further.
- Replace the remaining hardcoded UI copy with i18n keys in Iteration 3.
- Replace the current dark-only style system with tokenized light/dark/system theme handling in Iteration 4.

## Iteration 2: Tailwind And Component System Foundation

### Goal

Replace the hand-maintained page CSS approach with a reusable, tokenized component system based on Tailwind CSS and `shadcn-vue`.

### User-Visible Outcome

- The app has a more consistent UI foundation.
- Common controls look and behave consistently across pages and platforms.
- Future features can be added without reintroducing large page-specific CSS blocks.

### In Scope

- Add Tailwind CSS tooling and configuration.
- Add `shadcn-vue`.
- Introduce design tokens for color, spacing, radius, typography, and elevation.
- Build shared primitives such as button, input, textarea, select, card, badge, sheet, dialog, dropdown, tabs, scroll area, and toast.
- Refactor the most central screens to use the new primitives.

### Out Of Scope

- No full visual redesign of every future screen.
- No second component library.
- No partial migration that leaves both new and old systems growing in parallel without a transition plan.

### Frontend Changes

- Add Tailwind config and content paths.
- Add a UI components directory and conventions.
- Introduce business components above the raw primitives for device cards, session rows, provider badges, and detail panels.
- Gradually collapse old `styles.css` responsibilities into tokens and component variants.

### Configuration And Hardening Requirements

- Do not hardcode raw colors directly into page components when tokens or component variants should be used.
- Do not introduce one-off utility soups for recurring patterns that should be promoted into reusable components.
- Do not break Tauri or Android builds by assuming a web-only bundler path.

### Test Cases

- Run the web build after Tailwind and component setup.
- Render the main dashboard in Web and Tauri.
- Inspect Android-width layout.
- Validate that base inputs, buttons, cards, and dialogs render correctly.

### Acceptance Criteria

- Shared component primitives exist and are used by the core screen.
- New UI work no longer depends on expanding `styles.css` as the primary styling system.
- Web, Tauri, and Android builds still succeed.

### Failure Or Rollback Conditions

- If the app uses Tailwind for a few components but still depends on growing page CSS for the main layout, this iteration is incomplete.
- If the component setup introduces platform-specific styling regressions, fix them before closing the iteration.

### Post-Completion Update Rules

- Record the initial component inventory.
- Record the old CSS that remains intentionally for compatibility or tokens only.

### Implementation Notes

- 2026-03-27: Installed Tailwind CSS v4 and `@tailwindcss/vite` into `apps/vibe-app`.
- 2026-03-27: Added an `@/*` import alias in `apps/vibe-app/tsconfig.json` and `apps/vibe-app/vite.config.ts`.
- 2026-03-27: Replaced the hardcoded Vite `/api` proxy target with environment-driven configuration using `VITE_DEV_RELAY_PROXY_TARGET` first and `VITE_RELAY_BASE_URL` as a fallback.
- 2026-03-27: Initialized `shadcn-vue`, generated `components.json`, and added `src/lib/utils.ts`.
- 2026-03-27: Added core UI primitives for button, card, badge, input, textarea, select, separator, and scroll-area under `src/components/ui`.
- 2026-03-27: Rebuilt the main dashboard with shadcn primitives and Tailwind utility classes instead of extending the old page-scoped CSS system.
- 2026-03-27: Reduced `src/styles.css` to global theme tokens, background styling, and base resets.

### Validation Results

- 2026-03-27: `cd apps/vibe-app && npm run build` passed.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` passed.

### Deviations

- 2026-03-27: The current dashboard still uses native `<select>` controls styled with Tailwind utilities instead of the generated shadcn select component.
- 2026-03-27: A manual live UI smoke test against a running relay and agent was not performed in this turn.

### Follow-up Items

- Use the new component and utility foundation to migrate all remaining user-visible copy into i18n keys in Iteration 3.
- Replace the temporary `.dark` wrapper introduced in `App.vue` with a theme runtime in Iteration 4.
- Continue collapsing any leftover page-specific styling into shared business components as the dashboard grows.

## Iteration 3: Full Internationalization Infrastructure

### Goal

Support full Chinese and English localization for all user-visible app copy.

### User-Visible Outcome

- Users can switch between Chinese and English.
- UI copy, status text, validation hints, and product labels all follow the selected locale.

### In Scope

- Add `vue-i18n`.
- Add `zh-CN` and `en` locale bundles.
- Add locale detection, persistence, and user override.
- Move all user-facing text into translation keys.
- Add mapping for status enums and product-facing error summaries.

### Out Of Scope

- No translation of raw provider stdout/stderr.
- No machine translation of arbitrary backend logs.

### Frontend Changes

- Add locale bootstrap during app startup.
- Add a locale store or runtime helper.
- Replace hardcoded UI strings in pages, forms, filters, state labels, and empty states.
- Add a language switcher in settings or top-level app chrome.

### Configuration And Hardening Requirements

- Do not derive translated strings by directly `replaceAll`-ing enum values in templates.
- Do not leave user-visible fallback strings hardcoded in components.
- Do not make locale behavior depend on platform-specific branching except where a system locale API differs.

### Test Cases

- Switch app locale from Chinese to English and back.
- Verify session, device, terminal, preview, and settings screens update correctly.
- Verify form placeholders, buttons, empty states, and status labels are localized.
- Verify persisted locale survives reload or app restart.

### Acceptance Criteria

- No user-visible hardcoded Chinese or English strings remain in core screens.
- Both `zh-CN` and `en` work across primary flows.
- Locale selection follows the documented precedence rules.

### Failure Or Rollback Conditions

- If major screens still contain mixed hardcoded strings, this iteration is incomplete.
- If locale switching requires refresh hacks or loses state unnecessarily, fix it before closure.

### Post-Completion Update Rules

- Record the locale key namespaces that were introduced.
- Record any intentionally deferred untranslated surfaces.

### Implementation Notes

- 2026-03-27: Added `vue-i18n` runtime bootstrap in `apps/vibe-app/src/main.ts` and moved route title handling to locale-aware `meta.titleKey` resolution in `apps/vibe-app/src/router.ts`.
- 2026-03-27: Added `apps/vibe-app/src/lib/i18n.ts` with locale detection, persistence, `zh-CN`/`en` support, and explicit `setAppLocale()` / `initializeLocale()` helpers.
- 2026-03-27: Introduced locale namespaces for `app`, `locale`, `common`, `connectionState`, `taskStatus`, `shellStatus`, `portForwardStatus`, `eventKind`, `stream`, `transport`, `auth`, `error`, and `dashboard`.
- 2026-03-27: Fully migrated `apps/vibe-app/src/views/DashboardView.vue` to translation keys, including relay/auth copy, dashboard hero text, device/session/workspace panels, advanced tools, empty states, status filters, event labels, and connection-state summaries.
- 2026-03-27: Added an in-app language switcher to the relay card so locale changes apply without refresh and without hardcoded platform-specific settings flows.
- 2026-03-27: Extended `apps/vibe-app/src/stores/control.ts` with an `errorCode` path for product-owned validation errors so client-visible errors do not rely on hardcoded English strings in the UI layer.

### Validation Results

- 2026-03-27: `cd apps/vibe-app && npm run build` passed after the dashboard i18n migration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after the i18n runtime and store changes.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` passed after the i18n runtime and store changes.

### Deviations

- 2026-03-27: Raw provider stdout/stderr content and backend-returned freeform error messages remain untranslated by design; only product-owned summaries, labels, and explicit client validation paths are localized.
- 2026-03-27: This turn did not include manual interactive locale-switch and persisted-locale smoke testing against live Web, Tauri, or Android clients.

### Follow-up Items

- Begin Iteration 4 by replacing the temporary `.dark` wrapper in `apps/vibe-app/src/App.vue` with a proper light/dark/system theme runtime.
- Run manual QA for locale switching and locale persistence on Web, Tauri, and Android before release.
- Keep all future screens on the same locale namespace discipline so new product flows do not reintroduce hardcoded copy.

## Iteration 4: Theme System And Night Mode

### Goal

Introduce a proper theme system with light, dark, and system modes.

### User-Visible Outcome

- Users can choose light mode, dark mode, or follow system.
- The app remains usable and visually coherent in all theme modes.

### In Scope

- Add theme runtime and persistence.
- Use class-based theme switching.
- Map component styling to semantic tokens.
- Add a theme switcher surface.
- Update core screens and states for both themes.

### Out Of Scope

- No dark-only fallback as the main product behavior.
- No one-off page theming outside the token system.

### Frontend Changes

- Add a theme store or runtime helper.
- Apply the theme class at app root.
- Convert hardcoded dark-root assumptions into semantic token usage.
- Adapt cards, forms, badges, dialogs, sheets, lists, empty states, and error banners.

### Configuration And Hardening Requirements

- Do not lock `:root` to dark mode.
- Do not scatter theme conditionals across page templates when component variants should handle them.
- Do not make system theme support web-only; Tauri and Android must persist and restore correctly.

### Test Cases

- Toggle between light, dark, and system.
- Verify theme survives reload and app restart.
- Verify session, device, preview, and advanced-tool UI states in both themes.
- Verify contrast and focus treatment on buttons, inputs, and links.

### Acceptance Criteria

- All primary screens render correctly in light and dark modes.
- Theme selection follows the documented precedence and persistence rules.
- No major UI path still depends on the old dark-only root styling.

### Failure Or Rollback Conditions

- If shallow theme support leaves dialogs or forms unreadable in one mode, the iteration is incomplete.
- If system mode behaves differently across Web, Tauri, and Android without a documented reason, the iteration is incomplete.

### Post-Completion Update Rules

- Record the token groups used for color semantics.
- Record any temporary theme gaps as explicit follow-ups.

### Implementation Notes

- 2026-03-27: Added `apps/vibe-app/src/lib/theme.ts` with `light` / `dark` / `system` theme modes, persisted user preference, system-theme detection, and a non-hardcoded app-config theme extractor for future server defaults.
- 2026-03-27: Updated `apps/vibe-app/src/main.ts` to initialize theme state before mounting and updated `apps/vibe-app/src/App.vue` to sync optional relay-provided theme defaults once `appConfig` becomes available.
- 2026-03-27: Removed the temporary dark-only root wrapper from `apps/vibe-app/src/App.vue` and replaced it with class-based theme handling on the document root.
- 2026-03-27: Retokenized `apps/vibe-app/src/styles.css` to support light and dark background atmospheres through semantic variables instead of a fixed dark gradient.
- 2026-03-27: Added an in-app theme switcher to `apps/vibe-app/src/views/DashboardView.vue` and migrated the dashboard’s primary surfaces, cards, banners, code blocks, and advanced-tool panels from hardcoded dark colors to semantic theme-aware classes.

### Validation Results

- 2026-03-27: `cd apps/vibe-app && npm run build` passed after the theme runtime and dashboard retokenization.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after the theme runtime and dashboard retokenization.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` passed after the theme runtime and dashboard retokenization.

### Deviations

- 2026-03-27: This turn did not include live manual theme QA across Web, Tauri, and Android, so contrast and system-theme switching were validated by implementation review plus compile/test coverage rather than interactive smoke.
- 2026-03-27: Native `<select>` controls remain in use on the current screen, but their surrounding surfaces now follow the shared theme tokens.

### Follow-up Items

- Begin Iteration 5 by adding workspace browsing and read-only file preview inside the session workspace.
- Run manual QA for light/dark/system switching, persisted theme restore, and Android-width layout before release.
- Continue promoting repeated dashboard view patterns into higher-level business components as new themed surfaces are added.

## Iteration 5: Workspace Browse And File Preview

### Goal

Let users inspect the workspace directly from the session workflow without dropping into terminal for basic file access.

### User-Visible Outcome

- Users can open a workspace tree and preview files.
- Users can inspect common text files and understand when a file cannot be previewed.

### In Scope

- Directory listing under working root.
- File metadata retrieval.
- Read-only text preview.
- Image or binary metadata summary.
- Large-file and non-previewable-file handling.
- Security bounds to the working root.

### Out Of Scope

- No file editing.
- No file sync.
- No arbitrary filesystem browsing outside the configured workspace boundary.

### Relay And Agent Changes

- Add explicit workspace browse endpoints.
- Add bounded agent-side file access routines.
- Surface meaningful, product-facing errors for forbidden paths and unreadable files.

### Frontend Changes

- Add a workspace tree panel within the session workspace.
- Add file preview and metadata panels.
- Add loading, empty, and unsupported states.

### Configuration And Hardening Requirements

- Do not implement workspace browsing by proxying raw shell commands as the primary feature path.
- Do not allow `..` traversal or path escaping outside the working root.
- Do not rely on client-side path validation alone.

### Test Cases

- Browse nested directories.
- Open a text file preview.
- Attempt to access a path outside working root.
- Open a large file and verify truncation/guarding behavior.
- Open a binary or image and verify metadata-only fallback.

### Acceptance Criteria

- Users can browse and preview workspace files from the main product flow.
- Security boundaries are enforced server-side or agent-side.
- Error and empty states are productized, not raw IO dumps.

### Failure Or Rollback Conditions

- If workspace access can escape the configured working root, the iteration is incomplete.
- If the feature only works through raw shell output parsing, the iteration is incomplete.

### Post-Completion Update Rules

- Record the final API shape and path-boundary rules.
- Record any file-size thresholds or preview truncation policies implemented.

### Implementation Notes

- 2026-03-27: Added explicit workspace protocol types to `crates/vibe-core/src/lib.rs`, including browse and preview requests, bounded result envelopes, and relay-agent claim/complete payloads.
- 2026-03-27: Added `apps/vibe-relay/src/workspace.rs` plus new protected routes in `apps/vibe-relay/src/main.rs` for `/api/workspace/browse`, `/api/workspace/preview`, `/api/devices/:device_id/workspace/claim-next`, and `/api/workspace/requests/:request_id/complete`.
- 2026-03-27: Implemented agent-side workspace polling and bounded file access in `apps/vibe-agent/src/workspace_runtime.rs`, including directory listing, text preview, binary detection, and truncation guardrails.
- 2026-03-27: Kept workspace requests in relay memory only instead of persisting them into the store file, because the queue is short-lived transport state rather than durable product data.
- 2026-03-27: Added a localized workspace browser and preview panel to `apps/vibe-app/src/views/DashboardView.vue`, wired through `apps/vibe-app/src/lib/api.ts`, `apps/vibe-app/src/types.ts`, and the bilingual locale bundles.
- 2026-03-27: Implemented the current preview policy as `128 KiB` max file read, default preview from line `1`, default line limit `200`, and hard maximum preview line limit `500`.

### Validation Results

- 2026-03-27: `cd apps/vibe-app && npm run build` passed after the workspace browser and preview integration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after the workspace API and agent runtime changes.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` passed after adding workspace path-boundary and preview tests.

### Deviations

- 2026-03-27: The delivered security boundary is the resolved session root, not the entire agent working root, so workspace browsing is intentionally stricter than the original high-level plan wording.
- 2026-03-27: This turn did not include live manual QA against a running relay/agent pair, so large-directory behavior and Android-width ergonomics still rely on implementation review plus compile/test coverage.

### Follow-up Items

- Begin Iteration 6 by adding explicit Git inspection and result-supervision surfaces inside the same session workspace.
- Run manual QA for large directories, binary-file previews, and mobile-width usability before release.
- Revisit whether future product requirements justify broadening browse scope from the session root to the whole working root.

## Iteration 6: Git Inspect And Session Supervision

### Goal

Give users enough Git context inside the session workspace to evaluate AI results quickly.

### User-Visible Outcome

- Users can see changed files, branch context, and summarized diff state.
- Users can understand if an AI session produced meaningful repo changes.

### In Scope

- Git status summary.
- Branch name.
- Changed files list.
- Recent commits.
- Diff summary or changed-file summary.
- Session-side summary composition.

### Out Of Scope

- No full diff editor.
- No merge conflict resolution workflow.
- No complete code review platform.

### Relay And Agent Changes

- Add explicit Git inspect routines.
- Normalize product-facing errors when Git is unavailable or the directory is not a repository.

### Frontend Changes

- Add Git summary surfaces in the session workspace.
- Connect changed files with the workspace tree when available.
- Highlight result summaries alongside session completion state.

### Configuration And Hardening Requirements

- Do not surface raw shell snippets as the main user experience.
- Do not assume every workspace is a Git repository.
- Do not hardcode branch names or repository-relative paths.

### Test Cases

- Show status in a clean repository.
- Show changed files after a session modifies files.
- Handle non-repository directories gracefully.
- Handle Git executable or permission failures gracefully.

### Acceptance Criteria

- Users can inspect repo change state from the session workspace.
- Git failures are understandable without opening terminal.
- The UX reinforces the AI-session-first workflow instead of creating a detached Git console.

### Failure Or Rollback Conditions

- If users must open terminal for basic changed-file visibility, this iteration is incomplete.
- If the feature breaks on non-Git workspaces without graceful fallback, this iteration is incomplete.

### Post-Completion Update Rules

- Record Git command coverage and unsupported cases.
- Record how Git summaries are connected to workspace browsing.

### Implementation Notes

- 2026-03-27: Added `GitInspect` capability and explicit Git-inspection protocol types to `crates/vibe-core/src/lib.rs`, including structured repo state, changed-file summaries, recent commits, staged/unstaged diff stats, and relay-agent claim/complete payloads.
- 2026-03-27: Added `apps/vibe-relay/src/git.rs` and new protected routes in `apps/vibe-relay/src/main.rs` for `/api/git/inspect`, `/api/devices/:device_id/git/claim-next`, and `/api/git/requests/:request_id/complete`, using the same explicit request queue pattern as workspace browsing.
- 2026-03-27: Implemented agent-side Git inspection in `apps/vibe-agent/src/git_runtime.rs` using direct `git` commands for `rev-parse`, `status --porcelain`, `diff --numstat`, and `log`, instead of scraping shell transcripts.
- 2026-03-27: Normalized non-repository and missing-Git situations into `not_repository` and `git_unavailable` response states so the product layer can render clear supervision states without forcing terminal fallback.
- 2026-03-27: Added a localized session-supervision card and Git inspect panel to `apps/vibe-app/src/views/DashboardView.vue`, including branch/upstream context, workspace-scoped changed files, recent commits, and direct handoff from changed files into the existing workspace file preview.
- 2026-03-27: Added a relay `app-config` feature flag named `session_git_inspect` and updated the agent capability advertisement so the new surface can evolve behind explicit product-layer gates instead of hardcoded UI assumptions.

### Validation Results

- 2026-03-27: `cd apps/vibe-app && npm run build` passed after the Git-inspect and supervision UI integration.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after the Git protocol, relay queue, and agent runtime changes.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` passed after adding agent Git tests for non-repository and changed-repository cases.

### Deviations

- 2026-03-27: Changed-file and diff summaries are scoped to the selected session workspace path when the session cwd is nested inside a larger repository, while branch/upstream and recent-commit metadata stay repo-wide for context.
- 2026-03-27: The delivered diff summary is intentionally aggregate-only, split into staged and unstaged line counts, and does not attempt a full patch viewer or exact final-review diff model.
- 2026-03-27: This turn did not include live manual QA against running Web, Tauri, or Android clients, so long changed-file lists and deleted-file preview ergonomics still need manual verification.

### Follow-up Items

- Begin Iteration 7 by productizing preview flows on top of the existing port-forward runtime instead of exposing raw tunnel management as the primary UX.
- Run manual QA for Git inspect refresh behavior, deleted-file handling, nested-session scope behavior, and Android-width layout before release.
- Consider file-level diff preview, approval workflow, and richer result-review affordances in a future post-Iteration-6 review round rather than expanding this iteration into a full code-review system.

## Iteration 7: Preview Productization

### Goal

Turn the raw port-forward capability into a product-facing preview flow that matches common remote development expectations.

### User-Visible Outcome

- Users think in terms of opening previews, not managing raw TCP forwards.
- Common local web previews are easier to start and inspect.

### In Scope

- Product rename from raw port-forward to preview.
- Preview-first launch points from session or workspace context.
- HTTP preview emphasis.
- Advanced raw tunnel details retained behind secondary surfaces.

### Out Of Scope

- No removal of the low-level tunnel runtime.
- No attempt to GUI-wrap every possible TCP troubleshooting path.

### Frontend Changes

- Build preview-focused cards and detail views.
- Hide raw networking details by default.
- Add session-context actions such as “Open Preview”.

### Relay And Agent Changes

- Reuse existing port-forward runtime where possible.
- Add metadata or helper fields if needed to support preview-oriented UX.

### Configuration And Hardening Requirements

- Do not synthesize preview URLs by assuming a fixed host or fixed relay domain.
- Do not remove access to raw tunnel data needed for debugging self-hosted setups.
- Do not assume HTTP is the only protocol when storing the underlying runtime state.

### Test Cases

- Start an HTTP preview from a running local service.
- Reopen or inspect an existing preview.
- Surface failures when the target service is not reachable.
- Reach raw tunnel diagnostics from the advanced path.

### Acceptance Criteria

- The default user path is preview-centric, not tunnel-centric.
- Raw tunnel capability remains available for advanced users.
- Preview addresses and links are derived from actual runtime or config data.

### Failure Or Rollback Conditions

- If preview only works by assuming localhost on the client, this iteration is incomplete.
- If advanced tunnel diagnostics disappear entirely, this iteration is incomplete.

### Post-Completion Update Rules

- Record how preview URLs are derived.
- Record what remains intentionally advanced-only.

### Implementation Notes

- 2026-03-27: Kept the existing relay and agent port-forward runtime intact and rebuilt the product layer around `Preview` terminology instead of expanding raw tunnel language further.
- 2026-03-27: Added preview-oriented launch controls to `apps/vibe-app/src/views/DashboardView.vue` inside the session workspace so users can start a preview from the current device/session context without dropping into the lower advanced panel first.
- 2026-03-27: Reworked the advanced preview card to show preview URLs and open-in-browser affordances first, while moving raw relay endpoint, target endpoint, transport, and protocol details into a secondary advanced-connection block.
- 2026-03-27: Derived preview URLs in the product layer from runtime relay host/port data plus the current relay context instead of hardcoding client-side hostnames or assuming localhost from the viewer device.
- 2026-03-27: Added mobile loopback warnings for preview URLs that resolve to loopback hosts, so the app surfaces misconfiguration explicitly instead of silently generating unusable links on Android or other remote clients.
- 2026-03-27: Updated the bilingual locale bundles so preview-specific language now says `Preview`, `service host`, `service port`, `preview URL`, and `advanced connection details` instead of centering `tunnel` terminology.

### Validation Results

- 2026-03-27: `cd apps/vibe-app && npm run build` passed after the preview-productization UI rewrite.
- 2026-03-27: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after validating the unchanged relay/agent runtime alongside the preview-focused frontend changes.
- 2026-03-27: `cargo test --workspace --all-targets -- --nocapture` passed after the preview-productization integration.

### Deviations

- 2026-03-27: The delivered preview URL is intentionally HTTP-first for common local web-preview use cases; raw relay/target transport details remain available below for non-HTTP or low-level troubleshooting cases.
- 2026-03-27: Preview URLs are still derived in the client product layer from returned runtime host/port data rather than being server-authored signed links or custom-domain preview addresses.
- 2026-03-27: This turn did not include live manual preview smoke testing against Web, Tauri, or Android clients, so browser-opening behavior and self-hosted forward-host ergonomics still need manual verification.

### Follow-up Items

- Begin Iteration 8 by adding notification and async workflow handling for session outcomes and preview-ready states.
- Run manual QA for preview opening, relay-forward-host misconfiguration warnings, and Android/Tauri browser handoff behavior before release.
- Revisit preview auth, TLS strategy, and hosted-compatible/custom-domain URL handling in later self-hosted and enterprise rounds instead of overloading this iteration.

## Iteration 8: Notification And Async Workflow

### Goal

Support asynchronous work by notifying users when AI sessions succeed, fail, or need attention.

### User-Visible Outcome

- Users can leave the app and still be brought back to the right context when something important happens.

### In Scope

- Notification model for session success, failure, needs-attention, and preview-ready events.
- Notification center or recent activity surface.
- Desktop and Android notification integration.
- Deep-link or route return to the relevant context.

### Out Of Scope

- No enterprise alert routing.
- No complicated notification rules engine.

### Frontend Changes

- Add activity or notification state.
- Add permission-request UX where platform APIs require it.
- Add links back into the relevant session or preview.

### Relay And Agent Changes

- Add notification-worthy event classification on top of session state transitions.
- Avoid overusing low-level event spam as notifications.

### Configuration And Hardening Requirements

- Do not hardcode platform notification behavior into shared UI logic.
- Do not emit duplicate notifications on reconnect or reload without deduplication logic.
- Do not couple notifications to one locale or one theme.

### Test Cases

- Finish a session and receive a success notification.
- Fail a session and receive a failure notification.
- Tap or open the notification and return to the correct session.
- Verify notification state survives app resume scenarios where appropriate.

### Acceptance Criteria

- Notifications are meaningful, sparse, and actionable.
- Desktop and Android both support the initial notification set.
- Notification text respects locale and theme settings where visible.

### Failure Or Rollback Conditions

- If notifications are just raw event spam, the iteration is incomplete.
- If clicking or opening a notification does not restore context, the iteration is incomplete.

### Post-Completion Update Rules

- Record notification event classes and deduplication behavior.
- Record platform-specific permission or routing caveats.

### Implementation Notes

- 2026-03-28: Added `apps/vibe-app/src/lib/notifications.ts` and store-level activity state so task success, failure, cancel, preview ready, and preview failure are elevated into sparse product notifications instead of raw event spam.
- 2026-03-28: Added an activity center to `apps/vibe-app/src/views/DashboardView.vue` with unread state, deduplication by resource/category fingerprint, and click-to-restore context actions that reopen the relevant task or preview.
- 2026-03-28: Wired system-notification delivery through the active runtime's Notification API when the current platform capability and relay-provided notification channels allow it, while always preserving an in-app activity surface.
- 2026-03-28: Kept notification copy localized by generating activity titles and descriptions through i18n keys instead of hardcoded English strings.

### Validation Results

- 2026-03-28: `cd apps/vibe-app && npm run build` passed after the activity center and notification runtime integration.
- 2026-03-28: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after the Iteration 8 integration.
- 2026-03-28: `cargo test --workspace --all-targets -- --nocapture` passed after the Iteration 8 integration.

### Deviations

- 2026-03-28: The delivered system notification path currently depends on Notification API support in the current runtime; there is no background push, server fanout, or OS-native bridge yet.

### Follow-up Items

- Fold future shell/audit-driven notifications into the same deduplicated activity model instead of introducing separate per-feature notification stacks.
- Revisit native notification bridges later if Tauri or Android packaging needs capabilities beyond the current runtime Notification API.

## Iteration 9: Platform Harmonization

### Goal

Make Web, Tauri Desktop, and Android behave like one product with platform-aware runtime differences, not three loosely related shells.

### User-Visible Outcome

- Core session workflows feel consistent across supported clients.
- Platform-specific differences are deliberate and documented.

### In Scope

- Shared capability matrix.
- Shared runtime rules for locale, theme, notifications, and URL handling.
- Responsive mobile workspace layout.
- Desktop-friendly workspace layout.
- Android-focused mobile polish on the core session flow.

### Out Of Scope

- No iOS delivery commitment in this iteration.
- No platform-specific feature forks that create different product models.

### Frontend Changes

- Separate layout primitives for desktop and mobile while sharing the same business components.
- Move platform checks into runtime utilities instead of scattered template logic.
- Tighten navigation patterns for narrow-width devices.

### Configuration And Hardening Requirements

- Do not write long-term platform branching directly in page components.
- Do not create Web-only assumptions for storage, locale, theme, or notification behavior.
- Do not use different product wording on different platforms.

### Test Cases

- Run the main session flow on Web.
- Run the main session flow in Tauri Desktop.
- Run the main session flow in Android-width layout.
- Verify locale, theme, and relay config persistence across platforms.

### Acceptance Criteria

- One core session workflow works coherently on all currently supported platforms.
- Platform-specific behavior is isolated to runtime helpers or capability adapters.
- Android is treated as a first-class control client, not only a packaging artifact.

### Failure Or Rollback Conditions

- If mobile remains a squeezed desktop grid, the iteration is incomplete.
- If platform-specific hacks spread across business components, the iteration is incomplete.

### Post-Completion Update Rules

- Record the platform capability matrix.
- Record any platform-specific limitations that remain intentionally unsupported.

### Implementation Notes

- 2026-03-28: Added `apps/vibe-app/src/lib/platform.ts` to centralize client-kind detection, explicit-remote relay preference, notification capability checks, and persisted-runtime-config capability checks.
- 2026-03-28: Replaced scattered mobile relay heuristics in the dashboard with shared runtime helpers and rendered the relay-provided platform capability matrix directly in the UI.
- 2026-03-28: Expanded the dashboard with a dedicated deployment/platform/governance tranche above the main workspace so narrow-width clients get critical configuration and activity context before entering the heavier session grid.
- 2026-03-28: Preserved one product model across Web and Tauri Desktop while keeping Android-focused relay guidance explicit through capability-driven copy rather than platform-specific page forks.

### Validation Results

- 2026-03-28: `cd apps/vibe-app && npm run build` passed after platform-helper and layout updates.
- 2026-03-28: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after platform harmonization changes.
- 2026-03-28: `cargo test --workspace --all-targets -- --nocapture` passed after platform harmonization changes.

### Deviations

- 2026-03-28: The active runtime currently distinguishes `web` and `tauri_desktop` directly; mobile-specific product guidance is capability-driven, but there is still no separate native-Android-only view hierarchy.

### Follow-up Items

- Revisit whether future Android packaging needs native-only navigation shells or offline/runtime background behavior beyond the current shared dashboard model.

## Iteration 10: Self-Hosted Server Productization

### Goal

Turn self-hosting from an engineering capability into a documented product path.

### User-Visible Outcome

- Operators can understand how to deploy and connect to a self-hosted relay without reading source code.
- Clients can identify deployment mode through metadata rather than guesswork.

### In Scope

- Formal deployment metadata in app config or related config surfaces.
- Clear separation of hosted-compatible and self-hosted deployment concerns.
- Better storage/auth/deployment documentation and abstractions.
- Reduced local-only assumptions in product copy and client behavior.

### Out Of Scope

- No full managed cloud control plane.
- No cluster orchestration or high-availability deployment framework.

### Relay And Agent Changes

- Add clearer deployment metadata exposure.
- Isolate storage/auth config boundaries.
- Preserve self-hosted defaults while allowing hosted-compatible evolution.

### Frontend Changes

- Surface deployment mode in settings or connection details.
- Adapt connection guidance for self-hosted scenarios.

### Configuration And Hardening Requirements

- Do not detect hosted versus self-hosted by string-matching domain names.
- Do not assume HTTP loopback defaults are valid for all clients.
- Do not hide deployment-critical configuration behind undocumented environment defaults.

### Test Cases

- Connect the app to a self-hosted relay using an explicit LAN or public URL.
- Verify deployment metadata renders correctly.
- Verify no client path hardcodes `localhost` as the primary mobile suggestion.

### Acceptance Criteria

- Self-hosted deployment becomes a documented, user-facing path.
- The app can reason about deployment mode through explicit metadata.
- Connection guidance is safe for mobile, desktop, and remote usage.

### Failure Or Rollback Conditions

- If self-hosted use still depends on reading source or guessing config precedence, the iteration is incomplete.
- If the client still assumes loopback for remote/mobile use, the iteration is incomplete.

### Post-Completion Update Rules

- Record deployment metadata semantics and config precedence.
- Record any remaining operator-only rough edges.

### Implementation Notes

- 2026-03-28: Extended `vibe-core::AppConfig` and relay config exposure with deployment metadata, storage kind, auth mode, current actor, notification channels, and platform matrix fields so clients can reason about deployment explicitly.
- 2026-03-28: Updated `apps/vibe-app/src-tauri/src/lib.rs` to emit the expanded app-config surface for local Tauri startup instead of relying on the old reduced config payload.
- 2026-03-28: Added deployment, auth, storage, and relay-origin surfaces to the dashboard so operators and users no longer need to infer self-hosted behavior from source or hidden defaults.
- 2026-03-28: Added [`docs/self-hosted.md`](./self-hosted.md) to document relay, agent, storage, preview-host, and configuration-precedence rules for self-hosted deployments.

### Validation Results

- 2026-03-28: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after app-config and Tauri self-hosted metadata changes.
- 2026-03-28: `cd apps/vibe-app && npm run build` passed after self-hosted UI/documentation integration.
- 2026-03-28: `cargo test --workspace --all-targets -- --nocapture` passed after the Iteration 10 tranche.

### Deviations

- 2026-03-28: `external` storage is surfaced as an explicit mode in config and UI, but it still maps to file-backed persistence until a later backing-store integration lands.

### Follow-up Items

- Add richer operator documentation later for TLS termination, reverse proxying, and high-availability topologies instead of overloading the current MVP self-hosted notes.

## Iteration 11: Enterprise Foundations

### Goal

Prepare the repository for enterprise evolution without rewriting the product later.

### User-Visible Outcome

- Personal mode still works, but the system no longer structurally assumes permanent single-user operation.
- Future team and enterprise features have a stable foundation.

### In Scope

- Tenant abstraction.
- User and membership abstraction.
- Basic role model.
- Audit logging for critical actions.
- Persistence abstraction beyond a personal-edition state-file assumption.
- Auth interfaces that can grow past a single access token.

### Out Of Scope

- No full organization-management UI.
- No billing system.
- No complete compliance program.

### Relay And Agent Changes

- Remove long-term dependence on hardcoded personal tenant and user assumptions.
- Introduce audit-worthy event capture.
- Prepare storage interfaces for future backing stores.

### Frontend Changes

- Keep personal mode UX simple.
- Prepare settings and account surfaces to consume future organization metadata without major redesign.

### Configuration And Hardening Requirements

- Do not simulate multi-tenancy with front-end-only filters.
- Do not preserve hardcoded `DEFAULT_TENANT_ID` and `DEFAULT_USER_ID` as permanent business logic.
- Do not add role checks as stringly typed UI-only conditions without backend support.

### Test Cases

- Verify personal mode still functions when multi-tenant abstractions are introduced.
- Verify audit records are created for core control-plane actions.
- Verify role model boundaries can be enforced at the service layer.

### Acceptance Criteria

- The system can evolve to team and enterprise modes without a data-model reset.
- Core actions are auditable.
- Personal mode remains the default fallback path.

### Failure Or Rollback Conditions

- If multi-tenant support exists only as naming without backend enforcement points, the iteration is incomplete.
- If audit logging misses critical session and access actions, the iteration is incomplete.

### Post-Completion Update Rules

- Record the final tenant, user, membership, role, and audit boundaries.
- Record what still remains enterprise-future work versus what became active product behavior.

### Implementation Notes

- 2026-03-28: Extended `vibe-core` with actor, tenant, user, membership, audit, deployment, notification, and platform metadata types and threaded the new app-config fields through relay and app consumers.
- 2026-03-28: Added relay governance seeding, tenant filtering, role-based read/write checks, audit recording, and `/api/audit/events` so control-plane operations are now attributable and tenant-scoped.
- 2026-03-28: Updated relay task, shell, and preview handlers to construct records from explicit actor identity instead of assuming permanent personal-mode user constants.
- 2026-03-28: Updated the agent to accept `VIBE_TENANT_ID` and `VIBE_USER_ID`, send explicit actor headers, and register devices without hardcoding `personal` / `local-admin`.
- 2026-03-28: Added governance and audit surfaces to the frontend so the current actor, tenant scope, notification channels, and recent audit events are visible in the product layer.

### Validation Results

- 2026-03-28: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` passed after tenant/audit/storage foundation changes.
- 2026-03-28: `cd apps/vibe-app && npm run build` passed after the enterprise-foundation UI and type updates.
- 2026-03-28: `cargo test --workspace --all-targets -- --nocapture` passed after the enterprise-foundation relay test updates and actor-aware handler changes.

### Deviations

- 2026-03-28: Identity is currently derived from relay defaults or explicit request headers, so this iteration establishes backend enforcement points and audit structure without claiming a finished enterprise authentication system.

### Follow-up Items

- Layer a real auth/session provider on top of the current actor boundaries in a future roadmap instead of reintroducing implicit personal-mode assumptions.
- Replace the `external` storage placeholder with a real external persistence implementation when enterprise storage requirements become active.
