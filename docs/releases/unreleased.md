## Highlights

- Overlay smoke now accepts graceful task fallback and verifies automatic recovery back to overlay.
- Main CI now runs both `relay_polling` and `overlay` smoke coverage before release tagging.

## Included Iterations And Remediations

- Release verification alignment for overlay fallback and recovery behavior.

## Operator Notes

- No operator-facing configuration changes.

## Validation

- `bash -n scripts/dual-process-smoke.sh`
- `./scripts/dual-process-smoke.sh relay_polling`
- `./scripts/dual-process-smoke.sh overlay`
