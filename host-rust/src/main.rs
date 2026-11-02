// SPDX-License-Identifier: MPL-2.0
//! `lsa-sentinel` — CLI entry point.
//!
//! Usage:
//!   lsa-sentinel                  Collect live signals (Windows) and report.
//!   lsa-sentinel --snapshot FILE  Classify from a key=value snapshot (any OS).
//!   lsa-sentinel --json           Emit the stable machine line instead of prose.
//!   lsa-sentinel --help | -h      Print usage information and exit 0.
//!   lsa-sentinel --version | -V   Print crate version and exit 0.
//!
//! Exit code mirrors the verdict: 2 = BLOCKED-INERT (alarm), 1 = INDETERMINATE,
//! 0 = covered/moot. The alarm is deliberately un-suppressable: there is no
//! "don't show again".

use lsa_sentinel::classify::classify;
use lsa_sentinel::collectors::collect_live;
use lsa_sentinel::collectors::snapshot::Snapshot;
use lsa_sentinel::collectors::SignalSource;
use lsa_sentinel::report::{render, render_machine};

const USAGE: &str = "\
lsa-sentinel — detect silent 0patch lsass.exe coverage loss under Windows RunAsPPL

USAGE:
    lsa-sentinel [--json]
    lsa-sentinel --snapshot <FILE> [--json]
    lsa-sentinel --help | -h
    lsa-sentinel --version | -V

EXIT CODES:
    0   OPEN-COVERED or MOOT
    1   INDETERMINATE (insufficient or right-censored signal — investigate)
    2   BLOCKED-INERT (lsass patches present while loader is blocked — alarm)
   64   CLI usage error
   69   Live collection unavailable on non-Windows host
   74   Snapshot I/O error";

/// Parsed command-line action.
enum CliAction {
    Help,
    Version,
    Run(Option<String>, bool),
}

/// Parse command-line arguments sequentially so option values (such as a
/// `--snapshot` path named `-h`) are never misinterpreted as flags.
fn parse_args(args: &[String]) -> Result<CliAction, String> {
    let mut snapshot: Option<String> = None;
    let mut json = false;
    let mut it = args.iter();

    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(CliAction::Help),
            "-V" | "--version" => return Ok(CliAction::Version),
            "--json" => json = true,
            "--snapshot" => {
                let Some(path) = it.next() else {
                    return Err("--snapshot requires a file path".to_string());
                };
                snapshot = Some(path.clone());
            }
            other => return Err(format!("unknown argument '{other}' (try --help)")),
        }
    }

    Ok(CliAction::Run(snapshot, json))
}

/// Parse CLI flags, collect or load host signals, classify, and emit the verdict.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (snapshot, json) = match parse_args(&args) {
        Ok(CliAction::Help) => {
            println!("{USAGE}");
            return;
        }
        Ok(CliAction::Version) => {
            println!("lsa-sentinel {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Ok(CliAction::Run(snapshot, json)) => (snapshot, json),
        Err(msg) => {
            eprintln!("error: {msg}");
            std::process::exit(64);
        }
    };

    let signals = if let Some(path) = snapshot {
        match Snapshot::from_path(&path).and_then(|s| s.collect()) {
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
