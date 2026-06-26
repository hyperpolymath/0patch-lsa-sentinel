// SPDX-License-Identifier: MPL-2.0
//! The classifier — Rust reference mirror of the SPARK kernel.
//!
//! This MUST stay semantically identical to
//! `core-spark/src/sentinel-classifier.adb`. The SPARK side is the proved
//! source of truth; `tests/classify_tests.rs` checks this mirror against the
//! same truth table the SPARK postcondition encodes, and (under
//! `verified-core`) against the SPARK object code itself.

use crate::model::{LoaderPresence, LsaProtection, PatchStatus, Signals, Verdict};

/// Decide whether 0patch's lsass coverage is live, silently inert, or moot.
///
/// Mirrors the four-branch SPARK body exactly.
pub fn classify(s: Signals) -> Verdict {
    // 1. No lsass-targeted patches outstanding -> the conflict is moot.
    if s.patches == PatchStatus::Absent {
        return Verdict::Moot;
    }

    // 2. Genuinely covered: protection off, loader in, patches positively present.
    if s.protection == LsaProtection::Off
        && s.loader == LoaderPresence::Loaded
        && s.patches == PatchStatus::Present
    {
        return Verdict::OpenCovered;
    }

    // 3. Silent failure: patches positively present, and either LSA Protection
    //    is on (necessarily blocking the loader) or the loader is observed out.
    if s.patches == PatchStatus::Present
        && (s.protection == LsaProtection::On || s.loader == LoaderPresence::Blocked)
    {
        return Verdict::BlockedInert;
    }

    // 4. Not enough positive information.
    Verdict::Indeterminate
}

/// Call the SPARK-proved classifier over the C ABI. Available only when the
/// `verified-core` feature links `libsentinel_core.a`.
#[cfg(feature = "verified-core")]
pub fn classify_verified(s: Signals) -> Verdict {
    let raw = crate::ffi::classify_raw(s.protection as i32, s.loader as i32, s.patches as i32);
    // Fail-safe: an out-of-range code (impossible for the proved core) maps to
    // Indeterminate rather than panicking — never to a positive "covered".
    Verdict::try_from(raw).unwrap_or(Verdict::Indeterminate)
}
