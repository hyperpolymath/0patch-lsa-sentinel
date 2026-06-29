# SPDX-License-Identifier: MPL-2.0
<#
.SYNOPSIS
  Remove the lsa-sentinel scheduled task and installed files.
#>
[CmdletBinding()]
param(
    [string]$InstallDir = "$env:ProgramFiles\lsa-sentinel"
)
$ErrorActionPreference = 'Stop'
$TaskName = 'lsa-sentinel'

if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    throw 'Run this uninstaller from an elevated (Administrator) PowerShell.'
}

if (Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue) {
    Unregister-ScheduledTask -TaskName $TaskName -Confirm:$false
    Write-Host "Removed scheduled task '$TaskName'."
}
if (Test-Path $InstallDir) {
    Remove-Item -Recurse -Force $InstallDir
    Write-Host "Removed $InstallDir."
}
Write-Host "Note: the 'lsa-sentinel' Application event source and $env:ProgramData\lsa-sentinel logs are left in place for audit history."
