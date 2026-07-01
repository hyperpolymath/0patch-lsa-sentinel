// SPDX-License-Identifier: MPL-2.0
//! Pure parsers for the live collectors.
//!
//! These functions take raw command output / file text as `&str` (plus one
//! byte-level UTF-16LE decoder) and contain no I/O, so they compile and
//! unit-test on every platform (the Windows collectors that actually invoke
//! `reg.exe` / `wevtutil` / read the 0patch agent log live in the
//! `#[cfg(windows)]` modules and delegate here). Fixtures in the tests below
//! are modelled on real tool output; see each function's note on how confident
//! we are that the format matches reality. The `reg.exe` and 0patch-log
//! formats were field-confirmed on a live Windows host on 2026-07-02 (see
//! `docs/FIELD-VALIDATION.md`).

use crate::model::{LoaderPresence, LsaProtection, PatchStatus};

/// Parse `RunAsPPL` from `reg query HKLM\…\Lsa /v RunAsPPL` output.
///
/// Format is stable and well-documented:
/// `    RunAsPPL    REG_DWORD    0x1`. `0` = off, `1` = enabled, `2` = enabled
/// with UEFI lock. A line we cannot parse yields `Unknown` (never a false Off).
/// FIELD-CONFIRMED 2026-07-02: on a real host with `RunAsPPL=0x1` (and
/// `RunAsPPLBoot=0x1`) this parser returned `On` against live `reg.exe` output.
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
/// embeds the full `\Device\…\0patchLoaderX64.dll` path. Field note
/// (2026-07-02): the `wevtutil` query itself ran fine on a real host, but no
/// 3033/3063 events mentioning 0patch existed there (no user-mode lsass
/// injection attempt was occurring), so the positive-match path is still
/// unobserved live.
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

/// Parse the 0patch agent log (`C:\ProgramData\0patch\Logs\0patch.log`) for an
/// outstanding lsass-targeted patch application.
///
/// FIELD-CONFIRMED FORMAT (2026-07-02, real install, see
/// `docs/FIELD-VALIDATION.md`): the log is UTF-16LE text whose
/// patch-application lines look exactly like
/// `2026/07/01 21:41:35 Patch 2088 applied in application wevtutil.exe`.
/// An lsass-targeted application would therefore read
/// `Patch NNNN applied in application lsass.exe`.
///
/// Semantics:
/// * `Present` — any `applied in application lsass.exe` line exists.
/// * `Absent`  — the log decoded fine and is non-trivial (non-empty text) but
///   contains no such line.
/// * `Unknown` — empty / trivial text (the caller also maps a missing or
///   undecodable file to `Unknown`).
///
/// HONESTY NOTE: while the *line format* is field-confirmed, the inference
/// "no lsass line in the recent log ⇒ no lsass patch is being applied"
/// (`Absent`) is a heuristic — the Present path has never been observed live
/// because no lsass-targeted 0patch patch was outstanding on the validation
/// host. Anything ambiguous stays `Unknown`, so the classifier errs toward
/// INDETERMINATE (investigate), never a false all-clear.
pub fn parse_opatch_log(log_text: &str) -> PatchStatus {
    let lower = log_text.to_ascii_lowercase();
    if lower.contains("applied in application lsass.exe") {
        PatchStatus::Present
    } else if log_text.trim().is_empty() {
        PatchStatus::Unknown
    } else {
        PatchStatus::Absent
    }
}

