<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-02

### Added

- **Public alert + design** (`README.md`, `DESIGN.md`): documents the
  0patch ↔ Windows LSA Protection conflict — LSA Protection (`RunAsPPL`)
  silently blocks the 0patch loader from injecting into `lsass.exe`, so
  lsass-targeted micropatches stop applying with no loud warning. The design
  argues why "just turn off LSA Protection" is not an acceptable default and
  specifies a two-layer reverse-isolation container (Layer A: penned
  `lsa-open` machine-class policy; Layer B: a read-only sentinel that routes
  the suppressed signal to an unsilenceable work-order: 🟢 covered /
  🔴 silently-inert / ⚪ moot).
- **SPARK classifier kernel** (`core-spark/`): `Sentinel.Classifier.Classify`,
  proved pure and total by `gnatprove`, mapping collected signals to the
  three-state coverage verdict. Builds `libsentinel_core.a`.
- **Idris2 ABI proofs** (`abi-idris2/`): `Sentinel.ABI` — totality-checked
  conformance proofs for the C-ABI contract shared across the toolchains.
- **Rust host** (`host-rust/`): the host application and collectors
  (registry, event-log, 0patch, snapshot) calling the SPARK classifier over a
  single isolated, documented, fail-safe `unsafe` FFI wrapper.
- **Zig FFI seam** (`ffi-zig/`): the C-ABI seam and an independent reference
  implementation cross-checking the classifier contract.
- **Cross-toolchain build orchestration** (`Justfile`): `just all` runs the
  full four-language gate (SPARK build + prove, Idris2 ABI check, Zig FFI,
  Rust host).
- **Audit records** (`audits/`): accepted structural residuals
  (`audit-residuals.adoc`) and assail classifications.

### Security

- The single Rust `unsafe` block is reduced to the FFI boundary, documented
  with a `# Safety` contract, and fail-safe decoded: an out-of-range classifier
  code maps to `Indeterminate`, never to a false "covered" verdict.

### Notes

- This is a DESIGN-stage release. The classifier core is verified; the Windows
  collectors have not been tested on a real host and the 0patch CLI collector
  is speculative. See `AFFIRMATION.adoc`.

[Unreleased]: https://github.com/hyperpolymath/0patch-lsa-sentinel/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/hyperpolymath/0patch-lsa-sentinel/releases/tag/v0.1.0
