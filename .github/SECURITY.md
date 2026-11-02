<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Security Policy

Canonical security policy: [`SECURITY.adoc`](../SECURITY.adoc).

## Reporting a Vulnerability

Please report security vulnerabilities privately — **do not** open a public
issue, pull request, or discussion:

1. **Preferred — GitHub Security Advisories:**
   <https://github.com/hyperpolymath/0patch-lsa-sentinel/security/advisories/new>
2. **Email fallback:** `j.d.a.jewell@open.ac.uk`

Any defect that could cause `0patch-lsa-sentinel` to emit a false all-clear
(`OPEN-COVERED` or `MOOT` / exit code `0`) when a host's `lsass.exe`
micropatch coverage is silently blocked or right-censored is treated as a
high-severity vulnerability.

See [`SECURITY.adoc`](../SECURITY.adoc) for full scope, response targets, and
coordinated disclosure details.
