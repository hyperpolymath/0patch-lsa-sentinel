// SPDX-License-Identifier: MPL-2.0
//! Outstanding lsass-targeted patch status from the 0patch agent (Windows).
//!
//! FIELD-CONFIRMED source (2026-07-02, real install — see
//! `docs/FIELD-VALIDATION.md`): the authoritative local record of patch
//! applications is the agent log `C:\ProgramData\0patch\Logs\0patch.log`
//! (UTF-16LE), with lines like
//! `2026/07/01 21:41:35 Patch 2088 applied in application wevtutil.exe`.
//! An lsass-targeted application appears as
//! `Patch NNNN applied in application lsass.exe`. The previously assumed
//! `0patchConsole.exe list-patches` CLI was a guess with no supporting
//! evidence and has been removed.
//!
//! The collector reads at most the last [`TAIL_CAP`] bytes of the log (it
//! grows to ~10 MB), decodes UTF-16LE, and delegates the verdict to the pure,
//! cross-platform-tested [`crate::collectors::parse::parse_opatch_log`].
//!
//! HONESTY NOTE: the log *format* is field-confirmed, but the inference
//! "no `lsass.exe` line in the recent log ⇒ `Absent`" is a heuristic — the
//! `Present` path has never been observed live (no lsass-targeted patch was
//! outstanding on the validation host). Fail-safe behaviour is preserved:
//! missing / unreadable / undecodable log ⇒ `Unknown`, so the classifier
//! reports INDETERMINATE (investigate) rather than a false all-clear. The
//! agent install directory (real casing: `...\0patch\Agent`, confirmed
//! on-disk) is probed only to distinguish "0patch not installed" from
//! "installed but no readable log" in the diagnostic message.

use crate::collectors::parse::{decode_utf16le, parse_opatch_log};
use crate::model::PatchStatus;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Authoritative local record of patch applications (field-confirmed path).
const LOG_PATH: &str = r"C:\ProgramData\0patch\Logs\0patch.log";
/// Agent install dir (field-confirmed casing: `Agent`, not `agent` — Windows
/// is case-insensitive, but we record the real name).
const AGENT_DIR: &str = r"C:\Program Files (x86)\0patch\Agent";
/// Read at most this much of the log tail (the file grows to ~10 MB).
const TAIL_CAP: u64 = 2 * 1024 * 1024;

pub fn lsass_patch_status() -> Result<PatchStatus, String> {
    match read_log_tail_utf16le(Path::new(LOG_PATH), TAIL_CAP) {
        Some(text) => Ok(parse_opatch_log(&text)),
        None => {
            // No decodable log. Distinguish the two Unknown flavours for the
            // operator, but stay fail-safe either way.
            if Path::new(AGENT_DIR).exists() {
                eprintln!(
                    "0patch agent found at {AGENT_DIR} but its log \
                     {LOG_PATH} is missing/unreadable/undecodable -> patch status Unknown"
                );
            } else {
                eprintln!(
                    "0patch does not appear to be installed \
                     ({AGENT_DIR} absent) -> patch status Unknown"
                );
            }
            Ok(PatchStatus::Unknown)
        }
    }
}

/// Read up to `cap` bytes from the end of `path` and decode as UTF-16LE.
///
/// The seek offset is aligned down to an even byte so the tail window starts
/// on a UTF-16 code-unit boundary (the log starts with a 2-byte BOM, so code
/// units sit at even offsets). Returns `None` on any I/O or decode failure.
fn read_log_tail_utf16le(path: &Path, cap: u64) -> Option<String> {
    let mut f = File::open(path).ok()?;
    let len = f.metadata().ok()?.len();
    let start = len.saturating_sub(cap) & !1; // even-align the window start
    f.seek(SeekFrom::Start(start)).ok()?;
    let mut bytes = Vec::with_capacity((len - start) as usize);
    f.read_to_end(&mut bytes).ok()?;
    decode_utf16le(&bytes)
}
