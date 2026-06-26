// SPDX-License-Identifier: MPL-2.0
//! Outstanding lsass-targeted patch status from the 0patch agent (Windows).
//!
//! Best-effort: queries the 0patch CLI for applied patches and looks for any
//! whose module target is `lsass.exe`. If the CLI is absent or its format
//! changes, we return `Unknown` rather than guess — the classifier then
//! reports INDETERMINATE rather than a false all-clear.

use crate::model::PatchStatus;
use std::path::Path;
use std::process::Command;

const CLI_CANDIDATES: [&str; 2] = [
    r"C:\Program Files (x86)\0patch\agent\0patchConsole.exe",
    r"C:\Program Files (x86)\0patch\agent\0patchService.exe",
];

pub fn lsass_patch_status() -> Result<PatchStatus, String> {
    let cli = CLI_CANDIDATES.iter().find(|p| Path::new(p).exists());
    let Some(cli) = cli else {
        return Ok(PatchStatus::Unknown); // agent CLI not found
    };

    let out = Command::new(cli)
        .arg("list-patches")
        .output()
        .map_err(|e| format!("0patch CLI failed: {e}"))?;

    if !out.status.success() {
        return Ok(PatchStatus::Unknown);
    }

    let text = String::from_utf8_lossy(&out.stdout).to_ascii_lowercase();
    if text.contains("lsass.exe") {
        Ok(PatchStatus::Present)
    } else if text.trim().is_empty() {
        Ok(PatchStatus::Unknown)
    } else {
        Ok(PatchStatus::Absent)
    }
}
