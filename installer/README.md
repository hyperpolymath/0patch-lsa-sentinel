<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Installing lsa-sentinel (Windows)

> The sentinel turns 0patch's *silent* lsass coverage loss into a loud,
> un-silenceable signal. It runs as a SYSTEM scheduled task and writes its
> verdict to the **Application event log** every interval. The dangerous state
> (`BLOCKED-INERT`) is an **Error** event — pick it up in Event Viewer or your
> SIEM. There is no "don't show again".

## Get the binary

`lsa-sentinel.exe` is built and published by CI on tagged releases (see the
repo's **Releases** page). It is **not** built on non-Windows machines. To build
it yourself on Windows: `cargo build --release` in `host-rust/`.

## Install (elevated PowerShell)

```powershell
# in the extracted release folder (Install-LsaSentinel.ps1 next to lsa-sentinel.exe)
.\Install-LsaSentinel.ps1                       # default: hourly
.\Install-LsaSentinel.ps1 -IntervalMinutes 15   # more frequent
```

## What you'll see

| Exit | Verdict | Event (source `lsa-sentinel`) |
|------|---------|-------------------------------|
| 2 | `BLOCKED-INERT` — lsass patches exist but the loader is blocked | **Error**, id 1001 |
| 1 | `INDETERMINATE` — investigate | Warning, id 1002 |
| 0 | `OPEN-COVERED` / `MOOT` | Information, id 1000 |

A rolling log is also written to `%ProgramData%\lsa-sentinel\sentinel.log`.

## Try it without installing

```powershell
lsa-sentinel.exe                 # live collection (needs SYSTEM for full signal)
lsa-sentinel.exe --snapshot host.snap   # classify from a key=value file (any user)
lsa-sentinel.exe --json          # stable machine line
```

## Uninstall

```powershell
.\Uninstall-LsaSentinel.ps1
```

## ⚠️ Status note

The classifier is formally verified, but the live **collectors have not yet
been validated on a real machine**, and the **0patch patch-status collector is a
placeholder** pending confirmation of the real local query (it fails safe to
INDETERMINATE). See `AFFIRMATION.adoc`. Treat early output as advisory until the
collectors are field-confirmed.
