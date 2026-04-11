# Module Plan: vibe-app-tauri/inbox-and-notification-closure

## Status

- planned for Wave 10

## Purpose

Define a real product contract for inbox, feed, unread state, local alerts, and notification-driven
navigation.

## Source Of Truth

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
- `/root/happy/packages/happy-app/sources/components/InboxView.tsx`
- `/root/happy/packages/happy-app/sources/utils/notificationRouting.ts`
- `/root/happy/packages/happy-app/sources/sync/pushRegistration.ts`

## Target Location

- inbox routes, notification helpers, and supporting state in `packages/vibe-app-tauri`

## Responsibilities

- classify inbox items versus feed items versus notifications
- define unread semantics
- define local-notification and platform-notification scope
- define event source taxonomy for session, relationship, artifact, terminal, and system events

## Non-Goals

- implementing Android push as a hidden side effect without explicit planning
- claiming a complete notification system from local desktop notices alone

## Dependencies

- `validation-and-customer-capability-contract`

## Implementation Steps

1. Build the Wave 10 taxonomy for inbox, feed, and notification objects.
2. Define supported notification sources and platform scope.
3. Decide which current events stay user-visible and which remain internal.
4. Align copy, route structure, and validation to the resulting taxonomy.

## Edge Cases And Failure Modes

- feed and notification concepts collapsing into one generic list
- unread state drifting across session and non-session events
- Android support being implied without real delivery semantics

## Tests

- inbox route coverage
- unread/read-state tests for supported item types
- notification source classification review

## Acceptance Criteria

- inbox and notification behavior is described by one coherent contract
- active docs can explain supported notification behavior without hand-waving
- platform differences for notification delivery and routing are explicit

## Locked Decisions

- notification taxonomy must be product-first, not implementation-first
- unsupported delivery mechanisms must be described as unsupported, not postponed implicitly
