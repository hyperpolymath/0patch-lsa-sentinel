// SPDX-License-Identifier: MPL-2.0
//! `lsa-sentinel` — CLI entry point.
//!
//! Usage:
//!   lsa-sentinel                  Collect live signals (Windows) and report.
//!   lsa-sentinel --snapshot FILE  Classify from a key=value snapshot (any OS).
//!   lsa-sentinel --json           Emit the stable machine line instead of prose.
//!
//! Exit code mirrors the verdict: 2 = BLOCKED-INERT (alarm), 1 = INDETERMINATE,
//! 0 = covered/moot. The alarm is deliberately un-suppressable: there is no
//! "don't show again".

use lsa_sentinel::classify::classify;
use lsa_sentinel::collectors::{collect_live, snapshot::Snapshot, SignalSource};
use lsa_sentinel::report::{render, render_machine};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");

    let signals = if let Some(i) = args.iter().position(|a| a == "--snapshot") {
        let Some(path) = args.get(i + 1) else {
            eprintln!("error: --snapshot requires a file path");
            std::process::exit(64);
        };
        match Snapshot::from_path(path).and_then(|s| s.collect()) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(74);
            }
        }
    } else {
        match collect_live() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(69);
            }
        }
    };

    let verdict = classify(signals);

    if json {
        println!("{}", render_machine(signals, verdict));
    } else {
        println!("{}", render(signals, verdict));
    }

    std::process::exit(verdict.exit_code());
}
