// SPDX-License-Identifier: MPL-2.0
//! LSA Protection state from the registry (Windows only).
//!
//! Reads `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\RunAsPPL` via `reg.exe`.
//! `RunAsPPL = 1` (or `2`, audit) means LSA Protection is enabled, so the
//! loader will be kept out of `lsass.exe`.

use crate::model::LsaProtection;
use std::process::Command;

pub fn lsa_protection() -> Result<LsaProtection, String> {
    let out = Command::new("reg")
        .args([
            "query",
            r"HKLM\SYSTEM\CurrentControlSet\Control\Lsa",
            "/v",
            "RunAsPPL",
        ])
        .output()
        .map_err(|e| format!("reg query failed: {e}"))?;

    if !out.status.success() {
        // Value absent -> protection not configured -> effectively off.
        return Ok(LsaProtection::Off);
    }

    let text = String::from_utf8_lossy(&out.stdout);
    // Expect a line like: "    RunAsPPL    REG_DWORD    0x1"
    for line in text.lines() {
        if let Some(idx) = line.find("0x") {
            let hex = line[idx + 2..].split_whitespace().next().unwrap_or("");
            return Ok(match u32::from_str_radix(hex.trim(), 16) {
                Ok(0) => LsaProtection::Off,
                Ok(_) => LsaProtection::On, // 1 = enabled, 2 = audit-with-UEFI
                Err(_) => LsaProtection::Unknown,
            });
        }
    }
    Ok(LsaProtection::Unknown)
}
