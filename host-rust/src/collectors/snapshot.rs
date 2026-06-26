// SPDX-License-Identifier: MPL-2.0
//! Platform-independent snapshot collector.
//!
//! Parses a tiny `key = value` file (comments with `#`) describing the three
//! signals, e.g.:
//!
//! ```text
//! # one host's observed state
//! protection = on        # on | off | unknown
//! loader     = blocked   # loaded | blocked | unknown
//! patches    = present   # present | absent | unknown
//! ```
//!
//! Unrecognised or missing values degrade to `Unknown`, never to a positive
//! "covered" signal — the same fail-safe direction as the SPARK ABI.

use crate::collectors::SignalSource;
use crate::model::{LoaderPresence, LsaProtection, PatchStatus, Signals};

pub struct Snapshot {
    text: String,
}

impl Snapshot {
    pub fn from_str(text: impl Into<String>) -> Self {
        Snapshot { text: text.into() }
    }

    pub fn from_path(path: &str) -> Result<Self, String> {
        use std::io::Read;
        // Bounded read: a host snapshot is a handful of lines. Capping at 64 KiB
        // means a huge or malicious file cannot cause an unbounded allocation.
        const LIMIT: u64 = 64 * 1024;
        let mut f =
            std::fs::File::open(path).map_err(|e| format!("cannot open snapshot {path}: {e}"))?;
        let mut buf = String::new();
        f.by_ref()
            .take(LIMIT)
            .read_to_string(&mut buf)
            .map_err(|e| format!("cannot read snapshot {path}: {e}"))?;
        Ok(Snapshot::from_str(buf))
    }

    fn value(&self, key: &str) -> Option<String> {
        for line in self.text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if let Some((k, v)) = line.split_once('=') {
                if k.trim().eq_ignore_ascii_case(key) {
                    return Some(v.trim().to_ascii_lowercase());
                }
            }
        }
        None
    }
}

impl SignalSource for Snapshot {
    fn collect(&self) -> Result<Signals, String> {
        let protection = match self.value("protection").as_deref() {
            Some("on") => LsaProtection::On,
            Some("off") => LsaProtection::Off,
            _ => LsaProtection::Unknown,
        };
        let loader = match self.value("loader").as_deref() {
            Some("loaded") => LoaderPresence::Loaded,
            Some("blocked") => LoaderPresence::Blocked,
            _ => LoaderPresence::Unknown,
        };
        let patches = match self.value("patches").as_deref() {
            Some("present") => PatchStatus::Present,
            Some("absent") => PatchStatus::Absent,
            _ => PatchStatus::Unknown,
        };
        Ok(Signals { protection, loader, patches })
    }
}
