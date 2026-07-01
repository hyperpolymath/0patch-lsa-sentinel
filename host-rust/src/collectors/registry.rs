// SPDX-License-Identifier: MPL-2.0
//! LSA Protection state from the registry (Windows only).
//!
//! Reads `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\RunAsPPL` via `reg.exe`
//! and delegates parsing to [`crate::collectors::parse::parse_runas_ppl`]
//! (which is unit-tested cross-platform). Value absent (`reg` exits non-zero)
//! means protection is not configured, i.e. effectively off.
//! FIELD-VALIDATED 2026-07-02 on a real host (`RunAsPPL=0x1` correctly read
//! as On) — see `docs/FIELD-VALIDATION.md`.

use crate::collectors::parse::parse_runas_ppl;
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
        return Ok(LsaProtection::Off);
    }
    Ok(parse_runas_ppl(&String::from_utf8_lossy(&out.stdout)))
}
