<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk> -->

# Field validation — 2026-07-02

First validation of the Windows collectors against a **real machine**: the
physical Windows host running this project's WSL2 development environment,
with a live, licensed 0patch agent installed and LSA Protection enabled.

- **Date:** 2026-07-02
- **Method:** WSL interop — Windows binaries (`reg.exe`, `wevtutil.exe`) and
  Windows filesystem paths (`/mnt/c/...`) invoked/inspected directly from the
  WSL2 session on the same machine. No VM, no mock.
- **Host state:** LSA Protection ON (`RunAsPPL=0x1`, `RunAsPPLBoot=0x1`),
  0patch agent installed and actively applying patches to other processes.

## Findings

### 1. Registry collector — VALIDATED

`reg.exe query "HKLM\SYSTEM\CurrentControlSet\Control\Lsa" /v RunAsPPL`
returned:

```
    RunAsPPL    REG_DWORD    0x1
```

(`RunAsPPLBoot` also `0x1`.) `parse_runas_ppl` parsed this correctly as
`LsaProtection::On`. The registry collector's assumed output format matches
reality exactly.

### 2. 0patch install paths — CONFIRMED (one casing correction)

`C:\Program Files (x86)\0patch\Agent\` exists and contains:
`0patchConsole.exe`, `0patchLoader.dll`, `0patchLoaderX64.dll`,
`0patchService.exe`, `0patchServicex64.exe`, `0patchDriver32.sys`,
`0patchDriver64.sys`.

**Correction:** the directory is `Agent` (capital A); our code said `agent`.
Windows is case-insensitive so this was harmless, but the string has been
fixed to the real casing.

### 3. Patch-status source — CORRECTED (the big one)

The previously assumed `0patchConsole.exe list-patches` CLI was a **guess**
with no supporting evidence that the subcommand exists. It has been removed.

The authoritative local record of patch applications is the agent log
`C:\ProgramData\0patch\Logs\0patch.log` (also `0patchService.log`), which is
**UTF-16LE** encoded and grows to ~10 MB. Real lines observed:

```
2026/07/01 21:41:35 Patch 2088 applied in application wevtutil.exe
2026/07/01 21:41:35 Patch 1999 applied in application conhost.exe
```

An lsass-targeted patch application would appear as
`Patch NNNN applied in application lsass.exe`. On this machine there were
**zero** lsass mentions in the log — consistent with the verdict **MOOT**
(no lsass-targeted patch currently outstanding).

The service log also shows lines like
`Pid: 6184 Loader already injected permanently in 0patchDriver` — injection
happens via the kernel driver.

The collector (`host-rust/src/collectors/opatch.rs`) was rewritten
accordingly: read the last ≤2 MiB of the log, decode UTF-16LE (pure Rust,
BOM-tolerant, tail-cut-tolerant), `Present` on any
`applied in application lsass.exe` line, `Absent` on a decodable non-trivial
log without one, `Unknown` otherwise. The `Agent` directory probe is kept
only to distinguish "0patch not installed" from "installed but no readable
log" in the diagnostic output.

### 4. CodeIntegrity event log — QUERY PATH WORKS, MATCH PATH UNOBSERVED

The `wevtutil` query of the CodeIntegrity Operational log ran successfully,
but no 3033/3063 events mentioning 0patch were present — consistent with the
host state: no user-mode lsass injection attempt is currently being made
(protection is on and the loader stays out via the driver/other processes).

## Summary: validated / corrected / still open

| Item | Status |
|---|---|
| `parse_runas_ppl` vs real `reg.exe` output | **Validated** |
| 0patch install directory and file inventory | **Confirmed** (casing corrected to `Agent`) |
| `0patchConsole.exe list-patches` CLI | **Removed** — was speculative, no evidence it exists |
| Patch-status source = `0patch.log` (UTF-16LE, `applied in application X` lines) | **Field-confirmed format**; collector rewritten against it |
| `Present` path (an actual lsass-targeted patch application in the log) | **Still open** — never observed live; no lsass patch was outstanding on the validation host. The "no lsass line in recent log ⇒ Absent" inference is a heuristic. |
| CodeIntegrity 3033/3063 block-event path | **Still open** — query mechanism works, but no block is currently occurring, so a positive match has never been observed live |
