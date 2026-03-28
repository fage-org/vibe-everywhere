param(
  [ValidateSet("debug", "release")]
  [string]$Profile = "debug",
  [string]$DestinationDir,
  [string[]]$BinaryNames = @()
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RootDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$BuildDir = Join-Path $RootDir "target\$Profile"
if (-not $DestinationDir) {
  $DestinationDir = $BuildDir
}
New-Item -ItemType Directory -Force -Path $DestinationDir | Out-Null
$DestinationDir = (Resolve-Path $DestinationDir).Path

function Resolve-CargoHome {
  if ($env:CARGO_HOME) {
    return $env:CARGO_HOME
  }
  if ($env:USERPROFILE) {
    return (Join-Path $env:USERPROFILE ".cargo")
  }
  if ($env:HOME) {
    return (Join-Path $env:HOME ".cargo")
  }

  throw "unable to resolve CARGO_HOME"
}

function Find-RequiredFile {
  param(
    [Parameter(Mandatory = $true)]
    [string]$SearchRoot,
    [Parameter(Mandatory = $true)]
    [string]$Filter,
    [Parameter(Mandatory = $true)]
    [string]$Pattern
  )

  return Get-ChildItem -Path $SearchRoot -Recurse -File -Filter $Filter |
    Where-Object { $_.FullName -match $Pattern } |
    Select-Object -First 1
}

function Copy-IfFound {
  param(
    [System.IO.FileInfo]$File,
    [Parameter(Mandatory = $true)]
    [string]$DestinationDir
  )

  if ($null -eq $File) {
    return
  }

  $destinationPath = Join-Path $DestinationDir $File.Name
  if ($File.FullName -ne $destinationPath) {
    Copy-Item -Path $File.FullName -Destination $destinationPath -Force
  }
}

$CargoHome = Resolve-CargoHome
$CargoCheckouts = Join-Path $CargoHome "git\checkouts"
if (-not (Test-Path $CargoCheckouts)) {
  throw "cargo git checkout directory not found: $CargoCheckouts"
}

$PacketDll = Find-RequiredFile `
  -SearchRoot $CargoCheckouts `
  -Filter "Packet.dll" `
  -Pattern '[\\/]+easytier[\\/]+third_party[\\/]+x86_64[\\/]Packet\.dll$'
if ($null -eq $PacketDll) {
  throw "failed to locate Packet.dll in easytier third_party assets"
}

$WintunDll = Find-RequiredFile `
  -SearchRoot $CargoCheckouts `
  -Filter "wintun.dll" `
  -Pattern '[\\/]+easytier[\\/]+third_party[\\/]+x86_64[\\/]wintun\.dll$'
if ($null -eq $WintunDll) {
  throw "failed to locate wintun.dll in easytier third_party assets"
}

$WinDivertSys = Find-RequiredFile `
  -SearchRoot $CargoCheckouts `
  -Filter "WinDivert64.sys" `
  -Pattern '[\\/]+easytier[\\/]+third_party[\\/]+x86_64[\\/]WinDivert64\.sys$'
if ($null -eq $WinDivertSys) {
  throw "failed to locate WinDivert64.sys in easytier third_party assets"
}

$WinDivertDll = Find-RequiredFile `
  -SearchRoot $BuildDir `
  -Filter "WinDivert.dll" `
  -Pattern '[\\/]WinDivert\.dll$'

foreach ($binaryName in $BinaryNames) {
  $binarySource = Join-Path $BuildDir $binaryName
  if (-not (Test-Path $binarySource)) {
    throw "failed to locate built binary in $BuildDir: $binaryName"
  }

  Copy-Item -Path $binarySource -Destination (Join-Path $DestinationDir $binaryName) -Force
}

Copy-IfFound -File $PacketDll -DestinationDir $DestinationDir
Copy-IfFound -File $WintunDll -DestinationDir $DestinationDir
Copy-IfFound -File $WinDivertSys -DestinationDir $DestinationDir
Copy-IfFound -File $WinDivertDll -DestinationDir $DestinationDir

$StagedFiles = @(
  "Packet.dll",
  "wintun.dll",
  "WinDivert64.sys"
) | ForEach-Object {
  Join-Path $DestinationDir $_
} | Where-Object { Test-Path $_ } | Sort-Object

$OptionalFiles = @(
  "WinDivert.dll"
) | ForEach-Object {
  Join-Path $DestinationDir $_
} | Where-Object { Test-Path $_ } | Sort-Object

$BinaryFiles = @($BinaryNames | ForEach-Object {
  Join-Path $DestinationDir $_
} | Where-Object { Test-Path $_ } | Sort-Object)

$StagedFiles = @($BinaryFiles + $StagedFiles + $OptionalFiles)

if ($StagedFiles.Count -eq 0) {
  throw "no Windows runtime files were staged into $DestinationDir"
}

Write-Host "staged Windows runtime files into $DestinationDir"
$StagedFiles | ForEach-Object { Write-Host " - $($_ | Split-Path -Leaf)" }
