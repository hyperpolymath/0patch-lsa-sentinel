<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Security Policy

We take security seriously and appreciate responsible disclosure. This is a
security tool whose subject matter (Windows LSA Protection, credential
protection, the 0patch agent) is itself sensitive, so please report carefully.

## Reporting a Vulnerability

### Preferred: GitHub Security Advisories

1. Go to
   [Report a vulnerability](https://github.com/hyperpolymath/0patch-lsa-sentinel/security/advisories/new).
2. Complete the form with as much detail as possible.
3. Submit — the maintainer receives a private notification.

### Alternative: Email

If you cannot use GitHub Security Advisories, email
**j.d.a.jewell@open.ac.uk** with a clear subject line. Do **not** report
vulnerabilities through public issues, pull requests, or discussions.

## What to Include

- A clear description of the vulnerability and its impact.
- Affected component (file path, function, collector, or the SPARK/Idris2/Zig
  seam).
- Affected versions or commits.
- Steps to reproduce, and a proof of concept if available.
- A CVSS 3.1 assessment if you have one.

## Response Timeline

| Stage | Target |
|-------|--------|
| Initial acknowledgement | 48 hours |
| Triage / confirmation | 7 days |
| Status updates | every 7 days |
| Resolution / coordinated disclosure | 90 days (sooner if a fix is ready) |

These are targets, not guarantees; complex issues may take longer, and we will
communicate openly about any delay.

## Scope

### In scope

- This repository (`hyperpolymath/0patch-lsa-sentinel`) and its code.
- The SPARK classifier, the Idris2 ABI proofs, the Rust host and collectors,
  and the Zig FFI seam.
- A defect that could cause the sentinel to report a host as **covered when it
  is silently inert** (a false-negative coverage verdict) is treated as a
  high-severity issue, because false assurance is this project's primary threat
  model.

### Out of scope

- The 0patch agent and ACROS Security's products (report to the vendor).
- Microsoft Windows / LSA Protection itself (report to Microsoft).
- Vulnerabilities in third-party dependencies (report here; we coordinate
  upstream).
- Social engineering, physical security, and DoS testing.

## Disclosure Policy

We follow coordinated disclosure: you report privately, we investigate and
develop a fix, we coordinate timing, and we publish an advisory and fix
together. We will credit you unless you prefer anonymity, and we will not take
legal action against good-faith research conducted under this policy.

## Security Best Practices for Contributors

- Never commit secrets, credentials, or API keys.
- Use signed commits (`git config commit.gpgsign true` or the SSH equivalent).
- Keep the FFI `unsafe` surface minimal, documented, and fail-safe.
- Run `just prove` and `just abi-check` before submitting changes that touch
  the classifier or the ABI.

---

*Thank you for helping keep 0patch-lsa-sentinel and its users safe.*
