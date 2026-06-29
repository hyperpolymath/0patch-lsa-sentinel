<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# 0patch ↔ Windows LSA Protection — Reverse-Isolation Container

**Status:** DESIGN (no code yet) · **Date:** 2026-06-25 · **Owner:** Jonathan Jewell
**Prose licence:** CC-BY-SA-4.0 · **Code (when built):** MPL-2.0

> One-line: Windows LSA Protection silently blocks 0patch from patching `lsass.exe`.
> You can suppress the warning but you cannot fix the conflict. This document
> specifies a *container* that boxes the conflict into a named, monitored
> compartment so it stops contaminating the wider security posture **and stops
> failing silently** — plus the direct, on-the-record questions for the 0patch
> (ACROS Security) team.

---

## 1. The problem, exactly

Windows **LSA Protection** (a.k.a. `RunAsPPL` / running `lsass.exe` as a Protected
Process Light) permits only appropriately Microsoft-signed code to load into the
LSA process. The 0patch agent applies in-memory micropatches by **injecting its
loader** (`0PatchLoaderX64.dll`) into running processes — including `lsass.exe`.
Because that loader is not a Microsoft-signed PPL/LSA module, the protected-process
boundary blocks it. Windows logs CodeIntegrity **Event ID 3033/3063** and raises:

> *"This module is blocked from loading into the Local Security Authority:
> `\Device\HarddiskVolume3\Program Files (x86)\0patch\agent\0PatchLoaderX64.dll`"*

ACROS confirm (support KB, "This module is blocked from loading…") that **the only
resolution is to disable LSA Protection**, and warn that if *"Don't show this
message again"* is ticked, **lsass-targeted micropatches silently stop applying**.
Their cited example is the **PetitPotam** patch, which Microsoft never fully fixed.

### The trade-off has no clean side
| Lever | Popup | lsass micropatches | Credential-dump protection |
|---|---|---|---|
| **LSA Protection ON** (Microsoft default-ish) | gone if dismissed | **silently inert** | retained |
| **LSA Protection OFF** (`RunAsPPL=0`, `RunAsPPLBoot=0`) | gone | active | **forfeited** |

You are forced to choose which protection to give up, and the suppression UI
**hides** that choice rather than resolving it.

---

## ⚠️ READ THIS — why "just turn off LSA Protection" is not good enough

> **This section is the alert.** It exists because the vendor's own guidance, taken
> at face value, leads a busy operator to tick *"Don't show this message again"* and
> walk away — leaving a machine that **reports as protected while it is not.** Most
> people will do exactly that and hide it forever. They should not.

ACROS's position (paraphrased from their KB) is, in good faith, that 0patch coverage
is worth more than LSA Protection, that an admin attacker "can disable LSA Protection
anyway," and that it "isn't needed against a non-admin attacker." That reasoning is
**well-meant but naive**, and in places it quietly argues against itself:

1. **It recasts "we can't deliver here" as "you shouldn't want this control."** The
   block is a limitation of 0patch's *injection* architecture. The guidance responds
   by downplaying a strong, default-on Microsoft security boundary so that disabling
   it looks reasonable. That is advocacy for the agent, not neutral risk analysis.

2. **"An admin can disable it anyway" is a worst-case fallacy.** LSA Protection's
   entire purpose is to raise the bar against an *already-admin* attacker performing
   post-exploitation credential theft (Mimikatz-class lsass dumping). Disabling
   `RunAsPPL` generally requires a **reboot** to take effect — noisy, detectable, and
   not always available to the attacker. "They could eventually turn it off" is not
   "it protects nothing now." With **UEFI lock**, even that path closes — a fact the
   KB mentions only in passing.

3. **"Not needed against a non-admin attacker" contradicts point 2.** The headline
   value of LSA Protection *is* the admin/SYSTEM credential-dumping case — the very
   case point 2 waves away. The two arguments cancel.