/// Decode UTF-16LE bytes (the 0patch log encoding) into a `String`.
///
/// Tolerates: a leading BOM (`FF FE` / U+FEFF, stripped), a leading unpaired
/// *low* surrogate and a trailing unpaired *high* surrogate (both dropped —
/// they occur when the caller reads only the tail of a large log and the cut
/// lands mid-character). Returns `None` on odd byte length or any *interior*
/// unpaired surrogate, so a corrupt or mis-encoded file maps to `Unknown`
/// upstream rather than being half-read.
pub fn decode_utf16le(bytes: &[u8]) -> Option<String> {
    if bytes.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|p| u16::from_le_bytes([p[0], p[1]]))
        .collect();
    let mut units = units.as_slice();
    if units.first() == Some(&0xFEFF) {
        units = &units[1..]; // BOM
    }
    if matches!(units.first(), Some(u) if (0xDC00..=0xDFFF).contains(u)) {
        units = &units[1..]; // tail-read cut a surrogate pair: lone low surrogate
    }
    if matches!(units.last(), Some(u) if (0xD800..=0xDBFF).contains(u)) {
        units = &units[..units.len() - 1]; // lone high surrogate at EOF cut
    }
    char::decode_utf16(units.iter().copied())
        .collect::<Result<String, _>>()
        .ok()
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
        let ev = "Event[0]:\n  Log Name: Microsoft-Windows-CodeIntegrity/Operational\n  Id: 3033\n  Description: Code Integrity determined that a process \\Device\\HarddiskVolume3\\Windows\\System32\\lsass.exe attempted to load \\Device\\HarddiskVolume3\\Program Files (x86)\\0patch\\Agent\\0patchLoaderX64.dll that did not meet the Protected Process signing level requirements.";
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

    // ---- 0patch log parsing (line formats are verbatim from a real install,
    // ---- 2026-07-02; see docs/FIELD-VALIDATION.md) ----

    /// Real (field-captured) log lines, no lsass patch outstanding.
    const REAL_LOG_NO_LSASS: &str = "\
2026/07/01 21:41:35 Patch 2088 applied in application wevtutil.exe\n\
2026/07/01 21:41:35 Patch 1999 applied in application conhost.exe\n";

    #[test]
    fn opatch_log_present_on_lsass_application() {
        let log = format!(
            "{REAL_LOG_NO_LSASS}2026/07/01 21:42:00 Patch 2101 applied in application lsass.exe\n"
        );
        assert_eq!(parse_opatch_log(&log), PatchStatus::Present);
    }

    #[test]
    fn opatch_log_absent_when_no_lsass_line() {
        assert_eq!(parse_opatch_log(REAL_LOG_NO_LSASS), PatchStatus::Absent);
    }

    #[test]
    fn opatch_log_unknown_on_empty() {
        assert_eq!(parse_opatch_log(""), PatchStatus::Unknown);
        assert_eq!(parse_opatch_log("   \n  "), PatchStatus::Unknown);
    }

    #[test]
    fn opatch_log_bare_lsass_mention_is_not_present() {
        // e.g. a service-log line mentioning lsass without an application —
        // only the exact "applied in application lsass.exe" phrase counts.
        assert_eq!(
            parse_opatch_log("2026/07/01 21:41:35 Scanning lsass.exe modules\n"),
            PatchStatus::Absent
        );
    }

    // ---- UTF-16LE decoding ----

    fn utf16le(s: &str, bom: bool) -> Vec<u8> {
        let mut out = Vec::new();
        if bom {
            out.extend_from_slice(&[0xFF, 0xFE]);
        }
        for u in s.encode_utf16() {
            out.extend_from_slice(&u.to_le_bytes());
        }
        out
    }

    #[test]
    fn decode_utf16le_with_bom_roundtrips_real_line() {
        let line = "2026/07/01 21:41:35 Patch 2088 applied in application wevtutil.exe";
        assert_eq!(decode_utf16le(&utf16le(line, true)).as_deref(), Some(line));
    }

    #[test]
    fn decode_utf16le_without_bom_roundtrips() {
        let line = "Patch 1999 applied in application conhost.exe";
        assert_eq!(decode_utf16le(&utf16le(line, false)).as_deref(), Some(line));
    }

    #[test]
    fn decode_utf16le_rejects_odd_length() {
        assert_eq!(decode_utf16le(&[0x50, 0x00, 0x61]), None);
    }

    #[test]
    fn decode_utf16le_tolerates_tail_cut_surrogates() {
        // Simulate a tail read whose window starts on the low half of one
        // surrogate pair and ends on the high half of another.
        let mut bytes = 0xDC01u16.to_le_bytes().to_vec(); // lone low surrogate
        bytes.extend(utf16le("Patch 7 applied in application lsass.exe", false));
        bytes.extend_from_slice(&0xD801u16.to_le_bytes()); // lone high surrogate
        assert_eq!(
            decode_utf16le(&bytes).as_deref(),
            Some("Patch 7 applied in application lsass.exe")
        );
    }

    #[test]
    fn decode_utf16le_rejects_interior_lone_surrogate() {
        let mut bytes = utf16le("ok ", false);
        bytes.extend_from_slice(&0xD800u16.to_le_bytes()); // high surrogate…
        bytes.extend(utf16le(" not ok", false)); // …followed by non-low unit
        assert_eq!(decode_utf16le(&bytes), None);
    }

    #[test]
    fn decode_then_parse_end_to_end() {
        let bytes = utf16le(REAL_LOG_NO_LSASS, true);
        let text = decode_utf16le(&bytes).expect("decodes");
        assert_eq!(parse_opatch_log(&text), PatchStatus::Absent);
    }
}
