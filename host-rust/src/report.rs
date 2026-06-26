// SPDX-License-Identifier: MPL-2.0
//! Rendering the verdict.
//!
//! The whole point of the sentinel is that the dangerous state cannot be
//! silenced. `render` produces an operator line and a stable machine line; the
//! caller routes BLOCKED-INERT to an alert sink and exits non-zero.

use crate::model::{Signals, Verdict};

/// Human-facing one-liner.
pub fn render(signals: Signals, verdict: Verdict) -> String {
    let detail = match verdict {
        Verdict::OpenCovered => {
            "lsass micropatches LIVE — but LSA Protection is OFF, so credential-dump \
             protection is forfeited (recorded as an accepted lsa-open enrolment)."
        }
        Verdict::BlockedInert => {
            "ALARM: lsass-targeted patches exist but the loader is blocked from \
             lsass.exe. The host BELIEVES it is patched and is NOT. Raise a work-order."
        }
        Verdict::Moot => "no outstanding lsass-targeted patches; conflict currently inert.",
        Verdict::Indeterminate => {
            "insufficient signal to decide — investigate (agent state / console access)."
        }
    };
    format!(
        "{} {}  {}\n    signals: protection={:?} loader={:?} patches={:?}",
        verdict.glyph(),
        verdict.tag(),
        detail,
        signals.protection,
        signals.loader,
        signals.patches,
    )
}

/// Stable single-line machine record (grep/alert-sink friendly).
pub fn render_machine(signals: Signals, verdict: Verdict) -> String {
    format!(
        "lsa-sentinel verdict={} protection={} loader={} patches={} exit={}",
        verdict.tag(),
        signals.protection as i32,
        signals.loader as i32,
        signals.patches as i32,
        verdict.exit_code(),
    )
}
