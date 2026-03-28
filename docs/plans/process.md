# Vibe Everywhere Planning And Remediation Process

Last updated: 2026-03-28

## Purpose

This file defines the mandatory process for all future roadmap iterations and problem-remediation
work in this repository.

These rules are project governance, not optional suggestions.

## Plan Structure

Every active planning track must use versioned files under `docs/plans/`:

- one `summary` file for concise lookup
- one `details` file for full implementation guidance

The allowed pattern is:

- `docs/plans/iterations/vN-summary.md`
- `docs/plans/iterations/vN-details.md`
- `docs/plans/remediation/vN-summary.md`
- `docs/plans/remediation/vN-details.md`

Compatibility pointer files may remain at older locations, but they must stay short and redirect to
the versioned files.

## Summary Versus Details

`summary` files must stay concise and query-friendly. They should contain:

- active scope
- status table
- dependencies
- current target
- short acceptance summary

`details` files may be long. They should contain:

- goals
- problem statements
- implementation shape
- repair or iteration modes
- acceptance criteria
- validation rules
- completion records

## When To Create A New Version

Create a new `vN+1` plan set when any of the following happens:

- the current plan pair is no longer practical to scan quickly
- a completed phase gives way to a new phase with different goals
- historical context would become confusing if new work were appended to the old version

Do not rewrite an old version into a new phase. Start a new version and keep the old one as
history.

## Mandatory Update Flow

Before implementation:

1. identify whether the work belongs to iteration planning or remediation planning
2. locate the active version in [`README.md`](./README.md)
3. update the active `summary` and `details` files if the scope or status changed
4. update [`../../PLAN.md`](../../PLAN.md) if the active execution track changed

After implementation and verification:

1. update the active `summary` file
2. update the active `details` file
3. update [`../../PLAN.md`](../../PLAN.md)
4. update [`../../AGENTS.md`](../../AGENTS.md) if a new long-term rule was discovered
5. update docs or tests if the user-facing model changed
6. if the primary user-facing model changed, update [`../../README.md`](../../README.md),
   [`../../README.en.md`](../../README.en.md), and the manual checklist in
   [`../../TESTING.md`](../../TESTING.md) before closing the item
7. if the change affects developer onboarding, source-build instructions, or contributor entry
   points, update [`../../DEVELOPMENT.md`](../../DEVELOPMENT.md) before closing the item
8. if the work affects a shipped release, release packaging, release notes, or operator deployment
   flow, update the next-release note source under `docs/releases/` before closing the item
9. after pushing to GitHub, monitor the triggered GitHub Actions runs and do not consider the task
   delivered until the relevant workflows are either green or have a clearly documented failure
   diagnosis

Relevant workflow rule:

- a `main` push requires monitoring the `CI` workflow
- a pushed release tag such as `vX.Y.Z` requires monitoring both `CI` and `Release`
- if a workflow fails or behaves abnormally, capture the run URL, failing job, final conclusion,
  and next action in the delivery report
- release asset changes must also verify that published asset names remain versioned and that the
  release body is sourced from repository-owned notes rather than GitHub auto-generation alone

## Remediation Approval Rule

Every remediation item must define repair modes.

Before coding a remediation item:

1. present the available repair modes to the user
2. state the recommended mode
3. ask the user which mode to use
4. wait for that item-level choice before implementation

This is mandatory even when the overall remediation track is already approved.

Exception:

- if the user has already explicitly specified the repair shape, target structure, or delivery mode
  in a way that clearly maps to one repair mode, record that mode as `user-specified` and proceed
  without re-asking the same choice

## Iteration Approval Rule

For a new product iteration or roadmap phase:

1. define the scope in the active iteration `summary` and `details`
2. document acceptance criteria and validation
3. only then begin implementation

If the current plan set is already complete, start a new version rather than silently appending
new unrelated work.

## Guardrail Sync Rule

If a remediation or iteration uncovers a rule that should permanently constrain future work, add it
to [`../../AGENTS.md`](../../AGENTS.md) before the item is considered complete.

Examples include:

- UI information-architecture rules
- loopback/public-origin rules
- visibility-gating rules
- platform-surfacing rules
- documentation/update workflow rules
- verification/test-integrity rules
- release-packaging and release-notes workflow rules

## Documentation Surface Rule

- `README.md` and `README.en.md` are user/operator surfaces first.
- Do not write internal governance, anti-hardcoding requirements, release workflow mandates, or
  project-management instructions into the top-level README files.
- Contributor setup, local build commands, and source-development workflow belong in
  [`../../DEVELOPMENT.md`](../../DEVELOPMENT.md) or another dedicated developer document.
- Operator docs may explain runtime inputs and deployment steps, but they should stay user-facing
  and operational rather than governance-heavy.

## Workflow Cache Rule

- CI and release cache changes must use invalidation inputs tied to lockfiles, dependency manifests,
  or explicit tool versions.
- Prefer bounded caches such as npm, Rust, or Gradle dependency caches over broad SDK-directory
  snapshots unless the invalidation and failure mode are explicit.
- When cache scope or invalidation changes materially, reflect that in the active remediation or
  iteration details file so future updates do not silently degrade build reliability.
