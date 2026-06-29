<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Follow-up email to support@0patch.com

**To:** support@0patch.com
**Subject:** Follow-up: LSA Protection blocks 0patchLoader from lsass.exe — open analysis + the silent-coverage question

---

Hello 0patch / ACROS Security team,

I wrote recently about Windows LSA Protection (RunAsPPL) blocking
`0PatchLoaderX64.dll` from loading into `lsass.exe`, which silently disables
your lsass-targeted micropatches. In case that note didn't reach the right desk,
I'm following up — and I've since published an open, good-faith analysis (not
affiliated with ACROS) so the trade-off is documented rather than hidden:

  https://github.com/hyperpolymath/0patch-lsa-sentinel

The core concern is unchanged, and I'd genuinely value your view:

  - A micropatch that cannot be deployed onto a hardened (RunAsPPL) host is, in
    effect, a fix that exists on paper only. If the console reports it as present
    while the loader is blocked from lsass.exe, the operator gets false assurance.
  - The most dangerous behaviour — that ticking "Don't show this message again"
    silently stops lsass.exe patches from applying "without you noticing anything"
    — is, in your KB, mentioned almost in passing, after the UI has offered to
    suppress the only warning.

To make that failure mode *loud* on my own machines I built a small detector
(in the repo above) that re-raises the suppressed signal as an event-log alert.
It would be far better if the agent did this itself. My questions:

  1. Is a PPL/LSA-signed loader on the roadmap (Microsoft-signed, protected-
     process-compatible), or is an out-of-process patching mechanism for
     lsass-resident code feasible instead?
  2. When the loader is blocked from a protected process, does the agent surface
     the reduced coverage in the console/reporting?
  3. Is there any supported configuration that keeps BOTH LSA Protection and full
     0patch lsass coverage, or is the standing recommendation explicitly "pick one"?
  4. How can a customer enumerate, from the console, exactly which patches are
     currently inert on a given machine because of this block? (I'd use this to
     replace a placeholder in my detector with the real local query.)

Thank you — happy to share agent version, OS build, and CodeIntegrity event logs.

Jonathan Jewell
jonathan.jewell@gmail.com