4. **A patch you cannot deploy is not "showing you care" — it risks being coverage
   theatre.** This is the appalling part. Authoring a micropatch for an lsass-resident
   bug and then being unable to load it into a hardened (`RunAsPPL`) host means the
   fix exists **on paper only.** If the console/reporting counts that patch as
   present, the operator is handed **false assurance**: the dashboard says *protected*,
   the process is *not*. A security product's worst failure mode is not "no patch" —
   it is "a patch you believe is live but isn't." Having the patch is necessary; it is
   not the same as protecting the machine, and it must never be presented as if it
   were.

5. **The silent-failure is buried as a footnote — exactly backwards.** The single most
   dangerous behaviour ("our patches for lsass.exe may not be getting applied —
   without you noticing anything") is stated almost in passing, *after* the UI has
   already offered to suppress the only warning. A product that reduces your coverage
   should **shout, not whisper**, and must never default users toward silencing the
   notice. Safe-by-default is the inverse of what's on offer here.

6. **PetitPotam is carrying the whole argument.** The flagship justification — a real,
   never-fully-fixed bug — is largely mitigable by other means (disable NTLM, enforce
   EPA / channel binding, harden the relay *targets* such as AD CS). So the coverage
   actually forfeited by keeping LSA Protection on is often **smaller** than the
   framing implies, which makes "just turn it off" a worse trade, not a better one.

**This is not malice — it is a vendor optimising for "our agent keeps working" and
under-weighting a control most operators should keep.** The safe default is the
*inverse* of their recommendation: **keep LSA Protection on, treat lsass coverage as
forfeited there, and make the loss loud** (that is precisely what the sentinel in §3
does). Disable LSA Protection only by deliberate, recorded enrolment for a specific
patch you actually depend on — never as a reflex to silence a popup.

---

## 2. The hard constraint (what this container is NOT)

**You cannot wrap your way past the kernel boundary.** PPL is enforced by the kernel
Code Integrity check at DLL-load time, keyed on the module's signing certificate.
`lsass.exe` is a protected process or it is not — there is no user-space shim that
opens a selective hole for 0patch while preserving the guarantee for everything
else. The *only* key that fits that lock is a **Microsoft-issued PPL/LSA signature
on the loader** — which is ACROS's to obtain (a PKI / signing-program matter), not
ours to engineer around. Any "wrapper" claiming to bypass PPL would simply be
defeating the boundary, i.e. re-introducing exactly the exposure LSA Protection
exists to remove.

**Therefore the container is operational + observational, never a CI bypass:**
it isolates *where* the conflict is allowed to exist and makes it *loud*. It does
not — and must not — make an unsigned loader loadable into a protected process.

---

## 3. The container — two layers

```
            ┌─────────────────────────────────────────────────────────┐
            │  ESTATE: full LSA Protection, 0patch lsass-coverage = N/A │
            │                                                           │
            │   ┌───────────────────────────────────────────────┐      │
            │   │  QUARANTINE CLASS  (Layer A — policy)          │      │
            │   │  machines permitted LSA-OFF for 0patch reach   │      │
            │   │  · segmented · no high-value cached creds      │      │
            │   │  · tighter telemetry                           │      │
            │   │                                                │      │
            │   │   [ Layer B — sentinel watches BOTH classes ]  │      │
            │   └───────────────────────────────────────────────┘      │
            └─────────────────────────────────────────────────────────┘
```

### Layer A — Policy / quarantine baseline ("reverse isolation")
The containment is a **machine-class**, not a binary. Rather than flipping
`RunAsPPL=0` estate-wide, define an explicit, small **`lsa-open` class**:

- **Membership is deliberate and listed.** A machine is only `lsa-open` if there is
  a *named* lsass-targeted patch it depends on that is not otherwise mitigated.
- **Blast-radius controls** for that class: network segmentation, no high-value
  domain creds cached/logged-on there, stricter EDR/telemetry, shorter cred TTLs.
- **Everything outside the class keeps full LSA Protection.** The weakening is
  penned; it does not leak to the estate.
- **Default posture = LSA Protection ON.** A machine forfeits it only by explicit
  enrolment, recorded with the justifying patch ID.

### Layer B — The sentinel (the "filter" that kills silent failure)
A per-machine watcher — **`0patch-lsa-sentinel`** — that converts the KB's
"silently not applied" defect into a routed, visible signal. *(This is the
detection-with-a-routed-work-order pattern: a fan-out — the block event — must have
a fan-in.)*

**Signals it reads (all user-space, no PPL violation):**
1. Registry: `HKLM\SYSTEM\CurrentControlSet\Control\Lsa\RunAsPPL` and `RunAsPPLBoot`.
2. Effective lsass protection state (`PsProtectedSignerLsa` vs none) — via a
   protection-level query or a Process-Explorer-equivalent probe.
3. CodeIntegrity Operational log: events **3033/3063** naming `0patchLoader*`.
4. 0patch console/agent: which **lsass-targeted patches** exist and whether they are
   reported applied vs blocked.

**Tri-state output (and the action each triggers):**
| State | Condition | Meaning | Action |
|---|---|---|---|
| 🟢 `OPEN-COVERED` | LSA off **and** loader in lsass | lsass patches live; cred-dump protection forfeited | log as *accepted* against the `lsa-open` enrolment |
| 🔴 `BLOCKED-INERT` | LSA on **and** lsass-targeted patches exist but loader blocked | **the silent-failure state** — believed-protected, actually not | **raise work-order / alert** |
| ⚪ `MOOT` | no outstanding lsass-targeted patches | conflict currently inert | record only |

**Crucial inversion of the KB warning:** the sentinel *re-enables* the signal the
"Don't show this again" checkbox suppresses — but routes it to the operator/console
instead of a dismissable popup. The popup can be silenced; the sentinel cannot.

**Runtime:** scheduled task on **owned compute** (no metered CI), emitting to the
estate's existing alert sink. Read-only on the host; it changes nothing, it only
observes and reports.

---

## 4. The direct ask to 0patch (public, on the record)

The trade-off itself is only ACROS's to resolve. The open questions:

1. **Is a PPL/LSA-signed loader on the roadmap?** Loading into a `RunAsPPL` lsass
   requires a Microsoft-issued protected-process signature. Are you pursuing one
   (ELAM-adjacent / approved-LSA-plugin signing), or is an **out-of-process
   patching mechanism** for lsass-resident code feasible instead?
2. **Operator-visible coverage loss.** When the loader is blocked from a protected
   process, does the agent **surface the reduced coverage** in the console/reporting
   so customers aren't left believing they are protected? (Today the only signal is
   a dismissable, suppressible popup.)
