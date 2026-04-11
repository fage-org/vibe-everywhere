# Module Plan: vibe-app-tauri/remote-operations-surfaces

## Status

- planned for Wave 10

## Purpose

Turn the current terminal, machine, server, and related helper routes into one explicit
remote-operations workflow.

## Source Of Truth

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
- `/root/happy/packages/happy-app/sources/app/(app)/terminal/**`
- `/root/happy/packages/happy-app/sources/app/(app)/machine/**`
- `/root/happy/packages/happy-app/sources/app/(app)/server.tsx`

## Target Location

- remote-helper routes and supporting state in `packages/vibe-app-tauri`

## Responsibilities

- terminal helper and terminal connect surfaces
- machine detail surfaces
- server configuration surfaces
- route relationships between session, machine, and terminal actions

## Non-Goals

- redesigning remote control around a new backend object model
- adding new backend RPC contracts unless planning proves they are required

## Dependencies

- `validation-and-customer-capability-contract`
- `settings-and-connection-center`

## Implementation Steps

1. Define the user workflow for remote operations across the current helper surfaces.
2. Reclassify each route as supported workflow step, limited helper, or internal utility.
3. Align navigation and copy so the route set reads as one product flow.
4. Record platform differences where a helper exists on one runtime but not another.

## Edge Cases And Failure Modes

- machine detail pages acting as isolated diagnostics instead of workflow steps
- terminal approval routes implying richer remote control than the app actually provides
- server configuration routes mixing product and operator-only concerns

## Tests

- route coverage for visible remote-operation surfaces
- workflow review from machine list/detail to session and terminal helper entry points

## Acceptance Criteria

- the remote-operation helper routes tell one coherent story
- customer-safe wording distinguishes supported remote actions from helper-only actions
- route visibility and platform scope match the resulting workflow definition

## Locked Decisions

- handoff and helper routes are valid only when their limited role is explicit
- session, machine, server, and terminal objects must be described consistently across the app
