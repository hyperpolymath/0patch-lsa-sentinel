// SPDX-License-Identifier: MPL-2.0
//! Exhaustive truth-table tests for the classifier.
//!
//! There are exactly 3×3×3 = 27 input combinations. We assert every one, and
//! independently re-derive the SPARK postcondition properties over all of them
//! — so this catches any drift between the Rust mirror and the proved spec.
//! Under `--features verified-core`, we also diff against the SPARK object code.

use lsa_sentinel::classify::classify;
use lsa_sentinel::model::{LoaderPresence, LsaProtection, PatchStatus, Signals, Verdict};

const PROT: [LsaProtection; 3] =
    [LsaProtection::Unknown, LsaProtection::Off, LsaProtection::On];
const LOAD: [LoaderPresence; 3] =
    [LoaderPresence::Unknown, LoaderPresence::Blocked, LoaderPresence::Loaded];
const PAT: [PatchStatus; 3] =
    [PatchStatus::Unknown, PatchStatus::Absent, PatchStatus::Present];

fn all_signals() -> impl Iterator<Item = Signals> {
    PROT.iter().flat_map(|&p| {
        LOAD.iter()
            .flat_map(move |&l| PAT.iter().map(move |&pt| Signals::new(p, l, pt)))
    })
}

#[test]
fn covers_all_27_combinations() {
    assert_eq!(all_signals().count(), 27);
}

/// SPARK Post property 1: Moot iff and only iff patches are Absent.
#[test]
fn moot_iff_no_lsass_patches() {
    for s in all_signals() {
        let is_moot = classify(s) == Verdict::Moot;
        let no_patches = s.patches == PatchStatus::Absent;
        assert_eq!(is_moot, no_patches, "moot/absent mismatch for {s:?}");
    }
}

/// SPARK Post property 2: OpenCovered only for a genuinely covered host.
#[test]
fn open_covered_only_when_truly_covered() {
    for s in all_signals() {
        if classify(s) == Verdict::OpenCovered {
            assert_eq!(s.protection, LsaProtection::Off, "{s:?}");
            assert_eq!(s.loader, LoaderPresence::Loaded, "{s:?}");
            assert_eq!(s.patches, PatchStatus::Present, "{s:?}");
        }
    }
}

/// SPARK Post property 3: BlockedInert only when patches truly exist and
/// something is in fact keeping the loader out of lsass.
#[test]
fn blocked_inert_only_when_real_silent_failure() {
    for s in all_signals() {
        if classify(s) == Verdict::BlockedInert {
            assert_eq!(s.patches, PatchStatus::Present, "{s:?}");
            assert!(
                s.protection == LsaProtection::On || s.loader == LoaderPresence::Blocked,
                "blocked-inert with no blocking cause: {s:?}"
            );
        }
    }
}

/// The headline operational case: LSA on + lsass patch outstanding = alarm.
#[test]
fn the_dangerous_case_is_flagged() {
    let s = Signals::new(LsaProtection::On, LoaderPresence::Blocked, PatchStatus::Present);
    assert_eq!(classify(s), Verdict::BlockedInert);
    assert_eq!(classify(s).exit_code(), 2);
}

/// Fail-safe: all-unknown must never read as covered or moot.
#[test]
fn all_unknown_is_indeterminate() {
    let s = Signals::new(LsaProtection::Unknown, LoaderPresence::Unknown, PatchStatus::Unknown);
    assert_eq!(classify(s), Verdict::Indeterminate);
}

/// Explicit spot-checks of the spec table.
#[test]
fn spot_checks() {
    use LoaderPresence as L;
    use LsaProtection as P;
    use PatchStatus as S;
    let cases = [
        (P::Off, L::Loaded, S::Present, Verdict::OpenCovered),
        (P::On, L::Blocked, S::Present, Verdict::BlockedInert),
        (P::Off, L::Blocked, S::Present, Verdict::BlockedInert),
        (P::Off, L::Loaded, S::Absent, Verdict::Moot),
        (P::On, L::Blocked, S::Absent, Verdict::Moot),
        (P::Off, L::Loaded, S::Unknown, Verdict::Indeterminate),
        (P::Unknown, L::Unknown, S::Present, Verdict::Indeterminate),
    ];
    for (p, l, pt, want) in cases {
        assert_eq!(classify(Signals::new(p, l, pt)), want, "{p:?},{l:?},{pt:?}");
    }
}

/// Differential check against the SPARK-proved object code, when linked.
#[cfg(feature = "verified-core")]
#[test]
fn rust_mirror_matches_verified_core() {
    use lsa_sentinel::classify::classify_verified;
    for s in all_signals() {
        assert_eq!(classify(s), classify_verified(s), "drift at {s:?}");
    }
}
