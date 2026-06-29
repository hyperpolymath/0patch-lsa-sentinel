// SPDX-License-Identifier: MPL-2.0
//! Outstanding lsass-targeted patch status from the 0patch agent (Windows).
//!
//! ⚠️ UNVERIFIED collector. The local mechanism by which the 0patch agent
//! exposes "patches applicable to lsass.exe" has NOT been confirmed against a
//! real install — the CLI name and `list-patches` subcommand below are a
//! placeholder pending validation (see AFFIRMATION.adoc / docs). The collector
//! is deliberately fail-safe: if the CLI is absent or its output is ambiguous,
//! it returns `Unknown`, so the classifier reports INDETERMINATE (investigate)
//! rather than a false all-clear. Parsing itself is delegated to the
//! cross-platform-tested [`crate::collectors::parse::parse_lsass_patch_status`].

use crate::collectors::parse::parse_lsass_patch_status;
use crate::model::PatchStatus;
use std::path::Path;
use std::process::Command;

// Placeholder candidates — REPLACE once the real local query is confirmed.
const CLI_CANDIDATES: [&str; 1] =
    [r"C:\Program Files (x86)\0patch\agent\0patchConsole.exe"];

pub fn lsass_patch_status() -> Result<PatchStatus, String> {
    let Some(cli) = CLI_CANDIDATES.iter().find(|p| Path::new(p).exists()) else {
        return Ok(PatchStatus::Unknown); // agent CLI not found -> investigate
    };

    let out = Command::new(cli)
        .arg("list-patches")
        .output()
        .map_err(|e| format!("0patch CLI failed: {e}"))?;

    if !out.status.success() {
        return Ok(PatchStatus::Unknown);
    }
    Ok(parse_lsass_patch_status(&String::from_utf8_lossy(&out.stdout)))
}
