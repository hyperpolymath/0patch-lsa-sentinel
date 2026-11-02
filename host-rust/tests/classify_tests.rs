// SPDX-License-Identifier: MPL-2.0
//! Exhaustive truth-table tests for the classifier.
//!
//! There are exactly 3×3×3 = 27 input combinations. We assert every one, and
//! independently re-derive the SPARK postcondition properties over all of them
//! — so this catches any drift between the Rust mirror and the proved spec.
//! Under `--features verified-core`, we also diff against the SPARK object code.

use lsa_sentinel::classify::classify;
use lsa_sentinel::model::{LoaderPresence, LsaProtection, PatchStatus, Signals, Verdict};

const PROT: [LsaProtection; 3] = [
    LsaProtection::Unknown,
    LsaProtection::Off,
    LsaProtection::On,
];
const LOAD: [LoaderPresence; 3] = [
    LoaderPresence::Unknown,
    LoaderPresence::Blocked,
    LoaderPresence::Loaded,
];
const PAT: [PatchStatus; 3] = [
    PatchStatus::Unknown,
    PatchStatus::Absent,
    PatchStatus::Present,
];

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
    let s = Signals::new(
        LsaProtection::On,
        LoaderPresence::Blocked,
        PatchStatus::Present,
    );
    assert_eq!(classify(s), Verdict::BlockedInert);
    assert_eq!(classify(s).exit_code(), 2);
}

/// Fail-safe: all-unknown must never read as covered or moot.
#[test]
fn all_unknown_is_indeterminate() {
    let s = Signals::new(
        LsaProtection::Unknown,
        LoaderPresence::Unknown,
        PatchStatus::Unknown,
    );
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

/// Collector-to-classifier safety invariant: when LSA Protection is On (or a
/// CodeIntegrity loader-block event is present), `0patch.log` is right-censored
/// for `lsass.exe` and MUST NEVER produce a silent exit-0 verdict (`Moot` or
/// `OpenCovered`) for any possible log contents.
#[test]
fn protected_or_blocked_host_never_silently_clears_from_opatch_log() {
    use lsa_sentinel::collectors::parse::{loader_from_signals, parse_opatch_log};

    let sample_logs = [
        "",
        "   \n",
        "2026/07/01 21:41:35 0patch Service started\n",
        "2026/07/01 21:41:35 Patch 2088 applied in application wevtutil.exe\n",
        "2026/07/01 21:42:00 Patch 2101 applied in application lsass.exe\n",
    ];
    let blocked_hosts = [
        (LsaProtection::On, false),
        (LsaProtection::On, true),
        (LsaProtection::Off, true),
        (LsaProtection::Unknown, false),
        (LsaProtection::Unknown, true),
    ];

    for (protection, recent_block) in blocked_hosts {
        let loader = loader_from_signals(protection, recent_block);
        for log in sample_logs {
            let patches = parse_opatch_log(log, loader);
            let verdict = classify(Signals::new(protection, loader, patches));
            assert_ne!(
                verdict,
                Verdict::Moot,
                "false Moot on blocked/unknown host ({protection:?}, block={recent_block}) for log={log:?}"
            );
            assert_ne!(
                verdict,
                Verdict::OpenCovered,
                "false OpenCovered on blocked/unknown host ({protection:?}, block={recent_block}) for log={log:?}"
            );
            assert_ne!(
                verdict.exit_code(),
                0,
                "exit code 0 on blocked/unknown host ({protection:?}, block={recent_block})"
            );
        }
    }
}

/// CLI smoke check: `--help` and `--version` exit 0 with descriptive output.
#[test]
fn cli_help_and_version_succeed() {
    let exe = env!("CARGO_BIN_EXE_lsa-sentinel");
    for flag in ["--help", "-h"] {
        let out = std::process::Command::new(exe)
            .arg(flag)
            .output()
            .expect("spawn lsa-sentinel");
        assert!(out.status.success(), "{flag} exited {:?}", out.status);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("USAGE:"), "missing USAGE in {stdout:?}");
    }
    for flag in ["--version", "-V"] {
        let out = std::process::Command::new(exe)
            .arg(flag)
            .output()
            .expect("spawn lsa-sentinel");
        assert!(out.status.success(), "{flag} exited {:?}", out.status);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.starts_with("lsa-sentinel "),
            "unexpected version output {stdout:?}"
        );
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
