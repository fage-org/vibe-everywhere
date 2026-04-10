# Archive

Completed and historical planning documents. These are read-only reference material.

## Directory Layout

- `wave8/` — Wave 8 planning documents (desktop preview baseline, superseded by Wave 9)
- `wave9/` — Wave 9 planning documents (unified replacement execution, complete)
- `completed-projects/` — Project-level plans for finished projects (vibe-wire through vibe-app-tauri)
- `completed-modules/` — Module-level plans for `[done]` implementation tasks
- `historical/` — Other superseded documents (parity audits, pre-Wave-9 release models)

## Rules

- **Read-only**: Do not modify archived files. If an archived module needs revisiting, create a new module plan in the active `modules/` tree.
- **Reference only**: These files document decisions and designs that were implemented. They remain useful for traceability but are not implementation targets.
- **No dispatch**: Never dispatch AI implementation against an archived plan.

## Cross-References

When archived files are referenced from active docs, paths should point into this `archive/` directory. If you find a stale reference pointing to a moved file, update it.

## Wave 9 Archive Contents

- `wave9/execution-plan.md` — Module-level sequencing for all active waves
- `wave9/execution-batches.md` — AI dispatch batches for all batches
- `wave9/master-summary.md` — High-level objectives, milestones, risks
- `wave9/master-details.md` — Architecture, dependency graph, acceptance gates
- `wave9/vibe-app-tauri-wave9-unified-replacement-plan.md` — Wave 9 execution-facing replacement plan
- `wave9/vibe-app-tauri-wave9-migration-and-release-plan.md` — Migration and release ownership stages
- `wave9/vibe-app-tauri-wave9-route-and-capability-matrix.md` — Route families and capability owners
- `completed-projects/vibe-app-tauri.md` — Project plan for the active Wave 9 replacement package