# Vibe Everywhere Iteration Specs v6

Last updated: 2026-03-30

Version note:

- This file is the versioned detailed iteration plan for roadmap epoch `v6`.
- The concise lookup view lives in [`v6-summary.md`](./v6-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This epoch resets the product around the design baseline in
[`docs/ai_worktree_远程开发产品功能设计文档.md`](../../ai_worktree_%E8%BF%9C%E7%A8%8B%E5%BC%80%E5%8F%91%E4%BA%A7%E5%93%81%E5%8A%9F%E8%83%BD%E8%AE%BE%E8%AE%A1%E6%96%87%E6%A1%A3.md).
The goal is not to polish the current dashboard, but to replace it with a mobile-first remote AI
worktree product that keeps the object model stable across mobile and desktop:

1. server
2. host
3. project
4. conversation / task execution
5. code, Git, and runtime inspection

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record and decision log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level product and workflow guardrails

## Reset Mode

- Chosen implementation mode: `user-specified full reset`
- Meaning:
  the previous UI design, route structure, and page composition are not preserved as the product
  baseline. Reusable infrastructure may stay only when it remains structurally useful, but the
  shipped information architecture, copy, and feature layout must follow the new design document.

## Shared Guardrails

- The mobile experience is primary; desktop extends the same model instead of defining a separate
  object system.
- Do not collapse the new product back into one long mixed dashboard.
- Project detail must keep `conversation` primary and `changes/files/logs` secondary.
- Host and project context must remain continuously visible in the active workspace.
- Running AI work must show clear states: working, waiting for input, failed, stopped, completed.
- Code changes must stay modular. Do not reintroduce another monolithic app-wide control store or a
  page component that owns every workflow branch.
- Old UI files may be removed aggressively once the replacement path is in place.

## Roadmap Overview

| Iteration | Title | Status | Depends On |
| --- | --- | --- | --- |
| 16 | Product Reset And Information Architecture Rewrite | in_progress | Iteration roadmap `v5` |
| 17 | Mobile-First Host And Project Entry | planned | 16 |
| 18 | Project Workspace And ACP Conversation Experience | planned | 17 |
| 19 | Review, Logs, Notifications, And Safety Controls | planned | 18 |
| 20 | Desktop Workbench And Multi-Worktree Expansion | planned | 19 |

## Prioritized Remaining Backlog

1. Execution-policy and audit surfaces
   close the current gap where execution mode is visible but not yet backed by a fuller policy
   center; keep policy/audit visibility inside secondary project views instead of turning it into a
   top-level admin flow.
2. Conversation execution view refinement
   separate each turn into a clearer summary layer, event stream, and detailed output so long
   tasks remain understandable without reading raw event text end to end.
3. Multi-worktree lifecycle depth
   build on the current worktree create/list/reopen flow with safer cleanup, richer state hints,
   and additional management actions.
4. Host project discovery depth
   keep extending beyond the current working-root plus Git-worktree expansion model without
   regressing responsiveness or inventing an unbounded filesystem crawl.

## Iteration 16: Product Reset And Information Architecture Rewrite

### Goal

Replace the old control-console product framing, planning files, and frontend route skeleton with
the new AI worktree baseline.

### User-Visible Outcome

- Top-level docs describe the product as a remote AI worktree rather than a generic control plane.
- The app navigation shifts to `首页 / 项目 / 通知 / 我的`.
- Old dashboard routes and copy disappear from the default product surface.
- The first frontend replacement keeps server, host, project, and conversation context aligned.

### In Scope

- rewrite `README.md` and `README.en.md`
- rewrite `PLAN.md`, active plan index references, and iteration files
- update `TESTING.md` manual workflow expectations
- remove stale pointer docs outside the versioned planning structure
- replace the frontend route tree and primary page composition
- delete obsolete view code and deprecated design-specific helpers

### Acceptance Criteria

- versioned planning index points to `v6`
- top-level product docs reflect the AI worktree baseline
- mobile navigation is the default routing model
- old dashboard-first wording is removed from shipped UI
- frontend code is reorganized into feature modules instead of one app-wide page hierarchy
- shipped docs do not overstate incomplete host/project/review/security capabilities

### Validation

- `cd apps/vibe-app && npm run build`
- manual navigation pass over the new top-level tabs and project detail route

### Delivery Record

- Completed:
  product and plan narrative reset, removal of stale planning pointer docs, README correction so
  incomplete baseline items are not advertised as fully shipped, and manual regression guidance for
  the new AI worktree IA.
- Completed:
  project workspaces now honor a routed `conversationId` when entering from notifications, so the
  app can restore a specific conversation rather than only the project container.
- Completed:
  the mobile project list now merges history with a bounded host-side project inventory discovered
  from the agent working root, known-project revisit, and Git worktree expansion; project cards,
  workspace headers, and desktop trees now surface inventory availability so offline or
  temporarily unreachable projects remain visible instead of disappearing on refresh.
- Remaining before Iteration 16 can close:
  no additional Iteration 16 gaps remain; deeper inventory expansion continues under Iteration 17.

## Iteration 17: Mobile-First Host And Project Entry

### Goal

Deliver the first stable mobile entry flow from server connection to host and project selection.

### User-Visible Outcome

- 首页 shows current attention items:
  running AI work, recently opened projects, pending confirmations, and failure alerts.
- 项目页 supports browsing by host and project, plus search and quick filtering.
- 我的 aggregates server settings, appearance, language, and security-related preferences.
- The current server and host health remain visible without pushing infrastructure text into the
  center of the experience.

### Detailed Scope

- model the current relay as the active server context
- treat devices as hosts and group project records by `device + cwd`
- add recent-project persistence and home continuation cards
- add host summary cards with online state, provider availability, and running-task counts
- replace history-only project derivation with real host project inventory
- add project list search and filter states:
  all / running / recent / favorites-ready placeholder
- add empty states for:
  no server connected, no online hosts, no project history, no available providers
- ensure notification recall restores the exact project and conversation when that context exists

### Acceptance Criteria

- user can connect to a relay from `我的 > 服务器设置`
- user can return to 首页 and see host/project summaries without a management dashboard
- 项目页 shows grouped projects under hosts even before any prior conversation exists
- tapping a project opens the project detail workspace directly
- a notification can restore the exact target conversation inside its project workspace

### Validation

- manual narrow-width pass on mobile layout
- manual desktop pass confirming the same IA remains understandable

### Delivery Record

- Completed:
  host/project inventory is now retained as a bounded snapshot instead of a fragile one-pass scan:
  projects survive offline hosts and transient discovery failures, and UI surfaces availability
  states such as available, offline, unreachable, and history-only.
- Completed:
  project summaries now expose discovery source and retained verification context so the inventory
  model is explicit across project cards, workspace headers, and desktop project trees.
- Remaining before Iteration 17 can close:
  deeper host inventory expansion beyond the current working-root boundary and Git-worktree
  expansion is still not implemented.
- manual notification recall pass for both failed and waiting-input conversations

## Iteration 18: Project Workspace And ACP Conversation Experience

### Goal

Make project detail the primary work surface with conversation first and review tools adjacent.

### User-Visible Outcome

- project header always shows host, project, branch, and AI state
- default tab is `会话`
- remaining tabs are `变更 / 文件 / 日志`
- user can create a topic, continue a topic, answer provider prompts, stop a task, and inspect
  the latest related changes without leaving the project

### Detailed Scope

- project detail route with stable project identity
- topic list scoped to the current project
- message composer with prompt text, optional model override, and send action
- per-turn summaries that separate dialogue from machine events
- inline pending-input request card with option chips and custom text path
- AI-state summary strip:
  idle / running / waiting_input / failed / canceled / completed
- file-change summary tied to the latest task
- Git summary area:
  branch, changed file count, ahead/behind, recent commits
- workspace file browser:
  changed files first, then browsable tree
- runtime log tab with command/error emphasis

### Acceptance Criteria

- project opens on the conversation tab
- follow-up messages continue an existing conversation
- a new topic inherits the same host and project directory
- waiting-for-input states can be answered inline
- Git, files, and logs remain available as secondary tabs in the same route

### Validation

- `cargo check -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --workspace --all-targets -- --nocapture`
- `cd apps/vibe-app && npm run build`

### Delivery Record

- Completed:
  the `变更` tab now uses a review-first layout with Git scope summaries, file risk badges, and
  per-file diff inspection backed by a new relay/agent Git diff-file operation.
- Completed:
  the conversation composer now supports `只读 / 可改文件 / 可改并测试` execution modes, stores the
  selected mode on each task, forwards the mode into agent-side execution guidance, and adds a
  confirmation gate for obviously destructive prompts in writable modes.
- Completed:
  conversation turns now render as per-task cards with a clearer summary layer, recent execution
  events, and expandable raw event output instead of flattening all assistant and machine output
  into simple alternating chat bubbles.
- Completed:
  active task cards can now stop pending/assigned/running tasks directly from the conversation
  surface instead of forcing users back into secondary views just to cancel execution.
- Completed:
  completed or failed task cards now expose quick follow-up actions to retry the task or ask the
  AI to explain the previous result, reducing the need to manually restate common next-step prompts.
- Completed:
  completed and failed task cards now also expose direct navigation actions into `变更` and `日志`,
  so common review follow-up does not require users to manually switch secondary tabs after reading
  a task result.
- Remaining before Iteration 18 can close:
  no additional Iteration 18 gaps remain in the current v6 closure sequence.

## Iteration 19: Review, Logs, Notifications, And Safety Controls

### Goal

Close the personal-user loop from task completion to review, follow-up, and recall.

### User-Visible Outcome

- 通知页 becomes a first-class recall surface for:
  task completed, task failed, waiting confirmation, and test-result notifications
- project detail shows actionable review summaries before raw diffs/logs
- sensitive actions have explicit confirmation language and visible policy context

### Detailed Scope

- in-app notification center grouped by project and task outcome
- unread and recent sections
- quick actions from a notification:
  open conversation, open changes, open logs
- review summary cards on the `变更` tab
- per-file change risk labels and AI summary placeholder slots
- sensitive action confirmation model for:
  stop task, archive topic, rerun risky task, future destructive file actions
- audit trail visibility improvements in secondary views

### Acceptance Criteria

- user can return from a notification to the exact project workspace context
- failure and waiting-input states are visible from both 首页 and 通知
- review-first layout is present before raw diff/log detail

### Delivery Record

- Completed:
  the `日志` tab now surfaces error summaries before the full event stream and supports `全部 /
  错误 / 工具 / Provider` filtering for faster inspection.
- Completed:
  the 通知页 now includes a fuller notification-policy center with a global default preference,
  per-project overrides, unread/recent grouping, status filters, and mark-as-seen recall behavior
  instead of only a flat list plus project-level toggles.
- Completed:
  notification recall now restores the intended workspace tab as well as the project and
  conversation context, and each notification item exposes explicit quick actions for conversation,
  changes, and logs instead of relying only on one generic open-project entry point.
- Completed:
  project workspaces now load audit records for the active conversation tasks and show them ahead of
  raw runtime logs, giving the secondary inspection flow visible policy/audit context instead of
  only provider stdout/stderr.
- Completed:
  execution mode now starts to enforce real ACP-side boundaries in the agent runtime:
  `read_only` blocks file writes and terminal commands, while `workspace_write` blocks test-style
  terminal commands unless the task uses `workspace_write_and_test`.
- Completed:
  CLI provider enforcement is now aligned across the currently shipped CLI providers:
  `codex exec` uses explicit sandbox and approval flags for every execution mode, and Claude maps
  read-only sessions to native `plan` mode instead of inheriting a writable default.
- Completed:
  Claude read-only sessions now also carry a default disallowed-tools set for write and shell
  actions, so this CLI path no longer relies on `plan` mode alone to approximate read-only
  behavior.
- Completed:
  Claude workspace-write sessions now also carry a default disallowed-tools list for common
  test-style Bash commands, so the "edit but do not run tests yet" policy is no longer only a
  prompt-level instruction on that CLI path.
- Completed:
  the conversation surface now renders a user-visible "effective enforcement" summary for each
  task and the current composer mode, making ACP, Codex, Claude, and fallback policy differences
  visible without requiring users to infer them from runtime behavior alone.
- Completed:
  我的 now includes an editable policy-defaults area for execution mode, notification preference,
  and high-risk confirmation, plus a global audit trail view for recent task, shell, and preview
  events instead of only a read-only coverage summary.
- Remaining before Iteration 19 can close:
  no additional Iteration 19 gaps remain in the current v6 closure sequence.

### Validation

- `cd apps/vibe-app && npm run build`
- manual regression checklist in `TESTING.md`

## Iteration 20: Desktop Workbench And Multi-Worktree Expansion

### Goal

Extend the stable mobile object model into a higher-efficiency desktop workbench and prepare the
project model for future multi-worktree support.

### User-Visible Outcome

- desktop uses a true workbench layout:
  left tree, center conversation, right inspectors
- host and project switching become faster on wide screens
- project model is ready to support multiple worktrees under a single repository later

### Detailed Scope

- responsive desktop shell with persistent side panels
- server / host / project tree on desktop widths
- detachable or collapsible inspectors for Git, files, logs
- explicit project/worktree identity model extension
- groundwork for multiple active worktrees per project
- desktop-specific shortcuts and larger-screen diff ergonomics

### Acceptance Criteria

- wide-screen layout no longer behaves like a stretched mobile page
- the server/host/project hierarchy is visually explicit on desktop
- project identity can evolve from single active directory to multiple worktrees without route
  collapse

### Delivery Record

- Completed:
  wide-screen layouts now switch to a left-side app rail and a three-pane project workspace with
  host-project navigation, center conversation, and right-side inspectors.
- Completed:
  the desktop left rail inside project workspace now groups projects by host and path hierarchy
  instead of showing only a flat list for the current host.
- Completed:
  Git inspection and project summaries now carry shared-repository identity through
  `repoCommonDir`, allowing desktop project trees to group multiple worktrees under the same repo
  family instead of relying only on path proximity.
- Completed:
  Git inspection now returns a structured worktree list, and the project workspace header can show
  explicit worktree inventory instead of inferring everything from raw paths alone.
- Completed:
  desktop project workspaces can now create a sibling Git worktree on a new branch, with relay and
  agent support for a dedicated worktree-create request instead of read-only inventory display
  alone.
- Completed:
  the desktop sidebar can now list repository worktrees and directly reopen any worktree that has
  already been discovered as a project entry.
- Completed:
  project discovery now expands from the original working-root scan through Git-reported worktree
  paths, allowing newly created sibling worktrees to enter project inventory and desktop trees
  without requiring a deeper filesystem crawl.
- Completed:
  desktop project workspaces can now remove non-current sibling worktrees through relay and agent
  support for a dedicated worktree-remove request, and the refreshed project tree no longer keeps
  stale removed entries around.
- Completed:
  desktop worktree lists now surface richer lifecycle states such as current, detached, inventory
  missing, offline, unreachable, and remove-failed, and remove failures stay visible instead of
  being lost after the action returns.
- Remaining before Iteration 20 can close:
  no additional Iteration 20 gaps remain in the current v6 closure sequence.

### Validation

- `cd apps/vibe-app && npm run build`
- manual desktop regression checklist

## Implementation Record

- 2026-03-30:
  epoch `v6` created from the repository-owned AI worktree design baseline.
- 2026-03-30:
  mode recorded as `user-specified full reset`; prior dashboard-style UI is deprecated.
