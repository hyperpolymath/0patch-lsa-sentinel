// SPDX-License-Identifier: MPL-2.0
//! Loader presence from the CodeIntegrity log (Windows only).
//!
//! Delegates detection to the cross-platform-tested parsers in
//! [`crate::collectors::parse`]. When protection is on, the loader is
//! necessarily blocked and we skip the query; otherwise we corroborate with
//! recent CodeIntegrity Operational events 3033/3063 naming `0patchLoader`.

use crate::collectors::parse::{eventlog_mentions_loader_block, loader_from_signals};
use crate::model::{LoaderPresence, LsaProtection};
use std::process::Command;

const QUERY: &str = "*[System[(EventID=3033 or EventID=3063)]]";

pub fn loader_presence(protection: LsaProtection) -> Result<LoaderPresence, String> {
    // A protected lsass categorically excludes the unsigned loader.
    if protection == LsaProtection::On {
        return Ok(LoaderPresence::Blocked);
    }

    let out = Command::new("wevtutil")
        .args([
            "qe",
            "Microsoft-Windows-CodeIntegrity/Operational",
            &format!("/q:{QUERY}"),
            "/c:5",
            "/rd:true",
            "/f:text",
        ])
        .output()
        .map_err(|e| format!("wevtutil failed: {e}"))?;

    if !out.status.success() {
        return Ok(LoaderPresence::Unknown);
    }

    let text = String::from_utf8_lossy(&out.stdout);
    Ok(loader_from_signals(protection, eventlog_mentions_loader_block(&text)))
}
