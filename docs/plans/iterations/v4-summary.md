# Iteration Plan v4 Summary

Last updated: 2026-03-29

## Scope

Version 4 starts the next product-shaping epoch after workflow and release verification
normalization. This version focuses on making the control app feel more like a coherent product
again by compressing the everyday user journey into a session-first primary flow.

This version covers:

- moving relay connection inputs into the primary `Sessions` workflow
- keeping device selection, session launch, and result review on the same route
- demoting deployment metadata, governance context, and advanced tools into secondary views
- updating docs, testing guidance, and release notes to match the new primary workflow model

Full implementation detail lives in [`v4-details.md`](./v4-details.md).

## Status

| Iteration | Title | Status |
| --- | --- | --- |
| 14 | Session-First Primary Workflow Productization | completed |

## Current State

- Iteration 14 is complete locally after consolidating relay connection, device selection, session
  launch, event review, workspace browsing, and Git supervision into the `Sessions` route.
- `Devices` remains available as a secondary management surface and now carries deployment metadata,
  current-client runtime information, and governance context when feature-flagged.
- `Advanced` remains the exception path for terminal and preview workflows rather than a required
  part of the everyday user journey.
- README, testing guidance, release notes, planning indexes, and repository guardrails now reflect
  the new session-first primary workflow model.

## Lookup Notes

- Need the detailed acceptance criteria or implementation notes:
  read [`v4-details.md`](./v4-details.md).
- Need the previous workflow-normalization epoch:
  read [`v3-summary.md`](./v3-summary.md).
- Need the active remediation track:
  read [`../remediation/v10-summary.md`](../remediation/v10-summary.md).
