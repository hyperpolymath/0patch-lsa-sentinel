// SPDX-License-Identifier: MPL-2.0
//! Loader presence from the CodeIntegrity log (Windows only).
//!
//! When LSA Protection is on, the loader is necessarily blocked — we never
//! claim otherwise. When it is off, we corroborate by looking for recent
//! CodeIntegrity Operational events 3033/3063 naming `0patchLoader`: their
//! presence means a block was still occurring; their absence (with protection
//! off) is taken as the loader being in.

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

    let text = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    if text.contains("0patchloader") {
        Ok(LoaderPresence::Blocked)
    } else if protection == LsaProtection::Off {
        Ok(LoaderPresence::Loaded)
    } else {
        Ok(LoaderPresence::Unknown)
    }
}
