## Highlights

- Windows CLI packaging and installer flow now preserve EasyTier runtime files as side-by-side
  dependencies instead of installing only `vibe-relay.exe`.

## Included Iterations And Remediations

- Remediation v6 R1: Windows EasyTier runtime packaging alignment across smoke validation, release
  archives, and the Windows relay installer.

## Operator Notes

- On Windows, keep the extracted CLI runtime files beside `vibe-agent.exe`.
- The Windows relay installer now copies `Packet.dll`, `wintun.dll`, and `WinDivert64.sys`
  alongside `vibe-relay.exe`, plus `WinDivert.dll` when it is present in the archive.

## Validation

- `cargo fmt --all`
- `cargo check --locked -p vibe-relay -p vibe-agent`
- `pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/stage-windows-runtime.ps1',[ref]$null,[ref]$null)"`
- `pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/dual-process-smoke.ps1',[ref]$null,[ref]$null)"`
- `pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/install-relay.ps1',[ref]$null,[ref]$null)"`
- `./scripts/render-release-notes.sh v0.0.0 >/dev/null`