3. **Supported dual-on configuration.** Is there *any* configuration that keeps both
   LSA Protection and full 0patch lsass coverage, or is the standing recommendation
   explicitly "pick one"?
4. **Current exposure list.** How does a customer enumerate **which patches are
   currently inert** on a given machine due to this block, from the console?

---

## 5. "Make it public" — channel reality

- **ACROS's 0patch has no public GitHub / issue tracker.** `github.com/0patch` is an
  **unrelated namesake** (a 2016 personal account that forks/stars security tooling —
  `AD-Attack-Defense`, `amass`, `awesome-hacking`, etc.). Filing there reaches the
  wrong party. Do **not** raise the vendor question on that account.
- ACROS channels are `support@0patch.com`, `blog.0patch.com` (moderated comments),
  and **X / @0patch**.
- **The on-the-record surface we control is this estate repo.** Publish this analysis
  + the §4 questions in a public repo under `hyperpolymath/…`, then point ACROS at
  the permalink via email/X. That makes it public, durable, and citable without
  depending on a vendor-hosted tracker.

---

## 6. Build order (when greenlit)

1. Promote this doc into a right-sized public repo (`0patch-lsa-sentinel`).
2. Layer B v0: read-only detector script (registry + 3033/3063 + console cross-ref) →
   tri-state to stdout. No scheduling, no estate wiring yet — prove the signal.
3. Layer B v1: scheduled task on owned compute + alert-sink emission.
4. Layer A: write the `lsa-open` machine-class baseline (enrolment record, segmentation
   + cred-hygiene expectations, default-ON posture).
5. Send §4 to ACROS; link the public permalink.

> Scope discipline: v0 is a few-dozen-line read-only probe. Resist building a
> service before the signal is proven on one real machine.
