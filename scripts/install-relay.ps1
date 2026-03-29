[CmdletBinding()]
param(
  [ValidateSet("install", "update", "uninstall", "help")]
  [string]$Command = "help",
  [ValidateSet("all", "relay", "agent")]
  [string]$Component = $(if ($env:INSTALL_COMPONENT) { $env:INSTALL_COMPONENT } else { "all" }),
  [string]$InstallDir = $(if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "C:\Program Files\Vibe Everywhere" }),
  [string]$ReleaseTag = $env:VIBE_RELEASE_TAG,
  [string]$ArchiveUrl = $(if ($env:VIBE_CLI_ARCHIVE_URL) { $env:VIBE_CLI_ARCHIVE_URL } elseif ($env:RELAY_CLI_ARCHIVE_URL) { $env:RELAY_CLI_ARCHIVE_URL } else { "" }),
  [string]$ArchivePath = $(if ($env:VIBE_CLI_ARCHIVE_PATH) { $env:VIBE_CLI_ARCHIVE_PATH } elseif ($env:RELAY_CLI_ARCHIVE_PATH) { $env:RELAY_CLI_ARCHIVE_PATH } else { "" }),
  [string]$RepoOwner = $(if ($env:REPO_OWNER) { $env:REPO_OWNER } else { "fage-ac-org" }),
  [string]$RepoName = $(if ($env:REPO_NAME) { $env:REPO_NAME } else { "vibe-everywhere" }),
  [string]$GhProxy = $(if ($env:GH_PROXY) { $env:GH_PROXY } else { "https://ghfast.top/" }),
  [switch]$NoGhProxy
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Show-Help {
  @"
Vibe Everywhere CLI Binary Installer

Usage:
  .\install-relay.ps1 -Command <install|update|uninstall|help> [options]

Options:
  -Component <all|relay|agent>
                           Which binaries to manage (default: all)
  -InstallDir <path>       Target directory for installed binaries
  -ReleaseTag <tag>        Install a specific release tag, for example v0.1.9
  -ArchiveUrl <url>        Download from a custom archive URL
  -ArchivePath <path>      Install from a local archive path
  -GhProxy <url>           Prefix applied to GitHub release and redirect URLs
  -NoGhProxy               Disable the GitHub proxy prefix
  -RepoOwner <owner>       Override the GitHub repository owner
  -RepoName <name>         Override the GitHub repository name

Examples:
  .\install-relay.ps1 -Command install
  .\install-relay.ps1 -Command install -Component relay
  .\install-relay.ps1 -Command install -Component agent
  .\install-relay.ps1 -Command update -Component all -ReleaseTag v0.1.9
  .\install-relay.ps1 -Command uninstall -Component agent
  .\install-relay.ps1 -Command install -ArchivePath C:\Temp\vibe-everywhere-cli.zip

Notes:
  This script manages the published CLI binaries from the release archive.
  On Windows, the default install location is C:\Program Files\Vibe Everywhere, which follows the
  normal machine-wide application directory convention.
  It does not create services, write environment files, or start relay or agent processes.
"@
}

function Write-Info {
  param([string]$Message)
  Write-Host $Message -ForegroundColor Cyan
}

function Resolve-ProxiedUrl {
  param([Parameter(Mandatory = $true)][string]$Url)

  if ($NoGhProxy) {
    return $Url
  }

  return "$GhProxy$Url"
}

function Get-LatestReleaseTag {
  $latestUrl = Resolve-ProxiedUrl -Url "https://github.com/$RepoOwner/$RepoName/releases/latest"
  $response = Invoke-WebRequest -Uri $latestUrl -MaximumRedirection 5
  $finalUrl = $response.BaseResponse.ResponseUri.AbsoluteUri
  $match = [regex]::Match($finalUrl, "/releases/(?:tag|download)/([^/?#]+)")
  if (-not $match.Success) {
    throw "Failed to resolve the latest release tag from $finalUrl"
  }

  return $match.Groups[1].Value
}

function Build-ArchiveUrl {
  param(
    [Parameter(Mandatory = $true)][string]$Tag,
    [Parameter(Mandatory = $true)][string]$ArchiveName
  )

  $directUrl = "https://github.com/$RepoOwner/$RepoName/releases/download/$Tag/$ArchiveName"
  return Resolve-ProxiedUrl -Url $directUrl
}

function Get-SelectedBinaryNames {
  switch ($Component) {
    "all" {
      return @("vibe-relay.exe", "vibe-agent.exe")
    }
    "relay" {
      return @("vibe-relay.exe")
    }
    "agent" {
      return @("vibe-agent.exe")
    }
    default {
      throw "Unsupported component: $Component"
    }
  }
}

