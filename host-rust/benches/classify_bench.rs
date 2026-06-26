// SPDX-License-Identifier: MPL-2.0
//! Dependency-free benchmark (harness = false): times the classifier over all
//! 27 input combinations. No criterion, so it runs offline on stable Rust.

use lsa_sentinel::classify::classify;
use lsa_sentinel::model::{LoaderPresence, LsaProtection, PatchStatus, Signals};
use std::hint::black_box;
use std::time::Instant;

fn all_signals() -> Vec<Signals> {
    let prot = [LsaProtection::Unknown, LsaProtection::Off, LsaProtection::On];
    let load = [LoaderPresence::Unknown, LoaderPresence::Blocked, LoaderPresence::Loaded];
    let pat = [PatchStatus::Unknown, PatchStatus::Absent, PatchStatus::Present];
    let mut v = Vec::with_capacity(27);
    for &p in &prot {
        for &l in &load {
            for &pt in &pat {
                v.push(Signals::new(p, l, pt));
            }
        }
    }
    v
}

fn main() {
    let cases = all_signals();
    let iters: u64 = 20_000_000;

    // Warm up.
    let mut acc = 0i64;
    for s in &cases {
        acc += classify(*s) as i64;
    }
    black_box(acc);

    let start = Instant::now();
    let mut sink = 0i64;
    for i in 0..iters {
        let s = cases[(i as usize) % cases.len()];
        sink += classify(black_box(s)) as i64;
    }
    let elapsed = start.elapsed();
    black_box(sink);

    let per = elapsed.as_nanos() as f64 / iters as f64;
    println!(
        "classify: {iters} calls in {:?}  ->  {per:.2} ns/call  ({:.1} M calls/s)",
        elapsed,
        1000.0 / per
    );
}
