// SPDX-License-Identifier: MPL-2.0
//! Signal and verdict types.
//!
//! The integer encodings below are the canonical C ABI contract, shared
//! verbatim with `core-spark/src/sentinel-classifier.ads`, `ffi-zig/include/
//! sentinel.h`, and `abi-idris2/src/Sentinel/ABI.idr`. DO NOT renumber: the
//! Idris2 spec proves these exact codes round-trip, and the SPARK ABI maps
//! them identically.

/// Effective LSA Protection (RunAsPPL) state for `lsass.exe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LsaProtection {
    Unknown = 0,
    Off = 1,
    On = 2,
}

/// Whether 0patch's loader is actually inside `lsass.exe`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum LoaderPresence {
    Unknown = 0,
    Blocked = 1,
    Loaded = 2,
}

/// Whether any 0patch patch targeting `lsass.exe` is currently outstanding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PatchStatus {
    Unknown = 0,
    Absent = 1,
    Present = 2,
}

/// The classifier's tri-state-plus-fallback verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum Verdict {
    /// Inputs insufficient to assert any positive verdict — investigate.
    Indeterminate = 0,
    /// No outstanding lsass-targeted patches; the conflict is moot.
    Moot = 1,
    /// LSA off + loader in: lsass patches live (cred-dump protection forfeited).
    OpenCovered = 2,
    /// Patches exist but the loader is kept out: SILENT coverage loss.
    BlockedInert = 3,
}

impl Verdict {
    /// Process exit code: non-zero for the state operators must act on.
    pub fn exit_code(self) -> i32 {
        match self {
            Verdict::BlockedInert => 2, // the alarm state
            Verdict::Indeterminate => 1, // needs investigation
            Verdict::OpenCovered | Verdict::Moot => 0,
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Verdict::OpenCovered => "🟢",
            Verdict::BlockedInert => "🔴",
            Verdict::Moot => "⚪",
            Verdict::Indeterminate => "🟠",
        }
    }

    pub fn tag(self) -> &'static str {
        match self {
            Verdict::OpenCovered => "OPEN-COVERED",
            Verdict::BlockedInert => "BLOCKED-INERT",
            Verdict::Moot => "MOOT",
            Verdict::Indeterminate => "INDETERMINATE",
        }
    }
}

impl From<Verdict> for i32 {
    fn from(v: Verdict) -> i32 {
        v as i32
    }
}

impl TryFrom<i32> for Verdict {
    type Error = i32;
    fn try_from(x: i32) -> Result<Verdict, i32> {
        match x {
            0 => Ok(Verdict::Indeterminate),
            1 => Ok(Verdict::Moot),
            2 => Ok(Verdict::OpenCovered),
            3 => Ok(Verdict::BlockedInert),
            other => Err(other),
        }
    }
}

/// The three observed signals fed to the classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signals {
    pub protection: LsaProtection,
    pub loader: LoaderPresence,
    pub patches: PatchStatus,
}

impl Signals {
    pub fn new(protection: LsaProtection, loader: LoaderPresence, patches: PatchStatus) -> Self {
        Signals { protection, loader, patches }
    }
}