function Get-AllBinaryNames {
  return @("vibe-relay.exe", "vibe-agent.exe")
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

function Print-PostInstallNotes {
  param([Parameter(Mandatory = $true)][string[]]$InstalledBinaryPaths)

  $docsUrl = "https://github.com/$RepoOwner/$RepoName/blob/main/docs/relay-startup.md"
  $readmeUrl = "https://github.com/$RepoOwner/$RepoName/blob/main/README.en.md"

  Write-Host "Installed CLI binaries into $InstallDir"
  foreach ($binaryPath in $InstalledBinaryPaths) {
    Write-Host "Installed: $binaryPath"
  }
  Write-Host "Relay startup guide: $docsUrl"
  Write-Host "General usage guide: $readmeUrl"
  if (-not $NoGhProxy) {
    Write-Host ("Relay startup guide (accelerated): {0}" -f (Resolve-ProxiedUrl -Url $docsUrl))
    Write-Host ("General usage guide (accelerated): {0}" -f (Resolve-ProxiedUrl -Url $readmeUrl))
  }
  Write-Warning "This script only installs or updates binaries. Configure environment variables and startup separately."
}

function Install-OrUpdateCli {
  param([Parameter(Mandatory = $true)][string]$Mode)

  $selectedBinaryNames = @(Get-SelectedBinaryNames)
  if ($Mode -eq "update") {
    $existingBinaryCount = @(
      $selectedBinaryNames | Where-Object {
        Test-Path (Join-Path $InstallDir $_)
      }
    ).Count
    if ($existingBinaryCount -eq 0) {
      throw "No selected CLI binaries were found under $InstallDir. Run install first or choose a different -InstallDir."
    }
  }

  $resolvedReleaseTag = $ReleaseTag
  if (-not $resolvedReleaseTag -and -not $ArchiveUrl -and -not $ArchivePath) {
    Write-Info "Resolving latest release tag..."
    $resolvedReleaseTag = Get-LatestReleaseTag
  }

  $resolvedArchivePath = $ArchivePath
  if (-not $resolvedArchivePath) {
    $resolvedArchiveUrl = $ArchiveUrl
    if (-not $resolvedArchiveUrl) {
      $assetName = "vibe-everywhere-cli-$resolvedReleaseTag-x86_64-pc-windows-msvc.zip"
      $resolvedArchiveUrl = Build-ArchiveUrl -Tag $resolvedReleaseTag -ArchiveName $assetName
    }

    $resolvedArchivePath = Join-Path $env:TEMP ("vibe-cli-" + [guid]::NewGuid().ToString("N") + ".zip")
    Write-Info "Downloading CLI archive..."
    Write-Info "Archive URL: $resolvedArchiveUrl"
    Invoke-WebRequest -Uri $resolvedArchiveUrl -OutFile $resolvedArchivePath
  }

  $extractDir = Join-Path $env:TEMP ("vibe-cli-" + [guid]::NewGuid().ToString("N"))
  New-Item -ItemType Directory -Force -Path $extractDir | Out-Null
  Expand-Archive -Path $resolvedArchivePath -DestinationPath $extractDir -Force

  New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

  foreach ($binaryName in $selectedBinaryNames) {
    Copy-ArchiveFile -ExtractDir $extractDir -InstallDir $InstallDir -FileName $binaryName -Required
  }
  foreach ($runtimeFile in @("Packet.dll", "wintun.dll", "WinDivert64.sys")) {
    Copy-ArchiveFile -ExtractDir $extractDir -InstallDir $InstallDir -FileName $runtimeFile -Required
  }
  Copy-ArchiveFile -ExtractDir $extractDir -InstallDir $InstallDir -FileName "WinDivert.dll"

  Remove-Item -Path $extractDir -Recurse -Force
  if (-not $ArchivePath -and (Test-Path $resolvedArchivePath)) {
    Remove-Item -Path $resolvedArchivePath -Force
  }

  $installedBinaryPaths = @($selectedBinaryNames | ForEach-Object {
    Join-Path $InstallDir $_
  })
  Print-PostInstallNotes -InstalledBinaryPaths $installedBinaryPaths
}

function Uninstall-Cli {
  $selectedBinaryNames = @(Get-SelectedBinaryNames)
  $runtimeFiles = @("Packet.dll", "wintun.dll", "WinDivert64.sys", "WinDivert.dll")
  $removedBinaryCount = 0

  foreach ($binaryName in $selectedBinaryNames) {
    $binaryPath = Join-Path $InstallDir $binaryName
    if (Test-Path $binaryPath) {
      Remove-Item -Path $binaryPath -Force
      Write-Host "Removed $binaryPath"
      $removedBinaryCount += 1
    } else {
      Write-Warning "No binary found at $binaryPath"
    }
  }

  $remainingBinaryNames = @(
    Get-AllBinaryNames | Where-Object {
      Test-Path (Join-Path $InstallDir $_)
    }
  )

  if ($remainingBinaryNames.Count -eq 0) {
    foreach ($runtimeFile in $runtimeFiles) {
      $runtimePath = Join-Path $InstallDir $runtimeFile
      if (Test-Path $runtimePath) {
        Remove-Item -Path $runtimePath -Force
        Write-Host "Removed $runtimePath"
      }
    }
  } else {
    Write-Host ("Kept Windows runtime files because these binaries still exist: {0}" -f ($remainingBinaryNames -join ", "))
  }

  if ($removedBinaryCount -eq 0 -and $remainingBinaryNames.Count -eq 0) {
    Write-Warning "No selected CLI binaries were removed."
  }

  Write-Warning "This command removes only the selected CLI binaries. Windows runtime files are removed only when no CLI binaries remain in the install directory."
}

switch ($Command) {
  "help" {
    Show-Help
  }
  "install" {
    Install-OrUpdateCli -Mode "install"
  }
  "update" {
    Install-OrUpdateCli -Mode "update"
  }
  "uninstall" {
    Uninstall-Cli
  }
  default {
    throw "Unsupported command: $Command"
  }
}
