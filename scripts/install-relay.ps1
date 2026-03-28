[CmdletBinding()]
param(
  [string]$ReleaseTag = $env:VIBE_RELEASE_TAG,
  [string]$ArchiveUrl = $env:RELAY_CLI_ARCHIVE_URL,
  [string]$ArchivePath = $env:RELAY_CLI_ARCHIVE_PATH,
  [string]$InstallDir = "C:\Program Files\Vibe Everywhere",
  [string]$ConfigDir = "$env:ProgramData\Vibe Everywhere",
  [string]$TaskName = "VibeRelay",
  [string]$RelayBindHost = $(if ($env:RELAY_BIND_HOST) { $env:RELAY_BIND_HOST } else { "0.0.0.0" }),
  [int]$RelayPort = $(if ($env:RELAY_PORT) { [int]$env:RELAY_PORT } else { 8787 }),
  [string]$PublicRelayBaseUrl = $env:RELAY_PUBLIC_BASE_URL,
  [string]$RelayForwardHost = $env:RELAY_FORWARD_HOST,
  [string]$RelayAccessToken = $env:RELAY_ACCESS_TOKEN,
  [string]$RelayDeploymentMode = $(if ($env:RELAY_DEPLOYMENT_MODE) { $env:RELAY_DEPLOYMENT_MODE } else { "self_hosted" }),
  [switch]$SkipStartupTask
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Test-IsAdministrator {
  $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $principal = [Security.Principal.WindowsPrincipal]::new($identity)
  return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Get-LatestReleaseTag {
  $release = Invoke-RestMethod -Uri "https://api.github.com/repos/fage-ac-org/vibe-everywhere/releases/latest"
  return $release.tag_name
}

function Escape-SingleQuotedLiteral {
  param([string]$Value)
  return $Value -replace "'", "''"
}

function Copy-ArchiveFile {
  param(
    [Parameter(Mandatory = $true)]
    [string]$ExtractDir,
    [Parameter(Mandatory = $true)]
    [string]$InstallDir,
    [Parameter(Mandatory = $true)]
    [string]$FileName,
    [switch]$Required
  )

  $sourcePath = Join-Path $ExtractDir $FileName
  if (-not (Test-Path $sourcePath)) {
    if ($Required) {
      throw "Required file missing from Windows CLI archive: $FileName"
    }

    return
  }

  Copy-Item -Path $sourcePath -Destination (Join-Path $InstallDir $FileName) -Force
}

if (-not (Test-IsAdministrator)) {
  throw "This installer writes to Program Files and scheduled tasks. Run it in an elevated PowerShell session."
}

if (-not $ReleaseTag -and -not $ArchiveUrl -and -not $ArchivePath) {
  $ReleaseTag = Get-LatestReleaseTag
}

if (-not $RelayForwardHost -and $PublicRelayBaseUrl) {
  $RelayForwardHost = ([uri]$PublicRelayBaseUrl).Host
}

$downloadPath = $ArchivePath
if (-not $downloadPath) {
  if (-not $ArchiveUrl) {
    $assetName = "vibe-everywhere-cli-$ReleaseTag-x86_64-pc-windows-msvc.zip"
    $ArchiveUrl = "https://github.com/fage-ac-org/vibe-everywhere/releases/download/$ReleaseTag/$assetName"
  }

  $downloadPath = Join-Path $env:TEMP "vibe-everywhere-cli.zip"
  Invoke-WebRequest -Uri $ArchiveUrl -OutFile $downloadPath
}

$extractDir = Join-Path $env:TEMP ("vibe-everywhere-cli-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $extractDir | Out-Null
Expand-Archive -Path $downloadPath -DestinationPath $extractDir -Force

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $ConfigDir | Out-Null

$relayExePath = Join-Path $InstallDir "vibe-relay.exe"
Copy-ArchiveFile -ExtractDir $extractDir -InstallDir $InstallDir -FileName "vibe-relay.exe" -Required
foreach ($runtimeFile in @("Packet.dll", "wintun.dll", "WinDivert64.sys")) {
  Copy-ArchiveFile -ExtractDir $extractDir -InstallDir $InstallDir -FileName $runtimeFile -Required
}
Copy-ArchiveFile -ExtractDir $extractDir -InstallDir $InstallDir -FileName "WinDivert.dll"

$stateDir = Join-Path $ConfigDir "state"
New-Item -ItemType Directory -Force -Path $stateDir | Out-Null
$stateFile = Join-Path $stateDir "relay-state.json"

$envScriptPath = Join-Path $InstallDir "relay-env.ps1"
$launcherScriptPath = Join-Path $InstallDir "Start-VibeRelay.ps1"

$envLines = @(
  ('$env:VIBE_RELAY_HOST = ''{0}''' -f (Escape-SingleQuotedLiteral $RelayBindHost)),
  ('$env:VIBE_RELAY_PORT = ''{0}''' -f $RelayPort),
  ('$env:VIBE_RELAY_DEPLOYMENT_MODE = ''{0}''' -f (Escape-SingleQuotedLiteral $RelayDeploymentMode)),
  ('$env:VIBE_RELAY_STATE_FILE = ''{0}''' -f (Escape-SingleQuotedLiteral $stateFile))
)

if ($PublicRelayBaseUrl) {
  $envLines += ('$env:VIBE_PUBLIC_RELAY_BASE_URL = ''{0}''' -f (Escape-SingleQuotedLiteral $PublicRelayBaseUrl))
}

if ($RelayForwardHost) {
  $envLines += ('$env:VIBE_RELAY_FORWARD_HOST = ''{0}''' -f (Escape-SingleQuotedLiteral $RelayForwardHost))
}

if ($RelayAccessToken) {
  $envLines += ('$env:VIBE_RELAY_ACCESS_TOKEN = ''{0}''' -f (Escape-SingleQuotedLiteral $RelayAccessToken))
}

$envLines | Set-Content -Path $envScriptPath -Encoding UTF8

$launcherContent = @"
. "`$PSScriptRoot\relay-env.ps1"
`$relayExe = Join-Path `$PSScriptRoot "vibe-relay.exe"
Start-Process -FilePath `$relayExe -WorkingDirectory "$stateDir" -NoNewWindow -Wait
"@
Set-Content -Path $launcherScriptPath -Value $launcherContent -Encoding UTF8

if (-not $SkipStartupTask) {
  $action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument ('-NoProfile -ExecutionPolicy Bypass -File "{0}"' -f $launcherScriptPath)
  $trigger = New-ScheduledTaskTrigger -AtStartup
  $principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

  Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Principal $principal -Force | Out-Null
  Start-ScheduledTask -TaskName $TaskName
}

Remove-Item -Path $extractDir -Recurse -Force

Write-Host "Installed vibe-relay.exe to $relayExePath"
Write-Host "Installed Windows runtime files beside vibe-relay.exe"
Write-Host "Environment script: $envScriptPath"
Write-Host "Launcher script: $launcherScriptPath"
if (-not $SkipStartupTask) {
  Write-Host "Startup task: $TaskName"
}
if (-not $PublicRelayBaseUrl) {
  Write-Warning "No PublicRelayBaseUrl was provided. Set VIBE_PUBLIC_RELAY_BASE_URL in $envScriptPath before using remote/mobile clients."
}
