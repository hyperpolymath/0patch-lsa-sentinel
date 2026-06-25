<!--
SPDX-FileCopyrightText: 2026 Jonathan Jewell
SPDX-License-Identifier: CC-BY-SA-4.0
-->

# 0patch-lsa-sentinel

**A public alert + design for containing the 0patch ↔ Windows LSA Protection conflict.**

Windows **LSA Protection** (`RunAsPPL`) silently blocks the 0patch agent from loading
its loader into `lsass.exe`, which means **0patch's lsass-targeted micropatches stop
applying** — and, per the vendor's own guidance, you may never notice, because the
only warning is a dismissable popup with a *"Don't show this message again"* checkbox.

You can **suppress** this. You cannot **fix** it from the customer side. This repo
exists so the trade-off is **public, documented, and loud** instead of silently hidden.

➡️ **Read [`DESIGN.md`](./DESIGN.md)** — in particular the
[**⚠️ READ THIS** alert section](./DESIGN.md#%EF%B8%8F-read-this--why-just-turn-off-lsa-protection-is-not-good-enough)
on why "just turn off LSA Protection" is not good enough, and why **a patch that
cannot be deployed is not the same as a machine that is protected.**

## What this is

1. **An alert.** Most operators, faced with the popup, will tick "don't show again" and
   move on — leaving a host that *reports as protected while it is not.* This documents
   why that is the wrong default.
2. **A design (not yet built).** A two-layer "reverse isolation" container:
   - **Layer A — policy:** a penned `lsa-open` machine-class, so the weakening never
     leaks to the wider estate; default posture stays LSA-Protection-**on**.
   - **Layer B — sentinel:** a read-only watcher that re-enables the signal the popup
     suppresses and routes it to an **unsilenceable** work-order
     (🟢 covered / 🔴 silently-inert / ⚪ moot).
3. **Open questions for the 0patch / ACROS Security team** — see [`DESIGN.md` §4](./DESIGN.md#4-the-direct-ask-to-0patch-public-on-the-record).

## Status

`DESIGN` — documentation and questions only. No agent code yet. Build order in
[`DESIGN.md` §6](./DESIGN.md#6-build-order-when-greenlit).

## Note on the name

`github.com/0patch` is an **unrelated namesake** account, not the micropatching
vendor. ACROS Security's 0patch has no public issue tracker; this repo is an
independent, good-faith analysis, not affiliated with or endorsed by ACROS Security.

## Licence

Documentation/prose: **CC-BY-SA-4.0**. Any code added later: **MPL-2.0**.
