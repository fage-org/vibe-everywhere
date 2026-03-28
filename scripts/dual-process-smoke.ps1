param(
  [ValidateSet("relay_polling")]
  [string]$Mode = "relay_polling"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RootDir = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$ArtifactsRoot = if ($env:RUNNER_TEMP) {
  $env:RUNNER_TEMP
} elseif ($env:TEMP) {
  $env:TEMP
} else {
  [System.IO.Path]::GetTempPath()
}
$TmpDir = Join-Path $ArtifactsRoot ("vibe-everywhere-windows-smoke-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $TmpDir | Out-Null

$RelayProcess = $null
$AgentProcess = $null
$Succeeded = $false

function Stop-ChildProcess {
  param([System.Diagnostics.Process]$Process)

  if ($null -eq $Process) {
    return
  }

  try {
    if (-not $Process.HasExited) {
      Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
      $null = $Process.WaitForExit(5000)
    }
  } catch {
  }
}

function Show-Log {
  param(
    [string]$Label,
    [string]$Path
  )

  if (-not (Test-Path $Path)) {
    return
  }

  Write-Host "--- $Label ---"
  Get-Content -Path $Path -TotalCount 300
}

function Get-FreeTcpPort {
  param([string]$Address)

  $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Parse($Address), 0)
  try {
    $listener.Start()
    return ([System.Net.IPEndPoint]$listener.LocalEndpoint).Port
  } finally {
    $listener.Stop()
  }
}

function Invoke-JsonRequest {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Uri,
    [string]$Method = "GET",
    [object]$Body = $null
  )

  if ($null -eq $Body) {
    return Invoke-RestMethod -Uri $Uri -Method $Method -TimeoutSec 5
  }

  $jsonBody = $Body | ConvertTo-Json -Compress -Depth 10
  return Invoke-RestMethod -Uri $Uri -Method $Method -ContentType "application/json" -Body $jsonBody -TimeoutSec 5
}

