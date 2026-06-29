# SPDX-License-Identifier: MPL-2.0
<#
.SYNOPSIS
  Install lsa-sentinel and register a scheduled task that routes a
  BLOCKED-INERT verdict to the Windows Application event log — the
  un-silenceable sink. There is no "don't show again".

.DESCRIPTION
  Copies lsa-sentinel.exe to the install directory and registers a SYSTEM
  scheduled task that runs it on an interval. The task wrapper writes:
    * exit 2 (BLOCKED-INERT) -> Application event, source 'lsa-sentinel', id 1001, Error
    * exit 1 (INDETERMINATE) -> id 1002, Warning
    * exit 0 (covered/moot)  -> id 1000, Information
  SYSTEM is required so the agent can read the Lsa registry key, the
  CodeIntegrity log, and the 0patch agent state.

.PARAMETER InstallDir
  Target directory. Default: C:\Program Files\lsa-sentinel

.PARAMETER IntervalMinutes
  How often the check runs. Default: 60.

.EXAMPLE
  # From an elevated PowerShell, in the extracted release folder:
  .\Install-LsaSentinel.ps1
#>
[CmdletBinding()]
param(
    [string]$InstallDir = "$env:ProgramFiles\lsa-sentinel",
    [int]$IntervalMinutes = 60
)

$ErrorActionPreference = 'Stop'
$ExeName = 'lsa-sentinel.exe'
$TaskName = 'lsa-sentinel'
$EventSource = 'lsa-sentinel'
$LogDir = "$env:ProgramData\lsa-sentinel"

if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this installer from an elevated (Administrator) PowerShell.'
}

$srcExe = Join-Path $PSScriptRoot $ExeName
if (-not (Test-Path $srcExe)) {
    throw "Cannot find $ExeName next to this script. Extract the full release archive first."
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
Copy-Item -Force $srcExe (Join-Path $InstallDir $ExeName)
$exe = Join-Path $InstallDir $ExeName

# Register the event source so verdicts land in Event Viewer / SIEM.
if (-not [System.Diagnostics.EventLog]::SourceExists($EventSource)) {
    New-EventLog -LogName Application -Source $EventSource
}

# The task wrapper: run the sentinel, log the line, raise an event by exit code.
$wrapper = @"
`$out = & '$exe' --json 2>&1
`$code = `$LASTEXITCODE
Add-Content -Path '$LogDir\sentinel.log' -Value (\"{0}  {1}\" -f (Get-Date -Format o), `$out)
switch (`$code) {
  2 { Write-EventLog -LogName Application -Source '$EventSource' -EntryType Error       -EventId 1001 -Message `$out }
  1 { Write-EventLog -LogName Application -Source '$EventSource' -EntryType Warning     -EventId 1002 -Message `$out }
  default { Write-EventLog -LogName Application -Source '$EventSource' -EntryType Information -EventId 1000 -Message `$out }
}
"@
$wrapperPath = Join-Path $InstallDir 'run-sentinel.ps1'
Set-Content -Path $wrapperPath -Value $wrapper -Encoding UTF8

$action = New-ScheduledTaskAction -Execute 'powershell.exe' `
    -Argument "-NoProfile -NonInteractive -ExecutionPolicy Bypass -File `"$wrapperPath`""
$trigger = New-ScheduledTaskTrigger -Once -At (Get-Date) `
    -RepetitionInterval (New-TimeSpan -Minutes $IntervalMinutes)
$principal = New-ScheduledTaskPrincipal -UserId 'SYSTEM' -LogonType ServiceAccount -RunLevel Highest
$settings = New-ScheduledTaskSettingsSet -StartWhenAvailable -DontStopOnIdleEnd

Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger `
    -Principal $principal -Settings $settings -Force | Out-Null

Write-Host "Installed lsa-sentinel to $InstallDir and scheduled '$TaskName' every $IntervalMinutes min."
Write-Host "Verdicts: Event Viewer > Application, source 'lsa-sentinel' (Error 1001 = BLOCKED-INERT)."
Write-Host "Running once now..."
Start-ScheduledTask -TaskName $TaskName
