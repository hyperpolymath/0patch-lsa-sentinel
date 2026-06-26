// SPDX-License-Identifier: MPL-2.0
//! Signal collectors.
//!
//! Each collector resolves one of the three signals. The live collectors are
//! Windows-only and shell out to `reg.exe`, `wevtutil`, and the 0patch CLI
//! (no native crate dependency). The [`snapshot`] collector reads a small
//! key=value file and works on any platform — it powers tests and the
//! `--snapshot` CLI mode used for triage and CI on non-Windows runners.

pub mod snapshot;

#[cfg(windows)]
pub mod eventlog;
#[cfg(windows)]
pub mod opatch;
#[cfg(windows)]
pub mod registry;

use crate::model::Signals;

/// Anything that can produce the full signal triple for the host.
pub trait SignalSource {
    fn collect(&self) -> Result<Signals, String>;
}

/// Gather live signals from the running Windows host.
#[cfg(windows)]
pub fn collect_live() -> Result<Signals, String> {
    let protection = registry::lsa_protection()?;
    let loader = eventlog::loader_presence(protection)?;
    let patches = opatch::lsass_patch_status()?;
    Ok(Signals { protection, loader, patches })
}

#[cfg(not(windows))]
pub fn collect_live() -> Result<Signals, String> {
    Err("live collection is Windows-only; use --snapshot <file> on this platform".into())
}
