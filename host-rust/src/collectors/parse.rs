// SPDX-License-Identifier: MPL-2.0
//! Pure parsers for the live collectors.
//!
//! These functions take raw command output as `&str` and contain no I/O, so
//! they compile and unit-test on every platform (the Windows collectors that
//! actually invoke `reg.exe` / `wevtutil` / the 0patch CLI live in the
//! `#[cfg(windows)]` modules and delegate here). Fixtures in the tests below
//! are modelled on real tool output; see each function's note on how confident
//! we are that the format matches reality.

use crate::model::{LoaderPresence, LsaProtection, PatchStatus};

/// Parse `RunAsPPL` from `reg query HKLM\…\Lsa /v RunAsPPL` output.
///
/// Format is stable and well-documented:
/// `    RunAsPPL    REG_DWORD    0x1`. `0` = off, `1` = enabled, `2` = enabled
/// with UEFI lock. A line we cannot parse yields `Unknown` (never a false Off).
/// NOTE: the *absence* of the value (command exits non-zero) is handled by the
/// caller as `Off`; this function only sees present output.
pub fn parse_runas_ppl(stdout: &str) -> LsaProtection {
    for line in stdout.lines() {
        if line.to_ascii_lowercase().contains("runasppl") {
            if let Some(idx) = line.find("0x") {
                let hex: String = line[idx + 2..]
                    .chars()
                    .take_while(|c| c.is_ascii_hexdigit())
                    .collect();
                return match u32::from_str_radix(&hex, 16) {
                    Ok(0) => LsaProtection::Off,
                    Ok(_) => LsaProtection::On,
                    Err(_) => LsaProtection::Unknown,
                };
            }
        }
    }
    LsaProtection::Unknown
}

/// Detect a 0patch loader block in CodeIntegrity 3033/3063 event text.
///
/// The blocked-image events name the offending module in their message; we look
/// for `0patchLoader` case-insensitively. Confidence: high — the event message
/// embeds the full `\Device\…\0patchLoaderX64.dll` path.
pub fn eventlog_mentions_loader_block(event_text: &str) -> bool {
    event_text.to_ascii_lowercase().contains("0patchloader")
}

/// Resolve loader presence from protection state + event evidence.
///
/// A protected lsass (`On`) categorically excludes the unsigned loader, so we
/// report `Blocked` without needing the log. With protection `Off`, a recent
/// block event means it is still being kept out; no event means it is in.
pub fn loader_from_signals(protection: LsaProtection, recent_block: bool) -> LoaderPresence {
    match protection {
        LsaProtection::On => LoaderPresence::Blocked,
        LsaProtection::Off if recent_block => LoaderPresence::Blocked,
        LsaProtection::Off => LoaderPresence::Loaded,
        LsaProtection::Unknown => LoaderPresence::Unknown,
    }
}

/// Best-effort parse of 0patch agent output for an outstanding lsass-targeted
/// patch.
///
/// ⚠️ UNVERIFIED FORMAT. The exact 0patch local CLI / data source for "patches
/// applicable to lsass.exe" has NOT been confirmed against a real install; the
/// caller treats a missing CLI as `Unknown`. We only return `Present` on a
/// positive `lsass.exe` mention and `Absent` on clearly-non-empty output with
/// none — anything ambiguous stays `Unknown`, so the classifier errs toward
/// INDETERMINATE (investigate), never a false all-clear.
pub fn parse_lsass_patch_status(agent_output: &str) -> PatchStatus {
    let lower = agent_output.to_ascii_lowercase();
    if lower.contains("lsass.exe") {
        PatchStatus::Present
    } else if agent_output.trim().is_empty() {
        PatchStatus::Unknown
    } else {
        PatchStatus::Absent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runas_ppl_enabled() {
        let out = "\r\nHKEY_LOCAL_MACHINE\\SYSTEM\\CurrentControlSet\\Control\\Lsa\r\n    RunAsPPL    REG_DWORD    0x1\r\n";
        assert_eq!(parse_runas_ppl(out), LsaProtection::On);
    }

    #[test]
    fn runas_ppl_uefi_locked() {
        let out = "    RunAsPPL    REG_DWORD    0x2";
        assert_eq!(parse_runas_ppl(out), LsaProtection::On);
    }

    #[test]
    fn runas_ppl_disabled() {
        let out = "    RunAsPPL    REG_DWORD    0x0";
        assert_eq!(parse_runas_ppl(out), LsaProtection::Off);
    }

    #[test]
    fn runas_ppl_garbage_is_unknown() {
        assert_eq!(parse_runas_ppl("totally unrelated output"), LsaProtection::Unknown);
    }

    #[test]
    fn eventlog_detects_loader_block() {
        // Modelled on a CodeIntegrity 3033 message.
        let ev = "Event[0]:\n  Log Name: Microsoft-Windows-CodeIntegrity/Operational\n  Id: 3033\n  Description: Code Integrity determined that a process \\Device\\HarddiskVolume3\\Windows\\System32\\lsass.exe attempted to load \\Device\\HarddiskVolume3\\Program Files (x86)\\0patch\\agent\\0PatchLoaderX64.dll that did not meet the Protected Process signing level requirements.";
        assert!(eventlog_mentions_loader_block(ev));
    }

    #[test]
    fn eventlog_ignores_unrelated_block() {
        let ev = "Event[0]: ... attempted to load some_other_driver.sys ...";
        assert!(!eventlog_mentions_loader_block(ev));
    }

    #[test]
    fn loader_blocked_when_protected() {
        assert_eq!(
            loader_from_signals(LsaProtection::On, false),
            LoaderPresence::Blocked
        );
    }

    #[test]
    fn loader_loaded_when_off_and_no_block() {
        assert_eq!(
            loader_from_signals(LsaProtection::Off, false),
            LoaderPresence::Loaded
        );
    }

    #[test]
    fn loader_blocked_when_off_but_block_seen() {
        assert_eq!(
            loader_from_signals(LsaProtection::Off, true),
            LoaderPresence::Blocked
        );
    }

    #[test]
    fn patch_present_on_lsass_mention() {
        assert_eq!(
            parse_lsass_patch_status("Patch 1234 for lsass.exe (CVE-...) applied"),
            PatchStatus::Present
        );
    }

    #[test]
    fn patch_unknown_on_empty() {
        assert_eq!(parse_lsass_patch_status("   "), PatchStatus::Unknown);
    }

    #[test]
    fn patch_absent_when_listing_has_no_lsass() {
        assert_eq!(
            parse_lsass_patch_status("Patch 999 for chrome.exe applied"),
            PatchStatus::Absent
        );
    }
}
