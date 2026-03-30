# Iteration Plan v6 Summary

Last updated: 2026-03-30

## Scope

Version 6 starts a new product epoch based on
[`docs/ai_worktree_远程开发产品功能设计文档.md`](../../ai_worktree_%E8%BF%9C%E7%A8%8B%E5%BC%80%E5%8F%91%E4%BA%A7%E5%93%81%E5%8A%9F%E8%83%BD%E8%AE%BE%E8%AE%A1%E6%96%87%E6%A1%A3.md).
The product model is reset from the current mixed control-console surface to a mobile-first remote
AI worktree product built around:

- server
- host
- project
- conversation / task execution
- code, Git, and logs as secondary inspectors

This epoch uses `user-specified` full-reset mode: the prior UI information architecture is retired
instead of being incrementally adjusted.

Full implementation detail lives in [`v6-details.md`](./v6-details.md).

## Status

| Iteration | Title | Status |
| --- | --- | --- |
| 16 | Product Reset And Information Architecture Rewrite | in_progress |
| 17 | Mobile-First Host And Project Entry | planned |
| 18 | Project Workspace And ACP Conversation Experience | planned |
| 19 | Review, Logs, Notifications, And Safety Controls | planned |
| 20 | Desktop Workbench And Multi-Worktree Expansion | planned |

## Current State

- The design baseline is now the repository-owned AI worktree product design document rather than
  the prior conversation-home tranche.
- The target primary mobile navigation becomes `首页 / 项目 / 通知 / 我的`.
- The target project surface becomes `会话 / 变更 / 文件 / 日志`, with conversation as the
  default tab.
- The target desktop surface becomes a three-pane workbench:
  `服务器/主机/项目树 + AI 会话 + 代码/Git/日志`.
- Existing dashboard-style UI, route naming, copy, and code structure are treated as deprecated
  and are being removed in this epoch instead of preserved as compatibility chrome.
- Iteration 16 is now split into:
  `16A 文档与计划纠偏` and `16B 手机端个人闭环补齐`.
- 16A is delivered in the current tranche:
  docs, plan files, and regression guidance now distinguish between the shipped frontend skeleton
  and the still-missing baseline capabilities.
- 16B is partially delivered in the current tranche:
  the app now restores a specific conversation when a notification routes into a project workspace,
  derives a bounded host project inventory from the working root plus known-project and Git
  worktree expansion, surfaces inventory availability in project summaries, and keeps
  first-conversation entry usable for inventory-only projects with no prior history.
- Iteration 17 inventory depth has now improved:
  host project inventory is retained across offline or unreachable refreshes, and project summaries
  now carry discovery source plus availability state instead of behaving like a throwaway scan.
- Iteration 18 has started with review-first change inspection:
  the `变更` tab now shows review summaries, file risk labels, and file-level Git diff output
  instead of only a changed-file list.
- Iteration 19 has started with task-safety affordances:
  the conversation composer now records execution mode per task, shows that mode in the transcript,
  and requires an extra confirmation step for obviously risky prompts in writable modes.
- Iteration 19 also now includes project-level notification preferences:
  users can choose per project whether the notification list shows all completions or only failed
  and waiting-input work.
- Iteration 19 project logs now also surface audit records for the active conversation tasks before
  raw runtime output, improving policy visibility inside the secondary inspection flow.
- Iteration 19 execution mode now also enforces ACP-side boundaries in the agent runtime:
  `只读` blocks writes and terminal commands, while `可改文件` blocks test-style terminal commands
  until the task opts into `可改并测试`.
- Iteration 19 CLI provider enforcement is now aligned for the currently shipped CLI providers:
  Codex uses explicit sandbox/approval flags across all modes, and Claude read-only sessions map
  to native `plan` mode instead of inheriting a writable permission default.
- Iteration 19 Claude read-only sessions now also ship with a default write/shell tool blacklist,
  tightening that CLI path beyond permission-mode-only guidance.
- Iteration 19 Claude workspace-write sessions now also carry a default test-command blacklist,
  tightening the CLI path beyond prompt-only guidance.
- Iteration 19 conversation UI now also shows an explicit effective-enforcement summary for each
  task and the current composer mode, so provider/runtime policy differences are visible in the
  primary conversation flow.
- Iteration 19 我的 now also includes editable policy defaults for execution mode, notification
  preference, and high-risk confirmation, plus a global audit trail view.
- Iteration 19 通知页 now also includes a fuller notification-policy center with a default
  preference, per-project overrides, unread/recent grouping, status filters, and seen-state recall.
- Iteration 19 notification items now also expose explicit recall actions for conversation,
  changes, and logs, and can restore the matching workspace tab instead of only reopening the
  project container.
- Iteration 18 conversation turns now also render as structured task cards with per-turn summaries,
  recent execution events, and expandable raw event output.
- Iteration 18 task cards can now also stop pending or running work directly from the conversation
  surface.
- Iteration 18 task cards now also expose quick follow-up actions for retrying failed work or
  asking the AI to explain the previous result.
- Iteration 18 task cards now also expose direct review jumps into `变更` and `日志` for completed
  or failed work.
- Iteration 20 has started with a true wide-screen shell:
  desktop now switches from the mobile bottom-nav shell to a left-side app rail, and project
  workspace uses a three-pane layout instead of a stretched single-column page.
- Iteration 20 now also includes the first explicit worktree lifecycle action:
  desktop project workspaces can create a sibling Git worktree on a new branch, then refresh the
  host/project tree against the updated shared-repository inventory.
- Iteration 20 desktop sidebars can now also list the current repository worktrees and directly
  reopen any worktree that has already entered project inventory.
- Iteration 20 project discovery now expands through Git-reported worktree paths, so newly created
  sibling worktrees can enter project inventory even when they were not part of the original
  top-level directory scan.
- Iteration 20 desktop worktree lists now also support removing non-current sibling worktrees and
  refreshing the project tree afterward.
- Iteration 20 desktop worktree lists now also surface richer lifecycle states and keep
  remove-failure context visible.
- Still missing against the baseline document:
  deeper project discovery beyond the current host working-root scan.

## Lookup Notes

- Need the detailed phased implementation plan and acceptance criteria:
  read [`v6-details.md`](./v6-details.md).
- Need the prior roadmap epoch for historical reference:
  read [`v5-summary.md`](./v5-summary.md).
- Need the active remediation track:
  read [`../remediation/v12-summary.md`](../remediation/v12-summary.md).