try {
  # Keep the Windows smoke harness on loopback so GitHub-hosted runner interface selection does
  # not affect relay startup or local health checks. This is harness-only behavior, not a product
  # default.
  $HostIp = if ($env:VIBE_TEST_TCP_HOST) {
    $env:VIBE_TEST_TCP_HOST
  } else {
    "127.0.0.1"
  }
  $RelayPort = Get-FreeTcpPort $HostIp
  $BaseUrl = "http://$HostIp`:$RelayPort"
  $DeviceId = "smoke-agent"
  $DeviceName = "Windows Smoke Agent"
  $RelayStdout = Join-Path $TmpDir "relay.stdout.log"
  $RelayStderr = Join-Path $TmpDir "relay.stderr.log"
  $AgentStdout = Join-Path $TmpDir "agent.stdout.log"
  $AgentStderr = Join-Path $TmpDir "agent.stderr.log"
  $FakeCodex = Join-Path $TmpDir "fake-codex.cmd"

  Write-Host "building vibe-agent and vibe-relay binaries"
  Push-Location $RootDir
  try {
    cargo build -p vibe-relay -p vibe-agent | Out-Null
  } finally {
    Pop-Location
  }

  @'
@echo off
if /I "%~1"=="--version" (
  echo fake-codex 0.1.0
  exit /b 0
)
echo {"type":"thread.started","thread_id":"thread_smoke"}
echo {"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"dual-process smoke ok"}}
exit /b 0
'@ | Set-Content -Path $FakeCodex -Encoding Ascii

  Write-Host "starting vibe-relay on $BaseUrl (mode=$Mode)"
  $env:VIBE_RELAY_HOST = $HostIp
  $env:VIBE_RELAY_PORT = [string]$RelayPort
  $env:VIBE_PUBLIC_RELAY_BASE_URL = $BaseUrl
  $env:VIBE_RELAY_STATE_FILE = (Join-Path $TmpDir "relay-state.json")
  $env:VIBE_RELAY_FORWARD_HOST = $HostIp
  $env:VIBE_RELAY_FORWARD_BIND_HOST = $HostIp
  $RelayProcess = Start-Process `
    -FilePath (Join-Path $RootDir "target\debug\vibe-relay.exe") `
    -WorkingDirectory $RootDir `
    -RedirectStandardOutput $RelayStdout `
    -RedirectStandardError $RelayStderr `
    -PassThru

  $RelayHealthy = $false
  for ($attempt = 0; $attempt -lt 100; $attempt++) {
    try {
      $null = Invoke-JsonRequest -Uri "$BaseUrl/api/health"
      $RelayHealthy = $true
      break
    } catch {
      Start-Sleep -Milliseconds 200
    }
  }
  if (-not $RelayHealthy) {
    throw "relay did not become healthy in time"
  }

  Write-Host "starting vibe-agent"
  $env:VIBE_RELAY_URL = $BaseUrl
  $env:VIBE_DEVICE_ID = $DeviceId
  $env:VIBE_DEVICE_NAME = $DeviceName
  $env:VIBE_WORKING_ROOT = $RootDir
  $env:VIBE_POLL_INTERVAL_MS = "200"
  $env:VIBE_HEARTBEAT_INTERVAL_MS = "500"
  $env:VIBE_CODEX_COMMAND = $FakeCodex
  $AgentProcess = Start-Process `
    -FilePath (Join-Path $RootDir "target\debug\vibe-agent.exe") `
    -WorkingDirectory $RootDir `
    -RedirectStandardOutput $AgentStdout `
    -RedirectStandardError $AgentStderr `
    -PassThru

  Write-Host "waiting for agent registration"
  $RegisteredDevice = $null
  for ($attempt = 0; $attempt -lt 100; $attempt++) {
    try {
      $devices = @(Invoke-JsonRequest -Uri "$BaseUrl/api/devices")
      $RegisteredDevice = $devices | Where-Object {
        $_.id -eq $DeviceId -and
        $_.online -and
        (@($_.providers | Where-Object { $_.kind -eq "codex" -and $_.available })).Count -gt 0
      } | Select-Object -First 1

      if ($null -ne $RegisteredDevice) {
        break
      }
    } catch {
    }

    Start-Sleep -Milliseconds 200
  }
  if ($null -eq $RegisteredDevice) {
    throw "agent did not register in time"
  }

  Write-Host "creating task"
  $CreateTaskResponse = Invoke-JsonRequest `
    -Uri "$BaseUrl/api/tasks" `
    -Method "POST" `
    -Body @{
      deviceId = $DeviceId
      provider = "codex"
      prompt = "Say hello from the Windows dual-process smoke test"
      cwd = $null
      model = $null
      title = "Windows dual-process smoke task"
    }
  $TaskId = $CreateTaskResponse.task.id

  Write-Host "waiting for task $TaskId"
  $TaskDetail = $null
  for ($attempt = 0; $attempt -lt 180; $attempt++) {
    $TaskDetail = Invoke-JsonRequest -Uri "$BaseUrl/api/tasks/$TaskId"
    $TaskStatus = $TaskDetail.task.status
    if ($TaskStatus -eq "succeeded") {
      break
    }
    if ($TaskStatus -eq "failed" -or $TaskStatus -eq "canceled") {
      throw "task reached terminal failure state: $TaskStatus`n$($TaskDetail | ConvertTo-Json -Depth 10)"
    }
    Start-Sleep -Milliseconds 200
  }
  if ($null -eq $TaskDetail -or $TaskDetail.task.status -ne "succeeded") {
    throw "task did not finish successfully in time`n$($TaskDetail | ConvertTo-Json -Depth 10)"
  }

  $TaskMessages = @($TaskDetail.events | ForEach-Object { $_.message })
  if ($TaskDetail.task.transport -ne "relay_polling") {
    throw "task used unexpected transport in relay polling mode: $($TaskDetail.task.transport)`n$($TaskDetail | ConvertTo-Json -Depth 10)"
  }
  if (-not ($TaskMessages -contains "dual-process smoke ok")) {
    throw "task result did not include the expected smoke marker`n$($TaskDetail | ConvertTo-Json -Depth 10)"
  }

  Write-Host "creating shell session"
  $CreateShellResponse = Invoke-JsonRequest `
    -Uri "$BaseUrl/api/shell/sessions" `
    -Method "POST" `
    -Body @{
      deviceId = $DeviceId
      cwd = $null
    }
  $ShellId = $CreateShellResponse.session.id

  Write-Host "waiting for shell session $ShellId to become active"
  $ShellDetail = $null
  for ($attempt = 0; $attempt -lt 150; $attempt++) {
    $ShellDetail = Invoke-JsonRequest -Uri "$BaseUrl/api/shell/sessions/$ShellId"
    $ShellStatus = $ShellDetail.session.status
    $ShellTransport = $ShellDetail.session.transport

    if ($ShellTransport -ne "relay_polling") {
      throw "shell session used unexpected transport: $ShellTransport`n$($ShellDetail | ConvertTo-Json -Depth 10)"
    }

    if ($ShellStatus -eq "active") {
      break
    }
    if ($ShellStatus -eq "failed" -or $ShellStatus -eq "closed") {
      throw "shell session reached unexpected terminal state before input: $ShellStatus`n$($ShellDetail | ConvertTo-Json -Depth 10)"
    }

    Start-Sleep -Milliseconds 200
  }
  if ($null -eq $ShellDetail -or $ShellDetail.session.status -ne "active") {
    throw "shell session did not become active in time`n$($ShellDetail | ConvertTo-Json -Depth 10)"
  }

  Write-Host "sending shell input"
  $null = Invoke-JsonRequest `
    -Uri "$BaseUrl/api/shell/sessions/$ShellId/input" `
    -Method "POST" `
    -Body @{
      data = "echo __VIBE_SHELL_SMOKE__`r`nexit`r`n"
    }

  Write-Host "waiting for shell session $ShellId output and completion"
  $MarkerFound = $false
  for ($attempt = 0; $attempt -lt 180; $attempt++) {
    $ShellDetail = Invoke-JsonRequest -Uri "$BaseUrl/api/shell/sessions/$ShellId"
    $ShellStatus = $ShellDetail.session.status
    $Outputs = @($ShellDetail.outputs)
    $MarkerFound = (@($Outputs | Where-Object { $_.data -like "*__VIBE_SHELL_SMOKE__*" })).Count -gt 0

    if ($ShellDetail.session.transport -ne "relay_polling") {
      throw "shell session transport changed unexpectedly: $($ShellDetail.session.transport)`n$($ShellDetail | ConvertTo-Json -Depth 10)"
    }

    if ($ShellStatus -eq "succeeded" -and $MarkerFound) {
      break
    }
    if ($ShellStatus -eq "failed" -or $ShellStatus -eq "closed") {
      throw "shell session reached terminal failure state: $ShellStatus`n$($ShellDetail | ConvertTo-Json -Depth 10)"
    }

    Start-Sleep -Milliseconds 200
  }
  if ($null -eq $ShellDetail -or $ShellDetail.session.status -ne "succeeded" -or -not $MarkerFound) {
    throw "shell session did not produce marker and succeed in time`n$($ShellDetail | ConvertTo-Json -Depth 10)"
  }

  $Succeeded = $true
  Write-Host (
    @{
      mode = $Mode
      taskId = $TaskDetail.task.id
      taskTransport = $TaskDetail.task.transport
      shellSessionId = $ShellDetail.session.id
      shellTransport = $ShellDetail.session.transport
    } | ConvertTo-Json -Compress
  )
} finally {
  Stop-ChildProcess $AgentProcess
  Stop-ChildProcess $RelayProcess

  if (-not $Succeeded) {
    Write-Host "windows dual-process smoke test failed (mode=$Mode)"
    Show-Log -Label "relay.stdout.log" -Path (Join-Path $TmpDir "relay.stdout.log")
    Show-Log -Label "relay.stderr.log" -Path (Join-Path $TmpDir "relay.stderr.log")
    Show-Log -Label "agent.stdout.log" -Path (Join-Path $TmpDir "agent.stdout.log")
    Show-Log -Label "agent.stderr.log" -Path (Join-Path $TmpDir "agent.stderr.log")
    Write-Host "artifacts kept at $TmpDir"
  } elseif ($env:KEEP_SMOKE_ARTIFACTS -eq "1") {
    Write-Host "artifacts kept at $TmpDir"
  } else {
    Remove-Item -Path $TmpDir -Recurse -Force
  }
}
